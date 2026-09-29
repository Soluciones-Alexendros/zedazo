//! Adaptador CLI → `zedazo-carddav` (ADR-0018; sin GUI).

use std::io::{self, Write};
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

use anyhow::{bail, Context};
use zedazo_carddav::config::{load, ConfigOverrides};
use zedazo_carddav::filter::parse_category_filter;
use zedazo_carddav::url_policy::{parse_and_validate, resolve_href};
use zedazo_carddav::{CardDavClient, CardDavError, Url};

use super::cli::{CarddavAction, CarddavOpts};

pub fn run(action: CarddavAction) -> anyhow::Result<()> {
    match action {
        CarddavAction::List { opts } => list(opts),
        CarddavAction::Pull {
            opts,
            output,
            categories,
        } => pull(opts, output, categories),
        CarddavAction::Put {
            opts,
            href,
            input,
            etag,
            create,
            confirm,
        } => put(opts, href, input, etag, create, confirm),
        CarddavAction::Delete {
            opts,
            href,
            etag,
            confirm,
        } => delete(opts, href, etag, confirm),
        CarddavAction::Watch {
            opts,
            interval,
            max_cycles,
            once,
            output,
            categories,
        } => watch(opts, interval, max_cycles, once, output, categories),
    }
}

fn client_from_opts(opts: CarddavOpts) -> anyhow::Result<CardDavClient> {
    let config = load(ConfigOverrides {
        url: opts.url,
        username: opts.username,
        password: None,
        addressbook: opts.addressbook,
        config_path: opts.config,
    })
    .context("configuración CardDAV incompleta (ZEDAZO_CARDDAV_* / [carddav] / flags)")?;
    CardDavClient::new(config).context("no se pudo crear el cliente CardDAV")
}

fn parse_categories(raw: Vec<String>) -> anyhow::Result<Vec<String>> {
    raw.into_iter()
        .map(|c| parse_category_filter(&c).map_err(Into::into))
        .collect()
}

fn list(opts: CarddavOpts) -> anyhow::Result<()> {
    let client = client_from_opts(opts)?;
    let books = client.list_addressbooks()?;
    if books.is_empty() {
        anyhow::bail!("no se encontró ningún addressbook");
    }
    let stdout = io::stdout();
    let mut out = stdout.lock();
    for book in books {
        let name = book.displayname.as_deref().unwrap_or("-");
        writeln!(out, "{}\t{name}", book.url)?;
    }
    Ok(())
}

fn pull(opts: CarddavOpts, output: PathBuf, categories: Vec<String>) -> anyhow::Result<()> {
    let categories = parse_categories(categories)?;
    let client = client_from_opts(opts)?;
    let result = client.pull_filtered(None, &categories)?;
    write_vcf(&output, &result.concatenated_vcf())?;
    let name = result
        .addressbook
        .displayname
        .as_deref()
        .unwrap_or(result.addressbook.url.as_str());
    eprintln!(
        "Descargados {} vCard(s) de {name} → {}",
        result.cards.len(),
        output.display()
    );
    Ok(())
}

fn require_confirm(confirm: bool, op: &str) -> anyhow::Result<()> {
    if confirm {
        return Ok(());
    }
    bail!(
        "escritura CardDAV opt-in: {op} exige --confirm (no forma parte de `cribar`; \
         I7: el VCF local de origen no se sobrescribe; el remoto no se muta sin flag)"
    );
}

fn resolve_object_href(opts: &CarddavOpts, href: &str) -> anyhow::Result<Url> {
    if href.contains("://") {
        return parse_and_validate(href).map_err(Into::into);
    }
    if let Some(ab) = &opts.addressbook {
        let base = parse_and_validate(ab)?;
        return resolve_href(&base, href).map_err(Into::into);
    }
    if let Ok(ab) = std::env::var("ZEDAZO_CARDDAV_ADDRESSBOOK") {
        if !ab.trim().is_empty() {
            let base = parse_and_validate(&ab)?;
            return resolve_href(&base, href).map_err(Into::into);
        }
    }
    if let Some(url) = &opts.url {
        let base = parse_and_validate(url)?;
        return resolve_href(&base, href).map_err(Into::into);
    }
    bail!("href relativo: indica --addressbook o --url (o ZEDAZO_CARDDAV_*)");
}

fn put(
    opts: CarddavOpts,
    href: String,
    input: PathBuf,
    etag: Option<String>,
    create: bool,
    confirm: bool,
) -> anyhow::Result<()> {
    require_confirm(confirm, "put")?;
    let target = resolve_object_href(&opts, &href)?;
    let vcard = std::fs::read_to_string(&input)
        .with_context(|| format!("no se pudo leer {}", input.display()))?;
    let client = client_from_opts(opts)?;
    let result = if create {
        client.create(&target, &vcard)
    } else {
        let etag = etag.ok_or(CardDavError::MissingEtag)?;
        client.put(&target, &vcard, &etag)
    };
    match result {
        Ok(w) => {
            eprintln!(
                "PUT {} → HTTP {} (ETag {})",
                w.href,
                w.status,
                w.etag.as_deref().unwrap_or("-")
            );
            Ok(())
        }
        Err(CardDavError::PreconditionFailed { url, etag }) => {
            bail!(
                "conflicto CardDAV (HTTP 412) en {url}: If-Match {etag} no coincide; \
                 no se sobrescribió. Vuelve a hacer pull y revisa el ETag."
            )
        }
        Err(err) => Err(err.into()),
    }
}

fn delete(opts: CarddavOpts, href: String, etag: String, confirm: bool) -> anyhow::Result<()> {
    require_confirm(confirm, "delete")?;
    let target = resolve_object_href(&opts, &href)?;
    let client = client_from_opts(opts)?;
    match client.delete(&target, &etag) {
        Ok(w) => {
            eprintln!("DELETE {} → HTTP {}", w.href, w.status);
            Ok(())
        }
        Err(CardDavError::PreconditionFailed { url, etag }) => {
            bail!(
                "conflicto CardDAV (HTTP 412) en {url}: If-Match {etag} no coincide; \
                 no se borró."
            )
        }
        Err(err) => Err(err.into()),
    }
}

fn watch(
    opts: CarddavOpts,
    interval: u64,
    max_cycles: u32,
    once: bool,
    output: Option<PathBuf>,
    categories: Vec<String>,
) -> anyhow::Result<()> {
    let categories = parse_categories(categories)?;
    let client = client_from_opts(opts)?;
    let mut previous = client.collection_sync_state(None)?;
    eprintln!(
        "watch: línea base CTag={} sync-token={} (sin sync automático al arrancar)",
        previous.ctag.as_deref().unwrap_or("-"),
        previous.sync_token.as_deref().unwrap_or("-")
    );

    let cycles = if once { 1 } else { max_cycles };
    let mut n = 0u32;
    loop {
        if interval > 0 {
            thread::sleep(Duration::from_secs(interval));
        }
        let current = client.collection_sync_state(None)?;
        if current.changed_from(&previous) {
            eprintln!(
                "watch: cambio CTag {} → {} / token {} → {}",
                previous.ctag.as_deref().unwrap_or("-"),
                current.ctag.as_deref().unwrap_or("-"),
                previous.sync_token.as_deref().unwrap_or("-"),
                current.sync_token.as_deref().unwrap_or("-")
            );
            if let Some(path) = &output {
                let pulled = client.pull_filtered(None, &categories)?;
                write_vcf(path, &pulled.concatenated_vcf())?;
                eprintln!(
                    "watch: {} vCard(s) → {}",
                    pulled.cards.len(),
                    path.display()
                );
            }
            previous = current;
        }
        n += 1;
        if cycles > 0 && n >= cycles {
            break;
        }
    }
    Ok(())
}

fn write_vcf(output: &std::path::Path, vcf: &str) -> anyhow::Result<()> {
    if let Some(parent) = output.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    std::fs::write(output, vcf.as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::require_confirm;

    #[test]
    fn write_exige_confirm() {
        assert!(require_confirm(false, "put").is_err());
        assert!(require_confirm(true, "put").is_ok());
    }
}
