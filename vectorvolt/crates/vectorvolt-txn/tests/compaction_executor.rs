//! End-to-end compaction: plan, merge, swap, delete (VV-STO-015/026).
//!
//! The unit tests in `vectorvolt-storage::compaction` prove the merge is correct
//! in isolation. These prove the loop closes against a real manager: the manifest
//! reflects the swap, the PK index still resolves every surviving key, deleted
//! rows are actually reclaimed, and nothing is left on disk that should not be.

use tempfile::TempDir;
use vectorvolt_storage::{CompactionStrategy, WriteOptions, verify_partitions};
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
        1_000_000, // never auto-flush; tests flush explicitly
    )
}

fn vvsst_count(dir: &std::path::Path) -> usize {
    std::fs::read_dir(dir)
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|x| x == "vvsst"))
                .count()
        })
        .unwrap_or(0)
}

/// Six L0 partitions merge into one, and every key still reads back.
#[test]
fn compaction_merges_partitions_and_keeps_every_row_readable() {
    let dir = TempDir::new().unwrap();
    let m = manager(&dir, "t");
    m.enable_manifest().unwrap();

    for batch in 0..6u64 {
        for i in 0..20u64 {
            let pk = batch * 100 + i;
            m.insert(pk, vec![pk, pk * 7]).unwrap();
        }
        m.flush().unwrap();
    }
    assert_eq!(m.partitions().len(), 6);

    let outcomes = m
        .run_compaction(CompactionStrategy::default(), WriteOptions::default())
        .unwrap();

    assert_eq!(outcomes.len(), 1, "expected one L0 merge");
    let o = &outcomes[0];
    assert_eq!(o.rows_in, 120);
    assert_eq!(o.rows_out, 120, "no duplicates, so nothing to reclaim");

    // Manifest now names exactly the merged partition.
    let parts = m.partitions();
    assert_eq!(parts.len(), 1, "manifest still lists inputs: {parts:?}");
    assert_eq!(parts[0].partition_id, o.output.partition_id);
    assert_eq!(parts[0].level, 1, "output should be promoted to L1");
    assert_eq!(parts[0].row_count, 120);

    // Input files are gone; only the merged one remains.
    assert_eq!(vvsst_count(&dir.path().join("t")), 1);

    // And it verifies clean against the manifest.
    let report = verify_partitions(&parts).unwrap();
    assert!(report.is_clean(), "{:?}", report.findings);
}

/// **The PK index must still resolve every key after the merge.**
///
/// Compaction moves every row to a new `(file_id, partition_id, row_offset)`. An
/// index left pointing at the old coordinates addresses a file that was just
/// deleted — so this is the assertion that separates a working compaction from a
/// silently destructive one.
#[test]
fn pk_index_resolves_every_key_after_compaction() {
    let dir = TempDir::new().unwrap();
    let m = manager(&dir, "t");
    m.enable_manifest().unwrap();

    let mut expected: Vec<(u64, u64)> = Vec::new();
    for batch in 0..5u64 {
        for i in 0..30u64 {
            let pk = batch * 1000 + i;
            let amount = pk * 3 + 1;
            m.insert(pk, vec![pk, amount]).unwrap();
            expected.push((pk, amount));
        }
        m.flush().unwrap();
    }

    let outcomes = m
        .run_compaction(CompactionStrategy::default(), WriteOptions::default())
        .unwrap();
    assert_eq!(outcomes.len(), 1);

    let idx = m.pk_index.read();
    for (pk, amount) in &expected {
        let desc = idx
            .get(*pk)
            .unwrap_or_else(|| panic!("pk {pk} vanished from the index after compaction"));
        assert_eq!(
            desc.partition_id(),
            outcomes[0].output.partition_id,
            "pk {pk} still points at a deleted partition"
        );
        // Resolve through the descriptor to the real bytes.
        let reader = vectorvolt_storage::PartitionReader::open(&outcomes[0].output.path).unwrap();
        let row = desc.row_offset() as usize;
        assert_eq!(reader.read_value(0, row).unwrap(), *pk, "pk mismatch");
        assert_eq!(
            reader.read_value(1, row).unwrap(),
            *amount,
            "pk {pk} resolved to the wrong row"
        );
    }
}

/// Deleted rows are actually reclaimed, and stay deleted.
#[test]
fn deleted_rows_are_reclaimed_by_compaction() {
    let dir = TempDir::new().unwrap();
    let m = manager(&dir, "t");
    m.enable_manifest().unwrap();

    for batch in 0..5u64 {
        for i in 0..20u64 {
            let pk = batch * 100 + i;
            m.insert(pk, vec![pk, pk]).unwrap();
        }
        m.flush().unwrap();
    }
    // Delete half of them.
    let mut deleted = Vec::new();
    for batch in 0..5u64 {
        for i in 0..10u64 {
            let pk = batch * 100 + i;
            m.delete(pk).unwrap();
            deleted.push(pk);
        }
    }

    let outcomes = m
        .run_compaction(CompactionStrategy::default(), WriteOptions::default())
        .unwrap();
    let o = &outcomes[0];

    assert_eq!(o.rows_in, 100);
    assert_eq!(o.rows_out, 50, "deleted rows should not survive the merge");
    assert_eq!(o.rows_reclaimed(), 50);
    assert_eq!(m.partitions()[0].row_count, 50);

    // Deleted keys stay gone; survivors are still there.
    let idx = m.pk_index.read();
    for pk in &deleted {
        assert!(idx.get(*pk).is_none(), "deleted pk {pk} came back");
    }
    for batch in 0..5u64 {
        for i in 10..20u64 {
            let pk = batch * 100 + i;
            assert!(idx.get(pk).is_some(), "live pk {pk} was dropped");
        }
    }
}

/// Overwrites collapse to the newest value.
#[test]
fn superseded_rows_collapse_to_the_latest_write() {
    let dir = TempDir::new().unwrap();
    let m = manager(&dir, "t");
    m.enable_manifest().unwrap();

    // The same 25 keys written four times, each generation in its own partition.
    for generation in 1..=4u64 {
        for pk in 0..25u64 {
            m.insert(pk, vec![pk, generation * 1000 + pk]).unwrap();
        }
        m.flush().unwrap();
    }

    let outcomes = m
        .run_compaction(CompactionStrategy::default(), WriteOptions::default())
        .unwrap();
    let o = &outcomes[0];
    assert_eq!(o.rows_in, 100);
    assert_eq!(o.rows_out, 25, "four generations should collapse to one");

    let reader = vectorvolt_storage::PartitionReader::open(&o.output.path).unwrap();
    let idx = m.pk_index.read();
    for pk in 0..25u64 {
        let desc = idx.get(pk).unwrap();
        let row = desc.row_offset() as usize;
        assert_eq!(
            reader.read_value(1, row).unwrap(),
            4000 + pk,
            "pk {pk} kept a stale generation"
        );
    }
}

/// A WORM-locked partition is never rewritten.
///
/// The hold applies to those bytes; compaction destroys them. Filtered before
/// planning, so a locked partition is not even considered.
#[test]
fn worm_locked_partitions_are_excluded_from_compaction() {
    let dir = TempDir::new().unwrap();
    let m = manager(&dir, "t");
    m.enable_manifest().unwrap();

    for batch in 0..6u64 {
        for i in 0..10u64 {
            m.insert(batch * 100 + i, vec![batch * 100 + i, i]).unwrap();
        }
        m.flush().unwrap();
    }
    let locked = m.partitions()[0].clone();
    m.record(vectorvolt_storage::ManifestEdit::SetWormLock {
        file_id: locked.file_id,
        partition_id: locked.partition_id,
        locked: true,
    })
    .unwrap();

    m.run_compaction(CompactionStrategy::default(), WriteOptions::default())
        .unwrap();

    let after = m.partitions();
    assert!(
        after
            .iter()
            .any(|p| p.partition_id == locked.partition_id && p.worm_locked),
        "the WORM-locked partition was compacted away"
    );
    assert!(locked.path.exists(), "its file was deleted");
}

/// Compaction is idempotent: running it again when nothing qualifies is a no-op.
#[test]
fn a_second_run_with_nothing_to_do_changes_nothing() {
    let dir = TempDir::new().unwrap();
    let m = manager(&dir, "t");
    m.enable_manifest().unwrap();

    for batch in 0..6u64 {
        for i in 0..10u64 {
            m.insert(batch * 100 + i, vec![batch * 100 + i, i]).unwrap();
        }
        m.flush().unwrap();
    }
    assert_eq!(
        m.run_compaction(CompactionStrategy::default(), WriteOptions::default())
            .unwrap()
            .len(),
        1
    );
    let after_first = m.partitions();

    let second = m
        .run_compaction(CompactionStrategy::default(), WriteOptions::default())
        .unwrap();
    assert!(
        second.is_empty(),
        "a single L1 partition should not requalify"
    );
    assert_eq!(m.partitions(), after_first, "state changed on a no-op run");
    assert_eq!(vvsst_count(&dir.path().join("t")), 1);
}

/// No manifest means no compaction, rather than a guess.
#[test]
fn compaction_requires_a_manifest() {
    let dir = TempDir::new().unwrap();
    let m = manager(&dir, "t");
    for i in 0..50u64 {
        m.insert(i, vec![i, i]).unwrap();
    }
    m.flush().unwrap();
    let out = m
        .run_compaction(CompactionStrategy::default(), WriteOptions::default())
        .unwrap();
    assert!(
        out.is_empty(),
        "compacted without level or size information"
    );
}

/// The compacted state survives a restart.
///
/// The manifest swap is what makes this true — a restart replays it and sees only
/// the merged partition, never the inputs alongside it.
#[test]
fn compacted_state_survives_a_restart() {
    let dir = TempDir::new().unwrap();
    let flush_dir = dir.path().join("t");
    let merged_id;
    {
        let m = manager(&dir, "t");
        m.enable_manifest().unwrap();
        for batch in 0..5u64 {
            for i in 0..20u64 {
                m.insert(batch * 100 + i, vec![batch * 100 + i, i]).unwrap();
            }
            m.flush().unwrap();
        }
        let out = m
            .run_compaction(CompactionStrategy::default(), WriteOptions::default())
            .unwrap();
        merged_id = out[0].output.partition_id;
    }

    let wal_dir = dir.path().join("t2-wal");
    std::fs::create_dir_all(&wal_dir).unwrap();
    let cfg = WalConfig::default().with_dir(&wal_dir).with_fsync(false);
    let m2 = TxnManager::new(
        WalWriter::open(cfg).unwrap(),
        vec!["pk".into(), "amount".into()],
        &flush_dir,
        1_000_000,
    );
    let rec = m2.enable_manifest().unwrap();
    assert!(rec.is_clean(), "{rec:?}");

    let parts = m2.partitions();
    assert_eq!(
        parts.len(),
        1,
        "restart saw the pre-compaction set: {parts:?}"
    );
    assert_eq!(parts[0].partition_id, merged_id);
    assert_eq!(parts[0].row_count, 100);
    assert!(
        verify_partitions(&parts).unwrap().is_clean(),
        "the surviving partition does not verify"
    );
}

/// Compaction can compress its output, which is the natural place to do it: a
/// compacted partition is colder than a freshly flushed one.
#[test]
fn compaction_can_write_a_compressed_output() {
    let dir = TempDir::new().unwrap();
    let m = manager(&dir, "t");
    m.enable_manifest().unwrap();

    for batch in 0..5u64 {
        for i in 0..200u64 {
            let pk = batch * 1000 + i;
            m.insert(pk, vec![pk, i % 8]).unwrap();
        }
        m.flush().unwrap();
    }
    let bytes_before: u64 = m.partitions().iter().map(|p| p.size_bytes).sum();

    let out = m
        .run_compaction(
            CompactionStrategy::default(),
            WriteOptions::new().with_lz4(),
        )
        .unwrap();
    assert_eq!(out.len(), 1);
    assert!(
        out[0].bytes_out < bytes_before,
        "compressed merge {} not smaller than inputs {bytes_before}",
        out[0].bytes_out
    );

    // Still fully readable, and the index still resolves.
    let reader = vectorvolt_storage::PartitionReader::open(&out[0].output.path).unwrap();
    let idx = m.pk_index.read();
    for batch in 0..5u64 {
        for i in 0..200u64 {
            let pk = batch * 1000 + i;
            let row = idx.get(pk).unwrap().row_offset() as usize;
            assert_eq!(reader.read_value(0, row).unwrap(), pk);
        }
    }
}
