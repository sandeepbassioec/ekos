//! Layer 162 — fix 400x query read path (VV-QRY-001.4)
//! Tests that point reads via vvctl table get work correctly.

use std::path::PathBuf;
use tempfile::TempDir;
use vvctl::commands::table::{OutputFormat, TableCommand, run};

fn make_dir() -> TempDir {
    TempDir::new().unwrap()
}

fn tbl(dir: &TempDir) -> PathBuf {
    dir.path().join("t")
}

fn create(dir: &TempDir) {
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

// 1. get on missing pk returns Ok (not found)
#[test]
fn get_missing_pk_returns_ok() {
    let dir = make_dir();
    create(&dir);
    assert!(
        run(TableCommand::Get {
            dir: tbl(&dir),
            pk: 99,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

// 2. get after insert returns Ok
#[test]
fn get_after_insert_returns_ok() {
    let dir = make_dir();
    create(&dir);
    insert(&dir, 1, 42);
    assert!(
        run(TableCommand::Get {
            dir: tbl(&dir),
            pk: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

// 3. get json output for existing row returns Ok
#[test]
fn get_json_output_returns_ok() {
    let dir = make_dir();
    create(&dir);
    insert(&dir, 2, 100);
    assert!(
        run(TableCommand::Get {
            dir: tbl(&dir),
            pk: 2,
            output: OutputFormat::Json
        })
        .is_ok()
    );
}

// 4. get pk=0 returns Ok
#[test]
fn get_pk_zero_returns_ok() {
    let dir = make_dir();
    create(&dir);
    insert(&dir, 0, 7);
    assert!(
        run(TableCommand::Get {
            dir: tbl(&dir),
            pk: 0,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

// 5. get after flush returns Ok
#[test]
fn get_after_flush_returns_ok() {
    let dir = make_dir();
    create(&dir);
    insert(&dir, 5, 55);
    run(TableCommand::Flush { dir: tbl(&dir) }).unwrap();
    assert!(
        run(TableCommand::Get {
            dir: tbl(&dir),
            pk: 5,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

// 6. get on deleted pk returns Ok (not found)
#[test]
fn get_deleted_pk_returns_ok() {
    let dir = make_dir();
    create(&dir);
    insert(&dir, 10, 1);
    run(TableCommand::Flush { dir: tbl(&dir) }).unwrap();
    run(TableCommand::Delete {
        dir: tbl(&dir),
        pk: 10,
    })
    .unwrap();
    assert!(
        run(TableCommand::Get {
            dir: tbl(&dir),
            pk: 10,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

// 7. get on missing dir returns Err
#[test]
fn get_missing_dir_returns_err() {
    let dir = make_dir();
    assert!(
        run(TableCommand::Get {
            dir: dir.path().join("no_such"),
            pk: 1,
            output: OutputFormat::Table,
        })
        .is_err()
    );
}

// 8. get large pk value returns Ok
#[test]
fn get_large_pk_returns_ok() {
    let dir = make_dir();
    create(&dir);
    insert(&dir, u64::MAX - 1, 999);
    assert!(
        run(TableCommand::Get {
            dir: tbl(&dir),
            pk: u64::MAX - 1,
            output: OutputFormat::Table,
        })
        .is_ok()
    );
}

// 9. get on multi-column table returns Ok
#[test]
fn get_multi_column_returns_ok() {
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
        run(TableCommand::Get {
            dir: tbl(&dir),
            pk: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

// 10. multiple gets on same pk return Ok
#[test]
fn multiple_gets_same_pk_return_ok() {
    let dir = make_dir();
    create(&dir);
    insert(&dir, 3, 300);
    assert!(
        run(TableCommand::Get {
            dir: tbl(&dir),
            pk: 3,
            output: OutputFormat::Table
        })
        .is_ok()
    );
    assert!(
        run(TableCommand::Get {
            dir: tbl(&dir),
            pk: 3,
            output: OutputFormat::Json
        })
        .is_ok()
    );
}

// 11. get non-existent pk after flush returns Ok
#[test]
fn get_nonexistent_after_flush_returns_ok() {
    let dir = make_dir();
    create(&dir);
    insert(&dir, 100, 1000);
    run(TableCommand::Flush { dir: tbl(&dir) }).unwrap();
    assert!(
        run(TableCommand::Get {
            dir: tbl(&dir),
            pk: 999,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

// 12. get with bool column
#[test]
fn get_bool_column_returns_ok() {
    let dir = make_dir();
    run(TableCommand::Create {
        dir: tbl(&dir),
        columns: vec!["pk:u64".into(), "flag:bool".into()],
    })
    .unwrap();
    run(TableCommand::TypedInsert {
        dir: tbl(&dir),
        pk: "7".into(),
        values: vec!["true".into()],
    })
    .unwrap();
    assert!(
        run(TableCommand::Get {
            dir: tbl(&dir),
            pk: 7,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

// 13. get after compact returns Ok
#[test]
fn get_after_compact_returns_ok() {
    let dir = make_dir();
    create(&dir);
    insert(&dir, 50, 500);
    run(TableCommand::Flush { dir: tbl(&dir) }).unwrap();
    run(TableCommand::Compact { dir: tbl(&dir) }).unwrap();
    assert!(
        run(TableCommand::Get {
            dir: tbl(&dir),
            pk: 50,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
