//! Catalog integrity validation (VV-CTL-005).
//!
//! [`validate_catalog`] checks a catalog+registry pair for structural and
//! schema-level issues without requiring a live [`ShardCatalog`] instance.
//! It is safe to run against a stopped or freshly-started server.
//!
//! ## Checks performed
//!
//! **Per-table (schema-level):**
//! - At least one column defined.
//! - No duplicate column names.
//! - First column is `u64` (primary-key convention).
//!
//! **Structural (cross-table):**
//! - Every registry entry has a corresponding catalog directory on disk
//!   (`ghost_entries` — schema without data dir).
//! - Every catalog subdirectory has a corresponding registry entry
//!   (`orphan_dirs` — data dir without schema).

use std::collections::HashSet;
use std::path::Path;

use crate::error::{EngineError, EngineResult};
use crate::schema::ColumnType;
use crate::schema_registry::SchemaRegistry;

// ── Report ────────────────────────────────────────────────────────────────────

/// A single per-table validation finding.
#[derive(Debug, Clone, PartialEq)]
pub struct TableIssue {
    pub table: String,
    pub message: String,
    pub is_error: bool, // false = warning
}

/// Full report returned by [`validate_catalog`].
#[derive(Debug, Default)]
pub struct CatalogValidationReport {
    /// Tables that passed all checks.
    pub valid_tables: Vec<String>,
    /// Per-table schema findings (warnings + errors).
    pub issues: Vec<TableIssue>,
    /// Catalog dirs that have no matching registry entry.
    pub orphan_dirs: Vec<String>,
    /// Registry entries that have no matching catalog dir on disk.
    pub ghost_entries: Vec<String>,
}

impl CatalogValidationReport {
    /// `true` when there are no errors, no orphan dirs, and no ghost entries.
    pub fn is_healthy(&self) -> bool {
        self.issues.iter().all(|i| !i.is_error)
            && self.orphan_dirs.is_empty()
            && self.ghost_entries.is_empty()
    }

    /// `true` when there are no issues of any kind (including warnings).
    pub fn is_clean(&self) -> bool {
        self.issues.is_empty() && self.orphan_dirs.is_empty() && self.ghost_entries.is_empty()
    }

    /// Count of tables with at least one error-level issue.
    pub fn error_count(&self) -> usize {
        self.issues
            .iter()
            .filter(|i| i.is_error)
            .map(|i| i.table.as_str())
            .collect::<HashSet<_>>()
            .len()
    }

    /// Count of tables with at least one warning-level issue.
    pub fn warning_count(&self) -> usize {
        self.issues
            .iter()
            .filter(|i| !i.is_error)
            .map(|i| i.table.as_str())
            .collect::<HashSet<_>>()
            .len()
    }
}

// ── Validate ─────────────────────────────────────────────────────────────────

/// Validate a catalog+registry pair rooted at `catalog_dir` and `registry_dir`.
///
/// Reads the registry for the schema list and the catalog directory for the
/// on-disk subdirectory list.  Does not open WAL or TxnManager instances.
pub fn validate_catalog(
    catalog_dir: impl AsRef<Path>,
    registry_dir: impl AsRef<Path>,
) -> EngineResult<CatalogValidationReport> {
    let catalog_dir = catalog_dir.as_ref();
    let registry_dir = registry_dir.as_ref();

    let mut report = CatalogValidationReport::default();

    // ── Registry entries ─────────────────────────────────────────────────────
    let registry = SchemaRegistry::new(registry_dir);
    let schemas = if registry_dir.exists() {
        registry
            .load_all()
            .map_err(|e| EngineError::InvalidConfig(format!("registry load failed: {e}")))?
    } else {
        Default::default()
    };

    // ── Catalog subdirectories ────────────────────────────────────────────────
    let catalog_names: HashSet<String> = if catalog_dir.exists() {
        catalog_subdirs(catalog_dir)?
    } else {
        HashSet::new()
    };

    let registry_names: HashSet<String> = schemas.keys().cloned().collect();

    // ── Ghost entries: in registry but no catalog dir ─────────────────────────
    let mut ghosts: Vec<String> = registry_names.difference(&catalog_names).cloned().collect();
    ghosts.sort();
    report.ghost_entries = ghosts;

    // ── Orphan dirs: catalog dir but not in registry ──────────────────────────
    let mut orphans: Vec<String> = catalog_names.difference(&registry_names).cloned().collect();
    orphans.sort();
    report.orphan_dirs = orphans;

    // ── Per-table schema checks ───────────────────────────────────────────────
    let mut names: Vec<&str> = schemas.keys().map(String::as_str).collect();
    names.sort();

    for name in names {
        let schema = &schemas[name];
        let mut table_issues: Vec<TableIssue> = Vec::new();

        // Check 1: at least one column
        if schema.col_count() == 0 {
            table_issues.push(TableIssue {
                table: name.to_owned(),
                message: "no columns defined".into(),
                is_error: true,
            });
        } else {
            // Check 2: first column must be u64 (pk convention)
            if let Some(pk) = schema.get(0)
                && pk.col_type != ColumnType::U64
            {
                table_issues.push(TableIssue {
                    table: name.to_owned(),
                    message: format!(
                        "primary key column '{}' should be u64, found {}",
                        pk.name,
                        pk.col_type.as_str()
                    ),
                    is_error: false, // warn, not block — schema is still usable
                });
            }

            // Check 3: no duplicate column names
            let mut seen: HashSet<String> = HashSet::new();
            for i in 0..schema.col_count() {
                if let Some(col) = schema.get(i)
                    && !seen.insert(col.name.clone())
                {
                    table_issues.push(TableIssue {
                        table: name.to_owned(),
                        message: format!("duplicate column name '{}'", col.name),
                        is_error: true,
                    });
                }
            }
        }

        if table_issues.is_empty() {
            report.valid_tables.push(name.to_owned());
        } else {
            report.issues.extend(table_issues);
        }
    }

    report.valid_tables.sort();
    Ok(report)
}

// ── Internal ──────────────────────────────────────────────────────────────────

fn catalog_subdirs(catalog_dir: &Path) -> EngineResult<HashSet<String>> {
    let mut names = HashSet::new();
    for entry in std::fs::read_dir(catalog_dir).map_err(EngineError::Io)? {
        let entry = entry.map_err(EngineError::Io)?;
        if entry.file_type().map_err(EngineError::Io)?.is_dir()
            && let Some(name) = entry.file_name().to_str()
        {
            names.insert(name.to_owned());
        }
    }
    Ok(names)
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use tempfile::TempDir;

    use crate::schema::{ColumnDef, TableSchema};
    use crate::schema_registry::SchemaRegistry;

    fn two_col_schema() -> TableSchema {
        TableSchema::new(vec![ColumnDef::u64("pk"), ColumnDef::u64("val")]).unwrap()
    }

    fn setup(dir: &TempDir) -> (PathBuf, PathBuf) {
        let cat = dir.path().join("catalog");
        let reg = dir.path().join("registry");
        std::fs::create_dir_all(&cat).unwrap();
        std::fs::create_dir_all(&reg).unwrap();
        (cat, reg)
    }

    #[test]
    fn empty_catalog_is_healthy() {
        let dir = TempDir::new().unwrap();
        let (cat, reg) = setup(&dir);
        let report = validate_catalog(&cat, &reg).unwrap();
        assert!(report.is_healthy());
        assert!(report.valid_tables.is_empty());
    }

    #[test]
    fn valid_table_reports_ok() {
        let dir = TempDir::new().unwrap();
        let (cat, reg) = setup(&dir);
        let registry = SchemaRegistry::new(&reg);
        registry.save("orders", &two_col_schema()).unwrap();
        std::fs::create_dir_all(cat.join("orders")).unwrap();

        let report = validate_catalog(&cat, &reg).unwrap();
        assert!(report.is_healthy());
        assert!(report.valid_tables.contains(&"orders".to_string()));
    }

    #[test]
    fn ghost_entry_detected() {
        let dir = TempDir::new().unwrap();
        let (cat, reg) = setup(&dir);
        let registry = SchemaRegistry::new(&reg);
        registry.save("ghost", &two_col_schema()).unwrap();
        // no catalog dir created

        let report = validate_catalog(&cat, &reg).unwrap();
        assert!(!report.is_healthy());
        assert!(report.ghost_entries.contains(&"ghost".to_string()));
    }

    #[test]
    fn orphan_dir_detected() {
        let dir = TempDir::new().unwrap();
        let (cat, reg) = setup(&dir);
        std::fs::create_dir_all(cat.join("orphan")).unwrap();
        // no registry entry

        let report = validate_catalog(&cat, &reg).unwrap();
        assert!(!report.is_healthy());
        assert!(report.orphan_dirs.contains(&"orphan".to_string()));
    }
}
