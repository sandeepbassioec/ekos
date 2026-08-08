//! `vvctl connect` — interactive / scripted client for a running VectorVolt server.
//!
//! ## Subcommands
//!
//! ```text
//! vvctl connect ping                        # liveness check
//! vvctl connect get <pk>
//! vvctl connect insert <pk> [val...]
//! vvctl connect delete <pk>
//! vvctl connect scan --col 1 --lo 100 --hi 200 [--limit 50]
//! vvctl connect stats
//! vvctl connect flush
//! vvctl connect compact
//! ```
//!
//! ## Authentication
//!
//! Pass `--secret <hex>` to authenticate. The key-id defaults to `"default"` but
//! can be overridden with `--key-id`.
//!
//! ## TLS
//!
//! Pass `--ca <pem>` to trust a specific CA (e.g. a self-signed cert from `vvctl serve`).
//! Without `--ca`, the system trust store is used.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use clap::{Args, Subcommand};
use vectorvolt_net::Client;
use vectorvolt_net::tls::ClientTlsConfig;

// ── Top-level args shared by every subcommand ─────────────────────────────────

#[derive(Args)]
pub struct ConnectArgs {
    /// Server address.
    #[arg(long, short, default_value = "127.0.0.1:5995")]
    pub addr: SocketAddr,

    /// 32-byte shared secret (hex) for authentication.
    #[arg(long)]
    pub secret: Option<String>,

    /// Key ID to send with `--secret`.
    #[arg(long, default_value = "default")]
    pub key_id: String,

    /// PEM CA certificate to trust (for TLS with self-signed certs).
    #[arg(long)]
    pub ca: Option<PathBuf>,

    /// TLS server name (SNI). Defaults to the hostname in `--addr`.
    #[arg(long)]
    pub tls_name: Option<String>,

    /// Connection + call timeout in milliseconds.
    #[arg(long, default_value = "5000")]
    pub timeout_ms: u64,

    #[command(subcommand)]
    pub command: ConnectCommand,
}

#[derive(Subcommand)]
pub enum ConnectCommand {
    /// Check if the server is alive (no auth required).
    Ping,

    /// Retrieve a row by primary key.
    Get {
        /// Primary key.
        pk: u64,
    },

    /// Insert a row.
    Insert {
        /// Primary key.
        pk: u64,
        /// Column values in schema order (excluding pk).
        #[arg(num_args = 0..)]
        values: Vec<u64>,
    },

    /// Delete a row by primary key.
    Delete {
        /// Primary key.
        pk: u64,
    },

    /// Range scan on a column (both bounds inclusive).
    Scan {
        /// Column index (0 = pk).
        #[arg(long, default_value = "0")]
        col: u32,
        /// Minimum value (inclusive).
        #[arg(long, default_value = "0")]
        lo: u64,
        /// Maximum value (inclusive).
        #[arg(long, default_value = "18446744073709551615")]
        hi: u64,
        /// Maximum rows to return (0 = unlimited).
        #[arg(long, default_value = "0")]
        limit: u32,
    },

    /// Show server-side table statistics.
    Stats,

    /// Flush the write buffer to a new partition file.
    Flush,

    /// Compact all partitions into one, removing deleted rows.
    Compact,

    /// Secondary-index lookup: all rows where column `col_idx` equals `value`.
    SecondaryLookup {
        /// Column index to search (1 = first non-pk column).
        #[arg(long, default_value = "1")]
        col: u32,
        /// Exact column value to look up.
        value: u64,
    },

    /// Probabilistic FK existence check for a PK value.
    FkExists {
        /// PK value to check.
        value: u64,
    },

    /// Select the active table on this connection.
    UseTable {
        /// Table name.
        name: String,
    },

    /// Create a new table and select it.
    CreateTable {
        /// Table name.
        name: String,
        /// Column names in schema order (first = primary key).
        #[arg(long, num_args = 1..)]
        columns: Vec<String>,
    },

    /// Drop a named table.
    DropTable {
        /// Table name.
        name: String,
    },

    /// List all tables on the server.
    ListTables,

    /// Insert multiple rows in a single round-trip.
    ///
    /// Rows are specified as `pk:v1,v2,...` pairs separated by spaces.
    /// Example: `batch-insert 1:10,20 2:30,40`
    BatchInsert {
        /// Rows in `pk:v1,v2,...` format.
        #[arg(required = true)]
        rows: Vec<String>,
    },

    /// Delete multiple rows by primary key in a single round-trip.
    BatchDelete {
        /// Primary keys to delete.
        #[arg(required = true)]
        pks: Vec<u64>,
    },

    /// Begin a server-side transaction. Prints the txn-id to use with commit-txn/rollback-txn.
    BeginTxn,

    /// Commit the active transaction.
    CommitTxn {
        /// Transaction ID returned by begin-txn.
        txn_id: u64,
    },

    /// Roll back the active transaction, discarding all buffered ops.
    RollbackTxn {
        /// Transaction ID returned by begin-txn.
        txn_id: u64,
    },

    /// Insert or replace a row (delete existing row if present, then insert).
    Upsert {
        /// Primary key.
        pk: u64,
        /// Column values in schema order (excluding pk).
        #[arg(num_args = 0..)]
        values: Vec<u64>,
    },

    /// Fetch multiple rows by primary key in one round trip.
    ///
    /// Missing keys are silently omitted from the output.
    GetMany {
        /// Primary keys to fetch.
        #[arg(required = true)]
        pks: Vec<u64>,
    },

    /// Update an existing row. Errors if the primary key is not found.
    Update {
        /// Primary key.
        pk: u64,
        /// New column values in schema order (excluding pk).
        #[arg(num_args = 0..)]
        values: Vec<u64>,
    },

    /// Delete all rows from the selected table.
    Truncate,

    /// Insert or replace multiple rows in a single round trip.
    ///
    /// Rows are specified as `pk:v1,v2,...` pairs separated by spaces.
    /// Example: `batch-upsert 1:10,20 2:30,40`
    BatchUpsert {
        /// Rows in `pk:v1,v2,...` format.
        #[arg(required = true)]
        rows: Vec<String>,
    },

    /// Sum a column's values over a pk range (saturating at u64::MAX).
    Sum {
        /// Column index (0 = pk).
        #[arg(long, default_value = "0")]
        col: u32,
        /// Lower bound (inclusive).
        #[arg(long, default_value = "0")]
        lo: u64,
        /// Upper bound (inclusive).
        #[arg(long, default_value = "18446744073709551615")]
        hi: u64,
    },

    /// Return the min, max, and count of a column over a pk range.
    MinMax {
        /// Column index (0 = pk).
        #[arg(long, default_value = "0")]
        col: u32,
        /// Lower bound (inclusive).
        #[arg(long, default_value = "0")]
        lo: u64,
        /// Upper bound (inclusive).
        #[arg(long, default_value = "18446744073709551615")]
        hi: u64,
    },

    /// Range scan returning only primary keys. Same filter semantics as `scan`.
    ScanKeys {
        /// Column index to filter on (0 = pk).
        #[arg(long, default_value = "0")]
        col: u32,
        /// Lower bound (inclusive).
        #[arg(long, default_value = "0")]
        lo: u64,
        /// Upper bound (inclusive).
        #[arg(long, default_value = "18446744073709551615")]
        hi: u64,
        /// Maximum rows to return (0 = unlimited).
        #[arg(long, default_value = "0")]
        limit: u32,
    },

    /// Check whether a named table exists in the server catalog.
    TableExists {
        /// Table name to look up.
        name: String,
    },

    /// Print the column names for the currently selected table.
    DescribeTable,

    /// Fetch and atomically delete the row with the given primary key.
    ///
    /// Prints the row values if found, or "(not found)" if absent.
    GetAndDelete {
        /// Primary key of the row to fetch and delete.
        pk: u64,
    },

    /// Delete all rows where column `col` falls in `[lo, hi]`. Prints the deleted count.
    DeleteRange {
        /// Column index to filter on (0 = pk).
        #[arg(long, default_value = "0")]
        col: u32,
        /// Lower bound (inclusive).
        #[arg(long, default_value = "0")]
        lo: u64,
        /// Upper bound (inclusive).
        #[arg(long, default_value = "18446744073709551615")]
        hi: u64,
    },

    /// Insert a row only if the primary key is not already present.
    ///
    /// Prints "inserted" or "already exists".
    InsertIfAbsent {
        /// Primary key for the new row.
        pk: u64,
        /// Column values in schema order (excluding pk).
        #[arg(num_args = 0..)]
        values: Vec<u64>,
    },

    /// Set a single column of an existing row without supplying other column values.
    ///
    /// Prints "updated" or "(not found)".
    SetField {
        /// Primary key of the row.
        pk: u64,
        /// Column index to set (0 = pk).
        #[arg(long, default_value = "0")]
        col: u32,
        /// New value for the column.
        value: u64,
    },

    /// Conditional update: set a column to `replacement` iff it equals `expected`.
    ///
    /// Prints "swapped", "no-match", or "(not found)".
    CompareAndSwap {
        /// Primary key of the row.
        pk: u64,
        /// Column index to compare and swap (0 = pk).
        #[arg(long, default_value = "0")]
        col: u32,
        /// Value to compare against.
        expected: u64,
        /// Value to write if the comparison succeeds.
        replacement: u64,
    },

    /// Atomically add `delta` to a column value. Prints the new value or "(not found)".
    Increment {
        /// Primary key of the row to update.
        pk: u64,
        /// Column index to increment (0 = pk).
        #[arg(long, default_value = "0")]
        col: u32,
        /// Amount to add (saturating at u64::MAX).
        #[arg(long, default_value = "1")]
        delta: u64,
    },

    /// Update multiple existing rows in a single round trip. Rows not found are skipped.
    ///
    /// Rows are specified as `pk:v1,v2,...` pairs separated by spaces.
    /// Example: `batch-update 1:10,20 2:30,40`
    BatchUpdate {
        /// Rows in `pk:v1,v2,...` format.
        #[arg(required = true)]
        rows: Vec<String>,
    },

    /// Count rows where column `col` value falls in `[lo, hi]` (both inclusive).
    Count {
        /// Column index (0 = pk).
        #[arg(long, default_value = "0")]
        col: u32,
        /// Minimum value (inclusive).
        #[arg(long, default_value = "0")]
        lo: u64,
        /// Maximum value (inclusive).
        #[arg(long, default_value = "18446744073709551615")]
        hi: u64,
    },

    /// Subtract delta from a column, clamping at 0 (saturating subtract).
    ///
    /// Prints the new value or "not found".
    DecrementClamp {
        /// Primary key.
        #[arg(long)]
        pk: u64,
        /// Column index.
        #[arg(long)]
        col: u32,
        /// Amount to subtract.
        #[arg(long)]
        delta: u64,
    },

    /// Atomically fetch and delete the row with the minimum column value in [lo, hi].
    ///
    /// Prints the row values or "not found".
    PopMin {
        /// Column index to scan and rank on (0 = pk).
        #[arg(long, default_value = "0")]
        col: u32,
        /// Lower bound (inclusive).
        #[arg(long, default_value = "0")]
        lo: u64,
        /// Upper bound (inclusive).
        #[arg(long, default_value = "18446744073709551615")]
        hi: u64,
    },

    /// Atomically fetch and delete the row with the maximum column value in [lo, hi].
    ///
    /// Prints the row values or "not found".
    PopMax {
        /// Column index to scan and rank on (0 = pk).
        #[arg(long, default_value = "0")]
        col: u32,
        /// Lower bound (inclusive).
        #[arg(long, default_value = "0")]
        lo: u64,
        /// Upper bound (inclusive).
        #[arg(long, default_value = "18446744073709551615")]
        hi: u64,
    },

    /// Set multiple columns of an existing row atomically.
    ///
    /// Pairs are given as `col:value` arguments, e.g. `1:42 2:99`.
    MultiSet {
        /// Primary key of the row to update.
        #[arg(long)]
        pk: u64,
        /// Column-value pairs in `col:value` format (e.g. `1:42`).
        #[arg(required = true)]
        pairs: Vec<String>,
    },
}

// ── Entry point ───────────────────────────────────────────────────────────────

pub fn run(args: ConnectArgs) -> Result<()> {
    let timeout = Duration::from_millis(args.timeout_ms);
    let mut client = open_client(&args, timeout)?;

    match args.command {
        ConnectCommand::Ping => {
            client.ping().with_context(|| "ping")?;
            println!("pong — server is alive at {}", args.addr);
        }

        ConnectCommand::Get { pk } => {
            match client.get(pk).with_context(|| format!("get pk={pk}"))? {
                None => println!("not found: pk={pk}"),
                Some(values) => {
                    println!("pk={pk}");
                    for (i, v) in values.iter().enumerate() {
                        println!("  col[{i}] = {v}");
                    }
                }
            }
        }

        ConnectCommand::Insert { pk, values } => {
            client
                .insert(pk, values)
                .with_context(|| format!("insert pk={pk}"))?;
            println!("inserted pk={pk}");
        }

        ConnectCommand::Delete { pk } => {
            let existed = client
                .delete(pk)
                .with_context(|| format!("delete pk={pk}"))?;
            if existed {
                println!("deleted pk={pk}");
            } else {
                println!("not found (nothing deleted): pk={pk}");
            }
        }

        ConnectCommand::Scan { col, lo, hi, limit } => {
            let rows = client
                .range_scan(col, lo, hi, limit)
                .with_context(|| format!("scan col={col} [{lo},{hi}]"))?;
            if rows.is_empty() {
                println!("no rows matched col[{col}] in [{lo}, {hi}]");
                return Ok(());
            }
            // Print as tab-separated values; column headers use generic names
            // because the client does not know the schema.
            let col_count = rows[0].len();
            let header: Vec<String> = (0..col_count).map(|i| format!("col[{i}]")).collect();
            println!("{}", header.join("\t"));
            println!(
                "{}",
                header
                    .iter()
                    .map(|_| "────────")
                    .collect::<Vec<_>>()
                    .join("\t")
            );
            for row in &rows {
                let line: Vec<String> = row.iter().map(|v| v.to_string()).collect();
                println!("{}", line.join("\t"));
            }
            println!("\n{} row(s)", rows.len());
        }

        ConnectCommand::Stats => {
            let (rows, partitions, bytes) = client.stats().with_context(|| "stats")?;
            println!("rows       : {rows}");
            println!("partitions : {partitions}");
            println!("bytes      : {bytes}");
        }

        ConnectCommand::Flush => {
            client.flush().with_context(|| "flush")?;
            println!("flush ok");
        }

        ConnectCommand::Compact => {
            let kept = client.compact().with_context(|| "compact")?;
            println!("compact ok — {kept} row(s) kept");
        }

        ConnectCommand::SecondaryLookup { col, value } => {
            let rows = client
                .secondary_scan(col, value)
                .with_context(|| format!("secondary-lookup col={col} value={value}"))?;
            if rows.is_empty() {
                println!("no matches for col[{col}] = {value}");
            } else {
                let col_count = rows[0].len();
                let header: Vec<String> = (0..col_count).map(|i| format!("col[{i}]")).collect();
                println!("{}", header.join("\t"));
                for row in &rows {
                    let line: Vec<String> = row.iter().map(|v| v.to_string()).collect();
                    println!("{}", line.join("\t"));
                }
                println!("\n{} row(s)", rows.len());
            }
        }

        ConnectCommand::FkExists { value } => {
            let found = client
                .fk_exists(value)
                .with_context(|| format!("fk-exists value={value}"))?;
            if found {
                println!("exists (pk={value} found — may be false positive)");
            } else {
                println!("not found (pk={value})");
            }
        }

        ConnectCommand::UseTable { name } => {
            client
                .use_table(&name)
                .with_context(|| format!("use-table {name}"))?;
            println!("ok — active table: {name}");
        }

        ConnectCommand::CreateTable { name, columns } => {
            if columns.is_empty() {
                bail!("--columns is required for create-table");
            }
            client
                .create_table(&name, columns)
                .with_context(|| format!("create-table {name}"))?;
            println!("created and selected: {name}");
        }

        ConnectCommand::DropTable { name } => {
            client
                .drop_table(&name)
                .with_context(|| format!("drop-table {name}"))?;
            println!("dropped: {name}");
        }

        ConnectCommand::ListTables => {
            let names = client.list_tables().with_context(|| "list-tables")?;
            if names.is_empty() {
                println!("(no tables)");
            } else {
                for n in &names {
                    println!("{n}");
                }
            }
        }

        ConnectCommand::BatchInsert { rows: raw } => {
            let mut parsed: Vec<(u64, Vec<u64>)> = Vec::with_capacity(raw.len());
            for s in &raw {
                let (pk_str, vals_str) = s.split_once(':').with_context(|| {
                    format!("batch-insert row '{s}' must be in pk:v1,v2,... format")
                })?;
                let pk: u64 = pk_str
                    .parse()
                    .with_context(|| format!("pk '{pk_str}' is not a valid u64"))?;
                let values: Vec<u64> = if vals_str.is_empty() {
                    vec![]
                } else {
                    vals_str
                        .split(',')
                        .map(|v| {
                            v.parse::<u64>()
                                .with_context(|| format!("value '{v}' is not a valid u64"))
                        })
                        .collect::<Result<Vec<_>>>()?
                };
                parsed.push((pk, values));
            }
            let n = parsed.len();
            let (succeeded, skipped) = client
                .batch_insert(parsed)
                .with_context(|| "batch-insert")?;
            println!(
                "batch-insert: {n} row(s) submitted — {succeeded} succeeded, {skipped} skipped"
            );
        }

        ConnectCommand::BatchDelete { pks } => {
            let n = pks.len();
            let (succeeded, skipped) = client.batch_delete(pks).with_context(|| "batch-delete")?;
            println!(
                "batch-delete: {n} pk(s) submitted — {succeeded} deleted, {skipped} not found"
            );
        }

        ConnectCommand::BeginTxn => {
            let txn_id = client.begin_txn().with_context(|| "begin-txn")?;
            println!("txn started: txn-id={txn_id}");
        }

        ConnectCommand::CommitTxn { txn_id } => {
            let affected = client
                .commit_txn(txn_id)
                .with_context(|| format!("commit-txn txn-id={txn_id}"))?;
            println!("committed txn-id={txn_id} — {affected} op(s) applied");
        }

        ConnectCommand::RollbackTxn { txn_id } => {
            client
                .rollback_txn(txn_id)
                .with_context(|| format!("rollback-txn txn-id={txn_id}"))?;
            println!("rolled back txn-id={txn_id}");
        }

        ConnectCommand::Upsert { pk, values } => {
            client
                .upsert(pk, values)
                .with_context(|| format!("upsert pk={pk}"))?;
            println!("upserted pk={pk}");
        }

        ConnectCommand::Update { pk, values } => {
            client
                .update(pk, values)
                .with_context(|| format!("update pk={pk}"))?;
            println!("updated pk={pk}");
        }

        ConnectCommand::Truncate => {
            let deleted = client.truncate().with_context(|| "truncate")?;
            println!("truncated — {deleted} row(s) removed");
        }

        ConnectCommand::BatchUpsert { rows: raw } => {
            let mut parsed: Vec<(u64, Vec<u64>)> = Vec::with_capacity(raw.len());
            for s in &raw {
                let (pk_str, vals_str) = s.split_once(':').with_context(|| {
                    format!("batch-upsert row '{s}' must be in pk:v1,v2,... format")
                })?;
                let pk: u64 = pk_str
                    .parse()
                    .with_context(|| format!("pk '{pk_str}' is not a valid u64"))?;
                let values: Vec<u64> = if vals_str.is_empty() {
                    vec![]
                } else {
                    vals_str
                        .split(',')
                        .map(|v| {
                            v.parse::<u64>()
                                .with_context(|| format!("value '{v}' is not a valid u64"))
                        })
                        .collect::<Result<Vec<_>>>()?
                };
                parsed.push((pk, values));
            }
            let n = parsed.len();
            let affected = client
                .batch_upsert(parsed)
                .with_context(|| "batch-upsert")?;
            println!("batch-upsert: {n} row(s) submitted — {affected} applied");
        }

        ConnectCommand::Sum { col, lo, hi } => {
            let total = client
                .sum(col, lo, hi)
                .with_context(|| format!("sum col={col} [{lo},{hi}]"))?;
            println!("{total}");
        }

        ConnectCommand::MinMax { col, lo, hi } => {
            match client
                .min_max(col, lo, hi)
                .with_context(|| format!("min-max col={col} [{lo},{hi}]"))?
            {
                None => println!("(no rows in range)"),
                Some((min, max, count)) => {
                    println!("min={min}  max={max}  count={count}");
                }
            }
        }

        ConnectCommand::ScanKeys { col, lo, hi, limit } => {
            let pks = client
                .scan_keys(col, lo, hi, limit)
                .with_context(|| format!("scan-keys col={col} [{lo},{hi}]"))?;
            if pks.is_empty() {
                println!("(no keys)");
            } else {
                for pk in &pks {
                    println!("{pk}");
                }
                println!("\n{} key(s)", pks.len());
            }
        }

        ConnectCommand::TableExists { name } => {
            let found = client
                .table_exists(&name)
                .with_context(|| format!("table-exists {name}"))?;
            if found {
                println!("exists: {name}");
            } else {
                println!("not found: {name}");
            }
        }

        ConnectCommand::DeleteRange { col, lo, hi } => {
            let count = client
                .delete_range(col, lo, hi)
                .with_context(|| format!("delete-range col={col} [{lo},{hi}]"))?;
            println!("{count} row(s) deleted");
        }

        ConnectCommand::InsertIfAbsent { pk, values } => {
            let inserted = client
                .insert_if_absent(pk, values)
                .with_context(|| format!("insert-if-absent pk={pk}"))?;
            if inserted {
                println!("inserted");
            } else {
                println!("already exists");
            }
        }

        ConnectCommand::SetField { pk, col, value } => {
            let updated = client
                .set_field(pk, col, value)
                .with_context(|| format!("set-field pk={pk} col={col} value={value}"))?;
            if updated {
                println!("updated");
            } else {
                println!("(not found)");
            }
        }

        ConnectCommand::CompareAndSwap {
            pk,
            col,
            expected,
            replacement,
        } => {
            match client
                .compare_and_swap(pk, col, expected, replacement)
                .with_context(|| {
                    format!("compare-and-swap pk={pk} col={col} expected={expected} replacement={replacement}")
                })?
            {
                None => println!("(not found)"),
                Some(true) => println!("swapped"),
                Some(false) => println!("no-match"),
            }
        }

        ConnectCommand::Increment { pk, col, delta } => {
            match client
                .increment(pk, col, delta)
                .with_context(|| format!("increment pk={pk} col={col} delta={delta}"))?
            {
                None => println!("(not found)"),
                Some(new_val) => println!("{new_val}"),
            }
        }

        ConnectCommand::GetAndDelete { pk } => {
            match client
                .get_and_delete(pk)
                .with_context(|| format!("get-and-delete pk={pk}"))?
            {
                None => println!("(not found)"),
                Some(values) => {
                    let s: Vec<String> = values.iter().map(|v| v.to_string()).collect();
                    println!("{}", s.join(", "));
                }
            }
        }

        ConnectCommand::DescribeTable => {
            let columns = client.describe_table().with_context(|| "describe-table")?;
            if columns.is_empty() {
                println!("(no columns)");
            } else {
                for (i, col) in columns.iter().enumerate() {
                    println!("[{i}] {col}");
                }
            }
        }

        ConnectCommand::BatchUpdate { rows: raw } => {
            let mut parsed: Vec<(u64, Vec<u64>)> = Vec::with_capacity(raw.len());
            for s in &raw {
                let (pk_str, vals_str) = s.split_once(':').with_context(|| {
                    format!("batch-update row '{s}' must be in pk:v1,v2,... format")
                })?;
                let pk: u64 = pk_str
                    .parse()
                    .with_context(|| format!("pk '{pk_str}' is not a valid u64"))?;
                let values: Vec<u64> = if vals_str.is_empty() {
                    vec![]
                } else {
                    vals_str
                        .split(',')
                        .map(|v| {
                            v.parse::<u64>()
                                .with_context(|| format!("value '{v}' is not a valid u64"))
                        })
                        .collect::<Result<Vec<_>>>()?
                };
                parsed.push((pk, values));
            }
            let (succeeded, skipped) = client
                .batch_update(parsed)
                .with_context(|| "batch-update")?;
            println!("batch-update: {succeeded} updated, {skipped} skipped (not found)");
        }

        ConnectCommand::Count { col, lo, hi } => {
            let n = client
                .count(col, lo, hi)
                .with_context(|| format!("count col={col} [{lo},{hi}]"))?;
            println!("{n}");
        }

        ConnectCommand::PopMin { col, lo, hi } => {
            match client
                .pop_min(col, lo, hi)
                .with_context(|| format!("pop-min col={col} [{lo},{hi}]"))?
            {
                None => println!("not found"),
                Some(values) => println!("{}", values.iter().map(|v| v.to_string()).collect::<Vec<_>>().join("\t")),
            }
        }

        ConnectCommand::PopMax { col, lo, hi } => {
            match client
                .pop_max(col, lo, hi)
                .with_context(|| format!("pop-max col={col} [{lo},{hi}]"))?
            {
                None => println!("not found"),
                Some(values) => println!("{}", values.iter().map(|v| v.to_string()).collect::<Vec<_>>().join("\t")),
            }
        }

        ConnectCommand::DecrementClamp { pk, col, delta } => {
            match client
                .decrement_clamp(pk, col, delta)
                .with_context(|| format!("decrement-clamp pk={pk} col={col} delta={delta}"))?
            {
                None => println!("not found"),
                Some(v) => println!("{v}"),
            }
        }

        ConnectCommand::MultiSet { pk, pairs } => {
            let mut parsed = Vec::with_capacity(pairs.len());
            for s in &pairs {
                let (col_str, val_str) = s.split_once(':').with_context(|| {
                    format!("pair '{s}' must be col:value")
                })?;
                let col: u32 = col_str.parse().with_context(|| format!("invalid column '{col_str}'"))?;
                let value: u64 = val_str.parse().with_context(|| format!("invalid value '{val_str}'"))?;
                parsed.push((col, value));
            }
            let updated = client
                .multi_set(pk, parsed)
                .with_context(|| format!("multi-set pk={pk}"))?;
            if updated {
                println!("updated");
            } else {
                println!("not found");
            }
        }

        ConnectCommand::GetMany { pks } => {
            let n = pks.len();
            let entries = client.get_many(pks).with_context(|| "get-many")?;
            if entries.is_empty() {
                println!("no rows found (queried {n} key(s))");
            } else {
                let col_count = entries[0].1.len();
                let mut header = vec!["pk".to_string()];
                header.extend((0..col_count).map(|i| format!("col[{i}]")));
                println!("{}", header.join("\t"));
                for (pk, values) in &entries {
                    let mut row = vec![pk.to_string()];
                    row.extend(values.iter().map(|v| v.to_string()));
                    println!("{}", row.join("\t"));
                }
                println!("\n{} row(s) found of {n} requested", entries.len());
            }
        }
    }

    Ok(())
}

// ── Client construction ───────────────────────────────────────────────────────

fn open_client(args: &ConnectArgs, timeout: Duration) -> Result<Client> {
    // Parse optional secret before borrowing args.
    let secret_bytes: Option<([u8; 32], &str)> = if let Some(hex) = &args.secret {
        let bytes = hex::decode(hex.trim())
            .with_context(|| "secret must be hex (e.g. from `openssl rand -hex 32`)")?;
        if bytes.len() != 32 {
            bail!("secret must be exactly 32 bytes (64 hex chars)");
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Some((arr, args.key_id.as_str()))
    } else {
        None
    };

    let client = if let Some(ca_path) = &args.ca {
        // TLS with a custom CA (e.g. self-signed cert from `vvctl serve --cert`).
        let server_name = args
            .tls_name
            .clone()
            .unwrap_or_else(|| args.addr.ip().to_string());
        let ca_pem = std::fs::read(ca_path)
            .with_context(|| format!("read CA cert {}", ca_path.display()))?;
        let tls = ClientTlsConfig::pin_certificate(&ca_pem, &server_name)
            .with_context(|| format!("pin CA cert {}", ca_path.display()))?;

        let mut c = Client::connect_tls(args.addr, &tls)
            .with_context(|| format!("connect TLS to {}", args.addr))?;
        c.set_timeout(Some(timeout))?;

        if let Some((secret, key_id)) = &secret_bytes {
            c.authenticate(key_id, secret)
                .with_context(|| "authenticate")?;
        }
        c
    } else if let Some((secret, key_id)) = &secret_bytes {
        // Plaintext with auth.
        let c = Client::connect_authenticated(args.addr, key_id, secret)
            .with_context(|| format!("connect+auth to {}", args.addr))?;
        c.set_timeout(Some(timeout))?;
        c
    } else {
        // Plaintext, no auth.
        let c = Client::connect_timeout(args.addr, timeout)
            .with_context(|| format!("connect to {}", args.addr))?;
        c.set_timeout(Some(timeout))?;
        c
    };

    Ok(client)
}
