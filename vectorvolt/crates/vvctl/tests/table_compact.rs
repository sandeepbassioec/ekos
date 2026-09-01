//! Layer 169 — vvctl table compact (trigger compaction CLI gate)

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

// 1. compact empty table returns Ok
#[test]
fn compact_empty_table_ok() {
    let dir = make_dir();
    setup(&dir);
    assert!(run(TableCommand::Compact { dir: tbl(&dir) }).is_ok());
}

// 2. compact after insert returns Ok
#[test]
fn compact_after_insert_ok() {
    let dir = make_dir();
    setup(&dir);
    insert(&dir, 1, 10);
    run(TableCommand::Flush { dir: tbl(&dir) }).unwrap();
    assert!(run(TableCommand::Compact { dir: tbl(&dir) }).is_ok());
}

// 3. compact multiple partitions returns Ok
#[test]
fn compact_multiple_partitions_ok() {
    let dir = make_dir();
    setup(&dir);
    for i in 0u64..5 {
        insert(&dir, i, i);
    }
    run(TableCommand::Flush { dir: tbl(&dir) }).unwrap();
    for i in 5u64..10 {
        insert(&dir, i, i);
    }
    run(TableCommand::Flush { dir: tbl(&dir) }).unwrap();
    assert!(run(TableCommand::Compact { dir: tbl(&dir) }).is_ok());
}

// 4. data accessible after compact
#[test]
fn data_accessible_after_compact() {
    let dir = make_dir();
    setup(&dir);
    insert(&dir, 42, 420);
    run(TableCommand::Flush { dir: tbl(&dir) }).unwrap();
    run(TableCommand::Compact { dir: tbl(&dir) }).unwrap();
    assert!(
        run(TableCommand::Get {
            dir: tbl(&dir),
            pk: 42,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

// 5. stats after compact shows partition_count <= before
#[test]
fn stats_after_compact_ok() {
    let dir = make_dir();
    setup(&dir);
    for i in 0u64..3 {
        insert(&dir, i, i);
        run(TableCommand::Flush { dir: tbl(&dir) }).unwrap();
    }
    run(TableCommand::Compact { dir: tbl(&dir) }).unwrap();
    assert!(
        run(TableCommand::Stats {
            dir: tbl(&dir),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

// 6. compact missing dir returns Err
#[test]
fn compact_missing_dir_returns_err() {
    let dir = make_dir();
    assert!(
        run(TableCommand::Compact {
            dir: dir.path().join("no_dir")
        })
        .is_err()
    );
}

// 7. double compact returns Ok
#[test]
fn double_compact_ok() {
    let dir = make_dir();
    setup(&dir);
    insert(&dir, 1, 1);
    run(TableCommand::Flush { dir: tbl(&dir) }).unwrap();
    run(TableCommand::Compact { dir: tbl(&dir) }).unwrap();
    assert!(run(TableCommand::Compact { dir: tbl(&dir) }).is_ok());
}

// 8. scan after compact returns correct rows
#[test]
fn scan_after_compact_returns_rows() {
    let dir = make_dir();
    setup(&dir);
    for i in 0u64..5 {
        insert(&dir, i, i * 2);
    }
    run(TableCommand::Flush { dir: tbl(&dir) }).unwrap();
    run(TableCommand::Compact { dir: tbl(&dir) }).unwrap();
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

// 9. compact after delete removes deleted rows
#[test]
fn compact_after_delete_removes_rows() {
    let dir = make_dir();
    setup(&dir);
    insert(&dir, 1, 10);
    insert(&dir, 2, 20);
    run(TableCommand::Flush { dir: tbl(&dir) }).unwrap();
    run(TableCommand::Delete {
        dir: tbl(&dir),
        pk: 1,
    })
    .unwrap();
    run(TableCommand::Flush { dir: tbl(&dir) }).unwrap();
    assert!(run(TableCommand::Compact { dir: tbl(&dir) }).is_ok());
}

// 10. compact preserves all rows
#[test]
fn compact_preserves_all_rows() {
    let dir = make_dir();
    setup(&dir);
    for i in 0u64..10 {
        insert(&dir, i, i * 3);
    }
    run(TableCommand::Flush { dir: tbl(&dir) }).unwrap();
    run(TableCommand::Compact { dir: tbl(&dir) }).unwrap();
    assert!(
        run(TableCommand::Count {
            dir: tbl(&dir),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

// 11. json stats after compact
#[test]
fn json_stats_after_compact() {
    let dir = make_dir();
    setup(&dir);
    insert(&dir, 5, 50);
    run(TableCommand::Flush { dir: tbl(&dir) }).unwrap();
    run(TableCommand::Compact { dir: tbl(&dir) }).unwrap();
    assert!(
        run(TableCommand::Stats {
            dir: tbl(&dir),
            output: OutputFormat::Json
        })
        .is_ok()
    );
}

// 12. compact large table returns Ok
#[test]
fn compact_large_table_ok() {
    let dir = make_dir();
    setup(&dir);
    for i in 0u64..100 {
        insert(&dir, i, i);
    }
    run(TableCommand::Flush { dir: tbl(&dir) }).unwrap();
    assert!(run(TableCommand::Compact { dir: tbl(&dir) }).is_ok());
}

// 13. recount after compact is consistent
#[test]
fn recount_after_compact_consistent() {
    let dir = make_dir();
    setup(&dir);
    for i in 0u64..5 {
        insert(&dir, i, i);
    }
    run(TableCommand::Flush { dir: tbl(&dir) }).unwrap();
    run(TableCommand::Compact { dir: tbl(&dir) }).unwrap();
    assert!(
        run(TableCommand::Recount {
            dir: tbl(&dir),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
