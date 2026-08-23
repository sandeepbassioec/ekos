//! Catalog table rename (VV-CTL-013).
//!
//! [`rename_table`] atomically renames a table in both the catalog directory
//! and the schema registry:
//!
//! 1. Validates that `from` exists in the registry.
//! 2. Validates that `to` does not already exist (unless `--overwrite`).
//! 3. Saves the schema under the new name.
//! 4. Renames the catalog directory on disk (if it exists).
//! 5. Removes the old schema file.
//!
//! The operation is **not** crash-safe (no WAL for metadata moves), but is
//! idempotent at the registry level: if the process dies after step 3 but
//! before step 5, a subsequent call with the same arguments will fail at
//! step 2 (new name already exists) — the operator can clean up the orphan
//! old entry with `vvctl catalog gc`.

use std::path::Path;

use crate::error::{EngineError, EngineResult};
use crate::schema_registry::SchemaRegistry;

/// Rename `from` to `to` in both the catalog directory and the registry.
///
/// - `catalog_dir`: root directory containing one subdirectory per table.
/// - `registry_dir`: directory containing `.schema` files.
/// - `from`: current table name.
/// - `to`: desired new table name.
/// - `overwrite`: when `true`, allow renaming onto an existing target (the
///   target's schema and directory are replaced).
pub fn rename_table(
    catalog_dir: impl AsRef<Path>,
    registry_dir: impl AsRef<Path>,
    from: &str,
    to: &str,
    overwrite: bool,
) -> EngineResult<()> {
    let catalog_dir = catalog_dir.as_ref();
    let registry_dir = registry_dir.as_ref();

    let registry = SchemaRegistry::new(registry_dir);

    // 1. Source must exist.
    if !registry.exists(from) {
        return Err(EngineError::TableNotFound(from.to_owned()));
    }

    // 2. Target must not exist (unless overwrite).
    if registry.exists(to) && !overwrite {
        return Err(EngineError::TableAlreadyExists(to.to_owned()));
    }

    // 3. Save schema under new name.
    let schema = registry.load(from)?;
    registry.save(to, &schema)?;

    // 4. Rename catalog directory (best-effort: may not exist for ghost entries).
    let from_dir = catalog_dir.join(from);
    let to_dir = catalog_dir.join(to);
    if from_dir.is_dir() {
        if to_dir.exists() && overwrite {
            std::fs::remove_dir_all(&to_dir).map_err(EngineError::Io)?;
        }
        std::fs::rename(&from_dir, &to_dir).map_err(EngineError::Io)?;
    }

    // 5. Remove old schema file.
    registry.remove(from)?;

    Ok(())
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::schema::{ColumnDef, TableSchema};
    use crate::schema_registry::SchemaRegistry;
    use crate::shard_catalog::ShardCatalog;

    fn paths(dir: &TempDir) -> (std::path::PathBuf, std::path::PathBuf) {
        (dir.path().join("catalog"), dir.path().join("registry"))
    }

    fn create_table(dir: &TempDir, name: &str) {
        let (cat, reg) = paths(dir);
        let catalog = ShardCatalog::with_registry(&cat, &reg).unwrap();
        let schema = TableSchema::new(vec![ColumnDef::u64("pk"), ColumnDef::u64("val")]).unwrap();
        catalog.create_table(name, schema, 1024, 1).unwrap();
    }

    #[test]
    fn rename_moves_schema_and_dir() {
        let dir = TempDir::new().unwrap();
        create_table(&dir, "old");
        let (cat, reg) = paths(&dir);
        rename_table(&cat, &reg, "old", "new", false).unwrap();
        let registry = SchemaRegistry::new(&reg);
        assert!(registry.exists("new"));
        assert!(!registry.exists("old"));
        assert!(cat.join("new").is_dir());
        assert!(!cat.join("old").exists());
    }

    #[test]
    fn rename_nonexistent_table_errors() {
        let dir = TempDir::new().unwrap();
        let (cat, reg) = paths(&dir);
        std::fs::create_dir_all(&cat).unwrap();
        let err = rename_table(&cat, &reg, "ghost", "new", false).unwrap_err();
        assert!(matches!(err, EngineError::TableNotFound(_)));
    }

    #[test]
    fn rename_to_existing_name_errors_without_overwrite() {
        let dir = TempDir::new().unwrap();
        create_table(&dir, "a");
        create_table(&dir, "b");
        let (cat, reg) = paths(&dir);
        let err = rename_table(&cat, &reg, "a", "b", false).unwrap_err();
        assert!(matches!(err, EngineError::TableAlreadyExists(_)));
    }

    #[test]
    fn rename_with_overwrite_replaces_target() {
        let dir = TempDir::new().unwrap();
        create_table(&dir, "a");
        create_table(&dir, "b");
        let (cat, reg) = paths(&dir);
        rename_table(&cat, &reg, "a", "b", true).unwrap();
        let registry = SchemaRegistry::new(&reg);
        assert!(registry.exists("b"));
        assert!(!registry.exists("a"));
    }

    #[test]
    fn rename_preserves_schema_content() {
        let dir = TempDir::new().unwrap();
        create_table(&dir, "src");
        let (cat, reg) = paths(&dir);
        rename_table(&cat, &reg, "src", "dst", false).unwrap();
        let registry = SchemaRegistry::new(&reg);
        let schema = registry.load("dst").unwrap();
        assert_eq!(schema.col_count(), 2);
    }
}
