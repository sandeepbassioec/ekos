//! Catalog statistics — per-table disk usage and schema summary (VV-CTL-012).
//!
//! [`catalog_stats`] works entirely from the filesystem: it reads schemas from
//! the [`SchemaRegistry`] and computes disk usage by walking each table's
//! catalog directory.  No live server or [`ShardCatalog`] instance is needed,
//! so it is safe to run against a stopped or freshly-opened engine.
//!
//! ## Reported fields
//!
//! | Field | Source |
//! |---|---|
//! | `table_name` | registry listing |
//! | `column_count` | schema file |
//! | `columns` | schema file (name + type) |
//! | `disk_bytes` | recursive file-size walk of `<catalog_dir>/<table>` |
//! | `file_count` | number of files in the table directory |

use std::path::Path;

use crate::catalog_export::ExportedTable;
use crate::error::EngineResult;
use crate::schema_registry::SchemaRegistry;

// ── Types ─────────────────────────────────────────────────────────────────────

/// Statistics for a single table in the catalog.
#[derive(Debug, Clone)]
pub struct TableStats {
    pub name: String,
    /// Column count from the persisted schema.
    pub column_count: usize,
    /// Column descriptors in `"name:type"` format (same as [`ExportedTable`]).
    pub columns: Vec<String>,
    /// Total bytes in the table's catalog directory tree.
    pub disk_bytes: u64,
    /// Number of regular files in the table's catalog directory tree.
    pub file_count: usize,
}

/// Full stats report returned by [`catalog_stats`].
#[derive(Debug, Default)]
pub struct CatalogStatsReport {
    pub tables: Vec<TableStats>,
}

impl CatalogStatsReport {
    /// Total disk bytes across all tables.
    pub fn total_disk_bytes(&self) -> u64 {
        self.tables.iter().map(|t| t.disk_bytes).sum()
    }

    /// Total column count across all tables.
    pub fn total_column_count(&self) -> usize {
        self.tables.iter().map(|t| t.column_count).sum()
    }
}

// ── Implementation ────────────────────────────────────────────────────────────

/// Collect per-table statistics for all registered tables.
///
/// - `catalog_dir`: root directory containing one subdirectory per table.
/// - `registry_dir`: directory containing `.schema` files.
///
/// Tables that are registered but have no catalog directory on disk report
/// `disk_bytes = 0` and `file_count = 0` (they are ghost entries —
/// [`validate_catalog`] would flag them).
pub fn catalog_stats(
    catalog_dir: impl AsRef<Path>,
    registry_dir: impl AsRef<Path>,
) -> EngineResult<CatalogStatsReport> {
    let catalog_dir = catalog_dir.as_ref();
    let registry_dir = registry_dir.as_ref();

    let registry = SchemaRegistry::new(registry_dir);
    let mut names = registry.list_tables()?;
    names.sort();

    let mut tables = Vec::with_capacity(names.len());

    for name in names {
        let schema = registry.load(&name)?;

        // Build column list in "name:type" format.
        let columns: Vec<String> = (0..schema.col_count())
            .filter_map(|i| schema.get(i))
            .map(|c| format!("{}:{}", c.name, c.col_type.as_str()))
            .collect();
        let column_count = columns.len();

        // Walk the table directory for disk usage.
        let table_dir = catalog_dir.join(&name);
        let (disk_bytes, file_count) = dir_size(&table_dir);

        tables.push(TableStats {
            name,
            column_count,
            columns,
            disk_bytes,
            file_count,
        });
    }

    Ok(CatalogStatsReport { tables })
}

/// Walk `dir` recursively and return `(total_bytes, file_count)`.
/// Returns `(0, 0)` when the directory does not exist.
fn dir_size(dir: &Path) -> (u64, usize) {
    if !dir.is_dir() {
        return (0, 0);
    }
    let mut bytes = 0u64;
    let mut count = 0usize;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let (b, c) = dir_size(&path);
                bytes += b;
                count += c;
            } else if path.is_file() {
                bytes += entry.metadata().map(|m| m.len()).unwrap_or(0);
                count += 1;
            }
        }
    }
    (bytes, count)
}

/// Build an [`ExportedTable`] from a [`TableStats`] for cross-module reuse.
pub fn stats_to_exported(ts: &TableStats) -> ExportedTable {
    ExportedTable {
        name: ts.name.clone(),
        columns: ts.columns.clone(),
    }
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::schema::{ColumnDef, TableSchema};
    use crate::shard_catalog::ShardCatalog;

    fn make_table(dir: &TempDir, name: &str) {
        let cat = dir.path().join("catalog");
        let reg = dir.path().join("registry");
        let catalog = ShardCatalog::with_registry(&cat, &reg).unwrap();
        let schema = TableSchema::new(vec![ColumnDef::u64("pk"), ColumnDef::u64("val")]).unwrap();
        catalog.create_table(name, schema, 1024, 1).unwrap();
    }

    fn paths(dir: &TempDir) -> (std::path::PathBuf, std::path::PathBuf) {
        (dir.path().join("catalog"), dir.path().join("registry"))
    }

    #[test]
    fn empty_catalog_returns_empty_report() {
        let dir = TempDir::new().unwrap();
        let (cat, reg) = paths(&dir);
        std::fs::create_dir_all(&cat).unwrap();
        let report = catalog_stats(&cat, &reg).unwrap();
        assert!(report.tables.is_empty());
        assert_eq!(report.total_disk_bytes(), 0);
    }

    #[test]
    fn single_table_appears_in_report() {
        let dir = TempDir::new().unwrap();
        make_table(&dir, "orders");
        let (cat, reg) = paths(&dir);
        let report = catalog_stats(&cat, &reg).unwrap();
        assert_eq!(report.tables.len(), 1);
        assert_eq!(report.tables[0].name, "orders");
    }

    #[test]
    fn column_count_matches_schema() {
        let dir = TempDir::new().unwrap();
        make_table(&dir, "orders");
        let (cat, reg) = paths(&dir);
        let report = catalog_stats(&cat, &reg).unwrap();
        assert_eq!(report.tables[0].column_count, 2);
    }

    #[test]
    fn columns_list_has_correct_format() {
        let dir = TempDir::new().unwrap();
        make_table(&dir, "orders");
        let (cat, reg) = paths(&dir);
        let report = catalog_stats(&cat, &reg).unwrap();
        let cols = &report.tables[0].columns;
        assert!(cols.iter().all(|c| c.contains(':')));
    }

    #[test]
    fn multiple_tables_sorted_alphabetically() {
        let dir = TempDir::new().unwrap();
        make_table(&dir, "zzz");
        make_table(&dir, "aaa");
        let (cat, reg) = paths(&dir);
        let report = catalog_stats(&cat, &reg).unwrap();
        assert_eq!(report.tables[0].name, "aaa");
        assert_eq!(report.tables[1].name, "zzz");
    }

    #[test]
    fn disk_bytes_nonzero_after_table_created() {
        let dir = TempDir::new().unwrap();
        make_table(&dir, "t");
        let (cat, reg) = paths(&dir);
        let report = catalog_stats(&cat, &reg).unwrap();
        assert!(report.tables[0].disk_bytes > 0);
    }

    #[test]
    fn total_disk_bytes_sums_all_tables() {
        let dir = TempDir::new().unwrap();
        make_table(&dir, "a");
        make_table(&dir, "b");
        let (cat, reg) = paths(&dir);
        let report = catalog_stats(&cat, &reg).unwrap();
        let sum: u64 = report.tables.iter().map(|t| t.disk_bytes).sum();
        assert_eq!(report.total_disk_bytes(), sum);
    }
}
