use std::path::PathBuf;

use anyhow::Result;
use bytes::Bytes;
use serde_json::json;
use vectorvolt_store::{KvStore, ProjectionDispatcher, ScanRequest, StoreConfig};

// --------------------------------------------------------------------------
// Output format
// --------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, clap::ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
}

fn open_readonly(dir: &PathBuf) -> Result<std::sync::Arc<KvStore>> {
    Ok(KvStore::open(StoreConfig::new(dir).with_fsync(false))?)
}

fn open_write(dir: &PathBuf) -> Result<std::sync::Arc<KvStore>> {
    Ok(KvStore::open(StoreConfig::new(dir).with_fsync(true))?)
}

// --------------------------------------------------------------------------
// Commands
// --------------------------------------------------------------------------

pub fn cmd_stats(dir: PathBuf, fmt: OutputFormat) -> Result<()> {
    let store = open_readonly(&dir)?;
    let s = store.stats();

    match fmt {
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "dir": dir.display().to_string(),
                    "active_memtable_entries": s.active_memtable_entries,
                    "active_memtable_bytes":   s.active_memtable_bytes,
                    "sstable_count":           s.sstable_count,
                    "sstable_entries":         s.sstable_entries,
                }))?
            );
        }
        OutputFormat::Text => {
            println!("VectorVolt Store — {}", dir.display());
            println!("  Active memtable : {} entries ({} bytes)",
                s.active_memtable_entries, s.active_memtable_bytes);
            println!("  SSTables        : {}", s.sstable_count);
            println!("  SSTable entries : {}", s.sstable_entries);
        }
    }
    Ok(())
}

pub fn cmd_get(dir: PathBuf, key: String, fmt: OutputFormat) -> Result<()> {
    let store = open_readonly(&dir)?;
    match store.get(Bytes::from(key.clone()))? {
        Some(v) => {
            let value = std::str::from_utf8(&v).unwrap_or("<binary>").to_string();
            match fmt {
                OutputFormat::Json => println!("{}", json!({ "key": key, "value": value })),
                OutputFormat::Text => println!("{key} = {value}"),
            }
        }
        None => {
            match fmt {
                OutputFormat::Json => println!("{}", json!({ "key": key, "value": null })),
                OutputFormat::Text => {
                    eprintln!("{key}: not found");
                    std::process::exit(1);
                }
            }
        }
    }
    Ok(())
}

pub fn cmd_put(dir: PathBuf, key: String, value: String, fmt: OutputFormat) -> Result<()> {
    let store = open_write(&dir)?;
    store.put(Bytes::from(key.clone()), Bytes::from(value.clone()))?;
    store.flush()?;
    match fmt {
        OutputFormat::Json => println!("{}", json!({ "ok": true, "key": key, "value": value })),
        OutputFormat::Text => println!("PUT {key} = {value}"),
    }
    Ok(())
}

pub fn cmd_delete(dir: PathBuf, key: String, fmt: OutputFormat) -> Result<()> {
    let store = open_write(&dir)?;
    store.delete(Bytes::from(key.clone()))?;
    store.flush()?;
    match fmt {
        OutputFormat::Json => println!("{}", json!({ "ok": true, "key": key })),
        OutputFormat::Text => println!("DEL {key}"),
    }
    Ok(())
}

pub fn cmd_compact(dir: PathBuf, fmt: OutputFormat) -> Result<()> {
    let store = open_write(&dir)?;
    store.flush()?;
    let s = store.stats();
    match fmt {
        OutputFormat::Json => println!("{}", json!({
            "ok": true,
            "sstable_count":   s.sstable_count,
            "sstable_entries": s.sstable_entries,
        })),
        OutputFormat::Text => println!(
            "compaction complete — {} SSTables, {} entries",
            s.sstable_count, s.sstable_entries
        ),
    }
    Ok(())
}

pub fn cmd_scan(
    dir: PathBuf,
    prefix: Option<String>,
    start: Option<String>,
    end: Option<String>,
    limit: Option<usize>,
    keys_only: bool,
    fmt: OutputFormat,
) -> Result<()> {
    let store = open_readonly(&dir)?;
    let pd = ProjectionDispatcher::new(store);

    let req = if let Some(p) = prefix {
        let mut r = ScanRequest::prefix(Bytes::from(p));
        if let Some(n) = limit { r = r.with_limit(n); }
        r
    } else {
        let mut r = match (start, end) {
            (Some(s), Some(e)) => ScanRequest::inclusive(
                Bytes::from(s),
                Bytes::from(e),
            ),
            (Some(s), None) => ScanRequest::from_start(Bytes::from(s)),
            (None,    _)    => ScanRequest::full(),
        };
        if let Some(n) = limit { r = r.with_limit(n); }
        r
    };

    let entries = if keys_only {
        pd.keys(req)?
            .into_iter()
            .map(|k| (k, Bytes::new()))
            .collect::<Vec<_>>()
    } else {
        pd.scan(req)?
            .into_iter()
            .map(|e| (e.key, e.value))
            .collect::<Vec<_>>()
    };

    match fmt {
        OutputFormat::Json => {
            let items: Vec<_> = entries
                .iter()
                .map(|(k, v)| {
                    let ks = std::str::from_utf8(k).unwrap_or("<binary>");
                    if keys_only {
                        json!({ "key": ks })
                    } else {
                        let vs = std::str::from_utf8(v).unwrap_or("<binary>");
                        json!({ "key": ks, "value": vs })
                    }
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&items)?);
        }
        OutputFormat::Text => {
            if entries.is_empty() {
                println!("(no results)");
            } else {
                for (k, v) in &entries {
                    let ks = std::str::from_utf8(k).unwrap_or("<binary>");
                    if keys_only {
                        println!("{ks}");
                    } else {
                        let vs = std::str::from_utf8(v).unwrap_or("<binary>");
                        println!("{ks} = {vs}");
                    }
                }
                println!("({} entries)", entries.len());
            }
        }
    }
    Ok(())
}
