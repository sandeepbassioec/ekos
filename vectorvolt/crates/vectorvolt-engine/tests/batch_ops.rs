/// Layer 26 — batch operation integration tests.
///
/// `batch_get`, `batch_insert`, and `batch_delete` each process multiple
/// rows in a single engine call, eliminating per-row round-trip overhead.
use tempfile::TempDir;
use vectorvolt_engine::{Durability, EngineOptions, TableEngine};

fn open() -> (TableEngine, TempDir) {
    let dir = TempDir::new().unwrap();
    let eng = TableEngine::open_with(
        vec!["pk".into(), "val".into(), "tag".into()],
        dir.path(),
        EngineOptions::new(1024).with_durability(Durability::WriteOnly),
    )
    .unwrap();
    (eng, dir)
}

fn seed(eng: &TableEngine, n: u64) {
    for pk in 1..=n {
        eng.insert(pk, vec![pk, pk * 10, pk % 5]).unwrap();
    }
}

// ── batch_get ─────────────────────────────────────────────────────────────────

#[test]
fn batch_get_all_present() {
    let (eng, _dir) = open();
    seed(&eng, 5);
    let results = eng.batch_get(&[1, 2, 3, 4, 5]).unwrap();
    assert_eq!(results.len(), 5);
    for (i, row) in results.iter().enumerate() {
        let pk = (i + 1) as u64;
        let row = row.as_ref().unwrap();
        assert_eq!(row.pk, pk);
        assert_eq!(row.values[1], pk * 10);
    }
}

#[test]
fn batch_get_preserves_input_order() {
    let (eng, _dir) = open();
    seed(&eng, 5);
    let results = eng.batch_get(&[5, 3, 1]).unwrap();
    assert_eq!(results[0].as_ref().unwrap().pk, 5);
    assert_eq!(results[1].as_ref().unwrap().pk, 3);
    assert_eq!(results[2].as_ref().unwrap().pk, 1);
}

#[test]
fn batch_get_missing_pks_return_none() {
    let (eng, _dir) = open();
    seed(&eng, 3);
    let results = eng.batch_get(&[2, 99, 3, 100]).unwrap();
    assert!(results[0].is_some());
    assert!(results[1].is_none(), "pk=99 should be None");
    assert!(results[2].is_some());
    assert!(results[3].is_none(), "pk=100 should be None");
}

#[test]
fn batch_get_empty_input_returns_empty() {
    let (eng, _dir) = open();
    seed(&eng, 3);
    assert!(eng.batch_get(&[]).unwrap().is_empty());
}

#[test]
fn batch_get_all_missing() {
    let (eng, _dir) = open();
    let results = eng.batch_get(&[10, 20, 30]).unwrap();
    assert!(results.iter().all(|r| r.is_none()));
}

#[test]
fn batch_get_duplicate_pks_each_resolved() {
    let (eng, _dir) = open();
    seed(&eng, 2);
    let results = eng.batch_get(&[1, 1, 2]).unwrap();
    assert_eq!(results.len(), 3);
    assert_eq!(results[0].as_ref().unwrap().pk, 1);
    assert_eq!(results[1].as_ref().unwrap().pk, 1);
    assert_eq!(results[2].as_ref().unwrap().pk, 2);
}

// ── batch_insert ──────────────────────────────────────────────────────────────

#[test]
fn batch_insert_writes_all_rows() {
    let (eng, _dir) = open();
    let rows: Vec<(u64, Vec<u64>)> = (1u64..=5).map(|pk| (pk, vec![pk, pk * 100, 0])).collect();
    let count = eng.batch_insert(rows).unwrap();
    assert_eq!(count, 5);
    for pk in 1u64..=5 {
        assert_eq!(eng.get(pk).unwrap().unwrap().values[1], pk * 100);
    }
}

#[test]
fn batch_insert_overwrites_existing() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 10, 0]).unwrap();
    let count = eng
        .batch_insert(vec![(1u64, vec![1u64, 99u64, 0u64])])
        .unwrap();
    assert_eq!(count, 1);
    assert_eq!(eng.get(1).unwrap().unwrap().values[1], 99);
}

#[test]
fn batch_insert_empty_input_returns_zero() {
    let (eng, _dir) = open();
    let count = eng.batch_insert(Vec::<(u64, Vec<u64>)>::new()).unwrap();
    assert_eq!(count, 0);
}

#[test]
fn batch_insert_schema_mismatch_returns_error() {
    let (eng, _dir) = open();
    // schema has 3 cols; passing 1 value should fail
    let result = eng.batch_insert(vec![(1, vec![1])]);
    assert!(result.is_err(), "schema mismatch must return Err");
}

// ── batch_delete ──────────────────────────────────────────────────────────────

#[test]
fn batch_delete_removes_existing_rows() {
    let (eng, _dir) = open();
    seed(&eng, 5);
    let deleted = eng.batch_delete(&[1, 2, 3]).unwrap();
    assert_eq!(deleted, 3);
    assert!(eng.get(1).unwrap().is_none());
    assert!(eng.get(2).unwrap().is_none());
    assert!(eng.get(3).unwrap().is_none());
    assert!(eng.get(4).unwrap().is_some()); // untouched
    assert!(eng.get(5).unwrap().is_some()); // untouched
}

#[test]
fn batch_delete_skips_missing_pks() {
    let (eng, _dir) = open();
    seed(&eng, 3);
    // 2 is real, 99 and 100 are missing
    let deleted = eng.batch_delete(&[2, 99, 100]).unwrap();
    assert_eq!(deleted, 1, "only existing rows count");
}

#[test]
fn batch_delete_all_missing_returns_zero() {
    let (eng, _dir) = open();
    let deleted = eng.batch_delete(&[10, 20, 30]).unwrap();
    assert_eq!(deleted, 0);
}

#[test]
fn batch_delete_empty_input_returns_zero() {
    let (eng, _dir) = open();
    seed(&eng, 3);
    assert_eq!(eng.batch_delete(&[]).unwrap(), 0);
}

// ── round-trip ────────────────────────────────────────────────────────────────

#[test]
fn batch_insert_then_get_then_delete_round_trip() {
    let (eng, _dir) = open();
    let rows: Vec<(u64, Vec<u64>)> = (1u64..=10)
        .map(|pk| (pk, vec![pk, pk * 7, pk % 3]))
        .collect();
    assert_eq!(eng.batch_insert(rows).unwrap(), 10);

    let fetched = eng.batch_get(&(1u64..=10).collect::<Vec<_>>()).unwrap();
    assert_eq!(fetched.len(), 10);
    assert!(fetched.iter().all(|r| r.is_some()));

    let deleted = eng.batch_delete(&(1u64..=10).collect::<Vec<_>>()).unwrap();
    assert_eq!(deleted, 10);

    let after = eng.batch_get(&(1u64..=10).collect::<Vec<_>>()).unwrap();
    assert!(after.iter().all(|r| r.is_none()));
}
