//! [`TableRegistry`] — in-process catalog for multiple `TableEngine` instances.
//!
//! A single VectorVolt server process can hold many tables. This registry owns
//! them all: it creates tables on demand, hands out shared references for reads
//! and writes, and drops them on request.
//!
//! ## Naming rules
//!
//! Table names are non-empty ASCII strings of at most 64 bytes, consisting of
//! `[a-zA-Z0-9_-]`. Anything else is rejected at create/use time.
//!
//! ## Persistence layout
//!
//! Each table lives in its own subdirectory under `base_dir`:
//!
//! ```text
//! base_dir/
//!   default/
//!     schema.txt    ← column names, one per line
//!     wal/
//!     *.vvsst
//!   orders/
//!     schema.txt
//!     …
//! ```
//!
//! `schema.txt` is the catalog marker: its presence means the table exists, its
//! absence (or removal by `drop_table`) means it has been dropped. Subdirectories
//! without a `schema.txt` are ignored on `open()`.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use parking_lot::RwLock;
use vectorvolt_txn::ops::Mutation;
use vectorvolt_wal::{WalConfig, WalWriter};

use crate::schema::{ColumnDef, TableSchema};
use crate::{EngineOptions, EngineResult, TableEngine};

/// Filename written inside every table's directory to record its schema.
pub const SCHEMA_FILE: &str = "schema.txt";

/// Maximum table-name length, in bytes.
pub const MAX_TABLE_NAME: usize = 64;

// ── validation ────────────────────────────────────────────────────────────────

fn validate_name(name: &str) -> EngineResult<()> {
    use crate::EngineError;
    if name.is_empty() {
        return Err(EngineError::InvalidTableName(
            "table name must not be empty".into(),
        ));
    }
    if name.len() > MAX_TABLE_NAME {
        return Err(EngineError::InvalidTableName(format!(
            "table name '{name}' exceeds {MAX_TABLE_NAME}-byte limit"
        )));
    }
    if !name
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
    {
        return Err(EngineError::InvalidTableName(format!(
            "table name '{name}' contains invalid characters (only [a-zA-Z0-9_-] allowed)"
        )));
    }
    Ok(())
}

fn write_typed_schema(dir: &std::path::Path, schema: &TableSchema) -> std::io::Result<()> {
    std::fs::create_dir_all(dir)?;
    std::fs::write(dir.join(SCHEMA_FILE), schema.to_text())
}

fn read_typed_schema(dir: &std::path::Path) -> EngineResult<TableSchema> {
    let raw = std::fs::read_to_string(dir.join(SCHEMA_FILE))?;
    TableSchema::from_text(&raw)
}

// ── TableRegistry ─────────────────────────────────────────────────────────────

/// A catalog of named [`TableEngine`] instances backed by a base directory.
pub struct TableRegistry {
    base_dir: PathBuf,
    options: EngineOptions,
    tables: RwLock<HashMap<String, Arc<TableEngine>>>,
    /// Optional catalog WAL — when set, every CREATE / DROP TABLE is appended
    /// as a `DdlCreate` / `DdlDrop` mutation for audit / replay.
    catalog_wal: Option<WalWriter>,
}

impl TableRegistry {
    /// Create an empty in-memory registry backed by `base_dir`.
    ///
    /// Does NOT scan the directory — use [`open`](Self::open) to restore persisted tables.
    pub fn new(base_dir: impl Into<PathBuf>, options: EngineOptions) -> Self {
        TableRegistry {
            base_dir: base_dir.into(),
            options,
            tables: RwLock::new(HashMap::new()),
            catalog_wal: None,
        }
    }

    /// Open a registry by scanning `base_dir` for persisted tables.
    ///
    /// Every subdirectory containing a `schema.txt` is opened as a table. Subdirs
    /// without one are silently skipped (they may be temp files or orphaned data
    /// from a `drop_table`). Returns the first engine error encountered, if any.
    pub fn open(base_dir: impl Into<PathBuf>, options: EngineOptions) -> EngineResult<Self> {
        let base_dir = base_dir.into();
        let mut map = HashMap::new();

        if base_dir.is_dir() {
            for entry in std::fs::read_dir(&base_dir)? {
                let entry = entry?;
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }
                let schema_path = path.join(SCHEMA_FILE);
                if !schema_path.exists() {
                    continue;
                }
                // The subdir name is the table name. Silently skip anything that
                // fails our naming rules (could be system dirs like `.git`).
                let name = match path.file_name().and_then(|n| n.to_str()) {
                    Some(n) => n.to_owned(),
                    None => continue,
                };
                if validate_name(&name).is_err() {
                    continue;
                }
                let typed = read_typed_schema(&path)?;
                let engine = Arc::new(TableEngine::open_typed(typed, &path, options)?);
                map.insert(name, engine);
            }
        }

        Ok(TableRegistry {
            base_dir,
            options,
            tables: RwLock::new(map),
            catalog_wal: None,
        })
    }

    /// Create a registry with a pre-opened engine registered as `"default"`.
    ///
    /// Used by `vvctl serve` to maintain backward compatibility: a server
    /// started without an explicit `use-table` call auto-selects `"default"`.
    pub fn new_with_default(engine: Arc<TableEngine>) -> Self {
        let mut map = HashMap::new();
        map.insert("default".into(), engine);
        TableRegistry {
            base_dir: PathBuf::new(),
            options: EngineOptions::default(),
            tables: RwLock::new(map),
            catalog_wal: None,
        }
    }

    /// Create a table with the given schema, persist it, and open it.
    ///
    /// Writes `<base_dir>/<name>/schema.txt` so the table survives a restart.
    /// Returns `Err(TableAlreadyExists)` if the name is already registered.
    pub fn create_table(&self, name: &str, schema: Vec<String>) -> EngineResult<Arc<TableEngine>> {
        use crate::EngineError;
        validate_name(name)?;

        let mut tables = self.tables.write();
        if tables.contains_key(name) {
            return Err(EngineError::TableAlreadyExists(name.into()));
        }

        let dir = self.base_dir.join(name);
        // Write schema.txt before opening the engine: if the engine open fails,
        // the schema file stays on disk and will be picked up by `open()` on the
        // next restart — which is fine, the engine will just start empty.
        let typed = TableSchema::from_names(&schema);
        write_typed_schema(&dir, &typed)?;
        let engine = Arc::new(TableEngine::open_typed(typed, &dir, self.options)?);
        tables.insert(name.into(), Arc::clone(&engine));
        drop(tables);
        self.log_ddl(Mutation::DdlCreate {
            table_name: name.into(),
        });
        Ok(engine)
    }

    /// Create a table with a typed [`TableSchema`], persist it, and open it.
    ///
    /// Writes `<base_dir>/<name>/schema.txt` in the typed `"name:type"` format.
    /// Returns `Err(TableAlreadyExists)` if the name is already registered.
    pub fn create_table_typed(
        &self,
        name: &str,
        schema: TableSchema,
    ) -> EngineResult<Arc<TableEngine>> {
        use crate::EngineError;
        validate_name(name)?;

        let mut tables = self.tables.write();
        if tables.contains_key(name) {
            return Err(EngineError::TableAlreadyExists(name.into()));
        }

        let dir = self.base_dir.join(name);
        write_typed_schema(&dir, &schema)?;
        let engine = Arc::new(TableEngine::open_typed(schema, &dir, self.options)?);
        tables.insert(name.into(), Arc::clone(&engine));
        drop(tables);
        self.log_ddl(Mutation::DdlCreate {
            table_name: name.into(),
        });
        Ok(engine)
    }

    /// Read the persisted [`TableSchema`] for a table without opening its engine.
    pub fn read_schema(&self, name: &str) -> EngineResult<TableSchema> {
        use crate::EngineError;
        validate_name(name)?;
        let dir = self.base_dir.join(name);
        if !dir.join(SCHEMA_FILE).exists() {
            return Err(EngineError::TableNotFound(name.into()));
        }
        read_typed_schema(&dir)
    }

    /// Return a reference to an existing table, or `Err(TableNotFound)`.
    pub fn get(&self, name: &str) -> EngineResult<Arc<TableEngine>> {
        use crate::EngineError;
        self.tables
            .read()
            .get(name)
            .cloned()
            .ok_or_else(|| EngineError::TableNotFound(name.into()))
    }

    /// Drop a table: remove it from the registry and delete its `schema.txt`.
    ///
    /// The rest of the table's data directory is NOT deleted — that is an
    /// operator decision. Removing `schema.txt` is enough to prevent `open()`
    /// from restoring the table on the next server restart.
    ///
    /// Returns `Err(TableNotFound)` if the name is not registered.
    pub fn drop_table(&self, name: &str) -> EngineResult<()> {
        use crate::EngineError;
        let mut tables = self.tables.write();
        if tables.remove(name).is_none() {
            return Err(EngineError::TableNotFound(name.into()));
        }
        // Best-effort: remove schema.txt so the table is not re-opened on restart.
        let schema_path = self.base_dir.join(name).join(SCHEMA_FILE);
        let _ = std::fs::remove_file(schema_path);
        self.log_ddl(Mutation::DdlDrop {
            table_name: name.into(),
        });
        Ok(())
    }

    /// List all registered table names, sorted alphabetically.
    pub fn list_tables(&self) -> Vec<String> {
        let mut names: Vec<String> = self.tables.read().keys().cloned().collect();
        names.sort();
        names
    }

    /// True if `"default"` is registered (backward-compat check).
    pub fn has_default(&self) -> bool {
        self.tables.read().contains_key("default")
    }

    /// True if a table with the given name is currently registered.
    pub fn contains(&self, name: &str) -> bool {
        self.tables.read().contains_key(name)
    }

    /// Number of tables currently registered.
    pub fn table_count(&self) -> usize {
        self.tables.read().len()
    }

    /// Attach a catalog WAL so that CREATE / DROP TABLE events are durably
    /// logged.  `wal_dir` is the directory where the catalog WAL segments
    /// are stored (typically `<base_dir>/_catalog_wal`).
    ///
    /// Returns an error if the WAL directory cannot be opened.
    pub fn enable_catalog_wal(&mut self, wal_dir: impl Into<PathBuf>) -> EngineResult<()> {
        let cfg = WalConfig::default().with_dir(wal_dir).with_fsync(false);
        let writer = WalWriter::open(cfg)?;
        self.catalog_wal = Some(writer);
        Ok(())
    }

    /// Whether a catalog WAL is currently attached.
    pub fn has_catalog_wal(&self) -> bool {
        self.catalog_wal.is_some()
    }

    /// Create a table from a YAML schema file.
    ///
    /// The YAML must match the format expected by [`crate::yaml_schema::load_yaml_schema`].
    /// The table name is taken from the YAML `table` field, ignoring any name passed by
    /// the caller. Returns `Err(TableAlreadyExists)` if the table is already registered.
    pub fn create_from_yaml_file(&self, path: &std::path::Path) -> EngineResult<Arc<TableEngine>> {
        use crate::yaml_schema::{load_yaml_schema, yaml_def_to_schema};
        let def = load_yaml_schema(path)?;
        let schema = yaml_def_to_schema(&def)?;
        self.create_table_typed(&def.table, schema)
    }

    /// Alter a table's schema by adding and/or dropping columns.
    ///
    /// - `add`: new columns appended to the schema (in order). Errors if a name
    ///   already exists.
    /// - `drop_cols`: column names to remove. Silently ignores unknown names.
    ///   Errors if the result would have zero columns.
    ///
    /// The on-disk `schema.txt` is rewritten atomically (write-then-rename), the
    /// in-memory engine is replaced, and a `DdlAlter` record is appended to the
    /// catalog WAL if one is attached.
    ///
    /// Returns `Err(TableNotFound)` if the table is not registered.
    pub fn alter_table(
        &self,
        name: &str,
        add: &[ColumnDef],
        drop_cols: &[&str],
    ) -> EngineResult<Arc<TableEngine>> {
        use crate::EngineError;
        validate_name(name)?;

        let mut tables = self.tables.write();
        if !tables.contains_key(name) {
            return Err(EngineError::TableNotFound(name.into()));
        }

        let dir = self.base_dir.join(name);
        let old_schema = read_typed_schema(&dir)?;

        let new_schema = old_schema
            .with_added(add)
            .and_then(|s| s.with_dropped(drop_cols))?;

        // Write new schema atomically: temp file then rename.
        let tmp = dir.join("schema.txt.tmp");
        std::fs::write(&tmp, new_schema.to_text())?;
        std::fs::rename(&tmp, dir.join(SCHEMA_FILE))?;

        let engine = Arc::new(TableEngine::open_typed(new_schema, &dir, self.options)?);
        tables.insert(name.into(), Arc::clone(&engine));

        let add_names: Vec<String> = add.iter().map(|c| c.name.clone()).collect();
        let drop_names: Vec<String> = drop_cols.iter().map(|s| s.to_string()).collect();
        drop(tables);
        self.log_ddl(Mutation::DdlAlter {
            table_name: name.into(),
            add_columns: add_names,
            drop_columns: drop_names,
        });
        Ok(engine)
    }

    fn log_ddl(&self, mutation: Mutation) {
        if let Some(wal) = &self.catalog_wal {
            let payload = mutation.encode();
            let _ = wal.append(payload);
            let _ = wal.flush();
        }
    }
}
