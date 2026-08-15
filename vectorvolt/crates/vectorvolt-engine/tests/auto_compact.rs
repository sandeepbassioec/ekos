/// Layer 40 — auto-compaction trigger.
///
/// Verifies that setting `auto_compact_threshold` causes the engine to compact
/// automatically after a flush brings the partition count to or above the
/// threshold, and that all data remains correct and accessible afterwards.
use tempfile::TempDir;
use vectorvolt_engine::{Durability, EngineOptions, TableEngine};

fn open_with_threshold(threshold: usize) -> (TableEngine, TempDir) {
    let dir = TempDir::new().unwrap();
    let eng = TableEngine::open_with(
        vec!["pk".into(), "val".into()],
        dir.path(),
        EngineOptions::new(1)
            .with_durability(Durability::WriteOnly)
            .with_auto_compact(threshold),
    )
    .unwrap();
    (eng, dir)
}

fn open_no_auto() -> (TableEngine, TempDir) {
    let dir = TempDir::new().unwrap();
    let eng = TableEngine::open_with(
        vec!["pk".into(), "val".into()],
        dir.path(),
        EngineOptions::new(1).with_durability(Durability::WriteOnly),
    )
    .unwrap();
    (eng, dir)
}

// ── threshold configuration ───────────────────────────────────────────────────

#[test]
fn default_options_no_auto_compact_threshold() {
    let (eng, _dir) = open_no_auto();
    assert_eq!(eng.stats().auto_compact_threshold, None);
}

#[test]
fn with_auto_compact_sets_threshold_in_stats() {
    let (eng, _dir) = open_with_threshold(4);
    assert_eq!(eng.stats().auto_compact_threshold, Some(4));
}

#[test]
fn threshold_clamped_to_at_least_one() {
    let dir = TempDir::new().unwrap();
    let eng = TableEngine::open_with(
        vec!["pk".into(), "val".into()],
        dir.path(),
        EngineOptions::new(1)
            .with_durability(Durability::WriteOnly)
            .with_auto_compact(0),
    )
    .unwrap();
    assert_eq!(eng.stats().auto_compact_threshold, Some(1));
}

// ── auto-compact fires ────────────────────────────────────────────────────────

#[test]
fn partition_count_stays_below_threshold_after_flush() {
    // threshold=3: after 3 partitions form, they should be compacted to 1.
    let (eng, _dir) = open_with_threshold(3);

    eng.insert(1, vec![1, 10]).unwrap(); // flush_threshold=1 → auto-flush → 1 partition
    eng.insert(2, vec![2, 20]).unwrap(); // 2 partitions — below threshold
    assert_eq!(
        eng.stats().partition_count,
        2,
        "under threshold, no compact yet"
    );

    eng.insert(3, vec![3, 30]).unwrap(); // 3rd partition triggers auto-compact
    assert!(
        eng.stats().partition_count < 3,
        "auto-compact must reduce partition count; got {}",
        eng.stats().partition_count
    );
}

#[test]
fn threshold_two_compacts_on_second_partition() {
    let (eng, _dir) = open_with_threshold(2);

    eng.insert(1, vec![1, 10]).unwrap(); // 1 partition — no compact
    assert_eq!(eng.stats().partition_count, 1);

    eng.insert(2, vec![2, 20]).unwrap(); // 2nd partition → threshold reached → compact
    assert!(
        eng.stats().partition_count < 2,
        "threshold=2 must compact after 2nd partition"
    );
}

// ── no auto-compact without option ───────────────────────────────────────────

#[test]
fn without_threshold_partitions_accumulate() {
    let (eng, _dir) = open_no_auto();

    eng.insert(1, vec![1, 10]).unwrap();
    eng.insert(2, vec![2, 20]).unwrap();
    eng.insert(3, vec![3, 30]).unwrap();

    assert_eq!(
        eng.stats().partition_count,
        3,
        "without auto_compact partitions must accumulate"
    );
}

// ── data correctness after auto-compact ──────────────────────────────────────

#[test]
fn get_returns_correct_rows_after_auto_compact() {
    let (eng, _dir) = open_with_threshold(2);

    eng.insert(1, vec![1, 111]).unwrap();
    eng.insert(2, vec![2, 222]).unwrap(); // triggers compact

    assert_eq!(eng.get(1).unwrap().unwrap().values[1], 111);
    assert_eq!(eng.get(2).unwrap().unwrap().values[1], 222);
}

#[test]
fn scan_correct_after_auto_compact() {
    let (eng, _dir) = open_with_threshold(2);

    eng.insert(1, vec![1, 5]).unwrap();
    eng.insert(2, vec![2, 5]).unwrap(); // compact fires

    let rows = eng.scan(1, 5, 5).unwrap();
    let mut pks: Vec<u64> = rows.iter().map(|r| r.pk).collect();
    pks.sort_unstable();
    assert_eq!(pks, vec![1, 2]);
}

#[test]
fn count_range_correct_after_auto_compact() {
    let (eng, _dir) = open_with_threshold(2);

    for i in 1u64..=5 {
        eng.insert(i, vec![i, i * 10]).unwrap();
    }

    let n = eng.count_range(0, 1, 5).unwrap();
    assert_eq!(
        n, 5,
        "all 5 rows must be counted after repeated auto-compacts"
    );
}

#[test]
fn updated_row_correct_after_auto_compact() {
    let (eng, _dir) = open_with_threshold(2);

    eng.insert(1, vec![1, 100]).unwrap();
    eng.insert(2, vec![2, 200]).unwrap(); // compact → 1 partition
    eng.update_field(1, 1, 999).unwrap(); // writes to buffer, another flush
    eng.insert(3, vec![3, 300]).unwrap(); // may trigger another compact

    assert_eq!(
        eng.get(1).unwrap().unwrap().values[1],
        999,
        "update survived compaction"
    );
}

// ── explicit flush trigger ────────────────────────────────────────────────────

#[test]
fn explicit_flush_also_triggers_auto_compact() {
    let dir = TempDir::new().unwrap();
    // flush_threshold is high so auto-flush won't fire; we flush manually.
    let eng = TableEngine::open_with(
        vec!["pk".into(), "val".into()],
        dir.path(),
        EngineOptions::new(1000)
            .with_durability(Durability::WriteOnly)
            .with_auto_compact(2),
    )
    .unwrap();

    // First batch → flush → 1 partition.
    eng.insert(1, vec![1, 10]).unwrap();
    eng.insert(2, vec![2, 20]).unwrap();
    eng.flush().unwrap();
    assert_eq!(eng.stats().partition_count, 1);

    // Second batch → flush → 2 partitions → auto-compact fires.
    eng.insert(3, vec![3, 30]).unwrap();
    eng.flush().unwrap();
    assert!(
        eng.stats().partition_count < 2,
        "explicit flush must honour auto-compact threshold"
    );
}

// ── row count stable ─────────────────────────────────────────────────────────

#[test]
fn row_count_stable_after_multiple_auto_compacts() {
    let (eng, _dir) = open_with_threshold(2);

    for i in 1u64..=6 {
        eng.insert(i, vec![i, i]).unwrap();
    }

    // After any number of compactions, total distinct rows must still equal 6.
    let n = eng.count_range(0, 1, 6).unwrap();
    assert_eq!(n, 6);
}

// ── scan_deduped after auto-compact ──────────────────────────────────────────

#[test]
fn distinct_values_correct_after_auto_compact() {
    let (eng, _dir) = open_with_threshold(2);

    eng.insert(1, vec![1, 7]).unwrap();
    eng.insert(2, vec![2, 7]).unwrap(); // compact
    eng.insert(3, vec![3, 9]).unwrap();
    eng.insert(4, vec![4, 9]).unwrap(); // compact again

    let vals = eng.distinct_values(1, 1, u64::MAX).unwrap();
    assert_eq!(vals, vec![7, 9]);
}

#[test]
fn min_range_correct_after_auto_compact() {
    let (eng, _dir) = open_with_threshold(2);

    eng.insert(1, vec![1, 50]).unwrap();
    eng.insert(2, vec![2, 30]).unwrap(); // compact fires
    eng.insert(3, vec![3, 70]).unwrap();
    eng.insert(4, vec![4, 10]).unwrap(); // compact fires again

    let m = eng.min_range(1, 0, u64::MAX).unwrap();
    assert_eq!(m, Some(10), "min must be 10 after compactions");
}

#[test]
fn sum_range_correct_after_auto_compact() {
    let (eng, _dir) = open_with_threshold(2);

    eng.insert(1, vec![1, 10]).unwrap();
    eng.insert(2, vec![2, 20]).unwrap();
    eng.insert(3, vec![3, 30]).unwrap();
    eng.insert(4, vec![4, 40]).unwrap();

    let s = eng.sum_range(1, 1, u64::MAX).unwrap();
    assert_eq!(s, 100, "10+20+30+40 after repeated compacts");
}
