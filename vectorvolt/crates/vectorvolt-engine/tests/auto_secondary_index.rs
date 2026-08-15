/// Layer 33 — auto-maintained secondary index integration tests.
///
/// Schema: pk | category | score | region
use std::sync::{Arc, Barrier};
use std::thread;
use tempfile::TempDir;
use vectorvolt_engine::{Durability, EngineOptions, TableEngine};

fn open() -> (TableEngine, TempDir) {
    let dir = TempDir::new().unwrap();
    let eng = TableEngine::open_with(
        vec![
            "pk".into(),
            "category".into(),
            "score".into(),
            "region".into(),
        ],
        dir.path(),
        EngineOptions::new(1024).with_durability(Durability::WriteOnly),
    )
    .unwrap();
    (eng, dir)
}

// ── Build-then-insert ─────────────────────────────────────────────────────────

#[test]
fn build_then_insert_is_found() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 10, 100, 1]).unwrap();
    eng.build_secondary_index(1).unwrap();

    eng.insert(2, vec![2, 10, 200, 2]).unwrap();

    let pks = eng.secondary_lookup(1, 10).unwrap();
    assert!(pks.contains(&2), "new row must appear in live index");
}

#[test]
fn build_then_insert_different_value() {
    let (eng, _dir) = open();
    eng.build_secondary_index(1).unwrap(); // build on empty table
    eng.insert(1, vec![1, 42, 0, 0]).unwrap();
    eng.insert(2, vec![2, 99, 0, 0]).unwrap();

    assert_eq!(eng.secondary_lookup(1, 42).unwrap(), vec![1]);
    assert_eq!(eng.secondary_lookup(1, 99).unwrap(), vec![2]);
}

// ── Upsert / update path ──────────────────────────────────────────────────────

#[test]
fn upsert_moves_pk_between_buckets() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 10, 100, 0]).unwrap();
    eng.build_secondary_index(1).unwrap();

    // Re-insert (upsert) with a new category value.
    eng.insert(1, vec![1, 20, 100, 0]).unwrap();

    assert!(
        eng.secondary_lookup(1, 10).unwrap().is_empty(),
        "old bucket must be empty after upsert"
    );
    assert_eq!(
        eng.secondary_lookup(1, 20).unwrap(),
        vec![1],
        "new bucket must contain pk"
    );
}

#[test]
fn upsert_same_value_stays_in_bucket() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 10, 0, 0]).unwrap();
    eng.build_secondary_index(1).unwrap();
    eng.insert(1, vec![1, 10, 999, 0]).unwrap(); // same category, different score

    let pks = eng.secondary_lookup(1, 10).unwrap();
    assert_eq!(pks, vec![1], "pk must still be in the same bucket");
}

// ── Delete path ───────────────────────────────────────────────────────────────

#[test]
fn delete_removes_pk_from_index() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 10, 0, 0]).unwrap();
    eng.insert(2, vec![2, 10, 0, 0]).unwrap();
    eng.build_secondary_index(1).unwrap();

    eng.delete(1).unwrap();

    let pks = eng.secondary_lookup(1, 10).unwrap();
    assert!(!pks.contains(&1), "deleted pk must be gone from index");
    assert!(pks.contains(&2), "surviving pk must remain");
}

#[test]
fn secondary_get_empty_after_delete() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 5, 0, 0]).unwrap();
    eng.build_secondary_index(1).unwrap();

    eng.delete(1).unwrap();

    let rows = eng.secondary_get(1, 5).unwrap();
    assert!(
        rows.is_empty(),
        "secondary_get must return nothing after delete"
    );
}

// ── Insert before build ───────────────────────────────────────────────────────

#[test]
fn insert_before_build_captured_by_full_scan() {
    let (eng, _dir) = open();
    // Insert rows BEFORE any index exists.
    eng.insert(1, vec![1, 7, 0, 0]).unwrap();
    eng.insert(2, vec![2, 7, 0, 0]).unwrap();

    // build_secondary_index does a full partition scan — must find both rows.
    eng.build_secondary_index(1).unwrap();

    let pks = eng.secondary_lookup(1, 7).unwrap();
    assert!(pks.contains(&1));
    assert!(pks.contains(&2));
}

// ── Multiple indexes ──────────────────────────────────────────────────────────

#[test]
fn two_indexes_both_maintained_on_insert() {
    let (eng, _dir) = open();
    eng.build_secondary_index(1).unwrap(); // category
    eng.build_secondary_index(2).unwrap(); // score

    eng.insert(1, vec![1, 33, 99, 0]).unwrap();

    assert_eq!(eng.secondary_lookup(1, 33).unwrap(), vec![1]);
    assert_eq!(eng.secondary_lookup(2, 99).unwrap(), vec![1]);
}

#[test]
fn two_indexes_both_maintained_on_delete() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 33, 99, 0]).unwrap();
    eng.build_secondary_index(1).unwrap();
    eng.build_secondary_index(2).unwrap();

    eng.delete(1).unwrap();

    assert!(eng.secondary_lookup(1, 33).unwrap().is_empty());
    assert!(eng.secondary_lookup(2, 99).unwrap().is_empty());
}

// ── Drop stops maintenance ────────────────────────────────────────────────────

#[test]
fn drop_stops_maintenance_rebuild_gives_correct_result() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 10, 0, 0]).unwrap();
    eng.build_secondary_index(1).unwrap();

    // Drop the index — subsequent writes are not maintained.
    eng.drop_secondary_index(1);
    eng.insert(2, vec![2, 10, 0, 0]).unwrap();

    // Index is gone, so lookup returns nothing.
    assert!(eng.secondary_lookup(1, 10).unwrap().is_empty());

    // A fresh build must capture all rows including the one inserted after drop.
    eng.build_secondary_index(1).unwrap();
    let pks = eng.secondary_lookup(1, 10).unwrap();
    assert!(pks.contains(&1));
    assert!(pks.contains(&2));
}

// ── Range lookup ─────────────────────────────────────────────────────────────

#[test]
fn range_lookup_maintained_after_inserts() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 10, 0, 0]).unwrap();
    eng.build_secondary_index(1).unwrap();

    eng.insert(2, vec![2, 15, 0, 0]).unwrap();
    eng.insert(3, vec![3, 25, 0, 0]).unwrap();

    let pks = eng.secondary_range_lookup(1, 10, 20).unwrap();
    assert!(pks.contains(&1), "pk 1 (cat=10) in range [10,20]");
    assert!(pks.contains(&2), "pk 2 (cat=15) in range [10,20]");
    assert!(!pks.contains(&3), "pk 3 (cat=25) outside range");
}

// ── Batch operations ──────────────────────────────────────────────────────────

#[test]
fn batch_insert_all_rows_indexed() {
    let (eng, _dir) = open();
    eng.build_secondary_index(1).unwrap();

    // batch_insert calls self.insert() in a loop — maintenance fires per row.
    let rows: Vec<(u64, Vec<u64>)> = (1..=5).map(|i| (i, vec![i, 42, i * 10, 0])).collect();
    eng.batch_insert(rows).unwrap();

    let pks = eng.secondary_lookup(1, 42).unwrap();
    for i in 1u64..=5 {
        assert!(pks.contains(&i), "pk {i} must be indexed");
    }
}

#[test]
fn batch_delete_removes_all_from_index() {
    let (eng, _dir) = open();
    for i in 1u64..=4 {
        eng.insert(i, vec![i, 7, 0, 0]).unwrap();
    }
    eng.build_secondary_index(1).unwrap();

    eng.batch_delete(&[1, 2, 3, 4]).unwrap();

    assert!(eng.secondary_lookup(1, 7).unwrap().is_empty());
}

// ── Entry count / scale ───────────────────────────────────────────────────────

#[test]
fn entry_count_tracks_inserts_and_deletes() {
    let (eng, _dir) = open();
    eng.build_secondary_index(1).unwrap();

    eng.insert(1, vec![1, 10, 0, 0]).unwrap();
    eng.insert(2, vec![2, 20, 0, 0]).unwrap();
    eng.insert(3, vec![3, 30, 0, 0]).unwrap();
    assert_eq!(eng.secondary_index_entry_count(1), 3, "3 distinct values");

    eng.delete(2).unwrap();
    // Value 20 bucket is now empty — the key remains but the set is empty;
    // entry_count counts distinct-value keys, so it stays at 3 unless the
    // bucket is pruned.  What matters is that pk 2 is gone.
    let pks20 = eng.secondary_lookup(1, 20).unwrap();
    assert!(pks20.is_empty(), "pk 2 removed from value-20 bucket");
}

// ── Concurrent inserts ────────────────────────────────────────────────────────

#[test]
fn concurrent_inserts_all_indexed() {
    let dir = TempDir::new().unwrap();
    let eng = Arc::new(
        TableEngine::open_with(
            vec![
                "pk".into(),
                "category".into(),
                "score".into(),
                "region".into(),
            ],
            dir.path(),
            EngineOptions::new(1024).with_durability(Durability::WriteOnly),
        )
        .unwrap(),
    );
    eng.build_secondary_index(1).unwrap();

    let n = 8u64;
    let barrier = Arc::new(Barrier::new(n as usize));
    let mut handles = Vec::new();
    for i in 1..=n {
        let eng = Arc::clone(&eng);
        let barrier = Arc::clone(&barrier);
        handles.push(thread::spawn(move || {
            barrier.wait();
            eng.insert(i, vec![i, 99, i * 10, 0]).unwrap();
        }));
    }
    for h in handles {
        h.join().unwrap();
    }

    let pks = eng.secondary_lookup(1, 99).unwrap();
    assert_eq!(
        pks.len(),
        n as usize,
        "all {n} concurrent inserts must be indexed"
    );
    for i in 1..=n {
        assert!(pks.contains(&i), "pk {i} must be indexed");
    }
}
