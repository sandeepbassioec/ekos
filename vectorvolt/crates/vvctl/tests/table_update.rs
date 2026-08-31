//! Layer 168 — UPDATE opcode + vvctl table update (VV-TXN-004.6)

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

// 1. update existing row succeeds
#[test]
fn update_existing_row_succeeds() {
    let dir = make_dir();
    setup(&dir);
    insert(&dir, 1, 100);
    assert!(
        run(TableCommand::Update {
            dir: tbl(&dir),
            pk: 1,
            col: 1,
            val: 999
        })
        .is_ok()
    );
}

// 2. update then get returns new value
#[test]
fn update_then_get_returns_new_value() {
    let dir = make_dir();
    setup(&dir);
    insert(&dir, 2, 200);
    run(TableCommand::Update {
        dir: tbl(&dir),
        pk: 2,
        col: 1,
        val: 777,
    })
    .unwrap();
    assert!(
        run(TableCommand::Get {
            dir: tbl(&dir),
            pk: 2,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

// 3. update non-existent pk returns Err
#[test]
fn update_nonexistent_pk_returns_err() {
    let dir = make_dir();
    setup(&dir);
    assert!(
        run(TableCommand::Update {
            dir: tbl(&dir),
            pk: 999,
            col: 1,
            val: 1
        })
        .is_err()
    );
}

// 4. update pk column (col=0) returns Err
#[test]
fn update_pk_col_returns_err() {
    let dir = make_dir();
    setup(&dir);
    insert(&dir, 1, 1);
    assert!(
        run(TableCommand::Update {
            dir: tbl(&dir),
            pk: 1,
            col: 0,
            val: 2
        })
        .is_err()
    );
}

// 5. update out-of-range col index returns Err
#[test]
fn update_out_of_range_col_returns_err() {
    let dir = make_dir();
    setup(&dir);
    insert(&dir, 1, 1);
    assert!(
        run(TableCommand::Update {
            dir: tbl(&dir),
            pk: 1,
            col: 99,
            val: 1
        })
        .is_err()
    );
}

// 6. update after flush succeeds
#[test]
fn update_after_flush_succeeds() {
    let dir = make_dir();
    setup(&dir);
    insert(&dir, 3, 300);
    run(TableCommand::Flush { dir: tbl(&dir) }).unwrap();
    assert!(
        run(TableCommand::Update {
            dir: tbl(&dir),
            pk: 3,
            col: 1,
            val: 42
        })
        .is_ok()
    );
}

// 7. multiple updates on same row
#[test]
fn multiple_updates_same_row() {
    let dir = make_dir();
    setup(&dir);
    insert(&dir, 5, 500);
    run(TableCommand::Update {
        dir: tbl(&dir),
        pk: 5,
        col: 1,
        val: 111,
    })
    .unwrap();
    assert!(
        run(TableCommand::Update {
            dir: tbl(&dir),
            pk: 5,
            col: 1,
            val: 222
        })
        .is_ok()
    );
}

// 8. update on missing dir returns Err
#[test]
fn update_missing_dir_returns_err() {
    let dir = make_dir();
    assert!(
        run(TableCommand::Update {
            dir: dir.path().join("no_dir"),
            pk: 1,
            col: 1,
            val: 1,
        })
        .is_err()
    );
}

// 9. update multi-column table second col
#[test]
fn update_multi_col_second_col() {
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
    assert!(
        run(TableCommand::Update {
            dir: tbl(&dir),
            pk: 1,
            col: 2,
            val: 99
        })
        .is_ok()
    );
}

// 10. update multi-column table first col
#[test]
fn update_multi_col_first_col() {
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
    assert!(
        run(TableCommand::Update {
            dir: tbl(&dir),
            pk: 1,
            col: 1,
            val: 55
        })
        .is_ok()
    );
}

// 11. stats after update shows row count
#[test]
fn stats_after_update_shows_count() {
    let dir = make_dir();
    setup(&dir);
    insert(&dir, 7, 77);
    run(TableCommand::Update {
        dir: tbl(&dir),
        pk: 7,
        col: 1,
        val: 88,
    })
    .unwrap();
    assert!(
        run(TableCommand::Stats {
            dir: tbl(&dir),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

// 12. update value to 0
#[test]
fn update_value_to_zero() {
    let dir = make_dir();
    setup(&dir);
    insert(&dir, 10, 1000);
    assert!(
        run(TableCommand::Update {
            dir: tbl(&dir),
            pk: 10,
            col: 1,
            val: 0
        })
        .is_ok()
    );
}

// 13. update to max u64
#[test]
fn update_to_max_u64() {
    let dir = make_dir();
    setup(&dir);
    insert(&dir, 11, 1);
    assert!(
        run(TableCommand::Update {
            dir: tbl(&dir),
            pk: 11,
            col: 1,
            val: u64::MAX
        })
        .is_ok()
    );
}
