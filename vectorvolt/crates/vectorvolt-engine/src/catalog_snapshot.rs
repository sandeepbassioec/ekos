//! Catalog snapshot — point-in-time backup and restore (VV-CTL-008).
//!
//! A snapshot is a self-contained JSON file that bundles a [`CatalogExport`]
//! with metadata (timestamp, table count).  Snapshots are saved to a
//! `<dir>/snapshots/` directory with filenames of the form
//! `catalog-<unix_secs>.snapshot.json`.
//!
//! ## Workflow
//!
//! ```text
//! vvctl catalog snapshot-create               # write snapshots/<ts>.snapshot.json
//! vvctl catalog snapshot-list                 # list saved snapshots
//! vvctl catalog snapshot-restore <file>       # import snapshot into catalog
//! ```

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::catalog_export::{CatalogExport, ImportReport, export_catalog, import_catalog};
use crate::error::{EngineError, EngineResult};

// ── Types ─────────────────────────────────────────────────────────────────────

/// Metadata attached to every snapshot file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CatalogSnapshotMeta {
    /// Unix epoch seconds at snapshot creation time.
    pub created_at: u64,
    /// Number of tables captured.
    pub table_count: usize,
    /// Snapshot format version — always `1`.
    pub version: u32,
}

/// A complete point-in-time catalog snapshot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CatalogSnapshot {
    pub meta: CatalogSnapshotMeta,
    pub export: CatalogExport,
}

impl CatalogSnapshot {
    pub fn to_json(&self) -> EngineResult<String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| EngineError::InvalidConfig(format!("snapshot serialise failed: {e}")))
    }

    pub fn from_json(s: &str) -> EngineResult<Self> {
        serde_json::from_str(s)
            .map_err(|e| EngineError::InvalidConfig(format!("snapshot parse failed: {e}")))
    }
}

/// Lightweight descriptor returned by [`list_snapshots`].
#[derive(Debug, Clone)]
pub struct SnapshotInfo {
    pub path: PathBuf,
    pub created_at: u64,
    pub table_count: usize,
}

// ── Save ──────────────────────────────────────────────────────────────────────

/// Export the current registry and write a snapshot file into `snapshots_dir`.
///
/// Returns the path of the created snapshot file.
pub fn save_snapshot(
    registry_dir: impl AsRef<Path>,
    snapshots_dir: impl AsRef<Path>,
) -> EngineResult<PathBuf> {
    let snapshots_dir = snapshots_dir.as_ref();
    std::fs::create_dir_all(snapshots_dir).map_err(EngineError::Io)?;

    let export = export_catalog(registry_dir)?;
    let table_count = export.tables.len();
    let created_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let snapshot = CatalogSnapshot {
        meta: CatalogSnapshotMeta {
            created_at,
            table_count,
            version: 1,
        },
        export,
    };

    let filename = format!("catalog-{created_at}.snapshot.json");
    let path = snapshots_dir.join(&filename);
    let json = snapshot.to_json()?;
    std::fs::write(&path, json).map_err(EngineError::Io)?;

    Ok(path)
}

// ── List ──────────────────────────────────────────────────────────────────────

/// Return all snapshots in `snapshots_dir`, sorted oldest-first.
pub fn list_snapshots(snapshots_dir: impl AsRef<Path>) -> EngineResult<Vec<SnapshotInfo>> {
    let dir = snapshots_dir.as_ref();
    if !dir.exists() {
        return Ok(vec![]);
    }

    let mut infos: Vec<SnapshotInfo> = Vec::new();

    for entry in std::fs::read_dir(dir).map_err(EngineError::Io)? {
        let entry = entry.map_err(EngineError::Io)?;
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if !name.ends_with(".snapshot.json") {
            continue;
        }
        let content = std::fs::read_to_string(&path).map_err(EngineError::Io)?;
        let snap = CatalogSnapshot::from_json(&content)
            .map_err(|e| EngineError::InvalidConfig(format!("corrupt snapshot {name}: {e}")))?;
        infos.push(SnapshotInfo {
            path,
            created_at: snap.meta.created_at,
            table_count: snap.meta.table_count,
        });
    }

    infos.sort_by_key(|i| i.created_at);
    Ok(infos)
}

// ── Restore ───────────────────────────────────────────────────────────────────

/// Restore a snapshot file into the given catalog + registry directories.
///
/// Thin wrapper around [`import_catalog`] — all tables in the snapshot are
/// created as if they were imported from a plain export.
pub fn restore_snapshot(
    snapshot_path: impl AsRef<Path>,
    catalog_dir: impl AsRef<Path>,
    registry_dir: impl AsRef<Path>,
    flush_threshold: usize,
    partition_count: usize,
    skip_existing: bool,
) -> EngineResult<ImportReport> {
    let content = std::fs::read_to_string(snapshot_path.as_ref()).map_err(EngineError::Io)?;
    let snapshot = CatalogSnapshot::from_json(&content)?;
    import_catalog(
        catalog_dir,
        registry_dir,
        &snapshot.export,
        flush_threshold,
        partition_count,
        skip_existing,
    )
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::schema::{ColumnDef, TableSchema};
    use crate::schema_registry::SchemaRegistry;
    use crate::shard_catalog::ShardCatalog;

    fn setup_catalog_with_table(dir: &TempDir, name: &str) -> (PathBuf, PathBuf) {
        let cat = dir.path().join("catalog");
        let reg = dir.path().join("registry");
        let catalog = ShardCatalog::with_registry(&cat, &reg).unwrap();
        let schema = TableSchema::new(vec![ColumnDef::u64("pk"), ColumnDef::u64("val")]).unwrap();
        catalog.create_table(name, schema, 1024, 1).unwrap();
        (cat, reg)
    }

    #[test]
    fn save_snapshot_creates_file() {
        let dir = TempDir::new().unwrap();
        let (_, reg) = setup_catalog_with_table(&dir, "orders");
        let snaps = dir.path().join("snapshots");
        let path = save_snapshot(&reg, &snaps).unwrap();
        assert!(path.exists());
        assert!(path.to_str().unwrap().ends_with(".snapshot.json"));
    }

    #[test]
    fn save_snapshot_empty_registry_succeeds() {
        let dir = TempDir::new().unwrap();
        let reg = dir.path().join("registry");
        let snaps = dir.path().join("snapshots");
        let path = save_snapshot(&reg, &snaps).unwrap();
        assert!(path.exists());
    }

    #[test]
    fn snapshot_json_roundtrip() {
        let dir = TempDir::new().unwrap();
        let (_, reg) = setup_catalog_with_table(&dir, "events");
        let snaps = dir.path().join("snapshots");
        let path = save_snapshot(&reg, &snaps).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        let snap = CatalogSnapshot::from_json(&content).unwrap();
        assert_eq!(snap.meta.version, 1);
        assert_eq!(snap.meta.table_count, 1);
        assert_eq!(snap.export.tables[0].name, "events");
    }

    #[test]
    fn list_snapshots_empty_dir_returns_empty() {
        let dir = TempDir::new().unwrap();
        let snaps = dir.path().join("snapshots");
        let list = list_snapshots(&snaps).unwrap();
        assert!(list.is_empty());
    }

    #[test]
    fn list_snapshots_finds_created_snapshot() {
        let dir = TempDir::new().unwrap();
        let (_, reg) = setup_catalog_with_table(&dir, "t");
        let snaps = dir.path().join("snapshots");
        save_snapshot(&reg, &snaps).unwrap();
        let list = list_snapshots(&snaps).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].table_count, 1);
    }

    #[test]
    fn restore_snapshot_recreates_table() {
        let src = TempDir::new().unwrap();
        let (_, src_reg) = setup_catalog_with_table(&src, "users");
        let snaps = src.path().join("snapshots");
        let path = save_snapshot(&src_reg, &snaps).unwrap();

        let dst = TempDir::new().unwrap();
        let report = restore_snapshot(
            &path,
            dst.path().join("catalog"),
            dst.path().join("registry"),
            1024,
            1,
            false,
        )
        .unwrap();
        assert_eq!(report.imported, vec!["users"]);

        let schema = SchemaRegistry::new(dst.path().join("registry"))
            .load("users")
            .unwrap();
        assert_eq!(schema.col_count(), 2);
    }
}
