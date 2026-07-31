//! The manifest, driven by real flushes (VV-STO-025).
//!
//! The unit tests in `vectorvolt-storage::manifest` prove the log is crash-safe
//! in isolation. These prove the loop actually closes: a flush records a
//! partition, a restart recovers it, and the three planning layers that had no
//! input before can now consume real output.

use tempfile::TempDir;
use vectorvolt_storage::{
    CompactionStrategy, ManifestEdit, RetentionPolicy, RetentionRule, verify_partitions,
};
use vectorvolt_txn::TxnManager;
use vectorvolt_wal::{WalConfig, WalWriter};

fn manager(dir: &TempDir, sub: &str) -> TxnManager {
    let wal_dir = dir.path().join(format!("{sub}-wal"));
    std::fs::create_dir_all(&wal_dir).unwrap();
    let cfg = WalConfig::default().with_dir(&wal_dir).with_fsync(false);
    TxnManager::new(
        WalWriter::open(cfg).unwrap(),
        vec!["pk".into(), "amount".into()],
        dir.path().join(sub),
        100,
    )
}

fn fill(m: &TxnManager, base: u64, n: u64) {
    for i in base..base + n {
        m.insert(i, vec![i, i * 10]).unwrap();
    }
}

/// A flush records a partition whose metadata matches what was written.
#[test]
fn flush_records_accurate_partition_metadata() {
    let dir = TempDir::new().unwrap();
    let m = manager(&dir, "t");
    m.enable_manifest().unwrap();

    fill(&m, 100, 50);
    let result = m.flush().unwrap().expect("a flush should have happened");

    let parts = m.partitions();
    assert_eq!(parts.len(), 1);
    let p = &parts[0];

    assert_eq!(p.partition_id, result.partition_id);
    assert_eq!(p.row_count, 50);
    assert_eq!(p.path, result.path);
    assert_eq!(p.level, 0, "a fresh flush is L0");
    assert_eq!(p.tombstones, 0);
    assert!(!p.worm_locked);
    // Key range must be the real PK range, not a placeholder — the ZoneMap and
    // leveled compaction both make skip decisions from it.
    assert_eq!(p.min_key, 100);
    assert_eq!(p.max_key, 149);
    // Size must match the file on disk, since retention budgets are byte-based.
    let on_disk = std::fs::metadata(&p.path).unwrap().len();
    assert_eq!(
        p.size_bytes, on_disk,
        "recorded size disagrees with the file"
    );
    assert!(p.created_at_millis > 1_600_000_000_000, "timestamp not set");
}

/// The manifest survives a restart and reports the same partitions.
#[test]
fn partitions_survive_a_restart() {
    let dir = TempDir::new().unwrap();
    let flush_dir = dir.path().join("t");

    let before = {
        let m = manager(&dir, "t");
        m.enable_manifest().unwrap();
        for batch in 0..4 {
            fill(&m, batch * 1000, 30);
            m.flush().unwrap();
        }
        m.partitions()
    };
    assert_eq!(before.len(), 4);

    // A fresh manager over the same directory — nothing shared in memory.
    let wal_dir = dir.path().join("t2-wal");
    std::fs::create_dir_all(&wal_dir).unwrap();
    let cfg = WalConfig::default().with_dir(&wal_dir).with_fsync(false);
    let m2 = TxnManager::new(
        WalWriter::open(cfg).unwrap(),
        vec!["pk".into(), "amount".into()],
        &flush_dir,
        100,
    );
    let recovery = m2.enable_manifest().unwrap();
    assert!(
        recovery.is_clean(),
        "clean shutdown should discard nothing: {recovery:?}"
    );
    assert_eq!(recovery.edits_applied, 4);
    assert_eq!(m2.partitions(), before, "state changed across restart");
}

/// Without `enable_manifest`, nothing is recorded and nothing breaks.
///
/// The manifest is opt-in, so every existing caller must keep working untouched.
#[test]
fn manifest_is_optional_and_absent_by_default() {
    let dir = TempDir::new().unwrap();
    let m = manager(&dir, "t");
    fill(&m, 0, 40);
    let r = m.flush().unwrap();
    assert!(r.is_some(), "flush must still work without a manifest");
    assert!(m.partitions().is_empty());
    assert!(
        !m.record(ManifestEdit::Remove {
            file_id: 0,
            partition_id: 0
        })
        .unwrap()
    );
    // No MANIFEST file should have been created.
    assert!(!dir.path().join("t").join("MANIFEST").exists());
}

/// **The point of the whole module**: the three planning layers now run on real
/// data instead of hand-built fixtures.
///
/// Before this, `CompactionStrategy::plan`, `RetentionPolicy::plan`, and
/// `verify_partitions` were correct and untestable against anything the engine
/// actually produced, because nothing tracked levels, sizes, or creation times.
#[test]
fn planning_layers_consume_real_flush_output() {
    let dir = TempDir::new().unwrap();
    let m = manager(&dir, "t");
    m.enable_manifest().unwrap();

    for batch in 0..6 {
        fill(&m, batch * 1000, 25);
        m.flush().unwrap();
    }
    let parts = m.partitions();
    assert_eq!(parts.len(), 6);

    // Compaction: 6 L0 partitions is past the default trigger of 4.
    let jobs = CompactionStrategy::default().plan(&parts);
    assert_eq!(jobs.len(), 1, "expected one L0 merge, got {jobs:?}");
    assert_eq!(jobs[0].inputs.len(), 6, "all of L0 merges together");
    assert_eq!(jobs[0].output_level, 1);
    assert_eq!(jobs[0].input_rows(), 150);

    // Verification: the files really exist, so this must come back CLEAN. Every
    // earlier verify test ran against paths that did not exist and could only
    // check that findings were produced.
    let report = verify_partitions(&parts).unwrap();
    assert_eq!(report.checked, 6);
    assert!(
        report.is_clean(),
        "real flushed partitions should verify clean: {:?}",
        report.findings
    );

    // Retention: byte budget over real file sizes.
    let total: u64 = parts.iter().map(|p| p.size_bytes).sum();
    let plan = RetentionPolicy::new()
        .with_rule(RetentionRule::MaxTotalBytes(total / 2))
        .plan(&parts, u64::MAX / 2);
    assert!(
        !plan.delete.is_empty(),
        "half the budget should delete some"
    );
    assert!(
        plan.delete.len() < parts.len(),
        "retention must not select everything"
    );
}

/// Statistics updates flow through to the planners.
///
/// Marking a partition as mostly tombstones must change the compaction decision —
/// this is the write-back half of the loop, not just the read half.
#[test]
fn tombstone_updates_change_the_compaction_plan() {
    let dir = TempDir::new().unwrap();
    let m = manager(&dir, "t");
    m.enable_manifest().unwrap();

    fill(&m, 0, 60);
    let r = m.flush().unwrap().unwrap();

    // One L0 partition: below the trigger, so nothing to do.
    assert!(
        CompactionStrategy::default()
            .plan(&m.partitions())
            .is_empty()
    );

    // Now declare most of its rows dead.
    assert!(
        m.record(ManifestEdit::SetTombstones {
            file_id: r.file_id,
            partition_id: r.partition_id,
            tombstones: 55,
        })
        .unwrap()
    );

    let jobs = CompactionStrategy::default().plan(&m.partitions());
    assert_eq!(jobs.len(), 1, "a mostly-dead partition should be collected");
    assert_eq!(
        jobs[0].reason,
        vectorvolt_storage::CompactionReason::GarbageCollection
    );

    // And the update is durable.
    drop(m);
    let wal_dir = dir.path().join("t3-wal");
    std::fs::create_dir_all(&wal_dir).unwrap();
    let cfg = WalConfig::default().with_dir(&wal_dir).with_fsync(false);
    let m2 = TxnManager::new(
        WalWriter::open(cfg).unwrap(),
        vec!["pk".into(), "amount".into()],
        dir.path().join("t"),
        100,
    );
    m2.enable_manifest().unwrap();
    assert_eq!(m2.partitions()[0].tombstones, 55, "update not durable");
}

/// A WORM lock recorded through the manifest is honoured by retention after a
/// restart.
///
/// The compliance path end to end: set the hold, restart, confirm the policy
/// still refuses to delete it.
#[test]
fn worm_lock_persists_and_blocks_retention_after_restart() {
    let dir = TempDir::new().unwrap();
    let locked_id;
    {
        let m = manager(&dir, "t");
        m.enable_manifest().unwrap();
        for batch in 0..3 {
            fill(&m, batch * 1000, 20);
            m.flush().unwrap();
        }
        let first = m.partitions()[0].clone();
        locked_id = first.partition_id;
        m.record(ManifestEdit::SetWormLock {
            file_id: first.file_id,
            partition_id: locked_id,
            locked: true,
        })
        .unwrap();
    }

    let wal_dir = dir.path().join("t4-wal");
    std::fs::create_dir_all(&wal_dir).unwrap();
    let cfg = WalConfig::default().with_dir(&wal_dir).with_fsync(false);
    let m2 = TxnManager::new(
        WalWriter::open(cfg).unwrap(),
        vec!["pk".into(), "amount".into()],
        dir.path().join("t"),
        100,
    );
    m2.enable_manifest().unwrap();

    let parts = m2.partitions();
    assert!(
        parts
            .iter()
            .any(|p| p.partition_id == locked_id && p.worm_locked),
        "the WORM lock did not survive the restart"
    );

    // A policy that would otherwise delete everything old.
    let plan = RetentionPolicy::new()
        .with_rule(RetentionRule::MaxAgeMillis(0))
        .plan(&parts, u64::MAX / 2);
    assert!(
        !plan
            .delete
            .iter()
            .any(|a| a.partition.partition_id == locked_id),
        "a WORM-locked partition was scheduled for deletion after restart"
    );
    assert!(
        plan.retained
            .iter()
            .any(|(p, _)| p.partition_id == locked_id),
        "the veto was not reported"
    );
}
