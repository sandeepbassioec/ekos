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
//!
//! ## Encryption at rest
//!
//! A registry opened with [`TableRegistry::open_with_encryption`] opens and
//! creates every table with [`TableEngine::open_with_encryption`], so each
//! table's WAL payloads are AES-256-GCM encrypted under the given key (PP-51).
//! Each table's `schema.txt` is then a sealed side file bound to the table's
//! identity (PP-54, see `crate::side_crypto`); a plaintext one left by an
//! older version is read and re-encrypted on open. The catalog WAL, when
//! enabled, uses the same `VVE2` + key-id-header encryption as table WALs.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use parking_lot::RwLock;
use vectorvolt_crypto::KeyStore;
use vectorvolt_txn::ops::Mutation;
use vectorvolt_wal::{WalConfig, WalWriter};

use crate::schema::{ColumnDef, TableSchema};
use crate::side_crypto::{SideKey, purpose};
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

fn write_typed_schema(
    dir: &std::path::Path,
    schema: &TableSchema,
    side: Option<&SideKey>,
) -> std::io::Result<()> {
    std::fs::create_dir_all(dir)?;
    crate::side_crypto::write_file(
        &dir.join(SCHEMA_FILE),
        side,
        purpose::SCHEMA,
        schema.to_text().as_bytes(),
    )
}

/// Read `schema.txt`; also returns whether it was encrypted. An encrypted
/// schema is bound to the table identity recorded in the table's WAL, so a
/// schema.txt copied from another table fails.
fn read_typed_schema(
    dir: &std::path::Path,
    key: Option<&Arc<KeyStore>>,
) -> EngineResult<(TableSchema, bool)> {
    let path = dir.join(SCHEMA_FILE);
    let bytes = std::fs::read(&path)?;
    let (raw, sealed) = if crate::side_crypto::is_sealed(&bytes) {
        let side = match key {
            Some(key) => crate::side_crypto::read_table_id(&dir.join("wal"), key)?
                .map(|id| SideKey::new(Arc::clone(key), id)),
            None => None,
        };
        let pt = match side {
            Some(side) => side.open(purpose::SCHEMA, b"", &bytes, &path)?,
            None if key.is_some() => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!(
                        "{}: encrypted, but the table's WAL has no table marker to bind it to",
                        path.display()
                    ),
                )
                .into());
            }
            None => crate::side_crypto::open(None, purpose::SCHEMA, b"", b"", &bytes, &path)?,
        };
        (pt, true)
    } else {
        (bytes, false)
    };
    let raw = String::from_utf8(raw).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("{}: {e}", path.display()),
        )
    })?;
    Ok((TableSchema::from_text(&raw)?, sealed))
}

/// Read every DDL record from a catalog WAL written by
/// [`TableRegistry::enable_catalog_wal`], decrypting it under `key` when the
/// catalog is encrypted (same `VVE2` + key-id-header path as table WALs).
pub fn read_catalog_wal(
    wal_dir: &std::path::Path,
    key: Option<&Arc<KeyStore>>,
) -> EngineResult<Vec<Mutation>> {
    crate::engine::check_wal_keys(wal_dir, key)?;
    let cfg = WalConfig::default().with_dir(wal_dir).with_fsync(false);
    let mut out = Vec::new();
    let mut err = None;
    vectorvolt_wal::WalReader::open(cfg).scan_with_header(
        vectorvolt_wal::WalCursor::beginning(),
        |frame, header| match vectorvolt_txn::manager::open_wal_payload(
            key.map(|k| k.as_ref()),
            header,
            frame.sequence,
            frame.payload,
        ) {
            Ok(p) => {
                if let Ok(m) = Mutation::decode(&p) {
                    out.push(m);
                }
                true
            }
            Err(e) => {
                err = Some(crate::EngineError::WalDecrypt {
                    seq: e.seq,
                    reason: format!("catalog WAL {}: {}", wal_dir.display(), e.reason),
                });
                false
            }
        },
    )?;
    match err {
        Some(e) => Err(e),
        None => Ok(out),
    }
}

/// Open one table's engine, with WAL encryption when `key` is set.
fn open_engine(
    schema: TableSchema,
    dir: &std::path::Path,
    options: EngineOptions,
    key: Option<&Arc<KeyStore>>,
) -> EngineResult<TableEngine> {
    match key {
        Some(key) => {
            let mut engine =
                TableEngine::open_with_encryption(schema.names(), dir, options, Arc::clone(key))?;
            engine.schema = schema;
            Ok(engine)
        }
        None => TableEngine::open_typed(schema, dir, options),
    }
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
    /// WAL encryption key applied to every table this registry opens or
    /// creates (`None` = plaintext WAL).
    encryption_key: Option<Arc<KeyStore>>,
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
            encryption_key: None,
        }
    }

    /// Open a registry by scanning `base_dir` for persisted tables.
    ///
    /// Every subdirectory containing a `schema.txt` is opened as a table. Subdirs
    /// without one are silently skipped (they may be temp files or orphaned data
    /// from a `drop_table`). Returns the first engine error encountered, if any.
    pub fn open(base_dir: impl Into<PathBuf>, options: EngineOptions) -> EngineResult<Self> {
        Self::open_with_key(base_dir.into(), options, None)
    }

    /// [`open`](Self::open), but every table — restored now or created later —
    /// is opened with [`TableEngine::open_with_encryption`] under `key`.
    ///
    /// Restoring a table whose WAL was written under a different key (or a
    /// registry opened without one) fails with `EngineError::WalDecrypt`.
    pub fn open_with_encryption(
        base_dir: impl Into<PathBuf>,
        options: EngineOptions,
        key: Arc<KeyStore>,
    ) -> EngineResult<Self> {
        Self::open_with_key(base_dir.into(), options, Some(key))
    }

    fn open_with_key(
        base_dir: PathBuf,
        options: EngineOptions,
        encryption_key: Option<Arc<KeyStore>>,
    ) -> EngineResult<Self> {
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
                // Key problems are reported against the WAL first, naming
                // the segment and both key ids (PP-52).
                crate::engine::check_wal_keys(&path.join("wal"), encryption_key.as_ref())?;
                let (typed, sealed) = read_typed_schema(&path, encryption_key.as_ref())?;
                let engine = open_engine(typed, &path, options, encryption_key.as_ref())?;
                // Migrate a plaintext schema.txt of an encrypted table.
                if !sealed && let Some(side) = engine.side_key.as_ref() {
                    write_typed_schema(&path, &engine.schema, Some(side))?;
                }
                map.insert(name, Arc::new(engine));
            }
        }

        Ok(TableRegistry {
            base_dir,
            options,
            tables: RwLock::new(map),
            catalog_wal: None,
            encryption_key,
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
            encryption_key: None,
        }
    }

    /// Whether tables are opened with WAL encryption at rest.
    pub fn is_encrypted(&self) -> bool {
        self.encryption_key.is_some()
    }

    /// The at-rest key every table in this registry is opened with, if any.
    pub fn encryption_key(&self) -> Option<&Arc<KeyStore>> {
        self.encryption_key.as_ref()
    }

    /// Returns the registry's base directory.
    pub fn base_dir(&self) -> &std::path::Path {
        &self.base_dir
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
        let typed = TableSchema::from_names(&schema);
        let engine = Arc::new(self.persist_and_open(typed, &dir)?);
        tables.insert(name.into(), Arc::clone(&engine));
        drop(tables);
        self.log_ddl(Mutation::DdlCreate {
            table_name: name.into(),
        })?;
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
        let engine = Arc::new(self.persist_and_open(schema, &dir)?);
        tables.insert(name.into(), Arc::clone(&engine));
        drop(tables);
        self.log_ddl(Mutation::DdlCreate {
            table_name: name.into(),
        })?;
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
        Ok(read_typed_schema(&dir, self.encryption_key.as_ref())?.0)
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
        if !tables.contains_key(name) {
            return Err(EngineError::TableNotFound(name.into()));
        }
        // Remove schema.txt so the table is not re-opened on restart. A failure
        // here would resurrect the dropped table, so it is returned, and the
        // table stays registered until the file is actually gone.
        let schema_path = self.base_dir.join(name).join(SCHEMA_FILE);
        match std::fs::remove_file(schema_path) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.into()),
        }
        tables.remove(name);
        self.log_ddl(Mutation::DdlDrop {
            table_name: name.into(),
        })?;
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
    ///
    /// On an encrypted registry the catalog WAL is encrypted the same way as
    /// a table WAL (see `log_ddl`); a catalog WAL written under another key
    /// is refused here. Read it back with [`read_catalog_wal`].
    pub fn enable_catalog_wal(&mut self, wal_dir: impl Into<PathBuf>) -> EngineResult<()> {
        let wal_dir = wal_dir.into();
        if wal_dir.is_dir() {
            crate::engine::check_wal_keys(&wal_dir, self.encryption_key.as_ref())?;
        }
        let cfg = WalConfig::default()
            .with_dir(wal_dir)
            .with_fsync(false)
            .with_encryption_key_id(self.encryption_key.as_ref().map(|k| k.key_id()));
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
        let (old_schema, _) = read_typed_schema(&dir, self.encryption_key.as_ref())?;

        let new_schema = old_schema
            .with_added(add)
            .and_then(|s| s.with_dropped(drop_cols))?;

        // Write new schema atomically: temp file then rename.
        let side = tables.get(name).and_then(|e| e.side_key.clone());
        write_typed_schema(&dir, &new_schema, side.as_ref())?;

        let engine = Arc::new(self.open_engine(new_schema, &dir)?);
        tables.insert(name.into(), Arc::clone(&engine));

        let add_names: Vec<String> = add.iter().map(|c| c.name.clone()).collect();
        let drop_names: Vec<String> = drop_cols.iter().map(|s| s.to_string()).collect();
        drop(tables);
        self.log_ddl(Mutation::DdlAlter {
            table_name: name.into(),
            add_columns: add_names,
            drop_columns: drop_names,
        })?;
        Ok(engine)
    }

    /// Open a table engine with this registry's options and encryption key.
    fn open_engine(&self, schema: TableSchema, dir: &std::path::Path) -> EngineResult<TableEngine> {
        open_engine(schema, dir, self.options, self.encryption_key.as_ref())
    }

    /// Persist `schema.txt` for a new table and open it.
    ///
    /// Unencrypted: schema.txt is written first; if the engine open fails it
    /// stays on disk and `open()` picks the (empty) table up on restart.
    /// Encrypted: the engine is opened first, because the sealed schema is
    /// bound to the table identity the open records in the WAL. A crash in
    /// between leaves a directory without schema.txt, which `open()` ignores.
    fn persist_and_open(
        &self,
        schema: TableSchema,
        dir: &std::path::Path,
    ) -> EngineResult<TableEngine> {
        if self.encryption_key.is_none() {
            write_typed_schema(dir, &schema, None)?;
            return self.open_engine(schema, dir);
        }
        let engine = self.open_engine(schema, dir)?;
        write_typed_schema(dir, &engine.schema, engine.side_key.as_ref())?;
        Ok(engine)
    }

    /// Append a DDL record to the catalog WAL and flush it.
    ///
    /// The schema change is already on disk when this runs, so an error here
    /// means the catalog log (used by `--restore-catalog`) is missing the
    /// change. It is returned to the caller rather than dropped.
    ///
    /// With a key the payload is encrypted exactly like a table WAL frame:
    /// `VVE2` with AAD = (key id, sequence), inside a segment whose header
    /// records the key id.
    fn log_ddl(&self, mutation: Mutation) -> EngineResult<()> {
        if let Some(wal) = &self.catalog_wal {
            let payload = mutation.encode();
            match &self.encryption_key {
                Some(key) => {
                    let key_id = key.key_id();
                    wal.append_with(|seq| {
                        let aad = vectorvolt_wal::segment::frame_aad(&key_id, seq);
                        vectorvolt_crypto::encryptor::encrypt_with_aad(key, &payload, &aad).map_err(
                            |e| crate::EngineError::Other(format!("catalog WAL encrypt: {e}")),
                        )
                    })?;
                }
                None => {
                    wal.append(payload)?;
                }
            }
            wal.flush()?;
        }
        Ok(())
    }
}
