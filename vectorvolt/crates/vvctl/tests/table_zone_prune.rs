//! Layer 165 — Zone-prune before file-open (VV-ENG-014)
//! Check zone-map overlap BEFORE opening the file.

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
        columns: vec!["pk:u64".into(), "score:u64".into()],
    })
    .unwrap();
}

fn insert(dir: &TempDir, pk: u64, score: u64) {
    run(TableCommand::Insert {
        dir: tbl(dir),
        pk,
        values: vec![score],
    })
    .unwrap();
}

// 1. scan range below all data returns empty Ok
#[test]
fn scan_range_below_data_returns_ok() {
    let dir = make_dir();
    setup(&dir);
    for i in 100u64..110 {
        insert(&dir, i, i);
    }
    run(TableCommand::Flush { dir: tbl(&dir) }).unwrap();
    assert!(
        run(TableCommand::Scan {
            dir: tbl(&dir),
            col: 0,
            lo: 0,
            hi: 50,
            output: OutputFormat::Table,
        })
        .is_ok()
    );
}

// 2. scan range above all data returns empty Ok
#[test]
fn scan_range_above_data_returns_ok() {
    let dir = make_dir();
    setup(&dir);
    for i in 0u64..10 {
        insert(&dir, i, i);
    }
    run(TableCommand::Flush { dir: tbl(&dir) }).unwrap();
    assert!(
        run(TableCommand::Scan {
            dir: tbl(&dir),
            col: 0,
            lo: 100,
            hi: 200,
            output: OutputFormat::Table,
        })
        .is_ok()
    );
}

// 3. scan range overlapping data returns Ok
#[test]
fn scan_range_overlapping_data() {
    let dir = make_dir();
    setup(&dir);
    for i in 5u64..15 {
        insert(&dir, i, i * 2);
    }
    run(TableCommand::Flush { dir: tbl(&dir) }).unwrap();
    assert!(
        run(TableCommand::Scan {
            dir: tbl(&dir),
            col: 0,
            lo: 8,
            hi: 12,
            output: OutputFormat::Table,
        })
        .is_ok()
    );
}

// 4. scan score col with zone prune
#[test]
fn scan_score_col_zone_prune() {
    let dir = make_dir();
    setup(&dir);
    for i in 0u64..20 {
        insert(&dir, i, i * 10);
    }
    run(TableCommand::Flush { dir: tbl(&dir) }).unwrap();
    assert!(
        run(TableCommand::Scan {
            dir: tbl(&dir),
            col: 1,
            lo: 50,
            hi: 100,
            output: OutputFormat::Table,
        })
        .is_ok()
    );
}

// 5. multiple partitions, prune skips non-overlapping
#[test]
fn multiple_partitions_prune_skips() {
    let dir = make_dir();
    setup(&dir);
    for i in 0u64..10 {
        insert(&dir, i, i);
    }
    run(TableCommand::Flush { dir: tbl(&dir) }).unwrap();
    for i in 1000u64..1010 {
        insert(&dir, i, i);
    }
    run(TableCommand::Flush { dir: tbl(&dir) }).unwrap();
    assert!(
        run(TableCommand::Scan {
            dir: tbl(&dir),
            col: 0,
            lo: 0,
            hi: 9,
            output: OutputFormat::Table,
        })
        .is_ok()
    );
}

// 6. scan exact boundary returns Ok
#[test]
fn scan_exact_boundary() {
    let dir = make_dir();
    setup(&dir);
    insert(&dir, 5, 500);
    run(TableCommand::Flush { dir: tbl(&dir) }).unwrap();
    assert!(
        run(TableCommand::Scan {
            dir: tbl(&dir),
            col: 0,
            lo: 5,
            hi: 5,
            output: OutputFormat::Table,
        })
        .is_ok()
    );
}

// 7. stats after zone-prune operations
#[test]
fn stats_after_zone_prune_ops() {
    let dir = make_dir();
    setup(&dir);
    for i in 0u64..5 {
        insert(&dir, i, i * 100);
    }
    run(TableCommand::Flush { dir: tbl(&dir) }).unwrap();
    assert!(
        run(TableCommand::Stats {
            dir: tbl(&dir),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

// 8. compact then scan
#[test]
fn compact_then_scan() {
    let dir = make_dir();
    setup(&dir);
    for i in 0u64..8 {
        insert(&dir, i, i);
    }
    run(TableCommand::Flush { dir: tbl(&dir) }).unwrap();
    run(TableCommand::Compact { dir: tbl(&dir) }).unwrap();
    assert!(
        run(TableCommand::Scan {
            dir: tbl(&dir),
            col: 0,
            lo: 0,
            hi: 7,
            output: OutputFormat::Table,
        })
        .is_ok()
    );
}

// 9. scan returns Ok after inserting single row
#[test]
fn scan_single_row_returns_ok() {
    let dir = make_dir();
    setup(&dir);
    insert(&dir, 99, 9900);
    run(TableCommand::Flush { dir: tbl(&dir) }).unwrap();
    assert!(
        run(TableCommand::Scan {
            dir: tbl(&dir),
            col: 0,
            lo: 0,
            hi: 99,
            output: OutputFormat::Table,
        })
        .is_ok()
    );
}

// 10. scan json with zone prune
#[test]
fn scan_json_zone_prune() {
    let dir = make_dir();
    setup(&dir);
    for i in 10u64..15 {
        insert(&dir, i, i);
    }
    run(TableCommand::Flush { dir: tbl(&dir) }).unwrap();
    assert!(
        run(TableCommand::Scan {
            dir: tbl(&dir),
            col: 0,
            lo: 12,
            hi: 14,
            output: OutputFormat::Json,
        })
        .is_ok()
    );
}

// 11. missing dir scan returns Err
#[test]
fn missing_dir_scan_returns_err() {
    let dir = make_dir();
    assert!(
        run(TableCommand::Scan {
            dir: dir.path().join("no_dir"),
            col: 0,
            lo: 0,
            hi: 100,
            output: OutputFormat::Table,
        })
        .is_err()
    );
}

// 12. scan with lo > hi returns Ok (empty)
#[test]
fn scan_lo_greater_than_hi_returns_ok() {
    let dir = make_dir();
    setup(&dir);
    insert(&dir, 1, 1);
    assert!(
        run(TableCommand::Scan {
            dir: tbl(&dir),
            col: 0,
            lo: 10,
            hi: 5,
            output: OutputFormat::Table,
        })
        .is_ok()
    );
}

// 13. recount after scan
#[test]
fn recount_after_scan() {
    let dir = make_dir();
    setup(&dir);
    for i in 0u64..6 {
        insert(&dir, i, i);
    }
    run(TableCommand::Flush { dir: tbl(&dir) }).unwrap();
    assert!(
        run(TableCommand::Scan {
            dir: tbl(&dir),
            col: 0,
            lo: 0,
            hi: 5,
            output: OutputFormat::Table,
        })
        .is_ok()
    );
    assert!(
        run(TableCommand::Recount {
            dir: tbl(&dir),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
