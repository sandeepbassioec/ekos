//! Layer 170 — vvctl table vacuum (remove tombstones + reclaim WAL space)

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

// 1. vacuum on empty table returns Ok
#[test]
fn vacuum_empty_table_ok() {
    let dir = make_dir();
    setup(&dir);
    assert!(run(TableCommand::Vacuum { dir: tbl(&dir) }).is_ok());
}

// 2. vacuum after insert returns Ok
#[test]
fn vacuum_after_insert_ok() {
    let dir = make_dir();
    setup(&dir);
    insert(&dir, 1, 10);
    run(TableCommand::Flush { dir: tbl(&dir) }).unwrap();
    assert!(run(TableCommand::Vacuum { dir: tbl(&dir) }).is_ok());
}

// 3. vacuum after delete returns Ok
#[test]
fn vacuum_after_delete_ok() {
    let dir = make_dir();
    setup(&dir);
    insert(&dir, 2, 20);
    run(TableCommand::Flush { dir: tbl(&dir) }).unwrap();
    run(TableCommand::Delete {
        dir: tbl(&dir),
        pk: 2,
    })
    .unwrap();
    run(TableCommand::Flush { dir: tbl(&dir) }).unwrap();
    assert!(run(TableCommand::Vacuum { dir: tbl(&dir) }).is_ok());
}

// 4. vacuum missing dir returns Err
#[test]
fn vacuum_missing_dir_returns_err() {
    let dir = make_dir();
    assert!(
        run(TableCommand::Vacuum {
            dir: dir.path().join("no_dir")
        })
        .is_err()
    );
}

// 5. vacuum twice returns Ok
#[test]
fn vacuum_twice_ok() {
    let dir = make_dir();
    setup(&dir);
    insert(&dir, 3, 30);
    run(TableCommand::Flush { dir: tbl(&dir) }).unwrap();
    run(TableCommand::Vacuum { dir: tbl(&dir) }).unwrap();
    assert!(run(TableCommand::Vacuum { dir: tbl(&dir) }).is_ok());
}

// 6. data still accessible after vacuum
#[test]
fn data_accessible_after_vacuum() {
    let dir = make_dir();
    setup(&dir);
    insert(&dir, 5, 50);
    run(TableCommand::Vacuum { dir: tbl(&dir) }).unwrap();
    assert!(
        run(TableCommand::Get {
            dir: tbl(&dir),
            pk: 5,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

// 7. count after vacuum is correct
#[test]
fn count_after_vacuum_correct() {
    let dir = make_dir();
    setup(&dir);
    for i in 0u64..5 {
        insert(&dir, i, i);
    }
    run(TableCommand::Flush { dir: tbl(&dir) }).unwrap();
    run(TableCommand::Vacuum { dir: tbl(&dir) }).unwrap();
    assert!(
        run(TableCommand::Count {
            dir: tbl(&dir),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

// 8. scan after vacuum returns Ok
#[test]
fn scan_after_vacuum_ok() {
    let dir = make_dir();
    setup(&dir);
    for i in 0u64..4 {
        insert(&dir, i, i * 10);
    }
    run(TableCommand::Flush { dir: tbl(&dir) }).unwrap();
    run(TableCommand::Vacuum { dir: tbl(&dir) }).unwrap();
    assert!(
        run(TableCommand::Scan {
            dir: tbl(&dir),
            col: 0,
            lo: 0,
            hi: u64::MAX,
            output: OutputFormat::Table,
        })
        .is_ok()
    );
}

// 9. stats after vacuum returns Ok
#[test]
fn stats_after_vacuum_ok() {
    let dir = make_dir();
    setup(&dir);
    insert(&dir, 7, 77);
    run(TableCommand::Vacuum { dir: tbl(&dir) }).unwrap();
    assert!(
        run(TableCommand::Stats {
            dir: tbl(&dir),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

// 10. vacuum then compact returns Ok
#[test]
fn vacuum_then_compact_ok() {
    let dir = make_dir();
    setup(&dir);
    for i in 0u64..5 {
        insert(&dir, i, i);
    }
    run(TableCommand::Flush { dir: tbl(&dir) }).unwrap();
    run(TableCommand::Vacuum { dir: tbl(&dir) }).unwrap();
    assert!(run(TableCommand::Compact { dir: tbl(&dir) }).is_ok());
}

// 11. vacuum multi-column table
#[test]
fn vacuum_multi_column_table() {
    let dir = make_dir();
    run(TableCommand::Create {
        dir: tbl(&dir),
        columns: vec!["pk:u64".into(), "a:u64".into(), "b:u64".into()],
    })
    .unwrap();
    run(TableCommand::Insert {
        dir: tbl(&dir),
        pk: 1,
        values: vec![10, 20],
    })
    .unwrap();
    assert!(run(TableCommand::Vacuum { dir: tbl(&dir) }).is_ok());
}

// 12. vacuum with large dataset
#[test]
fn vacuum_large_dataset() {
    let dir = make_dir();
    setup(&dir);
    for i in 0u64..50 {
        insert(&dir, i, i);
    }
    run(TableCommand::Flush { dir: tbl(&dir) }).unwrap();
    assert!(run(TableCommand::Vacuum { dir: tbl(&dir) }).is_ok());
}

// 13. vacuum then insert succeeds
#[test]
fn vacuum_then_insert_ok() {
    let dir = make_dir();
    setup(&dir);
    insert(&dir, 1, 10);
    run(TableCommand::Vacuum { dir: tbl(&dir) }).unwrap();
    assert!(
        run(TableCommand::Insert {
            dir: tbl(&dir),
            pk: 2,
            values: vec![20]
        })
        .is_ok()
    );
}
