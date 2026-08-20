//! Catalog schema export/import for portability (VV-CTL-006).
//!
//! [`export_catalog`] serialises all schemas in a registry to a
//! [`CatalogExport`] value, which can be written as JSON and shipped
//! to another environment.  [`import_catalog`] is the inverse: it reads
//! a [`CatalogExport`] and materialises each table into a target catalog.
//!
//! ## JSON structure
//!
//! ```json
//! {
//!   "version": 1,
//!   "tables": [
//!     { "name": "orders", "columns": ["pk:u64", "amount:f64"] }
//!   ]
//! }
//! ```

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::{EngineError, EngineResult};
use crate::schema::{ColumnDef, TableSchema};
use crate::schema_registry::SchemaRegistry;
use crate::shard_catalog::ShardCatalog;

// ── Wire types ────────────────────────────────────────────────────────────────

/// A single table's schema in portable form.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExportedTable {
    pub name: String,
    /// Column definitions in `"name:type"` format (e.g. `"pk:u64"`).
    pub columns: Vec<String>,
}

/// Top-level export document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CatalogExport {
    /// Format version — always `1` for now.
    pub version: u32,
    pub tables: Vec<ExportedTable>,
}

impl CatalogExport {
    /// Serialise to a pretty-printed JSON string.
    pub fn to_json(&self) -> EngineResult<String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| EngineError::InvalidConfig(format!("JSON serialise failed: {e}")))
    }

    /// Deserialise from a JSON string.
    pub fn from_json(s: &str) -> EngineResult<Self> {
        serde_json::from_str(s)
            .map_err(|e| EngineError::InvalidConfig(format!("JSON parse failed: {e}")))
    }
}

// ── Export ────────────────────────────────────────────────────────────────────

/// Read every schema in `registry_dir` and return a [`CatalogExport`].
///
/// Tables are sorted alphabetically so output is deterministic.
pub fn export_catalog(registry_dir: impl AsRef<Path>) -> EngineResult<CatalogExport> {
    let registry_dir = registry_dir.as_ref();
    let registry = SchemaRegistry::new(registry_dir);

    let schemas = if registry_dir.exists() {
        registry
            .load_all()
            .map_err(|e| EngineError::InvalidConfig(format!("registry load failed: {e}")))?
    } else {
        Default::default()
    };

    let mut names: Vec<&str> = schemas.keys().map(String::as_str).collect();
    names.sort();

    let tables = names
        .into_iter()
        .map(|name| {
            let schema = &schemas[name];
            let columns: Vec<String> = (0..schema.col_count())
                .filter_map(|i| schema.get(i))
                .map(|col| col.to_line())
                .collect();
            ExportedTable {
                name: name.to_owned(),
                columns,
            }
        })
        .collect();

    Ok(CatalogExport { version: 1, tables })
}

// ── Import ────────────────────────────────────────────────────────────────────

/// Summary returned by [`import_catalog`].
#[derive(Debug, Default)]
pub struct ImportReport {
    /// Tables successfully created in this run.
    pub imported: Vec<String>,
    /// Tables skipped because they already existed in the catalog.
    pub skipped: Vec<String>,
}

/// Materialise the tables from `export` into `catalog_dir` / `registry_dir`.
///
/// - `skip_existing`: when `true`, silently skip tables that are already in
///   the catalog/registry; when `false`, return an error on conflict.
pub fn import_catalog(
    catalog_dir: impl AsRef<Path>,
    registry_dir: impl AsRef<Path>,
    export: &CatalogExport,
    flush_threshold: usize,
    partition_count: usize,
    skip_existing: bool,
) -> EngineResult<ImportReport> {
    let catalog_dir = catalog_dir.as_ref();
    let registry_dir = registry_dir.as_ref();

    std::fs::create_dir_all(catalog_dir).map_err(EngineError::Io)?;
    std::fs::create_dir_all(registry_dir).map_err(EngineError::Io)?;

    let catalog = ShardCatalog::with_registry(catalog_dir, registry_dir)?;
    // Load already-persisted tables so `contains()` is accurate.
    catalog.restore_all(flush_threshold, partition_count)?;

    let mut report = ImportReport::default();

    for entry in &export.tables {
        if catalog.contains(&entry.name) {
            if skip_existing {
                report.skipped.push(entry.name.clone());
                continue;
            } else {
                return Err(EngineError::InvalidConfig(format!(
                    "table '{}' already exists; use --skip-existing to ignore",
                    entry.name
                )));
            }
        }

        if entry.columns.is_empty() {
            return Err(EngineError::InvalidConfig(format!(
                "table '{}' has no columns in the export",
                entry.name
            )));
        }

        let defs: Vec<ColumnDef> = entry
            .columns
            .iter()
            .map(|c| ColumnDef::from_line(c))
            .collect();
        let schema = TableSchema::new(defs).map_err(|e| {
            EngineError::InvalidConfig(format!("invalid schema for table '{}': {e}", entry.name))
        })?;

        catalog.create_table(&entry.name, schema, flush_threshold, partition_count)?;
        report.imported.push(entry.name.clone());
    }

    Ok(report)
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::schema::{ColumnDef, TableSchema};
    use crate::schema_registry::SchemaRegistry;

    fn two_col_schema() -> TableSchema {
        TableSchema::new(vec![ColumnDef::u64("pk"), ColumnDef::u64("val")]).unwrap()
    }

    fn reg_with_table(dir: &TempDir, name: &str) -> (std::path::PathBuf, std::path::PathBuf) {
        let cat = dir.path().join("catalog");
        let reg = dir.path().join("registry");
        std::fs::create_dir_all(&cat).unwrap();
        let registry = SchemaRegistry::new(&reg);
        registry.save(name, &two_col_schema()).unwrap();
        std::fs::create_dir_all(cat.join(name)).unwrap();
        (cat, reg)
    }

    #[test]
    fn export_empty_registry_returns_empty_tables() {
        let dir = TempDir::new().unwrap();
        let reg = dir.path().join("registry");
        let export = export_catalog(&reg).unwrap();
        assert_eq!(export.version, 1);
        assert!(export.tables.is_empty());
    }

    #[test]
    fn export_single_table_roundtrip() {
        let dir = TempDir::new().unwrap();
        let (_, reg) = reg_with_table(&dir, "orders");
        let export = export_catalog(&reg).unwrap();
        assert_eq!(export.tables.len(), 1);
        assert_eq!(export.tables[0].name, "orders");
        assert_eq!(export.tables[0].columns, vec!["pk:u64", "val:u64"]);
    }

    #[test]
    fn export_tables_are_sorted() {
        let dir = TempDir::new().unwrap();
        let reg = dir.path().join("registry");
        let registry = SchemaRegistry::new(&reg);
        registry.save("zebra", &two_col_schema()).unwrap();
        registry.save("alpha", &two_col_schema()).unwrap();
        let export = export_catalog(&reg).unwrap();
        let names: Vec<&str> = export.tables.iter().map(|t| t.name.as_str()).collect();
        assert_eq!(names, vec!["alpha", "zebra"]);
    }

    #[test]
    fn json_roundtrip() {
        let dir = TempDir::new().unwrap();
        let (_, reg) = reg_with_table(&dir, "events");
        let export = export_catalog(&reg).unwrap();
        let json = export.to_json().unwrap();
        let parsed = CatalogExport::from_json(&json).unwrap();
        assert_eq!(export, parsed);
    }

    #[test]
    fn import_creates_table_in_fresh_catalog() {
        let src = TempDir::new().unwrap();
        let (_, src_reg) = reg_with_table(&src, "orders");
        let export = export_catalog(&src_reg).unwrap();

        let dst = TempDir::new().unwrap();
        let report = import_catalog(
            dst.path().join("catalog"),
            dst.path().join("registry"),
            &export,
            1024,
            1,
            false,
        )
        .unwrap();
        assert_eq!(report.imported, vec!["orders"]);
        assert!(report.skipped.is_empty());
    }

    #[test]
    fn import_skip_existing_does_not_error() {
        let src = TempDir::new().unwrap();
        let (_, src_reg) = reg_with_table(&src, "users");
        let export = export_catalog(&src_reg).unwrap();

        let dst = TempDir::new().unwrap();
        let dst_cat = dst.path().join("catalog");
        let dst_reg = dst.path().join("registry");

        // First import.
        import_catalog(&dst_cat, &dst_reg, &export, 1024, 1, false).unwrap();
        // Second import with skip_existing = true must not error.
        let report = import_catalog(&dst_cat, &dst_reg, &export, 1024, 1, true).unwrap();
        assert_eq!(report.skipped, vec!["users"]);
        assert!(report.imported.is_empty());
    }

    #[test]
    fn import_conflict_without_skip_returns_error() {
        let src = TempDir::new().unwrap();
        let (_, src_reg) = reg_with_table(&src, "t");
        let export = export_catalog(&src_reg).unwrap();

        let dst = TempDir::new().unwrap();
        let dst_cat = dst.path().join("catalog");
        let dst_reg = dst.path().join("registry");

        import_catalog(&dst_cat, &dst_reg, &export, 1024, 1, false).unwrap();
        let err = import_catalog(&dst_cat, &dst_reg, &export, 1024, 1, false);
        assert!(err.is_err());
    }
}
