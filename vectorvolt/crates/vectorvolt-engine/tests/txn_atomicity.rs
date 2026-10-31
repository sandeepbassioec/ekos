/// Transaction atomicity (PP-20/21): a commit is one WAL record, so a crash
/// leaves either the whole transaction or none of it.
use tempfile::TempDir;
use vectorvolt_engine::{Durability, EngineOptions, TableEngine};

fn open(dir: &TempDir) -> TableEngine {
    TableEngine::open_with(
        vec!["pk".into(), "v".into()],
        dir.path(),
        EngineOptions::new(1_000_000).with_durability(Durability::WriteOnly),
    )
    .unwrap()
}

fn last_wal_segment(dir: &TempDir) -> std::path::PathBuf {
    let mut segs: Vec<_> = std::fs::read_dir(dir.path().join("wal"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("vwal"))
        .collect();
    segs.sort();
    segs.pop().unwrap()
}

#[test]
fn committed_transaction_survives_restart_whole() {
    let dir = TempDir::new().unwrap();
    {
        let e = open(&dir);
        e.insert(5, vec![5, 0]).unwrap();
        let t = e.begin();
        e.insert_txn(t, 1, vec![1, 1]).unwrap();
        e.insert_txn(t, 2, vec![2, 2]).unwrap();
        e.delete_txn(t, 5).unwrap();
        e.commit_txn(t).unwrap();
    }
    let e = open(&dir);
    assert_eq!(e.get(1).unwrap().unwrap().values, vec![1, 1]);
    assert_eq!(e.get(2).unwrap().unwrap().values, vec![2, 2]);
    assert!(e.get(5).unwrap().is_none());
}

#[test]
fn a_torn_commit_record_leaves_none_of_the_transaction() {
    let dir = TempDir::new().unwrap();
    {
        let e = open(&dir);
        e.insert(5, vec![5, 0]).unwrap();
    }
    let before = std::fs::metadata(last_wal_segment(&dir)).unwrap().len();
    {
        let e = open(&dir);
        let t = e.begin();
        for pk in 11..=20u64 {
            e.insert_txn(t, pk, vec![pk, pk]).unwrap();
        }
        e.delete_txn(t, 5).unwrap();
        e.commit_txn(t).unwrap();
    }
    // Crash part-way through writing the commit record.
    let seg = last_wal_segment(&dir);
    let after = std::fs::metadata(&seg).unwrap().len();
    assert!(after > before + 20);
    let f = std::fs::OpenOptions::new().write(true).open(&seg).unwrap();
    f.set_len(before + (after - before) / 2).unwrap();
    drop(f);

    let e = open(&dir);
    for pk in 11..=20u64 {
        assert!(
            e.get(pk).unwrap().is_none(),
            "pk {pk} of a torn commit is visible"
        );
    }
    assert_eq!(
        e.get(5).unwrap().unwrap().values,
        vec![5, 0],
        "torn delete applied"
    );
}

#[test]
fn invalid_row_rejects_the_whole_batch() {
    let dir = TempDir::new().unwrap();
    let e = open(&dir);
    let err = e.apply_batch(vec![(1, Some(vec![1, 1])), (2, Some(vec![2]))]);
    assert!(err.is_err());
    assert!(
        e.get(1).unwrap().is_none(),
        "part of a rejected batch was applied"
    );
}

#[test]
fn apply_batch_is_idempotent() {
    let dir = TempDir::new().unwrap();
    let e = open(&dir);
    e.insert(3, vec![3, 3]).unwrap();
    let batch = vec![(1, Some(vec![1, 1])), (3, None), (4, None)];
    e.apply_batch(batch.clone()).unwrap();
    e.apply_batch(batch).unwrap();
    assert_eq!(e.get(1).unwrap().unwrap().values, vec![1, 1]);
    assert!(e.get(3).unwrap().is_none());
    assert!(e.get(4).unwrap().is_none());
}
