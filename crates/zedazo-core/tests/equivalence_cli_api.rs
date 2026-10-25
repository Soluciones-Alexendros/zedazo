//! Equivalencia process::run vs cribar::execute (paridad CLI/API core).
//! Smoke rápido multi-fixture; el cierre O10 lo firma `zedazo-api` equivalence_http.

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::Arc;

use tempfile::tempdir;
use zedazo_core::application::cribar;
use zedazo_core::application::process::{
    self, ArtifactKind, JobId, NullProgress, ProcessRequest, RulesConfigSource,
};
use zedazo_core::domain::screening::ScreeningDecision;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

const FIXTURES: &[&str] = &[
    "sample-contacts.vcf",
    "google_contactos.vcf",
    "google_otroscontactos.vcf",
    "proton_sample.vcf",
    "iso_sample.vcf",
    "duplicates.vcf",
    "edge_cases.vcf",
];

fn count_decision(
    contacts: &[zedazo_core::domain::contact::Contact],
    pred: fn(&ScreeningDecision) -> bool,
) -> usize {
    contacts.iter().filter(|x| pred(&x.decision)).count()
}

#[test]
fn equivalence_multi_fixture_counts_and_artifacts() {
    for name in FIXTURES {
        let input = fixture(name);
        assert!(input.exists(), "falta fixture {name}");

        let (_stats_cli, contacts_cli) =
            cribar::execute(&input, None, None, None, "auto", true, false, false).unwrap();

        let dir = tempdir().unwrap();
        let req = ProcessRequest {
            job_id: JobId(format!("test-{name}")),
            input_path: input.clone(),
            working_directory: dir.path().to_path_buf(),
            config: RulesConfigSource::BuiltIn,
            requested_artifacts: BTreeSet::from([
                ArtifactKind::StatsJson,
                ArtifactKind::AuditTsv,
                ArtifactKind::Vcf,
                ArtifactKind::Csv,
                ArtifactKind::Json,
            ]),
            source_override: "auto".into(),
            dry_run: false,
            strict: false,
            cancellation: Default::default(),
            progress: Arc::new(NullProgress),
        };
        let result = process::run(&req).unwrap();

        assert_eq!(
            count_decision(&contacts_cli, |d| matches!(d, ScreeningDecision::Conserved)),
            result.summary.retained,
            "{name}: retained"
        );
        assert_eq!(
            count_decision(&contacts_cli, |d| matches!(
                d,
                ScreeningDecision::NeedsReview(_)
            )),
            result.summary.needs_review,
            "{name}: needs_review"
        );
        assert_eq!(
            count_decision(&contacts_cli, |d| matches!(
                d,
                ScreeningDecision::Eliminated(_)
            )),
            result.summary.eliminated,
            "{name}: eliminated"
        );
        assert_eq!(
            count_decision(&contacts_cli, |d| matches!(
                d,
                ScreeningDecision::Quarantine(_)
            )),
            result.summary.quarantine,
            "{name}: quarantine"
        );
        assert_eq!(contacts_cli.len(), result.contacts.len(), "{name}: len");

        assert!(dir.path().join("stats.json").exists(), "{name}: stats.json");
        assert!(dir.path().join("audit.tsv").exists(), "{name}: audit.tsv");
        assert!(dir.path().join("output.vcf").exists(), "{name}: output.vcf");
        assert!(dir.path().join("contacts.csv").exists(), "{name}: csv");
        assert!(dir.path().join("contacts.json").exists(), "{name}: json");
        assert_eq!(result.artifacts.len(), 5, "{name}: artifact descriptors");
    }
}
