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
//!   default/       ← the "default" table, auto-created by new_with_default()
//!   orders/
//!   products/
//! ```
//!
//! The directory is created on first use of each table.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use parking_lot::RwLock;

use crate::{EngineOptions, EngineResult, TableEngine};

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

// ── TableRegistry ─────────────────────────────────────────────────────────────

/// A catalog of named [`TableEngine`] instances backed by a base directory.
pub struct TableRegistry {
    base_dir: PathBuf,
    options: EngineOptions,
    tables: RwLock<HashMap<String, Arc<TableEngine>>>,
}

impl TableRegistry {
    /// Create an empty registry. Tables are opened/created lazily.
    pub fn new(base_dir: impl Into<PathBuf>, options: EngineOptions) -> Self {
        TableRegistry {
            base_dir: base_dir.into(),
            options,
            tables: RwLock::new(HashMap::new()),
        }
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
        }
    }

    /// Create a table with the given schema and open it.
    ///
    /// Returns `Err(TableAlreadyExists)` if the name is already registered.
    pub fn create_table(&self, name: &str, schema: Vec<String>) -> EngineResult<Arc<TableEngine>> {
        use crate::EngineError;
        validate_name(name)?;

        let mut tables = self.tables.write();
        if tables.contains_key(name) {
            return Err(EngineError::TableAlreadyExists(name.into()));
        }

        let dir = self.base_dir.join(name);
        let engine = Arc::new(TableEngine::open_with(schema, &dir, self.options)?);
        tables.insert(name.into(), Arc::clone(&engine));
        Ok(engine)
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

    /// Drop a table: remove it from the registry (in-memory only).
    ///
    /// The table's data directory is NOT deleted — that is an operator decision.
    /// Returns `Err(TableNotFound)` if the name is not registered.
    pub fn drop_table(&self, name: &str) -> EngineResult<()> {
        use crate::EngineError;
        let mut tables = self.tables.write();
        if tables.remove(name).is_none() {
            return Err(EngineError::TableNotFound(name.into()));
        }
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
}
