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

use anyhow::{Context, Result, bail};
use clap::{Args, Subcommand};
use vectorvolt_engine::{
    CatalogExport, ColumnChangeKind, ColumnDef, SchemaRegistry, ShardCatalog, TableSchema,
    diff_catalogs, export_catalog, import_catalog, list_snapshots, restore_snapshot, save_snapshot,
    validate_catalog,
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
    }

    Ok(())
}
