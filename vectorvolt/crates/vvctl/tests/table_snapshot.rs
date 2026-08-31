//! Layer 172 — vvctl table snapshot + restore

use std::path::PathBuf;
use tempfile::TempDir;
use vvctl::commands::table::{OutputFormat, TableCommand, run};

fn make_dir() -> TempDir {
    TempDir::new().unwrap()
}

fn tbl(dir: &TempDir) -> PathBuf {
    dir.path().join("t")
}

fn setup(dir: &TempDir) {
    run(TableCommand::Create {
        dir: tbl(dir),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
}

fn insert(dir: &TempDir, pk: u64, val: u64) {
    run(TableCommand::Insert {
        dir: tbl(dir),
        pk,
        values: vec![val],
    })
    .unwrap();
}

// 1. snapshot on empty table returns Ok
#[test]
fn snapshot_empty_table_ok() {
    let dir = make_dir();
    setup(&dir);
    assert!(run(TableCommand::Snapshot { dir: tbl(&dir) }).is_ok());
}

// 2. snapshot creates snapshot.vvss file
#[test]
fn snapshot_creates_file() {
    let dir = make_dir();
    setup(&dir);
    run(TableCommand::Snapshot { dir: tbl(&dir) }).unwrap();
    assert!(tbl(&dir).join("snapshot.vvss").exists());
}

// 3. snapshot after insert returns Ok
#[test]
fn snapshot_after_insert_ok() {
    let dir = make_dir();
    setup(&dir);
    insert(&dir, 1, 10);
    assert!(run(TableCommand::Snapshot { dir: tbl(&dir) }).is_ok());
}

// 4. snapshot missing dir returns Err
#[test]
fn snapshot_missing_dir_returns_err() {
    let dir = make_dir();
    assert!(
        run(TableCommand::Snapshot {
            dir: dir.path().join("no_dir")
        })
        .is_err()
    );
}

// 5. restore with snapshot present returns Ok
#[test]
fn restore_with_snapshot_ok() {
    let dir = make_dir();
    setup(&dir);
    run(TableCommand::Snapshot { dir: tbl(&dir) }).unwrap();
    assert!(
        run(TableCommand::Restore {
            dir: tbl(&dir),
            lsn: 0
        })
        .is_ok()
    );
}

// 6. restore without snapshot returns Err
#[test]
fn restore_without_snapshot_returns_err() {
    let dir = make_dir();
    setup(&dir);
    assert!(
        run(TableCommand::Restore {
            dir: tbl(&dir),
            lsn: 0
        })
        .is_err()
    );
}

// 7. multiple snapshots return Ok
#[test]
fn multiple_snapshots_ok() {
    let dir = make_dir();
    setup(&dir);
    run(TableCommand::Snapshot { dir: tbl(&dir) }).unwrap();
    assert!(run(TableCommand::Snapshot { dir: tbl(&dir) }).is_ok());
}

// 8. restore to lsn=0 ok
#[test]
fn restore_lsn_zero_ok() {
    let dir = make_dir();
    setup(&dir);
    run(TableCommand::Snapshot { dir: tbl(&dir) }).unwrap();
    assert!(
        run(TableCommand::Restore {
            dir: tbl(&dir),
            lsn: 0
        })
        .is_ok()
    );
}

// 9. restore to large lsn ok
#[test]
fn restore_large_lsn_ok() {
    let dir = make_dir();
    setup(&dir);
    for i in 0u64..10 {
        insert(&dir, i, i);
    }
    run(TableCommand::Flush { dir: tbl(&dir) }).unwrap();
    run(TableCommand::Snapshot { dir: tbl(&dir) }).unwrap();
    assert!(
        run(TableCommand::Restore {
            dir: tbl(&dir),
            lsn: 999
        })
        .is_ok()
    );
}

// 10. pitr-list after snapshot returns Ok
#[test]
fn pitr_list_after_snapshot_ok() {
    let dir = make_dir();
    setup(&dir);
    run(TableCommand::Snapshot { dir: tbl(&dir) }).unwrap();
    assert!(
        run(TableCommand::PitrList {
            dir: tbl(&dir),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

// 11. pitr-list json after snapshot returns Ok
#[test]
fn pitr_list_json_ok() {
    let dir = make_dir();
    setup(&dir);
    run(TableCommand::Snapshot { dir: tbl(&dir) }).unwrap();
    assert!(
        run(TableCommand::PitrList {
            dir: tbl(&dir),
            output: OutputFormat::Json
        })
        .is_ok()
    );
}

// 12. pitr-list no snapshot returns Ok (empty list)
#[test]
fn pitr_list_no_snapshot_ok() {
    let dir = make_dir();
    setup(&dir);
    assert!(
        run(TableCommand::PitrList {
            dir: tbl(&dir),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

// 13. pitr-list missing dir returns Err
#[test]
fn pitr_list_missing_dir_returns_err() {
    let dir = make_dir();
    assert!(
        run(TableCommand::PitrList {
            dir: dir.path().join("no_dir"),
            output: OutputFormat::Table,
        })
        .is_err()
    );
}
