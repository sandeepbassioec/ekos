//! `vvctl catalog` — in-process ShardCatalog management (VV-CTL-002).
//!
//! Operates on a catalog directory without needing a running server.
//!
//! ## Subcommands
//!
//! ```text
//! vvctl catalog create-table <name> --columns id score        # create table + persist schema
//! vvctl catalog list-tables                                   # list all tables in the catalog
//! vvctl catalog drop-table <name>                             # remove table from catalog
//! vvctl catalog describe-table <name>                         # show column names
//! vvctl catalog restore                                       # reload all persisted tables
//! ```
//!
//! Tables are created inside `<dir>/catalog/<name>/` and their typed schemas
//! are persisted to `<dir>/registry/` via [`SchemaRegistry`] (VV-ENG-007, L103).

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};
use clap::{Args, Subcommand};
use vectorvolt_engine::{
    CatalogExport, CatalogStatsReport, ColumnChangeKind, ColumnDef, GcReport, SchemaRegistry,
    ShardCatalog, SnapshotFreshness, TableSchema, VvConfig, WatchControl, WatchEvent,
    catalog_stats, check_health, clear_tags, clone_table, create_alias, diff_catalogs,
    export_catalog, get_lock_info, get_tags, import_catalog, list_aliases, list_snapshots,
    lock_table, remove_alias, remove_tag, rename_table, resolve_alias, restore_snapshot,
    run_catalog_gc, save_snapshot, set_tag, unlock_table, validate_catalog, watch_catalog,
};

const DEFAULT_FLUSH_THRESHOLD: usize = 1_048_576;
const DEFAULT_PARTITION_COUNT: usize = 4;

#[derive(Args)]
pub struct CatalogArgs {
    /// Catalog root directory. Created if it does not exist.
    #[arg(long, short, default_value = ".")]
    pub dir: PathBuf,

    #[command(subcommand)]
    pub command: CatalogCommand,
}

#[derive(Subcommand)]
pub enum CatalogCommand {
    /// Create a new table and persist its schema.
    ///
    /// Column names are given in schema order; the first is the primary key.
    /// Types default to `u64`. Use `name:type` syntax for explicit types
    /// (e.g. `score:f64`). Supported types: `u64`, `i64`, `f64`, `bool`,
    /// `timestamp`, `decimal`.
    CreateTable {
        /// Table name (alphanumeric, underscores, and hyphens, max 64 chars).
        name: String,
        /// Column definitions in `name` or `name:type` format.
        #[arg(long, num_args = 1..)]
        columns: Vec<String>,
        /// Rows buffered before a WAL flush.
        #[arg(long, default_value_t = DEFAULT_FLUSH_THRESHOLD)]
        flush_threshold: usize,
        /// Number of write shards (partitions) for this table.
        #[arg(long, default_value_t = DEFAULT_PARTITION_COUNT)]
        partitions: usize,
    },

    /// List all tables in the catalog.
    ListTables,

    /// Remove a table from the catalog.
    ///
    /// This removes the in-process entry. Data directories on disk are NOT
    /// deleted — use standard filesystem tools for that if needed.
    DropTable {
        /// Table name.
        name: String,
    },

    /// Show column names for a table.
    DescribeTable {
        /// Table name.
        name: String,
    },

    /// Reload all tables persisted in the schema registry into the catalog.
    ///
    /// Equivalent to the automatic restore that `vvctl serve --restore-catalog`
    /// performs at startup.  Useful for one-shot CLI operations that need all
    /// historic tables available before continuing.
    Restore {
        /// Rows buffered before a WAL flush (applied to every restored table).
        #[arg(long, default_value_t = DEFAULT_FLUSH_THRESHOLD)]
        flush_threshold: usize,
        /// Number of write shards applied to every restored table.
        #[arg(long, default_value_t = DEFAULT_PARTITION_COUNT)]
        partitions: usize,
    },

    /// Aggregate health check: validate + snapshot freshness + optional baseline diff.
    ///
    /// Combines `catalog validate`, snapshot-age inspection, and an optional
    /// diff against a baseline export file.  Exits 0 only when all checks pass.
    /// Suitable for CI pipelines.
    CheckHealth {
        /// Fail if the newest snapshot is older than this many seconds.
        #[arg(long)]
        max_snapshot_age: Option<u64>,
        /// Path to a baseline export JSON file. Fails if the live catalog diverges.
        #[arg(long)]
        baseline: Option<std::path::PathBuf>,
        /// Treat schema warnings (e.g. non-u64 primary key) as errors.
        #[arg(long)]
        strict: bool,
    },

    /// Create a point-in-time snapshot of the catalog schema registry.
    ///
    /// Writes a `catalog-<unix_secs>.snapshot.json` file into `<dir>/snapshots/`.
    /// The snapshot can later be restored with `snapshot-restore`.
    SnapshotCreate {
        /// Directory to write the snapshot file into. Defaults to `<catalog-dir>/snapshots/`.
        #[arg(long)]
        output_dir: Option<std::path::PathBuf>,
    },

    /// List all available snapshots in the snapshots directory.
    SnapshotList,

    /// Restore a catalog snapshot file into the catalog.
    ///
    /// Equivalent to importing the export embedded in the snapshot.
    /// Use `--skip-existing` to silently skip tables already present.
    SnapshotRestore {
        /// Path to a `.snapshot.json` file.
        file: std::path::PathBuf,
        /// Rows buffered before a WAL flush (applied to every restored table).
        #[arg(long, default_value_t = DEFAULT_FLUSH_THRESHOLD)]
        flush_threshold: usize,
        /// Number of write shards applied to every restored table.
        #[arg(long, default_value_t = DEFAULT_PARTITION_COUNT)]
        partitions: usize,
        /// Silently skip tables that already exist instead of returning an error.
        #[arg(long)]
        skip_existing: bool,
    },

    /// Compare two catalog export files (or the live catalog against a file).
    ///
    /// Prints a human-readable diff: added tables (+), removed tables (-), and
    /// column-level changes within shared tables.
    ///
    /// One of `--left` or `--live` must be provided.
    /// `--right` is always required.
    Diff {
        /// Left-hand export JSON file. Mutually exclusive with `--live`.
        #[arg(long, conflicts_with = "live")]
        left: Option<std::path::PathBuf>,
        /// Use the current live catalog as the left side.
        #[arg(long, conflicts_with = "left")]
        live: bool,
        /// Right-hand export JSON file.
        #[arg(long)]
        right: std::path::PathBuf,
    },

    /// Export all schemas to a JSON file (or stdout) for portability.
    ///
    /// The export can be transferred to another environment and imported
    /// with `vvctl catalog import`.
    Export {
        /// Write JSON to this file. Omit to print to stdout.
        #[arg(long, short)]
        output: Option<std::path::PathBuf>,
    },

    /// Import schemas from a JSON export file.
    ///
    /// Creates each table in the catalog and persists its schema in the
    /// registry.  Tables that already exist are skipped (with `--skip-existing`)
    /// or cause an error (default).
    Import {
        /// Path to a JSON export file produced by `vvctl catalog export`.
        file: std::path::PathBuf,
        /// Rows buffered before a WAL flush (applied to every imported table).
        #[arg(long, default_value_t = DEFAULT_FLUSH_THRESHOLD)]
        flush_threshold: usize,
        /// Number of write shards applied to every imported table.
        #[arg(long, default_value_t = DEFAULT_PARTITION_COUNT)]
        partitions: usize,
        /// Silently skip tables that already exist instead of returning an error.
        #[arg(long)]
        skip_existing: bool,
    },

    /// Clone a table schema to a new name (VV-CTL-014).
    ///
    /// Copies the source table's schema to `dest` and creates a fresh catalog
    /// directory for the new name.  Only the schema is copied — no data is
    /// transferred.  Useful for scaffolding tables with an identical layout
    /// (e.g. staging mirrors, partitioned shards).
    CloneTable {
        /// Name of the existing source table.
        source: String,
        /// Name for the new cloned table.
        dest: String,
        /// Rows buffered before a WAL flush for the clone.
        #[arg(long, default_value_t = DEFAULT_FLUSH_THRESHOLD)]
        flush_threshold: usize,
        /// Number of write shards for the clone.
        #[arg(long, default_value_t = DEFAULT_PARTITION_COUNT)]
        partitions: usize,
        /// Replace the destination if it already exists.
        #[arg(long)]
        overwrite: bool,
    },

    /// Rename a table in the catalog directory and schema registry (VV-CTL-013).
    ///
    /// Renames the catalog subdirectory on disk and moves the schema file in
    /// the registry to the new name.  The operation is atomic at the registry
    /// level: the old name is removed only after the new schema file is written.
    ///
    /// Fails if the target name already exists unless `--overwrite` is given.
    RenameTable {
        /// Current table name.
        from: String,
        /// New table name.
        to: String,
        /// Replace the target if it already exists.
        #[arg(long)]
        overwrite: bool,
    },

    /// Show per-table disk usage and schema summary (VV-CTL-012).
    ///
    /// Reads schemas from the registry and walks each table's catalog
    /// directory to compute disk usage — no running server needed.
    ///
    /// Output: one line per table with column count, file count, and
    /// disk size; a totals line at the bottom.
    Stats {
        /// Show individual column names and types for each table.
        #[arg(long, short)]
        verbose: bool,
    },

    /// Poll the catalog for schema changes and stream diffs to stdout.
    ///
    /// Takes an initial snapshot then re-exports the catalog every `--interval`
    /// seconds.  When the schema set changes, a compact diff is printed showing
    /// added tables (+), removed tables (-), and column-level changes (~).
    ///
    /// The command runs until interrupted (Ctrl-C) or `--count` polls are done.
    Watch {
        /// Poll interval in seconds (default: 5).
        #[arg(long, short, default_value_t = 5)]
        interval: u64,
        /// Exit after this many polls (0 = run forever). Default: 0.
        #[arg(long, short, default_value_t = 0)]
        count: usize,
        /// Print a line even when nothing changed.
        #[arg(long)]
        verbose: bool,
    },

    /// Remove stale state: orphan dirs, ghost schema entries, and old snapshots.
    ///
    /// Three classes of stale state are cleaned up:
    ///
    ///   orphan dirs      — catalog subdirectories with no registry entry
    ///   ghost entries    — registry schema files with no catalog directory
    ///   stale snapshots  — snapshot files older than `--max-snapshot-age`
    ///
    /// Use `--dry-run` to preview what would be removed without touching disk.
    /// Exits 0 even when nothing is removed (a clean catalog is fine).
    Gc {
        /// Remove snapshots older than this many seconds.
        #[arg(long)]
        max_snapshot_age: Option<u64>,
        /// Preview removals without touching the filesystem.
        #[arg(long)]
        dry_run: bool,
    },

    /// Create a named alias pointing to an existing table (VV-CTL-017).
    ///
    /// An alias is a lightweight pointer stored in the registry directory.
    /// It lets you give a stable public name to a table that may be renamed,
    /// or refer to environment-specific tables by a canonical name.
    ///
    /// The alias is NOT automatically followed by the engine — use
    /// `alias-resolve` to look up the target before querying.
    AliasCreate {
        /// Alias name to create.
        alias: String,
        /// Existing table name the alias points to.
        target: String,
        /// Replace an existing alias without error.
        #[arg(long)]
        overwrite: bool,
    },

    /// Remove a named alias (VV-CTL-017).
    ///
    /// No-op if the alias does not exist.
    AliasRemove {
        /// Alias name to remove.
        alias: String,
    },

    /// Resolve an alias to its target table name (VV-CTL-017).
    ///
    /// Prints the target table name if `name` is an alias, or reports that
    /// `name` is not an alias (exits 1 so scripts can branch on it).
    AliasResolve {
        /// Alias name to resolve.
        name: String,
    },

    /// List all aliases in the registry (VV-CTL-017).
    AliasList,

    /// Place an advisory write-lock on a table (VV-CTL-016).
    ///
    /// Writes a `<table>.lock` file into the registry directory.  The engine
    /// never checks this lock during normal writes — it is a coordination
    /// mechanism for operators running maintenance (schema changes, bulk loads,
    /// backup windows).  Use `unlock-table` to release.
    ///
    /// Fails if the table is already locked unless `--force` is given.
    LockTable {
        /// Table name.
        table: String,
        /// Human-readable reason stored in the lock file.
        #[arg(long)]
        reason: Option<String>,
        /// Overwrite an existing lock without error.
        #[arg(long)]
        force: bool,
    },

    /// Release an advisory write-lock on a table (VV-CTL-016).
    ///
    /// No-op if the table is not currently locked.
    UnlockTable {
        /// Table name.
        table: String,
    },

    /// Print the lock status of a table (VV-CTL-016).
    LockStatus {
        /// Table name.
        table: String,
    },

    /// Attach or update a metadata tag on a table (VV-CTL-015).
    ///
    /// Tags are key/value string pairs stored alongside the schema in the
    /// registry. They are purely advisory (environment, owner, team, cost
    /// centre, etc.) and do not affect query execution.
    TagSet {
        /// Table name.
        table: String,
        /// Tag key (e.g. `env`, `owner`).
        key: String,
        /// Tag value.
        value: String,
    },

    /// Get one or all metadata tags for a table (VV-CTL-015).
    ///
    /// Without `--key`, prints every tag as `key=value`.
    /// With `--key <k>`, prints only the value for that key (exits 1 if absent).
    TagGet {
        /// Table name.
        table: String,
        /// Print only this tag's value.
        #[arg(long)]
        key: Option<String>,
    },

    /// Remove a metadata tag from a table (VV-CTL-015).
    ///
    /// Exits 0 whether or not the key was present.
    TagRemove {
        /// Table name.
        table: String,
        /// Tag key to remove.
        key: String,
    },

    /// Remove ALL metadata tags from a table (VV-CTL-015).
    TagClear {
        /// Table name.
        table: String,
    },

    /// Validate catalog and registry integrity (VV-CTL-005).
    ///
    /// Checks each registered schema for correctness (column count, duplicate
    /// names, primary-key convention) and detects structural mismatches:
    ///
    ///   ghost entries  — schema in registry but no catalog directory on disk
    ///   orphan dirs    — catalog directory on disk but no registry entry
    ///
    /// Exits 0 when the catalog is healthy (no errors, no ghost/orphan entries).
    /// With `--strict`, any warning also causes exit 1.
    Validate {
        /// Treat warnings (e.g. non-u64 primary key) as errors.
        #[arg(long)]
        strict: bool,
    },

    // ── Master config (VV-CFG-010) ────────────────────────────────────────────
    /// Show the resolved master config as YAML (VV-CFG-010).
    ///
    /// Parses the given file and prints its contents to stdout.
    ConfigShow {
        /// Path to a `vectorvolt.yaml` master config file.
        file: std::path::PathBuf,
    },

    /// Validate a master config file without starting the server (VV-CFG-010).
    ///
    /// Exits 0 if the file parses cleanly and has no constraint violations.
    /// Exits 1 if there are any warnings (or parse errors).
    ConfigValidate {
        /// Path to a `vectorvolt.yaml` master config file.
        file: std::path::PathBuf,
    },

    /// Set a single key in a master config file (VV-CFG-010).
    ///
    /// Uses dot-notation: `section.field`.  Creates the file if it does not
    /// exist.  Writes the updated config back to the same file.
    ///
    /// Example: `vvctl catalog config-set threads.writer_count 8`
    ConfigSet {
        /// Path to a `vectorvolt.yaml` master config file (created if absent).
        file: std::path::PathBuf,
        /// Dot-notation key (e.g. `threads.writer_count`).
        key: String,
        /// New value (always provided as a string and coerced to the field type).
        value: String,
    },
}

pub fn run(args: CatalogArgs) -> Result<()> {
    let catalog_dir = args.dir.join("catalog");
    let registry_dir = args.dir.join("registry");
    std::fs::create_dir_all(&catalog_dir)
        .with_context(|| format!("create catalog dir {}", catalog_dir.display()))?;

    let catalog = ShardCatalog::with_registry(&catalog_dir, &registry_dir)
        .with_context(|| "open catalog with registry")?;

    match args.command {
        CatalogCommand::CreateTable {
            name,
            columns,
            flush_threshold,
            partitions,
        } => {
            if columns.is_empty() {
                bail!("--columns is required (first column is the primary key)");
            }
            let defs: Vec<ColumnDef> = columns
                .iter()
                .map(|c| {
                    if c.contains(':') {
                        ColumnDef::from_line(c)
                    } else {
                        ColumnDef::from_line(&format!("{c}:u64"))
                    }
                })
                .collect();
            let schema = TableSchema::new(defs)
                .with_context(|| format!("build schema for table '{name}'"))?;
            catalog
                .create_table(&name, schema.clone(), flush_threshold, partitions)
                .with_context(|| format!("create table '{name}'"))?;
            println!("created: {name}");
            let col_strs: Vec<String> = (0..schema.col_count())
                .filter_map(|i| schema.get(i))
                .map(|c| format!("{}:{}", c.name, c.col_type.as_str()))
                .collect();
            println!("  columns   : {}", col_strs.join(", "));
            println!("  partitions: {partitions}");
            println!("  flush_thr : {flush_threshold}");
        }

        CatalogCommand::ListTables => {
            // Restore from registry so list reflects tables from prior sessions.
            catalog
                .restore_all(DEFAULT_FLUSH_THRESHOLD, DEFAULT_PARTITION_COUNT)
                .with_context(|| "restore catalog from registry")?;
            let names = catalog.list_tables();
            if names.is_empty() {
                println!("(no tables)");
            } else {
                for name in &names {
                    println!("{name}");
                }
                println!("\n{} table(s)", names.len());
            }
        }

        CatalogCommand::DropTable { name } => {
            // Restore to pick up tables persisted in prior sessions.
            catalog
                .restore_all(DEFAULT_FLUSH_THRESHOLD, DEFAULT_PARTITION_COUNT)
                .with_context(|| "restore catalog from registry")?;
            if !catalog.contains(&name) {
                bail!("table '{name}' not found in catalog or registry");
            }
            catalog
                .drop_table(&name)
                .with_context(|| format!("drop '{name}'"))?;
            println!("dropped: {name}");
        }

        CatalogCommand::DescribeTable { name } => {
            let schema = catalog
                .get(&name)
                .map(|e| e.schema.clone())
                .or_else(|_| SchemaRegistry::new(&registry_dir).load(&name))
                .with_context(|| format!("load schema for '{name}'"))?;
            println!("table: {name}");
            for i in 0..schema.col_count() {
                if let Some(col) = schema.get(i) {
                    let tag = if i == 0 { " (pk)" } else { "" };
                    println!("  [{i}] {}:{}{tag}", col.name, col.col_type.as_str());
                }
            }
        }

        CatalogCommand::Restore {
            flush_threshold,
            partitions,
        } => {
            let count = catalog
                .restore_all(flush_threshold, partitions)
                .with_context(|| "restore catalog from registry")?;
            if count == 0 {
                println!("nothing to restore");
            } else {
                println!("restored {count} table(s):");
                for name in catalog.list_tables() {
                    println!("  {name}");
                }
            }
        }

        CatalogCommand::CheckHealth {
            max_snapshot_age,
            baseline,
            strict,
        } => {
            let snapshots_dir = args.dir.join("snapshots");

            let baseline_export = baseline
                .map(|p| {
                    let json = std::fs::read_to_string(&p)
                        .with_context(|| format!("read baseline {}", p.display()))?;
                    CatalogExport::from_json(&json).with_context(|| "parse baseline JSON")
                })
                .transpose()?;

            let report = check_health(
                &catalog_dir,
                &registry_dir,
                &snapshots_dir,
                max_snapshot_age,
                baseline_export.as_ref(),
            )
            .with_context(|| "catalog health check")?;

            // ── Validation ─────────────────────────────────────────────
            if report.validation.valid_tables.is_empty()
                && report.validation.issues.is_empty()
                && report.validation.orphan_dirs.is_empty()
                && report.validation.ghost_entries.is_empty()
            {
                println!("  validation : ok (empty catalog)");
            } else {
                for name in &report.validation.valid_tables {
                    println!("  validation : ok  {name}");
                }
                for issue in &report.validation.issues {
                    let tag = if issue.is_error { "error  " } else { "warning" };
                    println!("  validation : {tag}  {} — {}", issue.table, issue.message);
                }
                for name in &report.validation.orphan_dirs {
                    println!("  validation : orphan  {name}");
                }
                for name in &report.validation.ghost_entries {
                    println!("  validation : ghost  {name}");
                }
            }

            // ── Snapshot freshness ──────────────────────────────────────
            match &report.freshness {
                SnapshotFreshness::Unchecked => {
                    println!(
                        "  snapshots  : {} snapshot(s) (freshness not checked)",
                        report.snapshot_count
                    );
                }
                SnapshotFreshness::Fresh { age_secs } => {
                    println!("  snapshots  : ok  newest is {age_secs}s old");
                }
                SnapshotFreshness::Stale {
                    age_secs,
                    max_age_secs,
                } => {
                    println!(
                        "  snapshots  : STALE  newest is {age_secs}s old (max {max_age_secs}s)"
                    );
                }
                SnapshotFreshness::Missing => {
                    println!("  snapshots  : MISSING  no snapshots found");
                }
            }

            // ── Baseline drift ──────────────────────────────────────────
            if let Some(ref drift) = report.baseline_drift {
                if drift.is_empty() {
                    println!("  baseline   : ok  no drift");
                } else {
                    println!(
                        "  baseline   : DRIFT  {} added, {} removed, {} changed",
                        drift.added.len(),
                        drift.removed.len(),
                        drift.changed.len()
                    );
                }
            }

            // ── Overall verdict ─────────────────────────────────────────
            let unhealthy = !report.is_healthy() || (strict && !report.is_clean());
            if unhealthy {
                bail!("catalog health check FAILED");
            }
            println!("\nhealth check PASSED");
        }

        CatalogCommand::SnapshotCreate { output_dir } => {
            let snapshots_dir = output_dir.unwrap_or_else(|| args.dir.join("snapshots"));
            let path = save_snapshot(&registry_dir, &snapshots_dir)
                .with_context(|| "create catalog snapshot")?;
            println!("snapshot: {}", path.display());
        }

        CatalogCommand::SnapshotList => {
            let snapshots_dir = args.dir.join("snapshots");
            let list = list_snapshots(&snapshots_dir).with_context(|| "list catalog snapshots")?;
            if list.is_empty() {
                println!("(no snapshots)");
            } else {
                for info in &list {
                    println!(
                        "  {} — {} table(s)",
                        info.path.file_name().unwrap_or_default().to_string_lossy(),
                        info.table_count
                    );
                }
                println!("\n{} snapshot(s)", list.len());
            }
        }

        CatalogCommand::SnapshotRestore {
            file,
            flush_threshold,
            partitions,
            skip_existing,
        } => {
            let report = restore_snapshot(
                &file,
                &catalog_dir,
                &registry_dir,
                flush_threshold,
                partitions,
                skip_existing,
            )
            .with_context(|| format!("restore snapshot {}", file.display()))?;
            for name in &report.imported {
                println!("  imported: {name}");
            }
            for name in &report.skipped {
                println!("  skipped : {name} (already exists)");
            }
            println!(
                "\nrestored {} table(s), skipped {}",
                report.imported.len(),
                report.skipped.len()
            );
        }

        CatalogCommand::Diff { left, live, right } => {
            let left_export = if live {
                export_catalog(&registry_dir)
                    .with_context(|| "export live catalog as diff left side")?
            } else {
                let path = left.context("--left or --live is required")?;
                let json = std::fs::read_to_string(&path)
                    .with_context(|| format!("read left file {}", path.display()))?;
                CatalogExport::from_json(&json).with_context(|| "parse left JSON")?
            };

            let right_json = std::fs::read_to_string(&right)
                .with_context(|| format!("read right file {}", right.display()))?;
            let right_export =
                CatalogExport::from_json(&right_json).with_context(|| "parse right JSON")?;

            let diff = diff_catalogs(&left_export, &right_export);

            if diff.is_empty() {
                println!("catalogs are identical");
                return Ok(());
            }

            for t in &diff.added {
                println!("+ {}", t.name);
                for col in &t.columns {
                    println!("    + {col}");
                }
            }
            for t in &diff.removed {
                println!("- {}", t.name);
                for col in &t.columns {
                    println!("    - {col}");
                }
            }
            for td in &diff.changed {
                println!("~ {}", td.name);
                for ch in &td.changes {
                    match &ch.kind {
                        ColumnChangeKind::Added { name, col_type } => {
                            println!("    + [{i}] {name}:{col_type}", i = ch.index);
                        }
                        ColumnChangeKind::Removed { name, col_type } => {
                            println!("    - [{i}] {name}:{col_type}", i = ch.index);
                        }
                        ColumnChangeKind::TypeChanged { name, from, to } => {
                            println!("    ~ [{i}] {name}: {from} -> {to}", i = ch.index);
                        }
                        ColumnChangeKind::Renamed { from, to, col_type } => {
                            println!("    ~ [{i}] {from} -> {to} ({col_type})", i = ch.index);
                        }
                    }
                }
            }

            println!(
                "\n{} added, {} removed, {} changed",
                diff.added.len(),
                diff.removed.len(),
                diff.changed.len()
            );
        }

        CatalogCommand::Export { output } => {
            let export = export_catalog(&registry_dir).with_context(|| "export catalog schemas")?;
            let json = export
                .to_json()
                .with_context(|| "serialise catalog to JSON")?;
            match output {
                Some(path) => {
                    std::fs::write(&path, &json)
                        .with_context(|| format!("write export to {}", path.display()))?;
                    println!(
                        "exported {} table(s) to {}",
                        export.tables.len(),
                        path.display()
                    );
                }
                None => println!("{json}"),
            }
        }

        CatalogCommand::Import {
            file,
            flush_threshold,
            partitions,
            skip_existing,
        } => {
            let json = std::fs::read_to_string(&file)
                .with_context(|| format!("read import file {}", file.display()))?;
            let export = CatalogExport::from_json(&json).with_context(|| "parse import JSON")?;
            let report = import_catalog(
                &catalog_dir,
                &registry_dir,
                &export,
                flush_threshold,
                partitions,
                skip_existing,
            )
            .with_context(|| "import catalog schemas")?;
            for name in &report.imported {
                println!("  imported: {name}");
            }
            for name in &report.skipped {
                println!("  skipped : {name} (already exists)");
            }
            println!(
                "\nimported {} table(s), skipped {}",
                report.imported.len(),
                report.skipped.len()
            );
        }

        CatalogCommand::CloneTable {
            source,
            dest,
            flush_threshold,
            partitions,
            overwrite,
        } => {
            clone_table(
                &catalog_dir,
                &registry_dir,
                &source,
                &dest,
                flush_threshold,
                partitions,
                overwrite,
            )
            .with_context(|| format!("clone table '{source}' -> '{dest}'"))?;
            println!("cloned: {source} -> {dest}");
        }

        CatalogCommand::RenameTable {
            from,
            to,
            overwrite,
        } => {
            rename_table(&catalog_dir, &registry_dir, &from, &to, overwrite)
                .with_context(|| format!("rename table '{from}' -> '{to}'"))?;
            println!("renamed: {from} -> {to}");
        }

        CatalogCommand::Stats { verbose } => {
            let report: CatalogStatsReport =
                catalog_stats(&catalog_dir, &registry_dir).with_context(|| "catalog stats")?;

            if report.tables.is_empty() {
                println!("(no tables)");
            } else {
                for t in &report.tables {
                    println!(
                        "  {:<32}  {:>2} col(s)  {:>4} file(s)  {:>10}",
                        t.name,
                        t.column_count,
                        t.file_count,
                        fmt_bytes(t.disk_bytes),
                    );
                    if verbose {
                        for col in &t.columns {
                            println!("    {col}");
                        }
                    }
                }
                println!(
                    "\n{} table(s) — {} total",
                    report.tables.len(),
                    fmt_bytes(report.total_disk_bytes()),
                );
            }
        }

        CatalogCommand::Watch {
            interval,
            count,
            verbose,
        } => {
            use std::time::Duration;
            let max = if count == 0 { None } else { Some(count) };
            let dur = Duration::from_secs(interval);

            watch_catalog(&registry_dir, dur, max, |event| match event {
                WatchEvent::Initial { export } => {
                    println!("[watch] initial snapshot: {} table(s)", export.tables.len());
                    for t in &export.tables {
                        println!("  {}", t.name);
                    }
                    WatchControl::Continue
                }
                WatchEvent::Unchanged => {
                    if verbose {
                        println!("[watch] no change");
                    }
                    WatchControl::Continue
                }
                WatchEvent::Changed { diff, current } => {
                    println!(
                        "[watch] changed: {} table(s) now — {} added, {} removed, {} changed",
                        current.tables.len(),
                        diff.added.len(),
                        diff.removed.len(),
                        diff.changed.len(),
                    );
                    for t in &diff.added {
                        println!("  + {}", t.name);
                    }
                    for t in &diff.removed {
                        println!("  - {}", t.name);
                    }
                    for td in &diff.changed {
                        println!("  ~ {}", td.name);
                    }
                    WatchControl::Continue
                }
            })
            .with_context(|| "catalog watch")?;
        }

        CatalogCommand::Gc {
            max_snapshot_age,
            dry_run,
        } => {
            let snapshots_dir = args.dir.join("snapshots");
            let report: GcReport = run_catalog_gc(
                &catalog_dir,
                &registry_dir,
                &snapshots_dir,
                max_snapshot_age,
                dry_run,
            )
            .with_context(|| "catalog gc")?;

            let prefix = if dry_run {
                "[dry-run] would remove"
            } else {
                "removed"
            };
            for p in &report.orphan_dirs_removed {
                println!("  {prefix} orphan   {}", p.display());
            }
            for name in &report.ghost_entries_removed {
                println!("  {prefix} ghost    {name}");
            }
            for p in &report.stale_snapshots_removed {
                println!(
                    "  {prefix} snapshot {}",
                    p.file_name().unwrap_or_default().to_string_lossy()
                );
            }

            if report.is_clean() {
                println!("catalog gc: nothing to remove");
            } else {
                println!(
                    "\n{}: {} orphan(s), {} ghost(s), {} stale snapshot(s)",
                    if dry_run { "would remove" } else { "removed" },
                    report.orphan_dirs_removed.len(),
                    report.ghost_entries_removed.len(),
                    report.stale_snapshots_removed.len(),
                );
            }
        }

        CatalogCommand::AliasCreate {
            alias,
            target,
            overwrite,
        } => {
            create_alias(&registry_dir, &alias, &target, overwrite)
                .with_context(|| format!("create alias '{alias}' -> '{target}'"))?;
            println!("alias created: {alias} -> {target}");
        }

        CatalogCommand::AliasRemove { alias } => {
            let existed = remove_alias(&registry_dir, &alias)
                .with_context(|| format!("remove alias '{alias}'"))?;
            if existed {
                println!("alias removed: {alias}");
            } else {
                println!("alias not found: {alias}");
            }
        }

        CatalogCommand::AliasResolve { name } => {
            match resolve_alias(&registry_dir, &name)
                .with_context(|| format!("resolve alias '{name}'"))?
            {
                Some(target) => println!("{target}"),
                None => bail!("'{name}' is not an alias"),
            }
        }

        CatalogCommand::AliasList => {
            let pairs = list_aliases(&registry_dir).with_context(|| "list aliases")?;
            if pairs.is_empty() {
                println!("(no aliases)");
            } else {
                for (alias, target) in &pairs {
                    println!("  {alias} -> {target}");
                }
                println!("\n{} alias(es)", pairs.len());
            }
        }

        CatalogCommand::LockTable {
            table,
            reason,
            force,
        } => {
            lock_table(&registry_dir, &table, reason.as_deref(), force)
                .with_context(|| format!("lock table '{table}'"))?;
            println!("locked: {table}");
        }

        CatalogCommand::UnlockTable { table } => {
            unlock_table(&registry_dir, &table)
                .with_context(|| format!("unlock table '{table}'"))?;
            println!("unlocked: {table}");
        }

        CatalogCommand::LockStatus { table } => {
            match get_lock_info(&registry_dir, &table)
                .with_context(|| format!("lock status for table '{table}'"))?
            {
                None => println!("unlocked: {table}"),
                Some(info) => {
                    let age = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .map(|d| d.as_secs())
                        .unwrap_or(0)
                        .saturating_sub(info.locked_at);
                    let reason_str = info
                        .reason
                        .as_deref()
                        .map(|r| format!(" — {r}"))
                        .unwrap_or_default();
                    println!("locked: {table}  ({age}s ago{reason_str})");
                }
            }
        }

        CatalogCommand::TagSet { table, key, value } => {
            set_tag(&registry_dir, &table, &key, &value)
                .with_context(|| format!("set tag '{key}' on table '{table}'"))?;
            println!("tag set: {table}  {key}={value}");
        }

        CatalogCommand::TagGet { table, key } => {
            let tags = get_tags(&registry_dir, &table)
                .with_context(|| format!("get tags for table '{table}'"))?;
            match key {
                Some(k) => {
                    let v = tags
                        .get(&k)
                        .with_context(|| format!("tag '{k}' not found on table '{table}'"))?;
                    println!("{v}");
                }
                None => {
                    if tags.is_empty() {
                        println!("(no tags)");
                    } else {
                        let mut pairs: Vec<_> = tags.iter().collect();
                        pairs.sort_by_key(|(k, _)| k.as_str());
                        for (k, v) in pairs {
                            println!("{k}={v}");
                        }
                    }
                }
            }
        }

        CatalogCommand::TagRemove { table, key } => {
            let existed = remove_tag(&registry_dir, &table, &key)
                .with_context(|| format!("remove tag '{key}' from table '{table}'"))?;
            if existed {
                println!("tag removed: {table}  {key}");
            } else {
                println!("tag not found: {table}  {key}");
            }
        }

        CatalogCommand::TagClear { table } => {
            clear_tags(&registry_dir, &table)
                .with_context(|| format!("clear tags for table '{table}'"))?;
            println!("tags cleared: {table}");
        }

        CatalogCommand::Validate { strict } => {
            let report = validate_catalog(&catalog_dir, &registry_dir)
                .with_context(|| "validate catalog")?;

            // Per-table results
            for name in &report.valid_tables {
                println!("  ok       {name}");
            }
            for issue in &report.issues {
                let tag = if issue.is_error { "error  " } else { "warning" };
                println!("  {tag}  {} — {}", issue.table, issue.message);
            }

            // Structural findings
            for name in &report.orphan_dirs {
                println!("  orphan   {name} (catalog dir without registry entry)");
            }
            for name in &report.ghost_entries {
                println!("  ghost    {name} (registry entry without catalog dir)");
            }

            // Summary
            let total = report.valid_tables.len()
                + report.error_count()
                + report.warning_count()
                + report.orphan_dirs.len()
                + report.ghost_entries.len();
            println!(
                "\n{} table(s) checked — {} ok, {} error(s), {} warning(s), {} orphan(s), {} ghost(s)",
                total,
                report.valid_tables.len(),
                report.error_count(),
                report.warning_count(),
                report.orphan_dirs.len(),
                report.ghost_entries.len(),
            );

            let unhealthy = !report.is_healthy() || (strict && !report.is_clean());
            if unhealthy {
                bail!("catalog validation failed");
            }
        }

        CatalogCommand::ConfigShow { file } => {
            let cfg = VvConfig::from_file(&file)
                .with_context(|| format!("load config '{}'", file.display()))?;
            let yaml = cfg.to_yaml().context("serialize config")?;
            print!("{yaml}");
        }

        CatalogCommand::ConfigValidate { file } => {
            let cfg = VvConfig::from_file(&file)
                .with_context(|| format!("load config '{}'", file.display()))?;
            let warns = cfg.validate();
            if warns.is_empty() {
                println!("config ok: {}", file.display());
            } else {
                for w in &warns {
                    eprintln!("warning: {w}");
                }
                bail!("config has {} warning(s)", warns.len());
            }
        }

        CatalogCommand::ConfigSet { file, key, value } => {
            let mut cfg = if file.exists() {
                VvConfig::from_file(&file)
                    .with_context(|| format!("load config '{}'", file.display()))?
            } else {
                VvConfig::default()
            };
            cfg.set_key(&key, &value)
                .with_context(|| format!("set key '{key}'"))?;
            let yaml = cfg.to_yaml().context("serialize config")?;
            std::fs::write(&file, &yaml)
                .with_context(|| format!("write config '{}'", file.display()))?;
            println!("set {key} = {value}  ({})", file.display());
        }
    }

    Ok(())
}

fn fmt_bytes(bytes: u64) -> String {
    const KB: u64 = 1_024;
    const MB: u64 = 1_024 * KB;
    const GB: u64 = 1_024 * MB;
    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{bytes} B")
    }
}
