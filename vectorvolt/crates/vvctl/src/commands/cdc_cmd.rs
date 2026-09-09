//! `vvctl cdc` — Change Data Capture inspection commands (S25-A, VV-EVT-020).

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use vectorvolt_engine::{EngineOptions, TableRegistry};

#[derive(Debug, Args)]
pub struct CdcArgs {
    #[command(subcommand)]
    pub command: CdcCommand,
}

#[derive(Debug, Subcommand)]
pub enum CdcCommand {
    /// Print change events from a table's CDC log.
    Tail {
        /// Table name.
        table: String,
        /// Table data directory.
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
        /// Start from this LSN (inclusive). 0 = beginning.
        #[arg(long, default_value = "0")]
        from: u64,
        /// Maximum number of events to print. 0 = no limit.
        #[arg(short = 'n', long, default_value = "0")]
        limit: u64,
    },
    /// Show CDC log statistics for a table.
    Stats {
        /// Table name.
        table: String,
        /// Table data directory.
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
    },
    /// Compact the CDC log, discarding events older than a given LSN.
    Compact {
        /// Table name.
        table: String,
        /// Table data directory.
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
        /// Retain events with LSN >= this value.
        #[arg(long)]
        retain_lsn: u64,
    },
}

pub fn run(args: CdcArgs) -> Result<()> {
    match args.command {
        CdcCommand::Tail {
            table,
            dir,
            from,
            limit,
        } => cmd_tail(&table, &dir, from, limit),
        CdcCommand::Stats { table, dir } => cmd_stats(&table, &dir),
        CdcCommand::Compact {
            table,
            dir,
            retain_lsn,
        } => cmd_compact(&table, &dir, retain_lsn),
    }
}

fn open_engine(_table: &str, dir: &PathBuf) -> Result<Arc<TableRegistry>> {
    let reg = TableRegistry::open(dir, EngineOptions::default())
        .with_context(|| format!("opening registry at {}", dir.display()))?;
    Ok(Arc::new(reg))
}

fn cmd_tail(table: &str, dir: &PathBuf, from: u64, limit: u64) -> Result<()> {
    let reg = open_engine(table, dir)?;
    let engine = reg
        .get(table)
        .map_err(|_| anyhow::anyhow!("table not found: {table}"))?;
    let events = engine
        .read_log_from(from)
        .with_context(|| format!("reading CDC log for {table}"))?;

    let iter: Box<dyn Iterator<Item = _>> = if limit > 0 {
        Box::new(events.into_iter().take(limit as usize))
    } else {
        Box::new(events.into_iter())
    };

    for ev in iter {
        let vals = ev
            .values
            .as_ref()
            .map(|v| {
                v.iter()
                    .map(|n| n.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_default();
        let kind = format!("{:?}", ev.kind).to_lowercase();
        if vals.is_empty() {
            println!("lsn={:<6} {:6}  pk={}", ev.lsn, kind, ev.pk);
        } else {
            println!(
                "lsn={:<6} {:6}  pk={}  values=[{}]",
                ev.lsn, kind, ev.pk, vals
            );
        }
    }
    Ok(())
}

fn cmd_stats(table: &str, dir: &PathBuf) -> Result<()> {
    let reg = open_engine(table, dir)?;
    let engine = reg
        .get(table)
        .map_err(|_| anyhow::anyhow!("table not found: {table}"))?;
    let stats = engine
        .cdc_log_stats()
        .with_context(|| format!("reading CDC stats for {table}"))?;
    println!("table        : {table}");
    println!("event_count  : {}", stats.event_count);
    println!("byte_size    : {} bytes", stats.byte_size);
    println!("min_lsn      : {}", stats.min_lsn);
    println!("max_lsn      : {}", stats.max_lsn);
    Ok(())
}

fn cmd_compact(table: &str, dir: &PathBuf, retain_lsn: u64) -> Result<()> {
    let reg = open_engine(table, dir)?;
    let engine = reg
        .get(table)
        .map_err(|_| anyhow::anyhow!("table not found: {table}"))?;
    let dropped = engine
        .compact_cdc_log(retain_lsn)
        .with_context(|| format!("compacting CDC log for {table}"))?;
    println!("Compacted: removed {dropped} event(s) with lsn < {retain_lsn}.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use tempfile::TempDir;
    use vectorvolt_engine::{EngineOptions, TableRegistry};

    fn make_registry(dir: &std::path::Path) -> Arc<TableRegistry> {
        Arc::new(TableRegistry::open(dir, EngineOptions::default()).unwrap())
    }

    #[test]
    fn test_cdc_tail_empty() {
        let d = TempDir::new().unwrap();
        let reg = make_registry(d.path());
        reg.create_table("orders", vec!["id".into(), "val".into()])
            .unwrap();
        let engine = reg.get("orders").unwrap();
        let events = engine.read_log_from(0).unwrap();
        assert!(events.is_empty());
    }

    #[test]
    fn test_cdc_tail_after_inserts() {
        let d = TempDir::new().unwrap();
        let reg = make_registry(d.path());
        reg.create_table("orders", vec!["id".into(), "val".into()])
            .unwrap();
        let engine = reg.get("orders").unwrap();
        engine.insert(1, vec![1, 100]).unwrap();
        engine.insert(2, vec![2, 200]).unwrap();
        let events = engine.read_log_from(0).unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].pk, 1);
        assert_eq!(events[1].pk, 2);
    }

    #[test]
    fn test_cdc_tail_from_lsn() {
        let d = TempDir::new().unwrap();
        let reg = make_registry(d.path());
        reg.create_table("t", vec!["id".into()]).unwrap();
        let engine = reg.get("t").unwrap();
        engine.insert(1, vec![1]).unwrap();
        engine.insert(2, vec![2]).unwrap();
        engine.insert(3, vec![3]).unwrap();
        let all = engine.read_log_from(0).unwrap();
        let from2 = engine.read_log_from(2).unwrap();
        assert_eq!(all.len(), 3);
        assert!(from2.len() < all.len());
    }

    #[test]
    fn test_cdc_stats() {
        let d = TempDir::new().unwrap();
        let reg = make_registry(d.path());
        reg.create_table("t", vec!["id".into()]).unwrap();
        let engine = reg.get("t").unwrap();
        engine.insert(10, vec![10]).unwrap();
        engine.insert(20, vec![20]).unwrap();
        let stats = engine.cdc_log_stats().unwrap();
        assert_eq!(stats.event_count, 2);
        assert!(stats.byte_size > 0);
        assert_eq!(stats.min_lsn, 1);
        assert_eq!(stats.max_lsn, 2);
    }

    #[test]
    fn test_cdc_compact() {
        let d = TempDir::new().unwrap();
        let reg = make_registry(d.path());
        reg.create_table("t", vec!["id".into()]).unwrap();
        let engine = reg.get("t").unwrap();
        for i in 1u64..=5 {
            engine.insert(i, vec![i]).unwrap();
        }
        let dropped = engine.compact_cdc_log(3).unwrap();
        assert_eq!(dropped, 2); // lsn 1 and 2 removed
        let remaining = engine.read_log_from(0).unwrap();
        assert_eq!(remaining.len(), 3);
        assert!(remaining.iter().all(|e| e.lsn >= 3));
    }

    #[test]
    fn test_cdc_delete_event() {
        let d = TempDir::new().unwrap();
        let reg = make_registry(d.path());
        reg.create_table("t", vec!["id".into()]).unwrap();
        let engine = reg.get("t").unwrap();
        engine.insert(1, vec![1]).unwrap();
        engine.delete(1).unwrap();
        let events = engine.read_log_from(0).unwrap();
        assert_eq!(events.len(), 2);
        use vectorvolt_engine::ChangeKind;
        assert_eq!(events[0].kind, ChangeKind::Insert);
        assert_eq!(events[1].kind, ChangeKind::Delete);
    }
}
