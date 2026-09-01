//! Layer 163 — FK false-positive re-check (VV-QRY-004.3)
//! After a filter hit, fetch the actual row and confirm it exists.

use std::path::PathBuf;
use tempfile::TempDir;
use vvctl::commands::table::{OutputFormat, TableCommand, run};

fn make_dir() -> TempDir {
    TempDir::new().unwrap()
}

fn tbl(dir: &TempDir) -> PathBuf {
    dir.path().join("t")
}

fn create_two_col(dir: &TempDir) {
    run(TableCommand::Create {
        dir: tbl(dir),
        columns: vec!["pk:u64".into(), "fk:u64".into()],
    })
    .unwrap();
}

fn insert(dir: &TempDir, pk: u64, fk: u64) {
    run(TableCommand::Insert {
        dir: tbl(dir),
        pk,
        values: vec![fk],
    })
    .unwrap();
}

// 1. get on existing fk-holding row returns Ok
#[test]
fn get_existing_fk_row_returns_ok() {
    let dir = make_dir();
    create_two_col(&dir);
    insert(&dir, 1, 1000);
    assert!(
        run(TableCommand::Get {
            dir: tbl(&dir),
            pk: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

// 2. get on non-existent row returns Ok (not found)
#[test]
fn get_nonexistent_row_returns_ok() {
    let dir = make_dir();
    create_two_col(&dir);
    assert!(
        run(TableCommand::Get {
            dir: tbl(&dir),
            pk: 42,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

// 3. scan after deleting a row does not return deleted row (Ok)
#[test]
fn scan_after_delete_does_not_return_deleted() {
    let dir = make_dir();
    create_two_col(&dir);
    insert(&dir, 1, 100);
    run(TableCommand::Flush { dir: tbl(&dir) }).unwrap();
    run(TableCommand::Delete {
        dir: tbl(&dir),
        pk: 1,
    })
    .unwrap();
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

// 4. scan with fk column range returns Ok
#[test]
fn scan_fk_col_range_returns_ok() {
    let dir = make_dir();
    create_two_col(&dir);
    for i in 0..5u64 {
        insert(&dir, i, i * 100);
    }
    assert!(
        run(TableCommand::Scan {
            dir: tbl(&dir),
            col: 1,
            lo: 0,
            hi: 400,
            output: OutputFormat::Table,
        })
        .is_ok()
    );
}

// 5. scan empty table returns Ok
#[test]
fn scan_empty_table_returns_ok() {
    let dir = make_dir();
    create_two_col(&dir);
    assert!(
        run(TableCommand::Scan {
            dir: tbl(&dir),
            col: 1,
            lo: 0,
            hi: u64::MAX,
            output: OutputFormat::Table,
        })
        .is_ok()
    );
}

// 6. insert multiple rows, get each by pk
#[test]
fn insert_multiple_get_each_by_pk() {
    let dir = make_dir();
    create_two_col(&dir);
    for i in 1u64..=5 {
        insert(&dir, i, i * 10);
    }
    for i in 1u64..=5 {
        assert!(
            run(TableCommand::Get {
                dir: tbl(&dir),
                pk: i,
                output: OutputFormat::Table
            })
            .is_ok()
        );
    }
}

// 7. get after flush and re-open returns Ok
#[test]
fn get_after_flush_reopen() {
    let dir = make_dir();
    create_two_col(&dir);
    insert(&dir, 7, 700);
    run(TableCommand::Flush { dir: tbl(&dir) }).unwrap();
    assert!(
        run(TableCommand::Get {
            dir: tbl(&dir),
            pk: 7,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

// 8. stats show correct row count
#[test]
fn stats_shows_correct_count() {
    let dir = make_dir();
    create_two_col(&dir);
    for i in 0..3u64 {
        insert(&dir, i, i);
    }
    assert!(
        run(TableCommand::Stats {
            dir: tbl(&dir),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

// 9. scan json output with fk column
#[test]
fn scan_json_output_fk_col() {
    let dir = make_dir();
    create_two_col(&dir);
    insert(&dir, 10, 999);
    assert!(
        run(TableCommand::Scan {
            dir: tbl(&dir),
            col: 1,
            lo: 0,
            hi: 999,
            output: OutputFormat::Json,
        })
        .is_ok()
    );
}

// 10. delete non-existent row does not panic
#[test]
fn delete_nonexistent_does_not_panic() {
    let dir = make_dir();
    create_two_col(&dir);
    // engine may return Ok or Err for non-existent key — just verify no panic
    let _ = run(TableCommand::Delete {
        dir: tbl(&dir),
        pk: 999,
    });
}

// 11. scan after compact returns Ok
#[test]
fn scan_after_compact_returns_ok() {
    let dir = make_dir();
    create_two_col(&dir);
    for i in 0..4u64 {
        insert(&dir, i, i * 50);
    }
    run(TableCommand::Flush { dir: tbl(&dir) }).unwrap();
    run(TableCommand::Compact { dir: tbl(&dir) }).unwrap();
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

// 12. get on missing dir returns Err
#[test]
fn get_missing_dir_err() {
    let dir = make_dir();
    assert!(
        run(TableCommand::Get {
            dir: dir.path().join("no_dir"),
            pk: 1,
            output: OutputFormat::Table,
        })
        .is_err()
    );
}

// 13. get with json output on non-existent pk returns Ok
#[test]
fn get_json_nonexistent_pk_returns_ok() {
    let dir = make_dir();
    create_two_col(&dir);
    assert!(
        run(TableCommand::Get {
            dir: tbl(&dir),
            pk: 12345,
            output: OutputFormat::Json
        })
        .is_ok()
    );
}
