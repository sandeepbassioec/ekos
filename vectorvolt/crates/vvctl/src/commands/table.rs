//! `vvctl table` — table-oriented commands backed by [`vectorvolt_engine::TableEngine`].
//!
//! ## Schema persistence
//!
//! The schema (column names) is written to `<dir>/schema.txt` (one name per line)
//! at `create` time and re-read by every other subcommand.  The PK column is
//! always index 0.
//!
//! ## Flush semantics
//!
//! Every `insert` and `delete` call flushes immediately so data survives the
//! process exit.  `get`, `scan`, and `stats` are read-only.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use clap::Subcommand;
use vectorvolt_engine::TableEngine;

const SCHEMA_FILE: &str = "schema.txt";
const FLUSH_THRESHOLD: usize = 1_048_576; // 1M rows — effectively manual-only from CLI

// ── Subcommand definitions ────────────────────────────────────────────────────

#[derive(Subcommand)]
pub enum TableCommand {
    /// Create a new table directory with the given schema.
    Create {
        /// Directory where table data will be stored.
        dir: PathBuf,
        /// Column names (first column is the primary key).
        #[arg(required = true, num_args = 1..)]
        columns: Vec<String>,
    },

    /// Insert a row (pk + one value per non-PK column).
    Insert {
        /// Table directory.
        dir: PathBuf,
        /// Primary key value (u64).
        pk: u64,
        /// Column values in schema order (excluding the PK column).
        #[arg(num_args = 0..)]
        values: Vec<u64>,
    },

    /// Look up a single row by primary key.
    Get {
        /// Table directory.
        dir: PathBuf,
        /// Primary key value (u64).
        pk: u64,
    },

    /// Range scan on a column (inclusive on both ends).
    Scan {
        /// Table directory.
        dir: PathBuf,
        /// Column index to scan (0 = PK column).
        #[arg(long, default_value = "0")]
        col: usize,
        /// Minimum value (inclusive).
        #[arg(long, default_value = "0")]
        lo: u64,
        /// Maximum value (inclusive).
        #[arg(long, default_value = "18446744073709551615")]
        hi: u64,
    },

    /// Delete a row by primary key.
    Delete {
        /// Table directory.
        dir: PathBuf,
        /// Primary key value (u64).
        pk: u64,
    },

    /// Show table statistics.
    Stats {
        /// Table directory.
        dir: PathBuf,
    },

    /// Force flush the write buffer to a new partition file.
    Flush {
        /// Table directory.
        dir: PathBuf,
    },

    /// Merge all partitions into one sorted file, removing deleted rows.
    Compact {
        /// Table directory.
        dir: PathBuf,
    },
}

// ── Public entry point ────────────────────────────────────────────────────────

pub fn run(cmd: TableCommand) -> Result<()> {
    match cmd {
        TableCommand::Create { dir, columns } => cmd_create(dir, columns),
        TableCommand::Insert { dir, pk, values } => cmd_insert(dir, pk, values),
        TableCommand::Get { dir, pk } => cmd_get(dir, pk),
        TableCommand::Scan { dir, col, lo, hi } => cmd_scan(dir, col, lo, hi),
        TableCommand::Delete { dir, pk } => cmd_delete(dir, pk),
        TableCommand::Stats { dir } => cmd_stats(dir),
        TableCommand::Flush { dir } => cmd_flush(dir),
        TableCommand::Compact { dir } => cmd_compact(dir),
    }
}

// ── Individual commands ───────────────────────────────────────────────────────

fn cmd_create(dir: PathBuf, columns: Vec<String>) -> Result<()> {
    if columns.is_empty() {
        bail!("at least one column (the PK) is required");
    }
    std::fs::create_dir_all(&dir).with_context(|| format!("create table dir {}", dir.display()))?;

    let schema_path = dir.join(SCHEMA_FILE);
    if schema_path.exists() {
        bail!("table already exists at {}", dir.display());
    }

    // Open the engine once to create WAL + data subdirectories.
    let schema = columns.clone();
    let _engine =
        TableEngine::open(schema, &dir, FLUSH_THRESHOLD).with_context(|| "open engine")?;

    std::fs::write(&schema_path, columns.join("\n")).with_context(|| "write schema.txt")?;

    println!("created table at {}", dir.display());
    println!("schema: {}", columns.join(", "));
    Ok(())
}

fn cmd_insert(dir: PathBuf, pk: u64, values: Vec<u64>) -> Result<()> {
    let schema = load_schema(&dir)?;
    let engine =
        TableEngine::open(schema.clone(), &dir, FLUSH_THRESHOLD).with_context(|| "open engine")?;

    // Build full row: pk column (index 0) + remaining values.
    let mut row = vec![pk];
    row.extend_from_slice(&values);

    if row.len() != schema.len() {
        bail!(
            "schema has {} columns but {} values provided (including pk)",
            schema.len(),
            row.len()
        );
    }

    engine.insert(pk, row).with_context(|| "insert")?;
    engine.flush().with_context(|| "flush after insert")?;

    println!("inserted pk={pk}");
    Ok(())
}

fn cmd_get(dir: PathBuf, pk: u64) -> Result<()> {
    let schema = load_schema(&dir)?;
    let engine =
        TableEngine::open(schema.clone(), &dir, FLUSH_THRESHOLD).with_context(|| "open engine")?;

    match engine.get(pk).with_context(|| "get")? {
        None => println!("not found: pk={pk}"),
        Some(row) => {
            for (name, val) in schema.iter().zip(row.values.iter()) {
                println!("{name}={val}");
            }
        }
    }
    Ok(())
}

fn cmd_scan(dir: PathBuf, col: usize, lo: u64, hi: u64) -> Result<()> {
    let schema = load_schema(&dir)?;
    let col_name = schema
        .get(col)
        .cloned()
        .unwrap_or_else(|| format!("col[{col}]"));
    let engine =
        TableEngine::open(schema.clone(), &dir, FLUSH_THRESHOLD).with_context(|| "open engine")?;

    let rows = engine.scan(col, lo, hi).with_context(|| "scan")?;
    if rows.is_empty() {
        println!("no rows matched {col_name} in [{lo}, {hi}]");
        return Ok(());
    }

    // Header
    println!("{}", schema.join("\t"));
    println!(
        "{}",
        schema
            .iter()
            .map(|_| "────────")
            .collect::<Vec<_>>()
            .join("\t")
    );
    for row in &rows {
        let line: Vec<String> = row.values.iter().map(|v| v.to_string()).collect();
        println!("{}", line.join("\t"));
    }
    println!("\n{} row(s) matched", rows.len());
    Ok(())
}

fn cmd_delete(dir: PathBuf, pk: u64) -> Result<()> {
    let schema = load_schema(&dir)?;
    let engine = TableEngine::open(schema, &dir, FLUSH_THRESHOLD).with_context(|| "open engine")?;

    engine.flush().with_context(|| "flush before delete")?;
    engine.delete(pk).with_context(|| "delete")?;

    println!("deleted pk={pk}");
    Ok(())
}

fn cmd_stats(dir: PathBuf) -> Result<()> {
    let schema = load_schema(&dir)?;
    let engine = TableEngine::open(schema, &dir, FLUSH_THRESHOLD).with_context(|| "open engine")?;

    let s = engine.stats();
    println!("schema:      {}", s.schema.join(", "));
    println!("buffered:    {} row(s)", s.rows_buffered);
    println!("flushed:     {} row(s)", s.rows_flushed);
    println!("partitions:  {}", s.partition_count);
    Ok(())
}

fn cmd_flush(dir: PathBuf) -> Result<()> {
    let schema = load_schema(&dir)?;
    let engine = TableEngine::open(schema, &dir, FLUSH_THRESHOLD).with_context(|| "open engine")?;

    match engine.flush().with_context(|| "flush")? {
        None => println!("nothing to flush"),
        Some(r) => println!("flushed {} row(s) → {}", r.row_count, r.path.display()),
    }
    Ok(())
}

fn cmd_compact(dir: PathBuf) -> Result<()> {
    let schema = load_schema(&dir)?;
    let engine = TableEngine::open(schema, &dir, FLUSH_THRESHOLD).with_context(|| "open engine")?;

    let before = engine.stats();
    match engine.compact().with_context(|| "compact")? {
        None => println!(
            "nothing to compact ({} partition(s), {} row(s) — already optimal)",
            before.partition_count, before.rows_flushed
        ),
        Some(r) => {
            println!(
                "compacted {} partition(s) → 1 partition",
                before.partition_count
            );
            println!("rows kept: {}", r.rows_kept);
            println!("output:    {}", r.path.display());
        }
    }
    Ok(())
}

// ── Helper ────────────────────────────────────────────────────────────────────

fn load_schema(dir: &Path) -> Result<Vec<String>> {
    let path = dir.join(SCHEMA_FILE);
    let text = std::fs::read_to_string(&path).with_context(|| {
        format!(
            "read schema from {} — did you run `table create` first?",
            path.display()
        )
    })?;
    Ok(text
        .lines()
        .map(str::to_owned)
        .filter(|s| !s.is_empty())
        .collect())
}
