//! SchemaRegistry — typed column persistence (VV-ENG-007).
//!
//! A lightweight standalone component that reads and writes [`TableSchema`]
//! files to disk. Unlike [`TableRegistry`], it has no dependency on
//! [`TableEngine`] or WAL — it is purely a schema store.
//!
//! ## On-disk layout
//!
//! ```text
//! base_dir/
//!   <table_name>.schema   ← typed "name:type" text, one column per line
//! ```
//!
//! Writes are atomic: the new content is written to `<name>.schema.tmp` and
//! then renamed over the target, so a crash mid-write leaves either the old
//! file or the new one, never a partial file.
//!
//! ## Example
//!
//! ```no_run
//! use vectorvolt_engine::schema_registry::SchemaRegistry;
//! use vectorvolt_engine::schema::{ColumnDef, TableSchema};
//!
//! let reg = SchemaRegistry::new("/tmp/schemas");
//! let schema = TableSchema::new(vec![ColumnDef::u64("id"), ColumnDef::u64("val")]).unwrap();
//! reg.save("orders", &schema).unwrap();
//! let loaded = reg.load("orders").unwrap();
//! assert_eq!(loaded.col_count(), 2);
//! ```

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::error::{EngineError, EngineResult};
use crate::schema::TableSchema;

const SCHEMA_EXT: &str = "schema";
const SCHEMA_TMP_EXT: &str = "schema.tmp";

// ── SchemaRegistry ────────────────────────────────────────────────────────────

/// Persistent store for [`TableSchema`] objects.
///
/// Each table maps to one file: `<base_dir>/<table_name>.schema`.
pub struct SchemaRegistry {
    base_dir: PathBuf,
}

impl SchemaRegistry {
    /// Create a registry backed by `base_dir`.
    ///
    /// The directory is created lazily on first write.
    pub fn new(base_dir: impl Into<PathBuf>) -> Self {
        Self {
            base_dir: base_dir.into(),
        }
    }

    /// Persist `schema` for `table` using an atomic write-then-rename.
    pub fn save(&self, table: &str, schema: &TableSchema) -> EngineResult<()> {
        validate_table_name(table)?;
        std::fs::create_dir_all(&self.base_dir).map_err(EngineError::Io)?;
        let tmp = self.path_tmp(table);
        let dst = self.path(table);
        std::fs::write(&tmp, schema.to_text()).map_err(EngineError::Io)?;
        std::fs::rename(&tmp, &dst).map_err(EngineError::Io)?;
        Ok(())
    }

    /// Load the schema for `table`.
    ///
    /// Returns `Err(TableNotFound)` if the file does not exist.
    pub fn load(&self, table: &str) -> EngineResult<TableSchema> {
        validate_table_name(table)?;
        let path = self.path(table);
        if !path.exists() {
            return Err(EngineError::TableNotFound(table.to_owned()));
        }
        let text = std::fs::read_to_string(&path).map_err(EngineError::Io)?;
        TableSchema::from_text(&text)
    }

    /// Return `true` if a schema file exists for `table`.
    pub fn exists(&self, table: &str) -> bool {
        if validate_table_name(table).is_err() {
            return false;
        }
        self.path(table).exists()
    }

    /// Remove the schema file for `table`.
    ///
    /// Returns `Err(TableNotFound)` if the file does not exist.
    pub fn remove(&self, table: &str) -> EngineResult<()> {
        validate_table_name(table)?;
        let path = self.path(table);
        if !path.exists() {
            return Err(EngineError::TableNotFound(table.to_owned()));
        }
        std::fs::remove_file(&path).map_err(EngineError::Io)?;
        Ok(())
    }

    /// List all table names that have a persisted schema file, sorted.
    pub fn list_tables(&self) -> EngineResult<Vec<String>> {
        if !self.base_dir.is_dir() {
            return Ok(vec![]);
        }
        let mut names = Vec::new();
        for entry in std::fs::read_dir(&self.base_dir).map_err(EngineError::Io)? {
            let entry = entry.map_err(EngineError::Io)?;
            let path = entry.path();
            let ext_ok = path.extension().and_then(|e| e.to_str()) == Some(SCHEMA_EXT);
            if !ext_ok {
                continue;
            }
            let stem = match path.file_stem().and_then(|s| s.to_str()) {
                Some(s) => s.to_owned(),
                None => continue,
            };
            if validate_table_name(&stem).is_ok() {
                names.push(stem);
            }
        }
        names.sort();
        Ok(names)
    }

    /// Load every persisted schema at once.
    ///
    /// Returns a map from table name to schema. Any file that fails to parse
    /// causes the entire call to return an error.
    pub fn load_all(&self) -> EngineResult<HashMap<String, TableSchema>> {
        let names = self.list_tables()?;
        let mut map = HashMap::with_capacity(names.len());
        for name in names {
            let schema = self.load(&name)?;
            map.insert(name, schema);
        }
        Ok(map)
    }

    /// Base directory for this registry.
    pub fn base_dir(&self) -> &Path {
        &self.base_dir
    }

    // ── private helpers ───────────────────────────────────────────────────────

    fn path(&self, table: &str) -> PathBuf {
        self.base_dir.join(format!("{}.{}", table, SCHEMA_EXT))
    }

    fn path_tmp(&self, table: &str) -> PathBuf {
        self.base_dir.join(format!("{}.{}", table, SCHEMA_TMP_EXT))
    }
}

fn validate_table_name(name: &str) -> EngineResult<()> {
    if name.is_empty() || name.len() > 64 {
        return Err(EngineError::InvalidTableName(name.to_owned()));
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return Err(EngineError::InvalidTableName(name.to_owned()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::ColumnDef;
    use tempfile::TempDir;

    fn two_col() -> TableSchema {
        TableSchema::new(vec![ColumnDef::u64("pk"), ColumnDef::u64("val")]).unwrap()
    }

    fn make_reg() -> (SchemaRegistry, TempDir) {
        let dir = TempDir::new().unwrap();
        (SchemaRegistry::new(dir.path()), dir)
    }

    #[test]
    fn save_and_load_roundtrip() {
        let (reg, _d) = make_reg();
        reg.save("orders", &two_col()).unwrap();
        let loaded = reg.load("orders").unwrap();
        assert_eq!(loaded.col_count(), 2);
    }

    #[test]
    fn save_preserves_column_names() {
        let (reg, _d) = make_reg();
        reg.save("users", &two_col()).unwrap();
        let loaded = reg.load("users").unwrap();
        assert_eq!(loaded.names(), vec!["pk", "val"]);
    }

    #[test]
    fn load_nonexistent_returns_table_not_found() {
        let (reg, _d) = make_reg();
        let err = reg.load("ghost").unwrap_err();
        assert!(matches!(err, EngineError::TableNotFound(_)));
    }

    #[test]
    fn exists_true_after_save() {
        let (reg, _d) = make_reg();
        assert!(!reg.exists("t"));
        reg.save("t", &two_col()).unwrap();
        assert!(reg.exists("t"));
    }

    #[test]
    fn remove_deletes_file() {
        let (reg, _d) = make_reg();
        reg.save("t", &two_col()).unwrap();
        reg.remove("t").unwrap();
        assert!(!reg.exists("t"));
    }

    #[test]
    fn remove_nonexistent_errors() {
        let (reg, _d) = make_reg();
        assert!(reg.remove("ghost").is_err());
    }

    #[test]
    fn list_tables_sorted() {
        let (reg, _d) = make_reg();
        reg.save("zebra", &two_col()).unwrap();
        reg.save("alpha", &two_col()).unwrap();
        assert_eq!(reg.list_tables().unwrap(), vec!["alpha", "zebra"]);
    }

    #[test]
    fn list_tables_empty_dir() {
        let (reg, _d) = make_reg();
        assert!(reg.list_tables().unwrap().is_empty());
    }

    #[test]
    fn list_tables_nonexistent_dir() {
        let reg = SchemaRegistry::new("/nonexistent/path/abc123");
        assert!(reg.list_tables().unwrap().is_empty());
    }

    #[test]
    fn load_all_returns_all_schemas() {
        let (reg, _d) = make_reg();
        reg.save("a", &two_col()).unwrap();
        reg.save("b", &two_col()).unwrap();
        let all = reg.load_all().unwrap();
        assert_eq!(all.len(), 2);
        assert!(all.contains_key("a"));
        assert!(all.contains_key("b"));
    }

    #[test]
    fn save_overwrites_existing() {
        let (reg, _d) = make_reg();
        reg.save("t", &two_col()).unwrap();
        let three = TableSchema::new(vec![
            ColumnDef::u64("a"),
            ColumnDef::u64("b"),
            ColumnDef::u64("c"),
        ])
        .unwrap();
        reg.save("t", &three).unwrap();
        assert_eq!(reg.load("t").unwrap().col_count(), 3);
    }

    #[test]
    fn invalid_name_errors() {
        let (reg, _d) = make_reg();
        assert!(matches!(
            reg.save("bad/name", &two_col()),
            Err(EngineError::InvalidTableName(_))
        ));
        assert!(matches!(
            reg.save("", &two_col()),
            Err(EngineError::InvalidTableName(_))
        ));
    }

    #[test]
    fn base_dir_accessible() {
        let dir = TempDir::new().unwrap();
        let reg = SchemaRegistry::new(dir.path());
        assert_eq!(reg.base_dir(), dir.path());
    }
}
