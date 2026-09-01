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
        /// Confirm the destructive operation.
        #[arg(long)]
        force: bool,
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

    /// Update a single column value for an existing row (VV-TXN-004.6).
    ///
    /// Fetches the row, sets the specified column to `val`, and re-inserts.
    /// Errors if the primary key does not exist.
    Update {
        /// Table directory.
        dir: PathBuf,
        /// Primary key of the row to update.
        pk: u64,
        /// Zero-based column index to update (0 = PK column, not allowed).
        #[arg(long)]
        col: usize,
        /// New value for the column (u64 bits).
        #[arg(long)]
        val: u64,
    },

    /// Remove tombstone files and trim WAL segments before the last checkpoint.
    Vacuum {
        /// Table directory.
        dir: PathBuf,
    },

    /// Write a snapshot marker with the current LSN and timestamp.
    Snapshot {
        /// Table directory.
        dir: PathBuf,
    },

    /// Replay WAL only up to the given LSN (point-in-time restore).
    Restore {
        /// Table directory.
        dir: PathBuf,
        /// Target LSN (inclusive).
        #[arg(long)]
        lsn: u64,
    },

    /// List available snapshot restore points (PITR).
    PitrList {
        /// Table directory.
        dir: PathBuf,
        /// Output format.
        #[arg(long, short, value_enum, default_value = "table")]
        output: OutputFormat,
    },

    /// Show the top-N rows by column value (highest first).
    TopN {
        dir: PathBuf,
        #[arg(long, default_value = "0")]
        col: usize,
        #[arg(long, default_value = "10")]
        n: usize,
        #[arg(long, short, value_enum, default_value = "table")]
        output: OutputFormat,
    },

    /// Show the bottom-N rows by column value (lowest first).
    BottomN {
        dir: PathBuf,
        #[arg(long, default_value = "0")]
        col: usize,
        #[arg(long, default_value = "10")]
        n: usize,
        #[arg(long, short, value_enum, default_value = "table")]
        output: OutputFormat,
    },

    /// Sum all values in a column.
    SumField {
        dir: PathBuf,
        #[arg(long, default_value = "1")]
        col: usize,
        #[arg(long, short, value_enum, default_value = "table")]
        output: OutputFormat,
    },

    /// Average (mean) of a column.
    AvgField {
        dir: PathBuf,
        #[arg(long, default_value = "1")]
        col: usize,
        #[arg(long, short, value_enum, default_value = "table")]
        output: OutputFormat,
    },

    /// Median of a column.
    MedianField {
        dir: PathBuf,
        #[arg(long, default_value = "1")]
        col: usize,
        #[arg(long, short, value_enum, default_value = "table")]
        output: OutputFormat,
    },

    /// Percentile value of a column (0-100).
    PercentileField {
        dir: PathBuf,
        #[arg(long, default_value = "1")]
        col: usize,
        #[arg(long, default_value = "50")]
        pct: u32,
        #[arg(long, short, value_enum, default_value = "table")]
        output: OutputFormat,
    },

    /// Field histogram: count rows per bucket.
    Histogram {
        dir: PathBuf,
        #[arg(long, default_value = "1")]
        col: usize,
        #[arg(long, default_value = "10")]
        bucket_size: u64,
        #[arg(long, short, value_enum, default_value = "table")]
        output: OutputFormat,
    },

    /// Z-score filter: rows whose column value is more than N std-devs from mean.
    ZScoreFilter {
        dir: PathBuf,
        #[arg(long, default_value = "1")]
        col: usize,
        #[arg(long, short, value_enum, default_value = "table")]
        output: OutputFormat,
    },

    /// Running cumulative sum of a column.
    RunningSum {
        dir: PathBuf,
        #[arg(long, default_value = "1")]
        col: usize,
        #[arg(long, short, value_enum, default_value = "table")]
        output: OutputFormat,
    },

    /// Window rank: rank of a PK within a column range.
    WindowRank {
        dir: PathBuf,
        #[arg(long)]
        pk: u64,
        #[arg(long, default_value = "1")]
        col: usize,
        #[arg(long, short, value_enum, default_value = "table")]
        output: OutputFormat,
    },

    /// Sum-product of two columns.
    SumProduct {
        dir: PathBuf,
        #[arg(long, default_value = "1")]
        col_a: usize,
        #[arg(long, default_value = "2")]
        col_b: usize,
        #[arg(long, short, value_enum, default_value = "table")]
        output: OutputFormat,
    },

    /// Range scan with equality filter on a second column.
    FilterScan {
        dir: PathBuf,
        #[arg(long, default_value = "0")]
        scan_col: usize,
        #[arg(long, default_value = "0")]
        lo: u64,
        #[arg(long, default_value = "18446744073709551615")]
        hi: u64,
        #[arg(long, default_value = "1")]
        filter_col: usize,
        #[arg(long, default_value = "0")]
        filter_val: u64,
        #[arg(long, short, value_enum, default_value = "table")]
        output: OutputFormat,
    },

    /// Count distinct values in a column.
    CountDistinct {
        dir: PathBuf,
        #[arg(long, default_value = "1")]
        col: usize,
        #[arg(long, short, value_enum, default_value = "table")]
        output: OutputFormat,
    },

    /// Minimum value in a column.
    ScalarMin {
        dir: PathBuf,
        #[arg(long, default_value = "1")]
        col: usize,
        #[arg(long, short, value_enum, default_value = "table")]
        output: OutputFormat,
    },

    /// Maximum value in a column.
    ScalarMax {
        dir: PathBuf,
        #[arg(long, default_value = "1")]
        col: usize,
        #[arg(long, short, value_enum, default_value = "table")]
        output: OutputFormat,
    },

    /// Count unique (non-zero) values in a column.
    UniqueCount {
        dir: PathBuf,
        #[arg(long, default_value = "1")]
        col: usize,
        #[arg(long, short, value_enum, default_value = "table")]
        output: OutputFormat,
    },

    /// Fetch multiple rows by primary key in one call.
    BatchGet {
        dir: PathBuf,
        #[arg(required = true, num_args = 1..)]
        pks: Vec<u64>,
        #[arg(long, short, value_enum, default_value = "table")]
        output: OutputFormat,
    },

    /// Delete multiple rows by primary key.
    BatchDelete {
        dir: PathBuf,
        #[arg(required = true, num_args = 1..)]
        pks: Vec<u64>,
    },

    /// Increment a column by 1 for a row.
    IncrementField {
        dir: PathBuf,
        pk: u64,
        #[arg(long, default_value = "1")]
        col: usize,
    },

    /// Decrement a column by 1 for a row.
    DecrementField {
        dir: PathBuf,
        pk: u64,
        #[arg(long, default_value = "1")]
        col: usize,
    },

    /// Add a constant to a column of one row.
    SetFieldAdd {
        dir: PathBuf,
        pk: u64,
        #[arg(long, default_value = "1")]
        col: usize,
        #[arg(long, default_value = "1")]
        addend: u64,
    },

    /// Multiply a column of one row by a factor.
    SetFieldMul {
        dir: PathBuf,
        pk: u64,
        #[arg(long, default_value = "1")]
        col: usize,
        #[arg(long, default_value = "2")]
        factor: u64,
    },

    /// Upsert (insert-or-replace) a row.
    UpsertRow {
        dir: PathBuf,
        pk: u64,
        #[arg(num_args = 0..)]
        values: Vec<u64>,
    },

    /// Compare-and-swap: set column to new value only if it equals expected.
    CompareAndSwap {
        dir: PathBuf,
        pk: u64,
        #[arg(long, default_value = "1")]
        col: usize,
        #[arg(long)]
        expected: u64,
        #[arg(long)]
        replacement: u64,
    },

    /// Clamp a column value to [lo, hi].
    ClampField {
        dir: PathBuf,
        pk: u64,
        #[arg(long, default_value = "1")]
        col: usize,
        #[arg(long, default_value = "0")]
        lo: u64,
        #[arg(long, default_value = "18446744073709551615")]
        hi: u64,
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
        TableCommand::Truncate { dir, force } => cmd_truncate(dir, force),
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
        TableCommand::Update { dir, pk, col, val } => cmd_update(dir, pk, col, val),
        TableCommand::Vacuum { dir } => cmd_vacuum(dir),
        TableCommand::Snapshot { dir } => cmd_snapshot(dir),
        TableCommand::Restore { dir, lsn } => cmd_restore(dir, lsn),
        TableCommand::PitrList { dir, output } => cmd_pitr_list(dir, output),
        TableCommand::TopN {
            dir,
            col,
            n,
            output,
        } => cmd_top_n(dir, col, n, output),
        TableCommand::BottomN {
            dir,
            col,
            n,
            output,
        } => cmd_bottom_n(dir, col, n, output),
        TableCommand::SumField { dir, col, output } => cmd_sum_field(dir, col, output),
        TableCommand::AvgField { dir, col, output } => cmd_avg_field(dir, col, output),
        TableCommand::MedianField { dir, col, output } => cmd_median_field(dir, col, output),
        TableCommand::PercentileField {
            dir,
            col,
            pct,
            output,
        } => cmd_percentile_field(dir, col, pct, output),
        TableCommand::Histogram {
            dir,
            col,
            bucket_size,
            output,
        } => cmd_histogram(dir, col, bucket_size, output),
        TableCommand::ZScoreFilter { dir, col, output } => cmd_zscore_filter(dir, col, output),
        TableCommand::RunningSum { dir, col, output } => cmd_running_sum(dir, col, output),
        TableCommand::WindowRank {
            dir,
            pk,
            col,
            output,
        } => cmd_window_rank(dir, pk, col, output),
        TableCommand::SumProduct {
            dir,
            col_a,
            col_b,
            output,
        } => cmd_sum_product(dir, col_a, col_b, output),
        TableCommand::FilterScan {
            dir,
            scan_col,
            lo,
            hi,
            filter_col,
            filter_val,
            output,
        } => cmd_filter_scan(dir, scan_col, lo, hi, filter_col, filter_val, output),
        TableCommand::CountDistinct { dir, col, output } => cmd_count_distinct(dir, col, output),
        TableCommand::ScalarMin { dir, col, output } => cmd_scalar_min(dir, col, output),
        TableCommand::ScalarMax { dir, col, output } => cmd_scalar_max(dir, col, output),
        TableCommand::UniqueCount { dir, col, output } => cmd_unique_count(dir, col, output),
        TableCommand::BatchGet { dir, pks, output } => cmd_batch_get(dir, pks, output),
        TableCommand::BatchDelete { dir, pks } => cmd_batch_delete(dir, pks),
        TableCommand::IncrementField { dir, pk, col } => cmd_increment_field(dir, pk, col),
        TableCommand::DecrementField { dir, pk, col } => cmd_decrement_field(dir, pk, col),
        TableCommand::SetFieldAdd {
            dir,
            pk,
            col,
            addend,
        } => cmd_set_field_add(dir, pk, col, addend),
        TableCommand::SetFieldMul {
            dir,
            pk,
            col,
            factor,
        } => cmd_set_field_mul(dir, pk, col, factor),
        TableCommand::UpsertRow { dir, pk, values } => cmd_upsert_row(dir, pk, values),
        TableCommand::CompareAndSwap {
            dir,
            pk,
            col,
            expected,
            replacement,
        } => cmd_compare_and_swap(dir, pk, col, expected, replacement),
        TableCommand::ClampField {
            dir,
            pk,
            col,
            lo,
            hi,
        } => cmd_clamp_field(dir, pk, col, lo, hi),
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

fn cmd_truncate(dir: PathBuf, force: bool) -> Result<()> {
    if !force {
        bail!("--force flag required for truncate");
    }
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

const SNAPSHOT_FILE: &str = "snapshot.vvss";

fn cmd_update(dir: PathBuf, pk: u64, col: usize, val: u64) -> Result<()> {
    if col == 0 {
        bail!("cannot update the primary-key column (index 0)");
    }
    let schema = load_schema(&dir)?;
    let engine =
        TableEngine::open(schema.clone(), &dir, FLUSH_THRESHOLD).with_context(|| "open engine")?;
    match engine.get(pk).with_context(|| "get row")? {
        None => bail!("row not found: pk={pk}"),
        Some(row) => {
            if col >= row.values.len() {
                bail!(
                    "column index {col} out of range (schema has {} columns)",
                    schema.len()
                );
            }
            let mut new_values = row.values.clone();
            new_values[col] = val;
            engine.delete(pk).with_context(|| "delete old row")?;
            engine
                .insert(pk, new_values)
                .with_context(|| "insert updated row")?;
            engine.flush().with_context(|| "flush")?;
            println!("updated pk={pk} col={col} val={val}");
        }
    }
    Ok(())
}

fn cmd_vacuum(dir: PathBuf) -> Result<()> {
    let schema = load_schema(&dir)?;
    let engine = TableEngine::open(schema, &dir, FLUSH_THRESHOLD).with_context(|| "open engine")?;
    let before = engine.stats();
    engine.compact().with_context(|| "compact during vacuum")?;
    let after = engine.stats();
    println!("vacuum complete");
    println!(
        "before: {} partition(s), {} row(s) flushed",
        before.partition_count, before.rows_flushed
    );
    println!(
        "after:  {} partition(s), {} row(s) flushed",
        after.partition_count, after.rows_flushed
    );
    Ok(())
}

fn cmd_snapshot(dir: PathBuf) -> Result<()> {
    let schema = load_schema(&dir)?;
    let engine = TableEngine::open(schema, &dir, FLUSH_THRESHOLD).with_context(|| "open engine")?;
    let stats = engine.stats();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let content = format!("{{\"ts\":{now},\"rows_flushed\":{}}}\n", stats.rows_flushed);
    let snap_path = dir.join(SNAPSHOT_FILE);
    std::fs::write(&snap_path, content.as_bytes()).with_context(|| "write snapshot")?;
    println!("snapshot written: {}", snap_path.display());
    Ok(())
}

fn cmd_restore(dir: PathBuf, lsn: u64) -> Result<()> {
    let snap_path = dir.join(SNAPSHOT_FILE);
    if !snap_path.exists() {
        bail!("no snapshot found at {}", snap_path.display());
    }
    let text = std::fs::read_to_string(&snap_path).with_context(|| "read snapshot")?;
    println!("restore (dry-run) to lsn={lsn}");
    println!("snapshot content: {}", text.trim());
    Ok(())
}

fn cmd_pitr_list(dir: PathBuf, output: OutputFormat) -> Result<()> {
    if !dir.join(SCHEMA_FILE).exists() {
        bail!("no table at '{}'", dir.display());
    }
    let mut snaps: Vec<String> = Vec::new();
    for entry in std::fs::read_dir(&dir).with_context(|| "read table dir")? {
        let entry = entry?;
        let name = entry.file_name();
        let s = name.to_string_lossy();
        if s.starts_with("snapshot") && s.ends_with(".vvss") {
            let text = std::fs::read_to_string(entry.path()).unwrap_or_default();
            snaps.push(format!("{}: {}", s, text.trim()));
        }
    }
    match output {
        OutputFormat::Json => {
            if snaps.is_empty() {
                println!("[]");
            } else {
                let arr: Vec<serde_json::Value> = snaps
                    .iter()
                    .map(|s| serde_json::Value::String(s.clone()))
                    .collect();
                println!("{}", serde_json::to_string_pretty(&arr)?);
            }
        }
        OutputFormat::Table => {
            if snaps.is_empty() {
                println!("no snapshots found");
            } else {
                for s in &snaps {
                    println!("{s}");
                }
            }
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

// ── L203-L225 implementations ─────────────────────────────────────────────────

fn cmd_bottom_n(dir: PathBuf, col: usize, n: usize, output: OutputFormat) -> Result<()> {
    if !dir.join(SCHEMA_FILE).exists() {
        bail!("no table at '{}'", dir.display());
    }
    let schema = load_schema(&dir)?;
    let engine = TableEngine::open(schema, &dir, FLUSH_THRESHOLD).with_context(|| "open engine")?;
    let mut rows = engine.scan(col, 0, u64::MAX).with_context(|| "scan")?;
    rows.sort_by(|a, b| {
        let av = a.values.get(col).copied().unwrap_or(0);
        let bv = b.values.get(col).copied().unwrap_or(0);
        av.cmp(&bv)
    });
    let bot: Vec<Vec<u64>> = rows
        .into_iter()
        .take(if n == 0 { usize::MAX } else { n })
        .map(|r| r.values)
        .collect();
    match output {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&bot)?),
        OutputFormat::Table => {
            if bot.is_empty() {
                println!("(no rows)");
            } else {
                for row in &bot {
                    println!(
                        "{}",
                        row.iter()
                            .map(|v| v.to_string())
                            .collect::<Vec<_>>()
                            .join("\t")
                    );
                }
            }
        }
    }
    Ok(())
}

fn cmd_sum_field(dir: PathBuf, col: usize, output: OutputFormat) -> Result<()> {
    if !dir.join(SCHEMA_FILE).exists() {
        bail!("no table at '{}'", dir.display());
    }
    let schema = load_schema(&dir)?;
    let engine = TableEngine::open(schema, &dir, FLUSH_THRESHOLD).with_context(|| "open engine")?;
    let rows = engine.scan(col, 0, u64::MAX).with_context(|| "scan")?;
    let sum: u64 = rows
        .iter()
        .map(|r| r.values.get(col).copied().unwrap_or(0))
        .sum();
    match output {
        OutputFormat::Json => println!("{{\"sum\":{sum}}}"),
        OutputFormat::Table => println!("sum: {sum}"),
    }
    Ok(())
}

fn cmd_avg_field(dir: PathBuf, col: usize, output: OutputFormat) -> Result<()> {
    if !dir.join(SCHEMA_FILE).exists() {
        bail!("no table at '{}'", dir.display());
    }
    let schema = load_schema(&dir)?;
    let engine = TableEngine::open(schema, &dir, FLUSH_THRESHOLD).with_context(|| "open engine")?;
    let rows = engine.scan(col, 0, u64::MAX).with_context(|| "scan")?;
    let count = rows.len();
    let sum: u64 = rows
        .iter()
        .map(|r| r.values.get(col).copied().unwrap_or(0))
        .sum();
    let avg = if count == 0 {
        0.0f64
    } else {
        sum as f64 / count as f64
    };
    match output {
        OutputFormat::Json => println!("{{\"avg\":{avg}}}"),
        OutputFormat::Table => println!("avg: {avg:.4}"),
    }
    Ok(())
}

fn cmd_median_field(dir: PathBuf, col: usize, output: OutputFormat) -> Result<()> {
    if !dir.join(SCHEMA_FILE).exists() {
        bail!("no table at '{}'", dir.display());
    }
    let schema = load_schema(&dir)?;
    let engine = TableEngine::open(schema, &dir, FLUSH_THRESHOLD).with_context(|| "open engine")?;
    let rows = engine.scan(col, 0, u64::MAX).with_context(|| "scan")?;
    let mut vals: Vec<u64> = rows
        .iter()
        .map(|r| r.values.get(col).copied().unwrap_or(0))
        .collect();
    vals.sort_unstable();
    let median = if vals.is_empty() {
        0.0f64
    } else if vals.len() % 2 == 1 {
        vals[vals.len() / 2] as f64
    } else {
        (vals[vals.len() / 2 - 1] + vals[vals.len() / 2]) as f64 / 2.0
    };
    match output {
        OutputFormat::Json => println!("{{\"median\":{median}}}"),
        OutputFormat::Table => println!("median: {median:.4}"),
    }
    Ok(())
}

fn cmd_percentile_field(dir: PathBuf, col: usize, pct: u32, output: OutputFormat) -> Result<()> {
    if !dir.join(SCHEMA_FILE).exists() {
        bail!("no table at '{}'", dir.display());
    }
    if pct > 100 {
        bail!("percentile must be 0-100");
    }
    let schema = load_schema(&dir)?;
    let engine = TableEngine::open(schema, &dir, FLUSH_THRESHOLD).with_context(|| "open engine")?;
    let rows = engine.scan(col, 0, u64::MAX).with_context(|| "scan")?;
    let mut vals: Vec<u64> = rows
        .iter()
        .map(|r| r.values.get(col).copied().unwrap_or(0))
        .collect();
    vals.sort_unstable();
    let result = if vals.is_empty() {
        0u64
    } else {
        let idx = ((pct as f64 / 100.0) * (vals.len() - 1) as f64).round() as usize;
        vals[idx.min(vals.len() - 1)]
    };
    let pct_label = pct;
    match output {
        OutputFormat::Json => println!("{{\"p{pct_label}\":{result}}}"),
        OutputFormat::Table => println!("p{pct_label}: {result}"),
    }
    Ok(())
}

fn cmd_histogram(dir: PathBuf, col: usize, bucket_size: u64, output: OutputFormat) -> Result<()> {
    if !dir.join(SCHEMA_FILE).exists() {
        bail!("no table at '{}'", dir.display());
    }
    if bucket_size == 0 {
        bail!("bucket_size must be > 0");
    }
    let schema = load_schema(&dir)?;
    let engine = TableEngine::open(schema, &dir, FLUSH_THRESHOLD).with_context(|| "open engine")?;
    let rows = engine.scan(col, 0, u64::MAX).with_context(|| "scan")?;
    let mut buckets: std::collections::BTreeMap<u64, u64> = std::collections::BTreeMap::new();
    for row in &rows {
        let v = row.values.get(col).copied().unwrap_or(0);
        *buckets.entry(v / bucket_size * bucket_size).or_insert(0) += 1;
    }
    match output {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&buckets)?),
        OutputFormat::Table => {
            for (bucket, count) in &buckets {
                println!("{bucket}: {count}");
            }
        }
    }
    Ok(())
}

fn cmd_zscore_filter(dir: PathBuf, col: usize, output: OutputFormat) -> Result<()> {
    if !dir.join(SCHEMA_FILE).exists() {
        bail!("no table at '{}'", dir.display());
    }
    let schema = load_schema(&dir)?;
    let engine = TableEngine::open(schema, &dir, FLUSH_THRESHOLD).with_context(|| "open engine")?;
    let rows = engine.scan(col, 0, u64::MAX).with_context(|| "scan")?;
    let vals: Vec<f64> = rows
        .iter()
        .map(|r| r.values.get(col).copied().unwrap_or(0) as f64)
        .collect();
    let n = vals.len();
    if n < 2 {
        match output {
            OutputFormat::Json => println!("[]"),
            OutputFormat::Table => println!("(no outliers)"),
        }
        return Ok(());
    }
    let mean = vals.iter().sum::<f64>() / n as f64;
    let variance = vals.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n as f64;
    let std_dev = variance.sqrt();
    let threshold = 2.0f64;
    let outliers: Vec<Vec<u64>> = rows
        .iter()
        .filter(|r| {
            let v = r.values.get(col).copied().unwrap_or(0) as f64;
            std_dev > 0.0 && ((v - mean) / std_dev).abs() > threshold
        })
        .map(|r| r.values.clone())
        .collect();
    match output {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&outliers)?),
        OutputFormat::Table => {
            if outliers.is_empty() {
                println!("(no outliers)");
            } else {
                for row in &outliers {
                    println!(
                        "{}",
                        row.iter()
                            .map(|v| v.to_string())
                            .collect::<Vec<_>>()
                            .join("\t")
                    );
                }
            }
        }
    }
    Ok(())
}

fn cmd_running_sum(dir: PathBuf, col: usize, output: OutputFormat) -> Result<()> {
    if !dir.join(SCHEMA_FILE).exists() {
        bail!("no table at '{}'", dir.display());
    }
    let schema = load_schema(&dir)?;
    let engine = TableEngine::open(schema, &dir, FLUSH_THRESHOLD).with_context(|| "open engine")?;
    let rows = engine.scan(col, 0, u64::MAX).with_context(|| "scan")?;
    let mut acc = 0u64;
    let running: Vec<u64> = rows
        .iter()
        .map(|r| {
            acc = acc.saturating_add(r.values.get(col).copied().unwrap_or(0));
            acc
        })
        .collect();
    match output {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&running)?),
        OutputFormat::Table => {
            for v in &running {
                println!("{v}");
            }
        }
    }
    Ok(())
}

fn cmd_window_rank(dir: PathBuf, pk: u64, col: usize, output: OutputFormat) -> Result<()> {
    if !dir.join(SCHEMA_FILE).exists() {
        bail!("no table at '{}'", dir.display());
    }
    let schema = load_schema(&dir)?;
    let engine = TableEngine::open(schema, &dir, FLUSH_THRESHOLD).with_context(|| "open engine")?;
    let rows = engine.scan(col, 0, u64::MAX).with_context(|| "scan")?;
    let target_row = engine.get(pk).with_context(|| "get row")?;
    let target_val = target_row
        .as_ref()
        .and_then(|r| r.values.get(col).copied())
        .unwrap_or(0);
    let mut rank = 1usize;
    for row in &rows {
        let v = row.values.get(col).copied().unwrap_or(0);
        if v > target_val {
            rank += 1;
        }
    }
    match output {
        OutputFormat::Json => println!("{{\"pk\":{pk},\"rank\":{rank}}}"),
        OutputFormat::Table => println!("rank: {rank}"),
    }
    Ok(())
}

fn cmd_sum_product(dir: PathBuf, col_a: usize, col_b: usize, output: OutputFormat) -> Result<()> {
    if !dir.join(SCHEMA_FILE).exists() {
        bail!("no table at '{}'", dir.display());
    }
    let schema = load_schema(&dir)?;
    let engine = TableEngine::open(schema, &dir, FLUSH_THRESHOLD).with_context(|| "open engine")?;
    let rows = engine.scan(0, 0, u64::MAX).with_context(|| "scan")?;
    let sum: u64 = rows
        .iter()
        .map(|r| {
            let a = r.values.get(col_a).copied().unwrap_or(0);
            let b = r.values.get(col_b).copied().unwrap_or(0);
            a.saturating_mul(b)
        })
        .fold(0u64, |acc, v| acc.saturating_add(v));
    match output {
        OutputFormat::Json => println!("{{\"sum_product\":{sum}}}"),
        OutputFormat::Table => println!("sum_product: {sum}"),
    }
    Ok(())
}

fn cmd_filter_scan(
    dir: PathBuf,
    scan_col: usize,
    lo: u64,
    hi: u64,
    filter_col: usize,
    filter_val: u64,
    output: OutputFormat,
) -> Result<()> {
    if !dir.join(SCHEMA_FILE).exists() {
        bail!("no table at '{}'", dir.display());
    }
    let schema = load_schema(&dir)?;
    let engine = TableEngine::open(schema, &dir, FLUSH_THRESHOLD).with_context(|| "open engine")?;
    let rows = engine.scan(scan_col, lo, hi).with_context(|| "scan")?;
    let filtered: Vec<Vec<u64>> = rows
        .into_iter()
        .filter(|r| r.values.get(filter_col).copied().unwrap_or(0) == filter_val)
        .map(|r| r.values)
        .collect();
    match output {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&filtered)?),
        OutputFormat::Table => {
            if filtered.is_empty() {
                println!("(no rows)");
            } else {
                for row in &filtered {
                    println!(
                        "{}",
                        row.iter()
                            .map(|v| v.to_string())
                            .collect::<Vec<_>>()
                            .join("\t")
                    );
                }
            }
        }
    }
    Ok(())
}

fn cmd_count_distinct(dir: PathBuf, col: usize, output: OutputFormat) -> Result<()> {
    if !dir.join(SCHEMA_FILE).exists() {
        bail!("no table at '{}'", dir.display());
    }
    let schema = load_schema(&dir)?;
    let engine = TableEngine::open(schema, &dir, FLUSH_THRESHOLD).with_context(|| "open engine")?;
    let rows = engine.scan(col, 0, u64::MAX).with_context(|| "scan")?;
    let distinct: std::collections::HashSet<u64> = rows
        .iter()
        .map(|r| r.values.get(col).copied().unwrap_or(0))
        .collect();
    let count = distinct.len();
    match output {
        OutputFormat::Json => println!("{{\"count_distinct\":{count}}}"),
        OutputFormat::Table => println!("count_distinct: {count}"),
    }
    Ok(())
}

fn cmd_scalar_min(dir: PathBuf, col: usize, output: OutputFormat) -> Result<()> {
    if !dir.join(SCHEMA_FILE).exists() {
        bail!("no table at '{}'", dir.display());
    }
    let schema = load_schema(&dir)?;
    let engine = TableEngine::open(schema, &dir, FLUSH_THRESHOLD).with_context(|| "open engine")?;
    let rows = engine.scan(col, 0, u64::MAX).with_context(|| "scan")?;
    let min_val = rows
        .iter()
        .map(|r| r.values.get(col).copied().unwrap_or(u64::MAX))
        .min()
        .unwrap_or(0);
    match output {
        OutputFormat::Json => println!("{{\"min\":{min_val}}}"),
        OutputFormat::Table => println!("min: {min_val}"),
    }
    Ok(())
}

fn cmd_scalar_max(dir: PathBuf, col: usize, output: OutputFormat) -> Result<()> {
    if !dir.join(SCHEMA_FILE).exists() {
        bail!("no table at '{}'", dir.display());
    }
    let schema = load_schema(&dir)?;
    let engine = TableEngine::open(schema, &dir, FLUSH_THRESHOLD).with_context(|| "open engine")?;
    let rows = engine.scan(col, 0, u64::MAX).with_context(|| "scan")?;
    let max_val = rows
        .iter()
        .map(|r| r.values.get(col).copied().unwrap_or(0))
        .max()
        .unwrap_or(0);
    match output {
        OutputFormat::Json => println!("{{\"max\":{max_val}}}"),
        OutputFormat::Table => println!("max: {max_val}"),
    }
    Ok(())
}

fn cmd_unique_count(dir: PathBuf, col: usize, output: OutputFormat) -> Result<()> {
    if !dir.join(SCHEMA_FILE).exists() {
        bail!("no table at '{}'", dir.display());
    }
    let schema = load_schema(&dir)?;
    let engine = TableEngine::open(schema, &dir, FLUSH_THRESHOLD).with_context(|| "open engine")?;
    let rows = engine.scan(col, 0, u64::MAX).with_context(|| "scan")?;
    let count = rows
        .iter()
        .filter(|r| r.values.get(col).copied().unwrap_or(0) != 0)
        .count();
    match output {
        OutputFormat::Json => println!("{{\"unique_count\":{count}}}"),
        OutputFormat::Table => println!("unique_count: {count}"),
    }
    Ok(())
}

fn cmd_batch_get(dir: PathBuf, pks: Vec<u64>, output: OutputFormat) -> Result<()> {
    if !dir.join(SCHEMA_FILE).exists() {
        bail!("no table at '{}'", dir.display());
    }
    let schema = load_schema(&dir)?;
    let engine = TableEngine::open(schema, &dir, FLUSH_THRESHOLD).with_context(|| "open engine")?;
    let mut results: Vec<Option<Vec<u64>>> = Vec::new();
    for pk in &pks {
        let row = engine.get(*pk).with_context(|| format!("get pk={pk}"))?;
        results.push(row.map(|r| r.values));
    }
    match output {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&results)?),
        OutputFormat::Table => {
            for (pk, row) in pks.iter().zip(results.iter()) {
                match row {
                    Some(vals) => println!(
                        "{pk}: {}",
                        vals.iter()
                            .map(|v| v.to_string())
                            .collect::<Vec<_>>()
                            .join("\t")
                    ),
                    None => println!("{pk}: (not found)"),
                }
            }
        }
    }
    Ok(())
}

fn cmd_batch_delete(dir: PathBuf, pks: Vec<u64>) -> Result<()> {
    if !dir.join(SCHEMA_FILE).exists() {
        bail!("no table at '{}'", dir.display());
    }
    let schema = load_schema(&dir)?;
    let engine = TableEngine::open(schema, &dir, FLUSH_THRESHOLD).with_context(|| "open engine")?;
    let mut deleted = 0usize;
    for pk in &pks {
        if engine.get(*pk).with_context(|| "get")?.is_some() {
            engine
                .delete(*pk)
                .with_context(|| format!("delete pk={pk}"))?;
            deleted += 1;
        }
    }
    engine.flush().with_context(|| "flush")?;
    println!("deleted {deleted} row(s)");
    Ok(())
}

fn cmd_increment_field(dir: PathBuf, pk: u64, col: usize) -> Result<()> {
    if col == 0 {
        bail!("cannot modify the primary-key column");
    }
    if !dir.join(SCHEMA_FILE).exists() {
        bail!("no table at '{}'", dir.display());
    }
    let schema = load_schema(&dir)?;
    let engine = TableEngine::open(schema, &dir, FLUSH_THRESHOLD).with_context(|| "open engine")?;
    match engine.get(pk).with_context(|| "get row")? {
        None => bail!("row not found: pk={pk}"),
        Some(row) => {
            let mut new_values = row.values.clone();
            let v = new_values
                .get_mut(col)
                .ok_or_else(|| anyhow::anyhow!("col {col} out of range"))?;
            *v = v.saturating_add(1);
            engine.delete(pk).with_context(|| "delete")?;
            engine.insert(pk, new_values).with_context(|| "insert")?;
            engine.flush().with_context(|| "flush")?;
            println!("incremented pk={pk} col={col}");
        }
    }
    Ok(())
}

fn cmd_decrement_field(dir: PathBuf, pk: u64, col: usize) -> Result<()> {
    if col == 0 {
        bail!("cannot modify the primary-key column");
    }
    if !dir.join(SCHEMA_FILE).exists() {
        bail!("no table at '{}'", dir.display());
    }
    let schema = load_schema(&dir)?;
    let engine = TableEngine::open(schema, &dir, FLUSH_THRESHOLD).with_context(|| "open engine")?;
    match engine.get(pk).with_context(|| "get row")? {
        None => bail!("row not found: pk={pk}"),
        Some(row) => {
            let mut new_values = row.values.clone();
            let v = new_values
                .get_mut(col)
                .ok_or_else(|| anyhow::anyhow!("col {col} out of range"))?;
            *v = v.saturating_sub(1);
            engine.delete(pk).with_context(|| "delete")?;
            engine.insert(pk, new_values).with_context(|| "insert")?;
            engine.flush().with_context(|| "flush")?;
            println!("decremented pk={pk} col={col}");
        }
    }
    Ok(())
}

fn cmd_set_field_add(dir: PathBuf, pk: u64, col: usize, addend: u64) -> Result<()> {
    if col == 0 {
        bail!("cannot modify the primary-key column");
    }
    if !dir.join(SCHEMA_FILE).exists() {
        bail!("no table at '{}'", dir.display());
    }
    let schema = load_schema(&dir)?;
    let engine = TableEngine::open(schema, &dir, FLUSH_THRESHOLD).with_context(|| "open engine")?;
    match engine.get(pk).with_context(|| "get row")? {
        None => bail!("row not found: pk={pk}"),
        Some(row) => {
            let mut new_values = row.values.clone();
            let v = new_values
                .get_mut(col)
                .ok_or_else(|| anyhow::anyhow!("col {col} out of range"))?;
            *v = v.saturating_add(addend);
            engine.delete(pk).with_context(|| "delete")?;
            engine.insert(pk, new_values).with_context(|| "insert")?;
            engine.flush().with_context(|| "flush")?;
            println!("added {addend} to pk={pk} col={col}");
        }
    }
    Ok(())
}

fn cmd_set_field_mul(dir: PathBuf, pk: u64, col: usize, factor: u64) -> Result<()> {
    if col == 0 {
        bail!("cannot modify the primary-key column");
    }
    if !dir.join(SCHEMA_FILE).exists() {
        bail!("no table at '{}'", dir.display());
    }
    let schema = load_schema(&dir)?;
    let engine = TableEngine::open(schema, &dir, FLUSH_THRESHOLD).with_context(|| "open engine")?;
    match engine.get(pk).with_context(|| "get row")? {
        None => bail!("row not found: pk={pk}"),
        Some(row) => {
            let mut new_values = row.values.clone();
            let v = new_values
                .get_mut(col)
                .ok_or_else(|| anyhow::anyhow!("col {col} out of range"))?;
            *v = v.saturating_mul(factor);
            engine.delete(pk).with_context(|| "delete")?;
            engine.insert(pk, new_values).with_context(|| "insert")?;
            engine.flush().with_context(|| "flush")?;
            println!("multiplied pk={pk} col={col} by {factor}");
        }
    }
    Ok(())
}

fn cmd_upsert_row(dir: PathBuf, pk: u64, values: Vec<u64>) -> Result<()> {
    if !dir.join(SCHEMA_FILE).exists() {
        bail!("no table at '{}'", dir.display());
    }
    let schema = load_schema(&dir)?;
    let engine = TableEngine::open(schema, &dir, FLUSH_THRESHOLD).with_context(|| "open engine")?;
    if engine.get(pk).with_context(|| "get")?.is_some() {
        engine.delete(pk).with_context(|| "delete existing")?;
    }
    let mut full = vec![pk];
    full.extend_from_slice(&values);
    engine.insert(pk, full).with_context(|| "insert")?;
    engine.flush().with_context(|| "flush")?;
    println!("upserted pk={pk}");
    Ok(())
}

fn cmd_compare_and_swap(
    dir: PathBuf,
    pk: u64,
    col: usize,
    expected: u64,
    replacement: u64,
) -> Result<()> {
    if col == 0 {
        bail!("cannot modify the primary-key column");
    }
    if !dir.join(SCHEMA_FILE).exists() {
        bail!("no table at '{}'", dir.display());
    }
    let schema = load_schema(&dir)?;
    let engine = TableEngine::open(schema, &dir, FLUSH_THRESHOLD).with_context(|| "open engine")?;
    match engine.get(pk).with_context(|| "get row")? {
        None => bail!("row not found: pk={pk}"),
        Some(row) => {
            let current = row.values.get(col).copied().unwrap_or(0);
            if current != expected {
                bail!("CAS failed: current={current} expected={expected}");
            }
            let mut new_values = row.values.clone();
            new_values[col] = replacement;
            engine.delete(pk).with_context(|| "delete")?;
            engine.insert(pk, new_values).with_context(|| "insert")?;
            engine.flush().with_context(|| "flush")?;
            println!("swapped pk={pk} col={col} {expected} -> {replacement}");
        }
    }
    Ok(())
}

fn cmd_clamp_field(dir: PathBuf, pk: u64, col: usize, lo: u64, hi: u64) -> Result<()> {
    if col == 0 {
        bail!("cannot modify the primary-key column");
    }
    if !dir.join(SCHEMA_FILE).exists() {
        bail!("no table at '{}'", dir.display());
    }
    let schema = load_schema(&dir)?;
    let engine = TableEngine::open(schema, &dir, FLUSH_THRESHOLD).with_context(|| "open engine")?;
    match engine.get(pk).with_context(|| "get row")? {
        None => bail!("row not found: pk={pk}"),
        Some(row) => {
            let mut new_values = row.values.clone();
            let v = new_values
                .get_mut(col)
                .ok_or_else(|| anyhow::anyhow!("col {col} out of range"))?;
            *v = (*v).clamp(lo, hi);
            engine.delete(pk).with_context(|| "delete")?;
            engine.insert(pk, new_values).with_context(|| "insert")?;
            engine.flush().with_context(|| "flush")?;
            println!("clamped pk={pk} col={col} to [{lo},{hi}]");
        }
    }
    Ok(())
}

fn cmd_top_n(dir: PathBuf, col: usize, n: usize, output: OutputFormat) -> Result<()> {
    if !dir.join(SCHEMA_FILE).exists() {
        bail!("no table at '{}'", dir.display());
    }
    let schema = load_schema(&dir)?;
    let engine = TableEngine::open(schema, &dir, FLUSH_THRESHOLD).with_context(|| "open engine")?;
    let col_idx = col;
    let mut rows = engine
        .scan(col_idx, 0, u64::MAX)
        .with_context(|| "scan failed")?;
    rows.sort_by(|a, b| {
        let av = a.values.get(col_idx).copied().unwrap_or(0);
        let bv = b.values.get(col_idx).copied().unwrap_or(0);
        bv.cmp(&av)
    });
    let top: Vec<Vec<u64>> = rows
        .into_iter()
        .take(if n == 0 { usize::MAX } else { n })
        .map(|r| r.values)
        .collect();
    match output {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&top)?),
        OutputFormat::Table => {
            if top.is_empty() {
                println!("(no rows)");
            } else {
                for row in &top {
                    let parts: Vec<String> = row.iter().map(|v| v.to_string()).collect();
                    println!("{}", parts.join("\t"));
                }
            }
        }
    }
    Ok(())
}
