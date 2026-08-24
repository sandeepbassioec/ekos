//! Catalog table clone — copy a schema to a new name (VV-CTL-014).
//!
//! [`clone_table`] copies an existing table's schema from the registry to a
//! new name and, optionally, creates the corresponding catalog directory
//! structure by opening a new [`ShardCatalog`] entry for the clone.
//!
//! The source table's *data* is never copied — only its schema definition.
//! This is useful for quickly scaffolding a new table that shares the same
//! column layout (e.g. partitioned shards, test mirrors, staging tables).

use std::path::Path;

use crate::error::{EngineError, EngineResult};
use crate::schema_registry::SchemaRegistry;
use crate::shard_catalog::ShardCatalog;

/// Clone `source` schema to `dest` in both the registry and catalog.
///
/// - `catalog_dir`: root directory containing one subdirectory per table.
/// - `registry_dir`: directory containing `.schema` files.
/// - `source`: name of the existing table to clone.
/// - `dest`: name for the new table.
/// - `flush_threshold`: WAL flush threshold for the new catalog entry.
/// - `partition_count`: number of write shards for the new catalog entry.
/// - `overwrite`: when `true`, allow cloning onto an existing target name
///   (drops the existing catalog entry and overwrites its schema file).
pub fn clone_table(
    catalog_dir: impl AsRef<Path>,
    registry_dir: impl AsRef<Path>,
    source: &str,
    dest: &str,
    flush_threshold: usize,
    partition_count: usize,
    overwrite: bool,
) -> EngineResult<()> {
    let catalog_dir = catalog_dir.as_ref();
    let registry_dir = registry_dir.as_ref();

    let registry = SchemaRegistry::new(registry_dir);

    // 1. Source must exist.
    if !registry.exists(source) {
        return Err(EngineError::TableNotFound(source.to_owned()));
    }

    // 2. Destination must not exist (unless overwrite).
    if registry.exists(dest) && !overwrite {
        return Err(EngineError::TableAlreadyExists(dest.to_owned()));
    }

    // 3. Load source schema.
    let schema = registry.load(source)?;

    // 4. Create the new catalog entry (creates catalog dir + writes schema).
    let catalog = ShardCatalog::with_registry(catalog_dir, registry_dir)?;
    if overwrite && registry.exists(dest) {
        // Drop existing entry first so create_table doesn't conflict.
        let _ = catalog.drop_table(dest);
    }
    catalog.create_table(dest, schema, flush_threshold, partition_count)?;

    Ok(())
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::schema::{ColumnDef, TableSchema};
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
    fn clone_creates_dest_schema() {
        let dir = TempDir::new().unwrap();
        create_table(&dir, "src");
        let (cat, reg) = paths(&dir);
        clone_table(&cat, &reg, "src", "dst", 1024, 1, false).unwrap();
        let registry = SchemaRegistry::new(&reg);
        assert!(registry.exists("dst"));
    }

    #[test]
    fn clone_preserves_source() {
        let dir = TempDir::new().unwrap();
        create_table(&dir, "src");
        let (cat, reg) = paths(&dir);
        clone_table(&cat, &reg, "src", "dst", 1024, 1, false).unwrap();
        let registry = SchemaRegistry::new(&reg);
        assert!(registry.exists("src"));
    }

    #[test]
    fn clone_dest_has_same_column_count() {
        let dir = TempDir::new().unwrap();
        create_table(&dir, "src");
        let (cat, reg) = paths(&dir);
        clone_table(&cat, &reg, "src", "dst", 1024, 1, false).unwrap();
        let registry = SchemaRegistry::new(&reg);
        let schema = registry.load("dst").unwrap();
        assert_eq!(schema.col_count(), 2);
    }

    #[test]
    fn clone_nonexistent_source_errors() {
        let dir = TempDir::new().unwrap();
        let (cat, reg) = paths(&dir);
        std::fs::create_dir_all(&cat).unwrap();
        let err = clone_table(&cat, &reg, "ghost", "dst", 1024, 1, false).unwrap_err();
        assert!(matches!(err, EngineError::TableNotFound(_)));
    }

    #[test]
    fn clone_to_existing_dest_errors_without_overwrite() {
        let dir = TempDir::new().unwrap();
        create_table(&dir, "src");
        create_table(&dir, "dst");
        let (cat, reg) = paths(&dir);
        let err = clone_table(&cat, &reg, "src", "dst", 1024, 1, false).unwrap_err();
        assert!(matches!(err, EngineError::TableAlreadyExists(_)));
    }

    #[test]
    fn clone_with_overwrite_replaces_dest() {
        let dir = TempDir::new().unwrap();
        create_table(&dir, "src");
        create_table(&dir, "dst");
        let (cat, reg) = paths(&dir);
        clone_table(&cat, &reg, "src", "dst", 1024, 1, true).unwrap();
        let registry = SchemaRegistry::new(&reg);
        assert!(registry.exists("dst"));
        assert!(registry.exists("src"));
    }

    #[test]
    fn clone_creates_catalog_dir_for_dest() {
        let dir = TempDir::new().unwrap();
        create_table(&dir, "src");
        let (cat, reg) = paths(&dir);
        clone_table(&cat, &reg, "src", "dst", 1024, 1, false).unwrap();
        assert!(cat.join("dst").is_dir());
    }
}
