//! Layer 174 — vvctl table flush

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

// 1. flush on empty table returns Ok
#[test]
fn flush_empty_table_ok() {
    let dir = make_dir();
    setup(&dir);
    assert!(run(TableCommand::Flush { dir: tbl(&dir) }).is_ok());
}

// 2. flush after insert returns Ok
#[test]
fn flush_after_insert_ok() {
    let dir = make_dir();
    setup(&dir);
    insert(&dir, 1, 10);
    assert!(run(TableCommand::Flush { dir: tbl(&dir) }).is_ok());
}

// 3. flush missing dir returns Err
#[test]
fn flush_missing_dir_returns_err() {
    let dir = make_dir();
    assert!(
        run(TableCommand::Flush {
            dir: dir.path().join("no_dir")
        })
        .is_err()
    );
}

// 4. data readable after flush
#[test]
fn data_readable_after_flush() {
    let dir = make_dir();
    setup(&dir);
    insert(&dir, 42, 420);
    run(TableCommand::Flush { dir: tbl(&dir) }).unwrap();
    assert!(
        run(TableCommand::Get {
            dir: tbl(&dir),
            pk: 42,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

// 5. stats after flush shows flushed rows
#[test]
fn stats_after_flush_shows_flushed() {
    let dir = make_dir();
    setup(&dir);
    insert(&dir, 1, 10);
    run(TableCommand::Flush { dir: tbl(&dir) }).unwrap();
    assert!(
        run(TableCommand::Stats {
            dir: tbl(&dir),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

// 6. multiple flushes return Ok
#[test]
fn multiple_flushes_ok() {
    let dir = make_dir();
    setup(&dir);
    insert(&dir, 1, 10);
    run(TableCommand::Flush { dir: tbl(&dir) }).unwrap();
    insert(&dir, 2, 20);
    assert!(run(TableCommand::Flush { dir: tbl(&dir) }).is_ok());
}

// 7. flush then scan returns Ok
#[test]
fn flush_then_scan_ok() {
    let dir = make_dir();
    setup(&dir);
    for i in 0u64..5 {
        insert(&dir, i, i * 2);
    }
    run(TableCommand::Flush { dir: tbl(&dir) }).unwrap();
    assert!(
        run(TableCommand::Scan {
            dir: tbl(&dir),
            col: 0,
            lo: 0,
            hi: 4,
            output: OutputFormat::Table,
        })
        .is_ok()
    );
}

// 8. flush with many rows returns Ok
#[test]
fn flush_many_rows_ok() {
    let dir = make_dir();
    setup(&dir);
    for i in 0u64..100 {
        insert(&dir, i, i);
    }
    assert!(run(TableCommand::Flush { dir: tbl(&dir) }).is_ok());
}

// 9. count after flush returns Ok
#[test]
fn count_after_flush_ok() {
    let dir = make_dir();
    setup(&dir);
    for i in 0u64..5 {
        insert(&dir, i, i);
    }
    run(TableCommand::Flush { dir: tbl(&dir) }).unwrap();
    assert!(
        run(TableCommand::Count {
            dir: tbl(&dir),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

// 10. flush then compact returns Ok
#[test]
fn flush_then_compact_ok() {
    let dir = make_dir();
    setup(&dir);
    insert(&dir, 1, 10);
    run(TableCommand::Flush { dir: tbl(&dir) }).unwrap();
    assert!(run(TableCommand::Compact { dir: tbl(&dir) }).is_ok());
}

// 11. flush with json stats returns Ok
#[test]
fn flush_json_stats_ok() {
    let dir = make_dir();
    setup(&dir);
    insert(&dir, 3, 30);
    run(TableCommand::Flush { dir: tbl(&dir) }).unwrap();
    assert!(
        run(TableCommand::Stats {
            dir: tbl(&dir),
            output: OutputFormat::Json
        })
        .is_ok()
    );
}

// 12. flush then head returns Ok
#[test]
fn flush_then_head_ok() {
    let dir = make_dir();
    setup(&dir);
    for i in 0u64..10 {
        insert(&dir, i, i);
    }
    run(TableCommand::Flush { dir: tbl(&dir) }).unwrap();
    assert!(
        run(TableCommand::Head {
            dir: tbl(&dir),
            n: 5,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

// 13. flush idempotent: flush already-flushed table returns Ok
#[test]
fn flush_already_flushed_ok() {
    let dir = make_dir();
    setup(&dir);
    insert(&dir, 1, 1);
    run(TableCommand::Flush { dir: tbl(&dir) }).unwrap();
    run(TableCommand::Flush { dir: tbl(&dir) }).unwrap();
    assert!(run(TableCommand::Flush { dir: tbl(&dir) }).is_ok());
}
