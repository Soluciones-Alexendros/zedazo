//! Caso de uso: CribarContacts — pipeline completo.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::application::stats::Stats;
use crate::domain::audit::AuditEntry;
use crate::domain::classification::classify;
use crate::domain::contact::Contact;
use crate::domain::identity::deduplicate;
use crate::domain::normalization::{normalize_fn, normalize_org, normalize_tel};
use crate::domain::pipeline::{ClassificationRuleProvider, ScreeningConfigProvider};
use crate::domain::screening::{decide, ScreeningDecision};
use crate::domain::verification::{verify, verify_post, InvariantError};
use crate::error::CribaError;

use crate::infrastructure::config::load_config;
use crate::infrastructure::encoding::ensure_utf8;
use crate::infrastructure::parser::{parse_vcards, unfold};
use crate::infrastructure::secure_fs::WriteOpts;
use crate::infrastructure::source::{detect_source, detect_version};
use crate::infrastructure::tsv_writer::write_audit_tsv;
use crate::infrastructure::v3_compat::adapt_v3;
use crate::infrastructure::writer::write_vcf;

/// Wrapper para errores de invariantes críticas que implementa Error + Display.
#[derive(Debug)]
struct CriticalInvariantErrors(Vec<InvariantError>);

impl std::fmt::Display for CriticalInvariantErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, err) in self.0.iter().enumerate() {
            if i > 0 {
                write!(f, "; ")?;
            }
            write!(f, "{}", err)?;
        }
        Ok(())
    }
}

impl std::error::Error for CriticalInvariantErrors {}

/// Ejecuta el pipeline completo de cribado.
/// Retorna estadísticas y los contactos procesados tras la deduplicación.
#[allow(clippy::too_many_arguments)]
pub fn execute(
    input: &Path,
    output: Option<&Path>,
    audit: Option<&Path>,
    config: Option<&Path>,
    source_override: &str,
    dry_run: bool,
    strict: bool,
    force: bool,
) -> Result<(Stats, Vec<Contact>), CribaError> {
    tracing::info!("Leyendo archivo: {}", input.display());
    let bytes = fs::read(input)?;

    if bytes.is_empty() {
        return Err(CribaError::EmptyVcf);
    }

    // 1. Transcodificar a UTF-8
    let utf8_text = ensure_utf8(&bytes)?;

    // 2. Desplegar líneas
    let unfolded = unfold(&utf8_text);

    // 3. Parsear
    let vcards = parse_vcards(&unfolded)?;
    let total_entrada = vcards.len();
    tracing::info!("Parseados {} contactos", total_entrada);

    // 4. Construir mapa UID -> &ParsedVCard (acceso O(1) para writer/auditoría)
    let mut vcard_map: HashMap<String, &crate::infrastructure::parser::ParsedVCard> =
        HashMap::with_capacity(vcards.len());
    for vcard in &vcards {
        vcard_map.insert(vcard.compute_uid(), vcard);
    }

    // 5. Detectar versión vCard
    let versions: Vec<String> = vcards.iter().filter_map(|v| v.version.clone()).collect();
    let detected_version = detect_version(&versions);
    tracing::info!("Versión vCard detectada: {}", detected_version);

    // 7. Cargar configuración (usando traits para inversión de dependencias)
    let app_config = load_config(config)?;
    let screening_config = app_config.screening_config();

    // 8. Procesar cada contacto
    let mut active_contacts: Vec<Contact> = Vec::with_capacity(vcards.len());
    let mut inactive_contacts: Vec<Contact> = Vec::with_capacity(vcards.len());
    let mut audit_entries = Vec::with_capacity(vcards.len());
    let mut eliminados = 0usize;
    let mut cuarentena = 0usize;

    for vcard in &vcards {
        let mut adapted = vcard.clone();
        if vcard.version.as_deref() == Some("3.0") {
            adapt_v3(&mut adapted);
        }

        let mut contact = adapted.to_contact()?;

        // Detectar fuente por contacto basado en UID
        let source_detail = if source_override != "auto" {
            match source_override.to_lowercase().as_str() {
                "proton" => crate::domain::contact::SourceDetail::ProtonAutosave,
                "google" => crate::domain::contact::SourceDetail::Google,
                "apple" => crate::domain::contact::SourceDetail::Apple,
                _ => detect_source(&[], &[contact.uid.clone()]),
            }
        } else {
            detect_source(&[], &[contact.uid.clone()])
        };
        contact.source_detail = source_detail;

        let trace = decide(&contact, screening_config);
        contact.screening_rule = trace.triggered_rule.clone();
        contact.decision = trace.outcome.clone();

        let is_active = matches!(
            contact.decision,
            ScreeningDecision::Conserved | ScreeningDecision::NeedsReview(_)
        );

        if is_active {
            // Normalización (solo conservados y needs_review)
            let (fn_normalized, title_extra, role_extra) = normalize_fn(&contact.fn_value);
            contact.fn_value = fn_normalized;
            if title_extra.is_some() && contact.title.is_none() {
                contact.title = title_extra;
            }
            if role_extra.is_some() && contact.role.is_none() {
                contact.role = role_extra;
            }

            for tel in &mut contact.tels {
                let tel_type = tel.tel_type;
                let normalized =
                    normalize_tel(&tel.value, &screening_config.prefijo_pais, tel_type);
                *tel = normalized;
            }

            if let Some(ref org) = contact.org {
                let (org_clean, legal_form) = normalize_org(org);
                contact.org = Some(org_clean);
                if contact.org_legal_form.is_none() {
                    contact.org_legal_form = legal_form;
                }
            }

            contact.categories = classify(&contact, app_config.classification_rules());

            active_contacts.push(contact);
        } else {
            match &contact.decision {
                ScreeningDecision::Eliminated(_) => eliminados += 1,
                ScreeningDecision::Quarantine(_) => cuarentena += 1,
                _ => {}
            }

            inactive_contacts.push(contact);
        }

        let fn_original = vcard
            .fn_raw
            .as_deref()
            .unwrap_or("Sin nombre")
            .replace(['\t', '\n'], " ");
        // Reconstruimos el contacto para la auditoría con el estado final.
        let contact_for_audit = if is_active {
            active_contacts.last().unwrap().clone()
        } else {
            inactive_contacts.last().unwrap().clone()
        };
        let audit_entry = AuditEntry::from_contact(&contact_for_audit, &fn_original, &trace);
        audit_entries.push(audit_entry);
    }

    // 9. Deduplicación solo de contactos activos
    let (active_contacts, fusionados) = deduplicate(active_contacts);

    // Actualizar entradas de auditoría para contactos fusionados
    let mut merged_map: HashMap<String, String> = HashMap::new();
    for c in &active_contacts {
        for merged_uid in &c.merged_uids {
            merged_map.insert(merged_uid.clone(), c.uid.clone());
        }
    }
    for entry in &mut audit_entries {
        if let Some(target) = merged_map.get(&entry.uid) {
            *entry = entry.clone().merged_into(target);
        }
    }

    // Recombinar activos e inactivos para salida y verificación
    let mut contacts = Vec::with_capacity(active_contacts.len() + inactive_contacts.len());
    contacts.extend(active_contacts);
    contacts.extend(inactive_contacts);

    // Recalcular métricas finales a partir de la lista consolidada
    let conservados = contacts
        .iter()
        .filter(|c| matches!(c.decision, ScreeningDecision::Conserved))
        .count();
    let needs_review = contacts
        .iter()
        .filter(|c| matches!(c.decision, ScreeningDecision::NeedsReview(_)))
        .count();

    // 10. Verificación de invariantes
    match verify(&contacts, Some(input), output, strict) {
        Ok(warnings) => {
            if !warnings.is_empty() {
                for warning in &warnings {
                    tracing::warn!("{}", warning);
                }
            }
        }
        Err(critical_errors) => {
            for err in &critical_errors {
                tracing::error!("{}", err);
            }
            let err_box: Box<dyn std::error::Error + Send + Sync> =
                Box::new(CriticalInvariantErrors(critical_errors));
            return Err(CribaError::InvariantError(err_box));
        }
    }

    tracing::info!(
        "Pipeline: {} entrada, {} conservados, {} eliminados, {} cuarentena, {} fusionados, {} needs_review",
        total_entrada, conservados, eliminados, cuarentena, fusionados, needs_review
    );

    let stats = Stats::compute(
        &contacts,
        total_entrada,
        conservados,
        eliminados,
        fusionados,
        cuarentena,
        needs_review,
    );

    // Ordenar contactos para que los activos aparezcan primero en el VCF
    contacts.sort_by_key(|c| match c.decision {
        ScreeningDecision::Conserved | ScreeningDecision::NeedsReview(_) => 0,
        _ => 1,
    });

    // 10. Salida
    if !dry_run {
        let write_opts = WriteOpts::new(force).protect(input);
        if let Some(out_path) = output {
            write_vcf(&contacts, &vcard_map, out_path, true, write_opts)?;
        }
        if let Some(audit_path) = audit {
            write_audit_tsv(&audit_entries, audit_path, write_opts)?;
        }
        // 11. Verificación post-escritura I4-I6 (no crítica, solo warnings)
        let post_warnings = verify_post(&contacts, output, audit);
        for w in &post_warnings {
            tracing::warn!("{}", w);
        }
    }

    Ok((stats, contacts))
}
