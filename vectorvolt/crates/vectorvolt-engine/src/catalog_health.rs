//! Catalog health check — aggregate validate + snapshot freshness (VV-CTL-009).
//!
//! [`check_health`] combines three independent signals into a single
//! [`HealthReport`]:
//!
//! 1. **Schema validation** — runs [`validate_catalog`] and surfaces errors /
//!    warnings / ghost entries / orphan dirs.
//! 2. **Snapshot freshness** — inspects the newest snapshot in `snapshots_dir`.
//!    When `max_snapshot_age_secs` is set, a stale (or missing) snapshot is
//!    treated as unhealthy.
//! 3. **Baseline drift** (optional) — if a [`CatalogExport`] baseline is
//!    supplied, runs [`diff_catalogs`] and flags any divergence as unhealthy.

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::catalog_diff::{CatalogDiff, diff_catalogs};
use crate::catalog_export::CatalogExport;
use crate::catalog_snapshot::list_snapshots;
use crate::catalog_validate::{CatalogValidationReport, validate_catalog};
use crate::error::EngineResult;

// ── Report ────────────────────────────────────────────────────────────────────

/// Freshness verdict for the snapshot store.
#[derive(Debug, Clone, PartialEq)]
pub enum SnapshotFreshness {
    /// No max-age constraint was given — freshness is not checked.
    Unchecked,
    /// Newest snapshot is within the allowed age.
    Fresh { age_secs: u64 },
    /// Newest snapshot is older than the allowed max age.
    Stale { age_secs: u64, max_age_secs: u64 },
    /// No snapshot exists and a max-age constraint was given.
    Missing,
}

/// Aggregated health report returned by [`check_health`].
#[derive(Debug)]
pub struct HealthReport {
    pub validation: CatalogValidationReport,
    pub snapshot_count: usize,
    pub freshness: SnapshotFreshness,
    /// Present when a baseline export was supplied and differs from the live catalog.
    pub baseline_drift: Option<CatalogDiff>,
}

impl HealthReport {
    /// `true` when all checks pass (validation healthy, snapshot fresh, no baseline drift).
    pub fn is_healthy(&self) -> bool {
        if !self.validation.is_healthy() {
            return false;
        }
        match &self.freshness {
            SnapshotFreshness::Stale { .. } | SnapshotFreshness::Missing => return false,
            _ => {}
        }
        if self.baseline_drift.as_ref().is_some_and(|d| !d.is_empty()) {
            return false;
        }
        true
    }

    /// `true` when healthy AND no validation warnings AND no baseline drift.
    pub fn is_clean(&self) -> bool {
        self.is_healthy() && self.validation.is_clean()
    }
}

// ── Check ─────────────────────────────────────────────────────────────────────

/// Run the aggregate health check.
///
/// - `max_snapshot_age_secs`: when `Some`, the newest snapshot must be no older
///   than this many seconds; when `None`, freshness is not checked.
/// - `baseline`: when `Some`, the live export is diffed against this snapshot;
///   any divergence marks the report as unhealthy.
pub fn check_health(
    catalog_dir: impl AsRef<Path>,
    registry_dir: impl AsRef<Path>,
    snapshots_dir: impl AsRef<Path>,
    max_snapshot_age_secs: Option<u64>,
    baseline: Option<&CatalogExport>,
) -> EngineResult<HealthReport> {
    // 1. Schema validation.
    let validation = validate_catalog(catalog_dir, &registry_dir)?;

    // 2. Snapshot freshness.
    let snapshots = list_snapshots(snapshots_dir)?;
    let snapshot_count = snapshots.len();

    let freshness = if let Some(max_age) = max_snapshot_age_secs {
        if let Some(newest) = snapshots.last() {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let age = now.saturating_sub(newest.created_at);
            if age <= max_age {
                SnapshotFreshness::Fresh { age_secs: age }
            } else {
                SnapshotFreshness::Stale {
                    age_secs: age,
                    max_age_secs: max_age,
                }
            }
        } else {
            SnapshotFreshness::Missing
        }
    } else {
        SnapshotFreshness::Unchecked
    };

    // 3. Baseline drift.
    let baseline_drift = if let Some(base) = baseline {
        let live = crate::catalog_export::export_catalog(&registry_dir)?;
        let diff = diff_catalogs(base, &live);
        Some(diff)
    } else {
        None
    };

    Ok(HealthReport {
        validation,
        snapshot_count,
        freshness,
        baseline_drift,
    })
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::catalog_snapshot::save_snapshot;
    use crate::schema::{ColumnDef, TableSchema};
    use crate::shard_catalog::ShardCatalog;

    fn cat_reg(dir: &TempDir) -> (std::path::PathBuf, std::path::PathBuf) {
        (dir.path().join("catalog"), dir.path().join("registry"))
    }

    fn create_table(dir: &TempDir, name: &str) {
        let (cat, reg) = cat_reg(dir);
        let catalog = ShardCatalog::with_registry(&cat, &reg).unwrap();
        let schema = TableSchema::new(vec![ColumnDef::u64("pk"), ColumnDef::u64("val")]).unwrap();
        catalog.create_table(name, schema, 1024, 1).unwrap();
    }

    fn snaps(dir: &TempDir) -> std::path::PathBuf {
        dir.path().join("snapshots")
    }

    #[test]
    fn empty_catalog_is_healthy() {
        let dir = TempDir::new().unwrap();
        let (cat, reg) = cat_reg(&dir);
        std::fs::create_dir_all(&cat).unwrap();
        let report = check_health(&cat, &reg, snaps(&dir), None, None).unwrap();
        assert!(report.is_healthy());
    }

    #[test]
    fn valid_table_is_healthy() {
        let dir = TempDir::new().unwrap();
        create_table(&dir, "orders");
        let (cat, reg) = cat_reg(&dir);
        let report = check_health(&cat, &reg, snaps(&dir), None, None).unwrap();
        assert!(report.is_healthy());
        assert_eq!(report.validation.valid_tables.len(), 1);
    }

    #[test]
    fn snapshot_count_reported() {
        let dir = TempDir::new().unwrap();
        create_table(&dir, "t");
        let (cat, reg) = cat_reg(&dir);
        let snaps_dir = snaps(&dir);
        save_snapshot(&reg, &snaps_dir).unwrap();
        let report = check_health(&cat, &reg, &snaps_dir, None, None).unwrap();
        assert_eq!(report.snapshot_count, 1);
    }

    #[test]
    fn no_max_age_freshness_is_unchecked() {
        let dir = TempDir::new().unwrap();
        let (cat, reg) = cat_reg(&dir);
        std::fs::create_dir_all(&cat).unwrap();
        let report = check_health(&cat, &reg, snaps(&dir), None, None).unwrap();
        assert_eq!(report.freshness, SnapshotFreshness::Unchecked);
    }

    #[test]
    fn missing_snapshot_with_max_age_is_unhealthy() {
        let dir = TempDir::new().unwrap();
        let (cat, reg) = cat_reg(&dir);
        std::fs::create_dir_all(&cat).unwrap();
        let report = check_health(&cat, &reg, snaps(&dir), Some(3600), None).unwrap();
        assert!(!report.is_healthy());
        assert_eq!(report.freshness, SnapshotFreshness::Missing);
    }

    #[test]
    fn fresh_snapshot_within_max_age_is_healthy() {
        let dir = TempDir::new().unwrap();
        create_table(&dir, "t");
        let (cat, reg) = cat_reg(&dir);
        let snaps_dir = snaps(&dir);
        save_snapshot(&reg, &snaps_dir).unwrap();
        // max age 1 hour — just-created snapshot is always within this
        let report = check_health(&cat, &reg, &snaps_dir, Some(3600), None).unwrap();
        assert!(report.is_healthy());
        assert!(matches!(report.freshness, SnapshotFreshness::Fresh { .. }));
    }
}
