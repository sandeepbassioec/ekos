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
use clap::{Subcommand, ValueEnum};
use vectorvolt_engine::{ColumnDef, ColumnType, TableEngine, TableSchema, TypedValue};

use serde_json;

/// Output format for read commands.
#[derive(Clone, Debug, ValueEnum, Default)]
pub enum OutputFormat {
    #[default]
    Table,
    Json,
}

/// Export format for table export (L151).
#[derive(Clone, Debug, ValueEnum, Default)]
pub enum ExportFormat {
    #[default]
    Json,
    Csv,
}

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
        /// Output format (table or json).
        #[arg(long, value_enum, default_value = "table")]
        output: OutputFormat,
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
        /// Output format (table or json).
        #[arg(long, value_enum, default_value = "table")]
        output: OutputFormat,
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
        /// Output format (table or json).
        #[arg(long, value_enum, default_value = "table")]
        output: OutputFormat,
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

    /// Print the typed column schema for an existing table.
    ///
    /// Shows each column's index, name, and type in declaration order.
    Schema {
        /// Table directory.
        dir: PathBuf,
        /// Output format (table or json).
        #[arg(long, value_enum, default_value = "table")]
        output: OutputFormat,
    },

    /// Add a new column to an existing table schema (DDL ALTER ADD COLUMN).
    ///
    /// The column spec uses `name:type` syntax, e.g. `active:bool`.
    /// Supported types: u64, i64, f64, bool, timestamp, decimal.
    /// The new column is appended; existing rows have an implicit value of 0.
    AlterAddCol {
        /// Table directory.
        dir: PathBuf,
        /// Column spec in `name:type` format (e.g. `status:i64`).
        column: String,
    },

    /// Drop a column from an existing table schema (DDL ALTER DROP COLUMN).
    ///
    /// The primary-key column (index 0) cannot be dropped.
    AlterDropCol {
        /// Table directory.
        dir: PathBuf,
        /// Name of the column to remove.
        column: String,
    },

    /// Insert a row using typed string values (one per non-PK column in schema order).
    ///
    /// Values are parsed according to each column's declared type.
    /// The PK column is always first; subsequent values match schema order.
    /// Examples: `true`/`false` for bool, `3.14` for f64, `-5` for i64, `1.500000` for decimal.
    TypedInsert {
        /// Table directory.
        dir: PathBuf,
        /// Primary key value (string, parsed as the PK column's type).
        pk: String,
        /// Non-PK column values in schema order (typed string representations).
        #[arg(num_args = 0..)]
        values: Vec<String>,
    },

    /// Delete the entire table directory (DDL DROP TABLE).
    ///
    /// Permanently removes all data, WAL, and schema for the table.
    /// Requires `--force` to prevent accidental deletion.
    Drop {
        /// Table directory to delete.
        dir: PathBuf,
        /// Confirm the destructive operation.
        #[arg(long)]
        force: bool,
    },

    /// Delete all row data while preserving the table schema (DDL TRUNCATE TABLE).
    ///
    /// Removes all `.vvsst` partition files and WAL segments; the `schema.txt`
    /// file is left intact so the table can be re-used immediately.
    Truncate {
        /// Table directory to truncate.
        dir: PathBuf,
    },

    /// Print the total row count (buffered + flushed) for a table.
    Count {
        /// Table directory.
        #[arg(short, long)]
        dir: PathBuf,
        /// Output format.
        #[arg(long, short, value_enum, default_value = "table")]
        output: OutputFormat,
    },

    /// Print the first N rows of a table ordered by primary key.
    Head {
        /// Table directory.
        #[arg(short, long)]
        dir: PathBuf,
        /// Number of rows to show.
        #[arg(short, long, default_value = "10")]
        n: usize,
        /// Output format.
        #[arg(long, short, value_enum, default_value = "table")]
        output: OutputFormat,
    },

    /// List all primary keys in a table.
    Keys {
        /// Table directory.
        #[arg(short, long)]
        dir: PathBuf,
        /// Output format.
        #[arg(long, short, value_enum, default_value = "table")]
        output: OutputFormat,
    },

    /// Check whether a table directory exists and has a valid schema.
    ///
    /// Exits 0 if valid, exits 1 if missing or corrupt.
    Exists {
        /// Table directory.
        #[arg(short, long)]
        dir: PathBuf,
    },

    /// Print the last N rows of a table ordered by primary key descending.
    Tail {
        /// Table directory.
        #[arg(short, long)]
        dir: PathBuf,
        /// Number of rows to show.
        #[arg(short, long, default_value = "10")]
        n: usize,
        /// Output format.
        #[arg(long, short, value_enum, default_value = "table")]
        output: OutputFormat,
    },

    /// Show the minimum and maximum primary key values in a table.
    PkRange {
        /// Table directory.
        #[arg(short, long)]
        dir: PathBuf,
        /// Output format.
        #[arg(long, short, value_enum, default_value = "table")]
        output: OutputFormat,
    },

    /// Show min/max/count statistics for a named column.
    ColStats {
        /// Table directory.
        #[arg(short, long)]
        dir: PathBuf,
        /// Column name to analyse.
        #[arg(long)]
        col: String,
        /// Output format.
        #[arg(long, short, value_enum, default_value = "table")]
        output: OutputFormat,
    },

    /// Show disk usage (data + WAL) for a table directory.
    Size {
        /// Table directory.
        #[arg(short, long)]
        dir: PathBuf,
        /// Output format.
        #[arg(long, short, value_enum, default_value = "table")]
        output: OutputFormat,
    },

    /// Combined schema + stats + partition list.
    Info {
        /// Table directory.
        #[arg(short, long)]
        dir: PathBuf,
        /// Output format.
        #[arg(long, short, value_enum, default_value = "table")]
        output: OutputFormat,
    },

    /// Integrity check: verify schema parses and engine opens cleanly.
    Check {
        /// Table directory.
        #[arg(short, long)]
        dir: PathBuf,
    },

    /// Export all rows as JSON array or CSV to stdout.
    Export {
        /// Table directory.
        #[arg(short, long)]
        dir: PathBuf,
        /// Export format (json or csv).
        #[arg(long, short, value_enum, default_value = "json")]
        format: ExportFormat,
    },

    /// Import rows from a JSON array file (upsert semantics).
    Import {
        /// Table directory.
        #[arg(short, long)]
        dir: PathBuf,
        /// JSON file to import (array of objects).
        #[arg(short, long)]
        file: PathBuf,
    },

    /// Rename a column in the schema.
    RenameCol {
        /// Table directory.
        #[arg(short, long)]
        dir: PathBuf,
        /// Current column name.
        #[arg(long)]
        from: String,
        /// New column name.
        #[arg(long)]
        to: String,
    },

    /// Copy all data from src table to dst table.
    Copy {
        /// Source table directory.
        src: PathBuf,
        /// Destination table directory (will be created).
        dst: PathBuf,
    },

    /// Compare two tables by PK — report added/removed/changed rows.
    Diff {
        /// First (base) table directory.
        a: PathBuf,
        /// Second table directory.
        b: PathBuf,
        /// Output format.
        #[arg(long, short, value_enum, default_value = "table")]
        output: OutputFormat,
    },

    /// Merge rows from src into dst (upsert semantics).
    Merge {
        /// Source table directory.
        src: PathBuf,
        /// Destination table directory.
        dst: PathBuf,
    },

    /// Recount rows by scanning partitions directly.
    Recount {
        /// Table directory.
        #[arg(short, long)]
        dir: PathBuf,
        /// Output format.
        #[arg(long, short, value_enum, default_value = "table")]
        output: OutputFormat,
    },
}

// ── Public entry point ────────────────────────────────────────────────────────

pub fn run(cmd: TableCommand) -> Result<()> {
    match cmd {
        TableCommand::Create { dir, columns } => cmd_create(dir, columns),
        TableCommand::Insert { dir, pk, values } => cmd_insert(dir, pk, values),
        TableCommand::Get { dir, pk, output } => cmd_get(dir, pk, output),
        TableCommand::Scan {
            dir,
            col,
            lo,
            hi,
            output,
        } => cmd_scan(dir, col, lo, hi, output),
        TableCommand::Delete { dir, pk } => cmd_delete(dir, pk),
        TableCommand::Stats { dir, output } => cmd_stats(dir, output),
        TableCommand::Flush { dir } => cmd_flush(dir),
        TableCommand::Compact { dir } => cmd_compact(dir),
        TableCommand::Schema { dir, output } => cmd_schema(dir, output),
        TableCommand::AlterAddCol { dir, column } => cmd_alter_add_col(dir, column),
        TableCommand::AlterDropCol { dir, column } => cmd_alter_drop_col(dir, column),
        TableCommand::TypedInsert { dir, pk, values } => cmd_typed_insert(dir, pk, values),
        TableCommand::Drop { dir, force } => cmd_drop(dir, force),
        TableCommand::Truncate { dir } => cmd_truncate(dir),
        TableCommand::Count { dir, output } => cmd_count(&dir, output),
        TableCommand::Head { dir, n, output } => cmd_head(&dir, n, output),
        TableCommand::Keys { dir, output } => cmd_keys(&dir, output),
        TableCommand::Exists { dir } => cmd_exists(&dir),
        TableCommand::Tail { dir, n, output } => cmd_tail(&dir, n, output),
        TableCommand::PkRange { dir, output } => cmd_pk_range(&dir, output),
        TableCommand::ColStats { dir, col, output } => cmd_col_stats(&dir, &col, output),
        TableCommand::Size { dir, output } => cmd_size(&dir, output),
        TableCommand::Info { dir, output } => cmd_info(&dir, output),
        TableCommand::Check { dir } => cmd_check(&dir),
        TableCommand::Export { dir, format } => cmd_export(&dir, format),
        TableCommand::Import { dir, file } => cmd_import(&dir, &file),
        TableCommand::RenameCol { dir, from, to } => cmd_rename_col(&dir, &from, &to),
        TableCommand::Copy { src, dst } => cmd_copy(&src, &dst),
        TableCommand::Diff { a, b, output } => cmd_table_diff(&a, &b, output),
        TableCommand::Merge { src, dst } => cmd_table_merge(&src, &dst),
        TableCommand::Recount { dir, output } => cmd_recount(&dir, output),
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

fn cmd_get(dir: PathBuf, pk: u64, output: OutputFormat) -> Result<()> {
    let schema = load_schema(&dir)?;
    let typed = load_typed_schema(&dir)?;
    let engine =
        TableEngine::open(schema.clone(), &dir, FLUSH_THRESHOLD).with_context(|| "open engine")?;

    match engine.get(pk).with_context(|| "get")? {
        None => match output {
            OutputFormat::Json => println!("null"),
            OutputFormat::Table => println!("not found: pk={pk}"),
        },
        Some(row) => {
            let fields: Vec<(String, String)> = schema
                .iter()
                .zip(row.values.iter())
                .enumerate()
                .map(|(i, (name, raw))| {
                    let col_type = typed.get(i).map(|c| c.col_type).unwrap_or(ColumnType::U64);
                    (
                        name.clone(),
                        TypedValue::from_bits(col_type, *raw).display_str(),
                    )
                })
                .collect();
            match output {
                OutputFormat::Json => {
                    let obj: serde_json::Map<String, serde_json::Value> = fields
                        .into_iter()
                        .map(|(k, v)| (k, serde_json::Value::String(v)))
                        .collect();
                    println!("{}", serde_json::to_string_pretty(&obj)?);
                }
                OutputFormat::Table => {
                    for (name, display) in &fields {
                        println!("{name}={display}");
                    }
                }
            }
        }
    }
    Ok(())
}

fn cmd_scan(dir: PathBuf, col: usize, lo: u64, hi: u64, output: OutputFormat) -> Result<()> {
    let schema = load_schema(&dir)?;
    let typed = load_typed_schema(&dir)?;
    let col_name = schema
        .get(col)
        .cloned()
        .unwrap_or_else(|| format!("col[{col}]"));
    let engine =
        TableEngine::open(schema.clone(), &dir, FLUSH_THRESHOLD).with_context(|| "open engine")?;

    let rows = engine.scan(col, lo, hi).with_context(|| "scan")?;

    match output {
        OutputFormat::Json => {
            let arr: Vec<serde_json::Value> = rows
                .iter()
                .map(|row| {
                    let obj: serde_json::Map<String, serde_json::Value> = schema
                        .iter()
                        .zip(row.values.iter())
                        .enumerate()
                        .map(|(i, (name, raw))| {
                            let col_type =
                                typed.get(i).map(|c| c.col_type).unwrap_or(ColumnType::U64);
                            (
                                name.clone(),
                                serde_json::Value::String(
                                    TypedValue::from_bits(col_type, *raw).display_str(),
                                ),
                            )
                        })
                        .collect();
                    serde_json::Value::Object(obj)
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&arr)?);
        }
        OutputFormat::Table => {
            if rows.is_empty() {
                println!("no rows matched {col_name} in [{lo}, {hi}]");
                return Ok(());
            }
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
                let line: Vec<String> = row
                    .values
                    .iter()
                    .enumerate()
                    .map(|(i, raw)| {
                        let col_type = typed.get(i).map(|c| c.col_type).unwrap_or(ColumnType::U64);
                        TypedValue::from_bits(col_type, *raw).display_str()
                    })
                    .collect();
                println!("{}", line.join("\t"));
            }
            println!("\n{} row(s) matched", rows.len());
        }
    }
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

fn cmd_stats(dir: PathBuf, output: OutputFormat) -> Result<()> {
    let schema = load_schema(&dir)?;
    let engine = TableEngine::open(schema, &dir, FLUSH_THRESHOLD).with_context(|| "open engine")?;

    let s = engine.stats();
    match output {
        OutputFormat::Json => {
            let obj = serde_json::json!({
                "schema": s.schema,
                "rows_buffered": s.rows_buffered,
                "rows_flushed": s.rows_flushed,
                "partition_count": s.partition_count,
            });
            println!("{}", serde_json::to_string_pretty(&obj)?);
        }
        OutputFormat::Table => {
            println!("schema:      {}", s.schema.join(", "));
            println!("buffered:    {} row(s)", s.rows_buffered);
            println!("flushed:     {} row(s)", s.rows_flushed);
            println!("partitions:  {}", s.partition_count);
        }
    }
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

fn cmd_typed_insert(dir: PathBuf, pk: String, values: Vec<String>) -> Result<()> {
    let typed = load_typed_schema(&dir)?;
    let cols = typed.columns();

    if cols.is_empty() {
        bail!("schema has no columns");
    }
    let pk_type = cols[0].col_type;
    let pk_bits = TypedValue::parse(pk_type, &pk)
        .map_err(|e| anyhow::anyhow!("pk: {e}"))?
        .to_bits();

    if values.len() != cols.len() - 1 {
        bail!(
            "schema has {} non-PK column(s) but {} value(s) provided",
            cols.len() - 1,
            values.len()
        );
    }

    let mut row = vec![pk_bits];
    for (i, (val_str, col)) in values.iter().zip(&cols[1..]).enumerate() {
        let bits = TypedValue::parse(col.col_type, val_str)
            .map_err(|e| anyhow::anyhow!("column '{}' (index {}): {e}", col.name, i + 1))?
            .to_bits();
        row.push(bits);
    }

    let schema_names = load_schema(&dir)?;
    let engine =
        TableEngine::open(schema_names, &dir, FLUSH_THRESHOLD).with_context(|| "open engine")?;
    engine.insert(pk_bits, row).with_context(|| "insert")?;
    engine.flush().with_context(|| "flush after insert")?;

    println!("inserted pk={pk}");
    Ok(())
}

fn cmd_schema(dir: PathBuf, output: OutputFormat) -> Result<()> {
    let schema = load_typed_schema(&dir)?;
    match output {
        OutputFormat::Json => {
            let cols: Vec<serde_json::Value> = schema
                .columns()
                .iter()
                .enumerate()
                .map(|(i, col)| {
                    serde_json::json!({
                        "index": i,
                        "name": col.name,
                        "type": col.col_type.as_str(),
                        "is_pk": i == 0,
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&cols)?);
        }
        OutputFormat::Table => {
            println!("schema: {} column(s)", schema.col_count());
            for (i, col) in schema.columns().iter().enumerate() {
                let pk_tag = if i == 0 { " (PK)" } else { "" };
                println!("  [{i}] {}:{}{pk_tag}", col.name, col.col_type.as_str());
            }
        }
    }
    Ok(())
}

fn cmd_alter_add_col(dir: PathBuf, column: String) -> Result<()> {
    let schema = load_typed_schema(&dir)?;
    let new_def = ColumnDef::from_line(&column);
    let updated = schema
        .with_added(std::slice::from_ref(&new_def))
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    write_typed_schema(&dir, &updated)?;
    println!(
        "added column '{}:{}' (index {})",
        new_def.name,
        new_def.col_type.as_str(),
        updated.col_count() - 1
    );
    Ok(())
}

fn cmd_alter_drop_col(dir: PathBuf, column: String) -> Result<()> {
    let schema = load_typed_schema(&dir)?;
    if schema.get(0).is_some_and(|pk| pk.name == column) {
        bail!("cannot drop the primary-key column '{column}'");
    }
    let updated = schema
        .with_dropped(&[column.as_str()])
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    write_typed_schema(&dir, &updated)?;
    println!("dropped column '{column}'");
    Ok(())
}

fn cmd_drop(dir: PathBuf, force: bool) -> Result<()> {
    if !dir.exists() {
        bail!("table directory '{}' does not exist", dir.display());
    }
    if !force {
        bail!("refusing to drop '{}' without --force", dir.display());
    }
    std::fs::remove_dir_all(&dir)
        .with_context(|| format!("remove table directory '{}'", dir.display()))?;
    println!("dropped table at {}", dir.display());
    Ok(())
}

fn cmd_truncate(dir: PathBuf) -> Result<()> {
    if !dir.join(SCHEMA_FILE).exists() {
        bail!(
            "no table found at '{}' — did you run `table create` first?",
            dir.display()
        );
    }
    let data_dir = dir.join("data");
    if data_dir.exists() {
        for entry in std::fs::read_dir(&data_dir)
            .with_context(|| format!("read data dir '{}'", data_dir.display()))?
        {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("vvsst") {
                std::fs::remove_file(&path)
                    .with_context(|| format!("remove '{}'", path.display()))?;
            }
        }
    }
    let wal_dir = dir.join("wal");
    if wal_dir.exists() {
        for entry in std::fs::read_dir(&wal_dir)
            .with_context(|| format!("read wal dir '{}'", wal_dir.display()))?
        {
            let entry = entry?;
            std::fs::remove_file(entry.path())
                .with_context(|| format!("remove WAL file '{}'", entry.path().display()))?;
        }
    }
    println!("truncated table at {} (schema preserved)", dir.display());
    Ok(())
}

fn cmd_count(dir: &Path, output: OutputFormat) -> Result<()> {
    let schema = load_schema(dir)?;
    let engine = TableEngine::open(schema, dir, FLUSH_THRESHOLD).with_context(|| "open engine")?;
    let s = engine.stats();
    let buffered = s.rows_buffered as u64;
    let flushed = s.rows_flushed;
    let total = buffered + flushed;
    match output {
        OutputFormat::Json => {
            println!("{{\"total\":{total},\"buffered\":{buffered},\"flushed\":{flushed}}}");
        }
        OutputFormat::Table => {
            println!("total:    {total}");
            println!("buffered: {buffered}");
            println!("flushed:  {flushed}");
        }
    }
    Ok(())
}

fn cmd_head(dir: &Path, n: usize, output: OutputFormat) -> Result<()> {
    let schema = load_schema(dir)?;
    let typed = load_typed_schema(dir)?;
    let engine =
        TableEngine::open(schema.clone(), dir, FLUSH_THRESHOLD).with_context(|| "open engine")?;
    let all_rows = engine.scan(0, 0, u64::MAX).with_context(|| "scan")?;
    let rows: Vec<_> = all_rows.into_iter().take(n).collect();
    match output {
        OutputFormat::Json => {
            let arr: Vec<serde_json::Value> = rows
                .iter()
                .map(|row| {
                    let obj: serde_json::Map<String, serde_json::Value> = schema
                        .iter()
                        .zip(row.values.iter())
                        .enumerate()
                        .map(|(i, (name, raw))| {
                            let col_type =
                                typed.get(i).map(|c| c.col_type).unwrap_or(ColumnType::U64);
                            (
                                name.clone(),
                                serde_json::Value::String(
                                    TypedValue::from_bits(col_type, *raw).display_str(),
                                ),
                            )
                        })
                        .collect();
                    serde_json::Value::Object(obj)
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&arr)?);
        }
        OutputFormat::Table => {
            if rows.is_empty() {
                println!("no rows");
                return Ok(());
            }
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
                let line: Vec<String> = row
                    .values
                    .iter()
                    .enumerate()
                    .map(|(i, raw)| {
                        let col_type = typed.get(i).map(|c| c.col_type).unwrap_or(ColumnType::U64);
                        TypedValue::from_bits(col_type, *raw).display_str()
                    })
                    .collect();
                println!("{}", line.join("\t"));
            }
            println!("\n{} row(s)", rows.len());
        }
    }
    Ok(())
}

fn cmd_keys(dir: &Path, output: OutputFormat) -> Result<()> {
    let schema = load_schema(dir)?;
    let typed = load_typed_schema(dir)?;
    let engine = TableEngine::open(schema, dir, FLUSH_THRESHOLD).with_context(|| "open engine")?;
    let rows = engine.scan(0, 0, u64::MAX).with_context(|| "scan")?;
    let pk_type = typed.get(0).map(|c| c.col_type).unwrap_or(ColumnType::U64);
    match output {
        OutputFormat::Json => {
            let arr: Vec<serde_json::Value> = rows
                .iter()
                .map(|row| {
                    let pk_raw = row.values.first().copied().unwrap_or(0);
                    serde_json::Value::String(TypedValue::from_bits(pk_type, pk_raw).display_str())
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&arr)?);
        }
        OutputFormat::Table => {
            for row in &rows {
                let pk_raw = row.values.first().copied().unwrap_or(0);
                println!("{}", TypedValue::from_bits(pk_type, pk_raw).display_str());
            }
            println!("\n{} key(s)", rows.len());
        }
    }
    Ok(())
}

fn cmd_exists(dir: &Path) -> Result<()> {
    let schema_path = dir.join("schema.txt");
    if schema_path.exists() {
        println!("exists: {}", dir.display());
        Ok(())
    } else {
        anyhow::bail!("no table at '{}'", dir.display())
    }
}

// ── L142-L161 commands ───────────────────────────────────────────────────────

fn cmd_tail(dir: &Path, n: usize, output: OutputFormat) -> Result<()> {
    let schema = load_schema(dir)?;
    let typed = load_typed_schema(dir)?;
    let engine =
        TableEngine::open(schema.clone(), dir, FLUSH_THRESHOLD).with_context(|| "open engine")?;
    let all_rows = engine.scan(0, 0, u64::MAX).with_context(|| "scan")?;
    let rows: Vec<_> = all_rows.iter().rev().take(n).rev().cloned().collect();
    match output {
        OutputFormat::Json => {
            let arr: Vec<serde_json::Value> = rows
                .iter()
                .map(|row| {
                    let obj: serde_json::Map<String, serde_json::Value> = schema
                        .iter()
                        .zip(row.values.iter())
                        .enumerate()
                        .map(|(i, (name, raw))| {
                            let col_type =
                                typed.get(i).map(|c| c.col_type).unwrap_or(ColumnType::U64);
                            (
                                name.clone(),
                                serde_json::Value::String(
                                    TypedValue::from_bits(col_type, *raw).display_str(),
                                ),
                            )
                        })
                        .collect();
                    serde_json::Value::Object(obj)
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&arr)?);
        }
        OutputFormat::Table => {
            if rows.is_empty() {
                println!("no rows");
                return Ok(());
            }
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
                let line: Vec<String> = row
                    .values
                    .iter()
                    .enumerate()
                    .map(|(i, raw)| {
                        let col_type = typed.get(i).map(|c| c.col_type).unwrap_or(ColumnType::U64);
                        TypedValue::from_bits(col_type, *raw).display_str()
                    })
                    .collect();
                println!("{}", line.join("\t"));
            }
            println!("\n{} row(s)", rows.len());
        }
    }
    Ok(())
}

fn cmd_pk_range(dir: &Path, output: OutputFormat) -> Result<()> {
    let schema = load_schema(dir)?;
    let typed = load_typed_schema(dir)?;
    let engine = TableEngine::open(schema, dir, FLUSH_THRESHOLD).with_context(|| "open engine")?;
    let rows = engine.scan(0, 0, u64::MAX).with_context(|| "scan")?;
    let pk_type = typed.get(0).map(|c| c.col_type).unwrap_or(ColumnType::U64);
    if rows.is_empty() {
        match output {
            OutputFormat::Json => println!("{{\"min\":null,\"max\":null}}"),
            OutputFormat::Table => println!("empty table"),
        }
        return Ok(());
    }
    let min_raw = rows
        .first()
        .and_then(|r| r.values.first())
        .copied()
        .unwrap_or(0);
    let max_raw = rows
        .last()
        .and_then(|r| r.values.first())
        .copied()
        .unwrap_or(0);
    let min_str = TypedValue::from_bits(pk_type, min_raw).display_str();
    let max_str = TypedValue::from_bits(pk_type, max_raw).display_str();
    match output {
        OutputFormat::Json => println!("{{\"min\":\"{min_str}\",\"max\":\"{max_str}\"}}"),
        OutputFormat::Table => {
            println!("min_pk: {min_str}");
            println!("max_pk: {max_str}");
        }
    }
    Ok(())
}

fn cmd_col_stats(dir: &Path, col_name: &str, output: OutputFormat) -> Result<()> {
    let schema = load_schema(dir)?;
    let typed = load_typed_schema(dir)?;
    let col_idx = schema
        .iter()
        .position(|n| n == col_name || n.split(':').next() == Some(col_name))
        .ok_or_else(|| anyhow::anyhow!("column '{col_name}' not found in schema"))?;
    let engine = TableEngine::open(schema, dir, FLUSH_THRESHOLD).with_context(|| "open engine")?;
    let rows = engine.scan(0, 0, u64::MAX).with_context(|| "scan")?;
    let col_type = typed
        .get(col_idx)
        .map(|c| c.col_type)
        .unwrap_or(ColumnType::U64);
    let count = rows.len();
    if count == 0 {
        match output {
            OutputFormat::Json => println!("{{\"count\":0,\"min\":null,\"max\":null}}"),
            OutputFormat::Table => println!("count: 0\nmin: (none)\nmax: (none)"),
        }
        return Ok(());
    }
    let mut min_raw = u64::MAX;
    let mut max_raw = 0u64;
    for row in &rows {
        let v = row.values.get(col_idx).copied().unwrap_or(0);
        if v < min_raw {
            min_raw = v;
        }
        if v > max_raw {
            max_raw = v;
        }
    }
    let min_str = TypedValue::from_bits(col_type, min_raw).display_str();
    let max_str = TypedValue::from_bits(col_type, max_raw).display_str();
    match output {
        OutputFormat::Json => {
            println!("{{\"count\":{count},\"min\":\"{min_str}\",\"max\":\"{max_str}\"}}")
        }
        OutputFormat::Table => {
            println!("count: {count}");
            println!("min:   {min_str}");
            println!("max:   {max_str}");
        }
    }
    Ok(())
}

fn cmd_size(dir: &Path, output: OutputFormat) -> Result<()> {
    if !dir.join(SCHEMA_FILE).exists() {
        anyhow::bail!("no table at '{}'", dir.display());
    }
    let data_dir = dir.join("data");
    let wal_dir = dir.join("wal");
    let data_bytes: u64 = if data_dir.exists() {
        std::fs::read_dir(&data_dir)
            .with_context(|| "read data dir")?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("vvsst"))
            .map(|e| std::fs::metadata(e.path()).map(|m| m.len()).unwrap_or(0))
            .sum()
    } else {
        0
    };
    let wal_bytes: u64 = if wal_dir.exists() {
        std::fs::read_dir(&wal_dir)
            .with_context(|| "read wal dir")?
            .filter_map(|e| e.ok())
            .map(|e| std::fs::metadata(e.path()).map(|m| m.len()).unwrap_or(0))
            .sum()
    } else {
        0
    };
    let total_bytes = data_bytes + wal_bytes;
    match output {
        OutputFormat::Json => println!(
            "{{\"data_bytes\":{data_bytes},\"wal_bytes\":{wal_bytes},\"total_bytes\":{total_bytes}}}"
        ),
        OutputFormat::Table => {
            println!("data_bytes:  {data_bytes}");
            println!("wal_bytes:   {wal_bytes}");
            println!("total_bytes: {total_bytes}");
        }
    }
    Ok(())
}

fn cmd_info(dir: &Path, output: OutputFormat) -> Result<()> {
    let schema = load_schema(dir)?;
    let typed = load_typed_schema(dir)?;
    let engine =
        TableEngine::open(schema.clone(), dir, FLUSH_THRESHOLD).with_context(|| "open engine")?;
    let stats = engine.stats();
    let data_dir = dir.join("data");
    let mut partitions: Vec<(String, u64)> = Vec::new();
    if data_dir.exists() {
        for entry in std::fs::read_dir(&data_dir).with_context(|| "read data dir")? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("vvsst") {
                let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_owned();
                partitions.push((name, size));
            }
        }
        partitions.sort();
    }
    match output {
        OutputFormat::Json => {
            let cols: Vec<serde_json::Value> = typed.columns().iter().enumerate().map(|(i, col)| {
                serde_json::json!({"index": i, "name": col.name, "type": col.col_type.as_str()})
            }).collect();
            let parts: Vec<serde_json::Value> = partitions
                .iter()
                .map(|(n, b)| serde_json::json!({"name": n, "bytes": b}))
                .collect();
            let obj = serde_json::json!({
                "schema": cols,
                "stats": {"rows_buffered": stats.rows_buffered, "rows_flushed": stats.rows_flushed, "partition_count": stats.partition_count},
                "partitions": parts,
            });
            println!("{}", serde_json::to_string_pretty(&obj)?);
        }
        OutputFormat::Table => {
            println!("=== schema ({} columns) ===", typed.col_count());
            for (i, col) in typed.columns().iter().enumerate() {
                let pk = if i == 0 { " (PK)" } else { "" };
                println!("  [{i}] {}:{}{pk}", col.name, col.col_type.as_str());
            }
            println!("=== stats ===");
            println!("  buffered: {} row(s)", stats.rows_buffered);
            println!("  flushed:  {} row(s)", stats.rows_flushed);
            println!("  partitions: {}", stats.partition_count);
            println!("=== partitions ===");
            if partitions.is_empty() {
                println!("  (none)");
            } else {
                for (name, bytes) in &partitions {
                    println!("  {name}  {bytes} bytes");
                }
            }
        }
    }
    Ok(())
}

fn cmd_check(dir: &Path) -> Result<()> {
    let schema_path = dir.join("schema.txt");
    if !schema_path.exists() {
        anyhow::bail!("no schema.txt at '{}'", dir.display());
    }
    let schema = load_schema(dir)?;
    let _engine = TableEngine::open(schema, dir, FLUSH_THRESHOLD).with_context(|| "open engine")?;
    println!("ok: {}", dir.display());
    Ok(())
}

fn cmd_export(dir: &Path, format: ExportFormat) -> Result<()> {
    let schema = load_schema(dir)?;
    let typed = load_typed_schema(dir)?;
    let engine =
        TableEngine::open(schema.clone(), dir, FLUSH_THRESHOLD).with_context(|| "open engine")?;
    let rows = engine.scan(0, 0, u64::MAX).with_context(|| "scan")?;
    match format {
        ExportFormat::Json => {
            let arr: Vec<serde_json::Value> = rows
                .iter()
                .map(|row| {
                    let obj: serde_json::Map<String, serde_json::Value> = schema
                        .iter()
                        .zip(row.values.iter())
                        .enumerate()
                        .map(|(i, (name, raw))| {
                            let col_type =
                                typed.get(i).map(|c| c.col_type).unwrap_or(ColumnType::U64);
                            (
                                name.clone(),
                                serde_json::Value::String(
                                    TypedValue::from_bits(col_type, *raw).display_str(),
                                ),
                            )
                        })
                        .collect();
                    serde_json::Value::Object(obj)
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&arr)?);
        }
        ExportFormat::Csv => {
            println!("{}", schema.join(","));
            for row in &rows {
                let line: Vec<String> = row
                    .values
                    .iter()
                    .enumerate()
                    .map(|(i, raw)| {
                        let col_type = typed.get(i).map(|c| c.col_type).unwrap_or(ColumnType::U64);
                        TypedValue::from_bits(col_type, *raw).display_str()
                    })
                    .collect();
                println!("{}", line.join(","));
            }
        }
    }
    Ok(())
}

fn cmd_import(dir: &Path, file: &Path) -> Result<()> {
    let text =
        std::fs::read_to_string(file).with_context(|| format!("read '{}'", file.display()))?;
    let arr: Vec<serde_json::Map<String, serde_json::Value>> =
        serde_json::from_str(&text).with_context(|| "parse JSON array")?;
    let typed = load_typed_schema(dir)?;
    let cols = typed.columns();
    if cols.is_empty() {
        bail!("schema has no columns");
    }
    let schema = load_schema(dir)?;
    let engine = TableEngine::open(schema, dir, FLUSH_THRESHOLD).with_context(|| "open engine")?;
    let mut count = 0usize;
    for obj in &arr {
        let pk_name = &cols[0].name;
        let pk_str = obj
            .get(pk_name)
            .map(|v| match v {
                serde_json::Value::String(s) => s.clone(),
                other => other.to_string(),
            })
            .unwrap_or_default();
        let pk_bits = TypedValue::parse(cols[0].col_type, &pk_str)
            .map_err(|e| anyhow::anyhow!("pk: {e}"))?
            .to_bits();
        let mut row = vec![pk_bits];
        for col in &cols[1..] {
            let val_str = obj
                .get(&col.name)
                .map(|v| match v {
                    serde_json::Value::String(s) => s.clone(),
                    other => other.to_string(),
                })
                .unwrap_or_default();
            let bits = TypedValue::parse(col.col_type, &val_str)
                .map_err(|e| anyhow::anyhow!("column '{}': {e}", col.name))?
                .to_bits();
            row.push(bits);
        }
        engine.insert(pk_bits, row).with_context(|| "insert")?;
        count += 1;
    }
    engine.flush().with_context(|| "flush")?;
    println!("imported {count} row(s)");
    Ok(())
}

fn cmd_rename_col(dir: &Path, from: &str, to: &str) -> Result<()> {
    let schema = load_typed_schema(dir)?;
    let idx = schema
        .columns()
        .iter()
        .position(|c| c.name == from)
        .ok_or_else(|| anyhow::anyhow!("column '{from}' not found"))?;
    if idx == 0 {
        bail!("cannot rename the primary-key column");
    }
    let new_col_type = schema.columns()[idx].col_type;
    let new_defs: Vec<_> = schema
        .columns()
        .iter()
        .enumerate()
        .map(|(i, col)| {
            if i == idx {
                vectorvolt_engine::ColumnDef::from_line(&format!(
                    "{}:{}",
                    to,
                    new_col_type.as_str()
                ))
            } else {
                col.clone()
            }
        })
        .collect();
    let new_schema = vectorvolt_engine::TableSchema::new(new_defs.clone())
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    write_typed_schema(dir, &new_schema)?;
    println!("renamed column '{from}' -> '{to}'");
    Ok(())
}

fn cmd_copy(src: &Path, dst: &Path) -> Result<()> {
    let schema = load_schema(src)?;
    std::fs::create_dir_all(dst).with_context(|| format!("create dst dir '{}'", dst.display()))?;
    let schema_src = src.join("schema.txt");
    let schema_dst = dst.join("schema.txt");
    std::fs::copy(&schema_src, &schema_dst).with_context(|| "copy schema.txt")?;
    let engine_src = TableEngine::open(schema.clone(), src, FLUSH_THRESHOLD)
        .with_context(|| "open src engine")?;
    let engine_dst =
        TableEngine::open(schema, dst, FLUSH_THRESHOLD).with_context(|| "open dst engine")?;
    let rows = engine_src
        .scan(0, 0, u64::MAX)
        .with_context(|| "scan src")?;
    let count = rows.len();
    for row in rows {
        let pk = row.values.first().copied().unwrap_or(0);
        engine_dst
            .insert(pk, row.values)
            .with_context(|| "insert")?;
    }
    engine_dst.flush().with_context(|| "flush dst")?;
    println!("{count} rows copied");
    Ok(())
}

fn cmd_table_diff(a: &Path, b: &Path, output: OutputFormat) -> Result<()> {
    let schema_a = load_schema(a)?;
    let schema_b = load_schema(b)?;
    let engine_a =
        TableEngine::open(schema_a, a, FLUSH_THRESHOLD).with_context(|| "open table a")?;
    let engine_b =
        TableEngine::open(schema_b, b, FLUSH_THRESHOLD).with_context(|| "open table b")?;
    let rows_a = engine_a.scan(0, 0, u64::MAX).with_context(|| "scan a")?;
    let rows_b = engine_b.scan(0, 0, u64::MAX).with_context(|| "scan b")?;
    use std::collections::HashMap;
    let map_a: HashMap<u64, Vec<u64>> = rows_a
        .into_iter()
        .map(|r| (r.values[0], r.values))
        .collect();
    let map_b: HashMap<u64, Vec<u64>> = rows_b
        .into_iter()
        .map(|r| (r.values[0], r.values))
        .collect();
    let mut added = 0usize;
    let mut removed = 0usize;
    let mut changed = 0usize;
    for (pk, vals_b) in &map_b {
        match map_a.get(pk) {
            None => added += 1,
            Some(vals_a) => {
                if vals_a != vals_b {
                    changed += 1;
                }
            }
        }
    }
    for pk in map_a.keys() {
        if !map_b.contains_key(pk) {
            removed += 1;
        }
    }
    match output {
        OutputFormat::Json => {
            println!("{{\"added\":{added},\"removed\":{removed},\"changed\":{changed}}}")
        }
        OutputFormat::Table => {
            println!("added:   {added}");
            println!("removed: {removed}");
            println!("changed: {changed}");
        }
    }
    Ok(())
}

fn cmd_table_merge(src: &Path, dst: &Path) -> Result<()> {
    let schema_src = load_schema(src)?;
    let schema_dst = load_schema(dst)?;
    let engine_src =
        TableEngine::open(schema_src, src, FLUSH_THRESHOLD).with_context(|| "open src")?;
    let engine_dst =
        TableEngine::open(schema_dst, dst, FLUSH_THRESHOLD).with_context(|| "open dst")?;
    let rows = engine_src
        .scan(0, 0, u64::MAX)
        .with_context(|| "scan src")?;
    let count = rows.len();
    for row in rows {
        let pk = row.values.first().copied().unwrap_or(0);
        engine_dst
            .insert(pk, row.values)
            .with_context(|| "insert")?;
    }
    engine_dst.flush().with_context(|| "flush dst")?;
    println!("{count} rows merged");
    Ok(())
}

fn cmd_recount(dir: &Path, output: OutputFormat) -> Result<()> {
    let schema = load_schema(dir)?;
    let engine = TableEngine::open(schema, dir, FLUSH_THRESHOLD).with_context(|| "open engine")?;
    let stats = engine.stats();
    let rows = engine.scan(0, 0, u64::MAX).with_context(|| "scan")?;
    let scan_count = rows.len() as u64;
    let stats_count = stats.rows_flushed;
    let matched = scan_count == stats_count;
    match output {
        OutputFormat::Json => println!(
            "{{\"scan_count\":{scan_count},\"stats_count\":{stats_count},\"match\":{matched}}}"
        ),
        OutputFormat::Table => {
            println!("scan_count:  {scan_count}");
            println!("stats_count: {stats_count}");
            println!("match:       {matched}");
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

fn load_typed_schema(dir: &Path) -> Result<TableSchema> {
    let path = dir.join(SCHEMA_FILE);
    let text = std::fs::read_to_string(&path).with_context(|| {
        format!(
            "read schema from {} — did you run `table create` first?",
            path.display()
        )
    })?;
    TableSchema::from_text(&text).map_err(|e| anyhow::anyhow!("{e}"))
}

fn write_typed_schema(dir: &Path, schema: &TableSchema) -> Result<()> {
    let path = dir.join(SCHEMA_FILE);
    std::fs::write(&path, schema.to_text()).with_context(|| "write schema.txt")
}
