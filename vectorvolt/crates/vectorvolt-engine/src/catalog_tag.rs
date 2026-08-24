//! Catalog table tags — per-table key/value metadata (VV-CTL-015).
//!
//! Tags are stored as a JSON object in `<registry_dir>/<table>.tags`.
//! They are purely advisory metadata (environment, owner, team, etc.)
//! and have no effect on query execution or WAL behaviour.

use std::collections::HashMap;
use std::path::Path;

use crate::error::{EngineError, EngineResult};
use crate::schema_registry::SchemaRegistry;

fn tags_path(registry_dir: &Path, table: &str) -> std::path::PathBuf {
    registry_dir.join(format!("{table}.tags"))
}

fn load_raw(registry_dir: &Path, table: &str) -> EngineResult<HashMap<String, String>> {
    let path = tags_path(registry_dir, table);
    if !path.exists() {
        return Ok(HashMap::new());
    }
    let json = std::fs::read_to_string(&path)?;
    let map: HashMap<String, String> = serde_json::from_str(&json)
        .map_err(|e| EngineError::InvalidConfig(format!("parse tags for '{table}': {e}")))?;
    Ok(map)
}

fn save_raw(registry_dir: &Path, table: &str, tags: &HashMap<String, String>) -> EngineResult<()> {
    std::fs::create_dir_all(registry_dir)?;
    let json = serde_json::to_string(tags)
        .map_err(|e| EngineError::InvalidConfig(format!("serialise tags for '{table}': {e}")))?;
    std::fs::write(tags_path(registry_dir, table), json)?;
    Ok(())
}

fn require_exists(registry_dir: &Path, table: &str) -> EngineResult<()> {
    if !SchemaRegistry::new(registry_dir).exists(table) {
        return Err(EngineError::TableNotFound(table.to_owned()));
    }
    Ok(())
}

/// Set (or overwrite) a single tag on `table`.
pub fn set_tag(
    registry_dir: impl AsRef<Path>,
    table: &str,
    key: &str,
    value: &str,
) -> EngineResult<()> {
    let dir = registry_dir.as_ref();
    require_exists(dir, table)?;
    let mut tags = load_raw(dir, table)?;
    tags.insert(key.to_owned(), value.to_owned());
    save_raw(dir, table, &tags)
}

/// Return all tags for `table`. Returns an empty map when no tags have been set.
pub fn get_tags(
    registry_dir: impl AsRef<Path>,
    table: &str,
) -> EngineResult<HashMap<String, String>> {
    let dir = registry_dir.as_ref();
    require_exists(dir, table)?;
    load_raw(dir, table)
}

/// Remove a single tag by key. Returns `true` if the key existed, `false` otherwise.
pub fn remove_tag(registry_dir: impl AsRef<Path>, table: &str, key: &str) -> EngineResult<bool> {
    let dir = registry_dir.as_ref();
    require_exists(dir, table)?;
    let mut tags = load_raw(dir, table)?;
    if tags.remove(key).is_some() {
        save_raw(dir, table, &tags)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Remove all tags for `table`.
pub fn clear_tags(registry_dir: impl AsRef<Path>, table: &str) -> EngineResult<()> {
    let dir = registry_dir.as_ref();
    require_exists(dir, table)?;
    let path = tags_path(dir, table);
    if path.exists() {
        std::fs::remove_file(&path)?;
    }
    Ok(())
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
    fn set_and_get_tag() {
        let dir = make_dir();
        create_table(&dir, "t1");
        let (_, reg) = paths(&dir);
        set_tag(&reg, "t1", "env", "prod").unwrap();
        let tags = get_tags(&reg, "t1").unwrap();
        assert_eq!(tags.get("env").map(String::as_str), Some("prod"));
    }

    #[test]
    fn get_tags_empty_returns_empty_map() {
        let dir = make_dir();
        create_table(&dir, "t1");
        let (_, reg) = paths(&dir);
        let tags = get_tags(&reg, "t1").unwrap();
        assert!(tags.is_empty());
    }

    #[test]
    fn set_tag_overwrites_existing() {
        let dir = make_dir();
        create_table(&dir, "t1");
        let (_, reg) = paths(&dir);
        set_tag(&reg, "t1", "env", "staging").unwrap();
        set_tag(&reg, "t1", "env", "prod").unwrap();
        let tags = get_tags(&reg, "t1").unwrap();
        assert_eq!(tags.get("env").map(String::as_str), Some("prod"));
    }

    #[test]
    fn remove_existing_tag_returns_true() {
        let dir = make_dir();
        create_table(&dir, "t1");
        let (_, reg) = paths(&dir);
        set_tag(&reg, "t1", "owner", "alice").unwrap();
        assert!(remove_tag(&reg, "t1", "owner").unwrap());
        let tags = get_tags(&reg, "t1").unwrap();
        assert!(!tags.contains_key("owner"));
    }

    #[test]
    fn remove_absent_tag_returns_false() {
        let dir = make_dir();
        create_table(&dir, "t1");
        let (_, reg) = paths(&dir);
        assert!(!remove_tag(&reg, "t1", "ghost_key").unwrap());
    }

    #[test]
    fn clear_tags_removes_all() {
        let dir = make_dir();
        create_table(&dir, "t1");
        let (_, reg) = paths(&dir);
        set_tag(&reg, "t1", "a", "1").unwrap();
        set_tag(&reg, "t1", "b", "2").unwrap();
        clear_tags(&reg, "t1").unwrap();
        let tags = get_tags(&reg, "t1").unwrap();
        assert!(tags.is_empty());
    }

    #[test]
    fn nonexistent_table_errors_on_set() {
        let dir = make_dir();
        let (_, reg) = paths(&dir);
        std::fs::create_dir_all(&reg).unwrap();
        let err = set_tag(&reg, "ghost", "k", "v").unwrap_err();
        assert!(matches!(err, EngineError::TableNotFound(_)));
    }
}
