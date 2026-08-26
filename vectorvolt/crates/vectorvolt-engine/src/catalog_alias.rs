//! Catalog table aliases — named pointers to existing tables (VV-CTL-017).
//!
//! An alias is a plain-text file `<registry_dir>/<alias>.alias` whose content
//! is the target table name.  Aliases are resolved by the CLI and any tooling
//! that wants to support them; the engine itself does not resolve them.
//!
//! Typical use: give a stable public name to a table that may be renamed
//! (`orders_v2` → `orders`) or shadow an environment-specific table
//! (`orders_staging` → `orders_prod`).

use std::path::Path;

use crate::error::{EngineError, EngineResult};
use crate::schema_registry::SchemaRegistry;

fn alias_path(registry_dir: &Path, alias: &str) -> std::path::PathBuf {
    registry_dir.join(format!("{alias}.alias"))
}

fn require_target_exists(registry_dir: &Path, target: &str) -> EngineResult<()> {
    if !SchemaRegistry::new(registry_dir).exists(target) {
        return Err(EngineError::TableNotFound(target.to_owned()));
    }
    Ok(())
}

/// Create an alias `alias` pointing to `target`.
///
/// Fails if `target` does not exist in the registry.
/// Fails if `alias` already exists as an alias and `overwrite` is `false`.
/// Does not prevent `alias` from shadowing a real table name — callers should
/// check for conflicts if that matters.
pub fn create_alias(
    registry_dir: impl AsRef<Path>,
    alias: &str,
    target: &str,
    overwrite: bool,
) -> EngineResult<()> {
    let dir = registry_dir.as_ref();
    require_target_exists(dir, target)?;

    let path = alias_path(dir, alias);
    if path.exists() && !overwrite {
        return Err(EngineError::InvalidConfig(format!(
            "alias '{alias}' already exists (use --overwrite to replace)"
        )));
    }

    std::fs::create_dir_all(dir)?;
    std::fs::write(&path, target)?;
    Ok(())
}

/// Remove an alias.  Returns `true` if the alias existed, `false` otherwise.
pub fn remove_alias(registry_dir: impl AsRef<Path>, alias: &str) -> EngineResult<bool> {
    let path = alias_path(registry_dir.as_ref(), alias);
    if path.exists() {
        std::fs::remove_file(&path)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Resolve `name` to its target.  Returns `Some(target)` if `name` is a
/// known alias, `None` if `name` is not an alias (it may still be a real
/// table — callers decide what to do with that).
pub fn resolve_alias(registry_dir: impl AsRef<Path>, name: &str) -> EngineResult<Option<String>> {
    let path = alias_path(registry_dir.as_ref(), name);
    if !path.exists() {
        return Ok(None);
    }
    let target = std::fs::read_to_string(&path)?.trim().to_owned();
    Ok(Some(target))
}

/// List all aliases as `(alias, target)` pairs, sorted by alias name.
pub fn list_aliases(registry_dir: impl AsRef<Path>) -> EngineResult<Vec<(String, String)>> {
    let dir = registry_dir.as_ref();
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut pairs = vec![];
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name();
        let s = name.to_string_lossy();
        if let Some(alias) = s.strip_suffix(".alias") {
            let target = std::fs::read_to_string(entry.path())?.trim().to_owned();
            pairs.push((alias.to_owned(), target));
        }
    }
    pairs.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(pairs)
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::schema::{ColumnDef, TableSchema};
    use crate::shard_catalog::ShardCatalog;

    fn make_dir() -> TempDir {
        TempDir::new().unwrap()
    }

    fn paths(dir: &TempDir) -> (std::path::PathBuf, std::path::PathBuf) {
        (dir.path().join("catalog"), dir.path().join("registry"))
    }

    fn create_table(dir: &TempDir, name: &str) {
        let (cat, reg) = paths(dir);
        let catalog = ShardCatalog::with_registry(&cat, &reg).unwrap();
        let schema = TableSchema::new(vec![ColumnDef::u64("pk")]).unwrap();
        catalog.create_table(name, schema, 1024, 1).unwrap();
    }

    #[test]
    fn create_and_resolve_alias() {
        let dir = make_dir();
        create_table(&dir, "orders_v2");
        let (_, reg) = paths(&dir);
        create_alias(&reg, "orders", "orders_v2", false).unwrap();
        let target = resolve_alias(&reg, "orders").unwrap();
        assert_eq!(target.as_deref(), Some("orders_v2"));
    }

    #[test]
    fn resolve_returns_none_for_unknown() {
        let dir = make_dir();
        let (_, reg) = paths(&dir);
        std::fs::create_dir_all(&reg).unwrap();
        let target = resolve_alias(&reg, "ghost").unwrap();
        assert!(target.is_none());
    }

    #[test]
    fn create_alias_nonexistent_target_errors() {
        let dir = make_dir();
        let (_, reg) = paths(&dir);
        std::fs::create_dir_all(&reg).unwrap();
        let err = create_alias(&reg, "a", "nope", false).unwrap_err();
        assert!(matches!(err, EngineError::TableNotFound(_)));
    }

    #[test]
    fn duplicate_alias_without_overwrite_errors() {
        let dir = make_dir();
        create_table(&dir, "t1");
        let (_, reg) = paths(&dir);
        create_alias(&reg, "ptr", "t1", false).unwrap();
        let err = create_alias(&reg, "ptr", "t1", false).unwrap_err();
        assert!(err.to_string().contains("ptr"));
    }

    #[test]
    fn overwrite_replaces_alias() {
        let dir = make_dir();
        create_table(&dir, "t1");
        create_table(&dir, "t2");
        let (_, reg) = paths(&dir);
        create_alias(&reg, "ptr", "t1", false).unwrap();
        create_alias(&reg, "ptr", "t2", true).unwrap();
        let target = resolve_alias(&reg, "ptr").unwrap();
        assert_eq!(target.as_deref(), Some("t2"));
    }

    #[test]
    fn remove_alias_returns_true_when_existed() {
        let dir = make_dir();
        create_table(&dir, "t1");
        let (_, reg) = paths(&dir);
        create_alias(&reg, "a", "t1", false).unwrap();
        assert!(remove_alias(&reg, "a").unwrap());
        assert!(resolve_alias(&reg, "a").unwrap().is_none());
    }

    #[test]
    fn remove_absent_alias_returns_false() {
        let dir = make_dir();
        let (_, reg) = paths(&dir);
        std::fs::create_dir_all(&reg).unwrap();
        assert!(!remove_alias(&reg, "ghost").unwrap());
    }
}
