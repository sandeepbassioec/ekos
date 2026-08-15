/// Layer 29 — TTL / row expiry integration tests.
///
/// Uses an opaque u64 "now" counter (no real clock) so tests are
/// deterministic and instant.  expire_at=10 means the row expires when
/// now >= 10.
use tempfile::TempDir;
use vectorvolt_engine::{Durability, EngineOptions, TableEngine};

fn open() -> (TableEngine, TempDir) {
    let dir = TempDir::new().unwrap();
    let eng = TableEngine::open_with(
        vec!["pk".into(), "val".into()],
        dir.path(),
        EngineOptions::new(1024).with_durability(Durability::WriteOnly),
    )
    .unwrap();
    (eng, dir)
}

// ── insert_with_ttl / row_expire_at / ttl_count ───────────────────────────────

#[test]
fn insert_with_ttl_records_deadline() {
    let (eng, _dir) = open();
    eng.insert_with_ttl(1, vec![1, 42], 100).unwrap();
    assert_eq!(eng.row_expire_at(1), Some(100));
}

#[test]
fn insert_with_ttl_overwrites_previous_deadline() {
    let (eng, _dir) = open();
    eng.insert_with_ttl(1, vec![1, 10], 50).unwrap();
    eng.insert_with_ttl(1, vec![1, 20], 200).unwrap();
    assert_eq!(eng.row_expire_at(1), Some(200));
}

#[test]
fn ttl_count_tracks_entries() {
    let (eng, _dir) = open();
    assert_eq!(eng.ttl_count(), 0);
    eng.insert_with_ttl(1, vec![1, 1], 10).unwrap();
    eng.insert_with_ttl(2, vec![2, 2], 20).unwrap();
    assert_eq!(eng.ttl_count(), 2);
}

#[test]
fn plain_insert_has_no_ttl_entry() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 99]).unwrap();
    assert_eq!(eng.row_expire_at(1), None);
    assert_eq!(eng.ttl_count(), 0);
}

// ── get_with_ttl ──────────────────────────────────────────────────────────────

#[test]
fn get_with_ttl_returns_row_before_deadline() {
    let (eng, _dir) = open();
    eng.insert_with_ttl(1, vec![1, 42], 100).unwrap();
    let row = eng.get_with_ttl(1, 99).unwrap();
    assert!(row.is_some());
    assert_eq!(row.unwrap().values[1], 42);
}

#[test]
fn get_with_ttl_returns_none_at_deadline() {
    let (eng, _dir) = open();
    eng.insert_with_ttl(1, vec![1, 42], 100).unwrap();
    // now == expire_at is expired
    assert!(eng.get_with_ttl(1, 100).unwrap().is_none());
}

#[test]
fn get_with_ttl_returns_none_after_deadline() {
    let (eng, _dir) = open();
    eng.insert_with_ttl(1, vec![1, 42], 100).unwrap();
    assert!(eng.get_with_ttl(1, 9999).unwrap().is_none());
}

#[test]
fn get_with_ttl_non_ttl_row_never_expires() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 55]).unwrap();
    // Very large now — still returned
    assert!(eng.get_with_ttl(1, u64::MAX).unwrap().is_some());
}

#[test]
fn get_with_ttl_missing_pk_returns_none() {
    let (eng, _dir) = open();
    assert!(eng.get_with_ttl(99, 0).unwrap().is_none());
}

// ── scan_with_ttl ─────────────────────────────────────────────────────────────

#[test]
fn scan_with_ttl_excludes_expired_rows() {
    let (eng, _dir) = open();
    eng.insert_with_ttl(1, vec![1, 10], 50).unwrap(); // expires at 50
    eng.insert_with_ttl(2, vec![2, 20], 200).unwrap(); // alive
    eng.insert(3, vec![3, 30]).unwrap(); // no TTL — always alive

    let rows = eng.scan_with_ttl(100).unwrap();
    let pks: Vec<u64> = rows.iter().map(|r| r.pk).collect();
    assert!(!pks.contains(&1), "pk=1 should be excluded (expired)");
    assert!(pks.contains(&2), "pk=2 should be present");
    assert!(pks.contains(&3), "pk=3 should be present (no TTL)");
}

#[test]
fn scan_with_ttl_all_expired_returns_empty() {
    let (eng, _dir) = open();
    for pk in 1u64..=5 {
        eng.insert_with_ttl(pk, vec![pk, pk], 10).unwrap();
    }
    assert!(eng.scan_with_ttl(10).unwrap().is_empty());
}

#[test]
fn scan_with_ttl_none_expired_returns_all() {
    let (eng, _dir) = open();
    for pk in 1u64..=5 {
        eng.insert_with_ttl(pk, vec![pk, pk], 1000).unwrap();
    }
    let rows = eng.scan_with_ttl(1).unwrap();
    assert_eq!(rows.len(), 5);
}

#[test]
fn scan_with_ttl_mixed_ttl_and_non_ttl() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 1]).unwrap(); // no TTL
    eng.insert_with_ttl(2, vec![2, 2], 5).unwrap(); // expires at 5
    eng.insert_with_ttl(3, vec![3, 3], 15).unwrap(); // alive at now=10

    let rows = eng.scan_with_ttl(10).unwrap();
    let pks: Vec<u64> = rows.iter().map(|r| r.pk).collect();
    assert!(pks.contains(&1));
    assert!(!pks.contains(&2));
    assert!(pks.contains(&3));
    assert_eq!(rows.len(), 2);
}

// ── purge_expired ─────────────────────────────────────────────────────────────

#[test]
fn purge_expired_removes_expired_rows() {
    let (eng, _dir) = open();
    eng.insert_with_ttl(1, vec![1, 1], 10).unwrap();
    eng.insert_with_ttl(2, vec![2, 2], 10).unwrap();
    eng.insert_with_ttl(3, vec![3, 3], 20).unwrap();

    let removed = eng.purge_expired(10).unwrap();
    assert_eq!(removed, 2);
    assert!(eng.get(1).unwrap().is_none());
    assert!(eng.get(2).unwrap().is_none());
    assert!(eng.get(3).unwrap().is_some()); // not yet expired
}

#[test]
fn purge_expired_removes_ttl_entries_from_index() {
    let (eng, _dir) = open();
    eng.insert_with_ttl(1, vec![1, 1], 10).unwrap();
    eng.insert_with_ttl(2, vec![2, 2], 20).unwrap();

    eng.purge_expired(10).unwrap();
    assert_eq!(eng.ttl_count(), 1, "only pk=2 TTL entry should remain");
    assert_eq!(eng.row_expire_at(1), None, "pk=1 TTL entry must be cleared");
}

#[test]
fn purge_expired_returns_zero_when_nothing_expired() {
    let (eng, _dir) = open();
    eng.insert_with_ttl(1, vec![1, 1], 100).unwrap();
    let removed = eng.purge_expired(10).unwrap();
    assert_eq!(removed, 0);
    assert!(eng.get(1).unwrap().is_some()); // still alive
}

#[test]
fn purge_expired_empty_table_returns_zero() {
    let (eng, _dir) = open();
    assert_eq!(eng.purge_expired(9999).unwrap(), 0);
}

#[test]
fn purge_expired_then_get_with_ttl_returns_none() {
    let (eng, _dir) = open();
    eng.insert_with_ttl(1, vec![1, 99], 10).unwrap();
    eng.purge_expired(10).unwrap();
    // Row deleted from engine; get_with_ttl must return None
    assert!(eng.get_with_ttl(1, 0).unwrap().is_none());
}

#[test]
fn purge_expired_does_not_affect_non_ttl_rows() {
    let (eng, _dir) = open();
    eng.insert(1, vec![1, 10]).unwrap(); // no TTL
    eng.insert_with_ttl(2, vec![2, 20], 5).unwrap();

    eng.purge_expired(5).unwrap();
    assert!(
        eng.get(1).unwrap().is_some(),
        "non-TTL row must survive purge"
    );
    assert!(eng.get(2).unwrap().is_none());
}
