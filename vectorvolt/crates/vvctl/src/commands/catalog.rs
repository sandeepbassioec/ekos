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
use vectorvolt_engine::{ColumnDef, SchemaRegistry, ShardCatalog, TableSchema};

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
    }

    Ok(())
}
