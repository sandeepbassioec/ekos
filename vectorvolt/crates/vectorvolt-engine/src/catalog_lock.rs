//! Catalog table advisory locks (VV-CTL-016).
//!
//! A lock is a JSON file `<registry_dir>/<table>.lock` that records who locked
//! the table and when.  Locks are **advisory** — the engine never checks them
//! during writes; they exist only to let operators coordinate maintenance.
//!
//! Workflow: `lock-table` → maintenance → `unlock-table`.

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::error::{EngineError, EngineResult};
use crate::schema_registry::SchemaRegistry;

fn lock_path(registry_dir: &Path, table: &str) -> std::path::PathBuf {
    registry_dir.join(format!("{table}.lock"))
}

fn require_exists(registry_dir: &Path, table: &str) -> EngineResult<()> {
    if !SchemaRegistry::new(registry_dir).exists(table) {
        return Err(EngineError::TableNotFound(table.to_owned()));
    }
    Ok(())
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Information stored inside a `.lock` file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockInfo {
    pub locked_at: u64,
    pub reason: Option<String>,
}

/// Lock `table`.  Fails with [`EngineError::TableAlreadyExists`] (reused as
/// "already locked") if a lock file already exists and `force` is `false`.
pub fn lock_table(
    registry_dir: impl AsRef<Path>,
    table: &str,
    reason: Option<&str>,
    force: bool,
) -> EngineResult<()> {
    let dir = registry_dir.as_ref();
    require_exists(dir, table)?;

    let path = lock_path(dir, table);
    if path.exists() && !force {
        return Err(EngineError::InvalidConfig(format!(
            "table '{table}' is already locked (use --force to override)"
        )));
    }

    let info = LockInfo {
        locked_at: now_secs(),
        reason: reason.map(str::to_owned),
    };
    let json = serde_json::to_string(&info)
        .map_err(|e| EngineError::InvalidConfig(format!("serialise lock for '{table}': {e}")))?;
    std::fs::write(&path, json)?;
    Ok(())
}

/// Unlock `table`.  No-op if the table is not locked.
pub fn unlock_table(registry_dir: impl AsRef<Path>, table: &str) -> EngineResult<()> {
    let dir = registry_dir.as_ref();
    require_exists(dir, table)?;

    let path = lock_path(dir, table);
    if path.exists() {
        std::fs::remove_file(&path)?;
    }
    Ok(())
}

/// Return `true` if `table` currently has a lock file.
pub fn is_locked(registry_dir: impl AsRef<Path>, table: &str) -> EngineResult<bool> {
    let dir = registry_dir.as_ref();
    require_exists(dir, table)?;
    Ok(lock_path(dir, table).exists())
}

/// Return the [`LockInfo`] for `table`, or `None` if not locked.
pub fn get_lock_info(
    registry_dir: impl AsRef<Path>,
    table: &str,
) -> EngineResult<Option<LockInfo>> {
    let dir = registry_dir.as_ref();
    require_exists(dir, table)?;

    let path = lock_path(dir, table);
    if !path.exists() {
        return Ok(None);
    }
    let json = std::fs::read_to_string(&path)?;
    let info: LockInfo = serde_json::from_str(&json)
        .map_err(|e| EngineError::InvalidConfig(format!("parse lock for '{table}': {e}")))?;
    Ok(Some(info))
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
    fn lock_then_is_locked() {
        let dir = make_dir();
        create_table(&dir, "t1");
        let (_, reg) = paths(&dir);
        lock_table(&reg, "t1", None, false).unwrap();
        assert!(is_locked(&reg, "t1").unwrap());
    }

    #[test]
    fn unlock_clears_lock() {
        let dir = make_dir();
        create_table(&dir, "t1");
        let (_, reg) = paths(&dir);
        lock_table(&reg, "t1", None, false).unwrap();
        unlock_table(&reg, "t1").unwrap();
        assert!(!is_locked(&reg, "t1").unwrap());
    }

    #[test]
    fn double_lock_without_force_errors() {
        let dir = make_dir();
        create_table(&dir, "t1");
        let (_, reg) = paths(&dir);
        lock_table(&reg, "t1", None, false).unwrap();
        let err = lock_table(&reg, "t1", None, false).unwrap_err();
        assert!(err.to_string().contains("already locked"));
    }

    #[test]
    fn force_lock_overwrites() {
        let dir = make_dir();
        create_table(&dir, "t1");
        let (_, reg) = paths(&dir);
        lock_table(&reg, "t1", Some("first"), false).unwrap();
        lock_table(&reg, "t1", Some("second"), true).unwrap();
        let info = get_lock_info(&reg, "t1").unwrap().unwrap();
        assert_eq!(info.reason.as_deref(), Some("second"));
    }

    #[test]
    fn unlock_noop_when_not_locked() {
        let dir = make_dir();
        create_table(&dir, "t1");
        let (_, reg) = paths(&dir);
        unlock_table(&reg, "t1").unwrap();
        assert!(!is_locked(&reg, "t1").unwrap());
    }

    #[test]
    fn lock_stores_reason() {
        let dir = make_dir();
        create_table(&dir, "t1");
        let (_, reg) = paths(&dir);
        lock_table(&reg, "t1", Some("maintenance window"), false).unwrap();
        let info = get_lock_info(&reg, "t1").unwrap().unwrap();
        assert_eq!(info.reason.as_deref(), Some("maintenance window"));
    }

    #[test]
    fn nonexistent_table_errors_on_lock() {
        let dir = make_dir();
        let (_, reg) = paths(&dir);
        std::fs::create_dir_all(&reg).unwrap();
        let err = lock_table(&reg, "ghost", None, false).unwrap_err();
        assert!(matches!(err, EngineError::TableNotFound(_)));
    }
}
