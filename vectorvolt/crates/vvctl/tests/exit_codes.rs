//! Layer 173 — Exit codes + error prefix consistency (VV-CTL-016)

use std::path::PathBuf;
use tempfile::TempDir;
use vvctl::commands::table::{OutputFormat, TableCommand, run as table_run};
use vvctl::commands::wal::{WalCommand, run as wal_run};

fn make_dir() -> TempDir {
    TempDir::new().unwrap()
}

fn tbl(dir: &TempDir) -> PathBuf {
    dir.path().join("t")
}

// 1. table create missing columns returns Err
#[test]
fn table_create_no_cols_returns_err() {
    let dir = make_dir();
    assert!(
        table_run(TableCommand::Create {
            dir: tbl(&dir),
            columns: vec![]
        })
        .is_err()
    );
}

// 2. table get missing dir returns Err
#[test]
fn table_get_missing_dir_returns_err() {
    let dir = make_dir();
    assert!(
        table_run(TableCommand::Get {
            dir: dir.path().join("gone"),
            pk: 1,
            output: OutputFormat::Table,
        })
        .is_err()
    );
}

// 3. table create success returns Ok
#[test]
fn table_create_success_returns_ok() {
    let dir = make_dir();
    assert!(
        table_run(TableCommand::Create {
            dir: tbl(&dir),
            columns: vec!["pk:u64".into(), "val:u64".into()],
        })
        .is_ok()
    );
}

// 4. table insert success returns Ok
#[test]
fn table_insert_success_returns_ok() {
    let dir = make_dir();
    table_run(TableCommand::Create {
        dir: tbl(&dir),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    assert!(
        table_run(TableCommand::Insert {
            dir: tbl(&dir),
            pk: 1,
            values: vec![10]
        })
        .is_ok()
    );
}

// 5. table scan missing dir returns Err
#[test]
fn table_scan_missing_dir_returns_err() {
    let dir = make_dir();
    assert!(
        table_run(TableCommand::Scan {
            dir: dir.path().join("gone"),
            col: 0,
            lo: 0,
            hi: 100,
            output: OutputFormat::Table,
        })
        .is_err()
    );
}

// 6. table update missing pk returns Err
#[test]
fn table_update_missing_pk_returns_err() {
    let dir = make_dir();
    table_run(TableCommand::Create {
        dir: tbl(&dir),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    assert!(
        table_run(TableCommand::Update {
            dir: tbl(&dir),
            pk: 999,
            col: 1,
            val: 1
        })
        .is_err()
    );
}

// 7. table flush success returns Ok
#[test]
fn table_flush_success_returns_ok() {
    let dir = make_dir();
    table_run(TableCommand::Create {
        dir: tbl(&dir),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    assert!(table_run(TableCommand::Flush { dir: tbl(&dir) }).is_ok());
}

// 8. wal stats missing dir returns Err
#[test]
fn wal_stats_missing_dir_returns_err() {
    let dir = make_dir();
    assert!(
        wal_run(WalCommand::Stats {
            dir: dir.path().join("no_wal"),
            output: vvctl::commands::wal::ReplayOutput::Table,
        })
        .is_err()
    );
}

// 9. table stats success returns Ok
#[test]
fn table_stats_success_returns_ok() {
    let dir = make_dir();
    table_run(TableCommand::Create {
        dir: tbl(&dir),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    assert!(
        table_run(TableCommand::Stats {
            dir: tbl(&dir),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

// 10. table exists success returns Ok
#[test]
fn table_exists_success_returns_ok() {
    let dir = make_dir();
    table_run(TableCommand::Create {
        dir: tbl(&dir),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    assert!(table_run(TableCommand::Exists { dir: tbl(&dir) }).is_ok());
}

// 11. table drop without force returns Err
#[test]
fn table_drop_without_force_returns_err() {
    let dir = make_dir();
    table_run(TableCommand::Create {
        dir: tbl(&dir),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    // Non-TTY with force=false should fail
    assert!(
        table_run(TableCommand::Drop {
            dir: tbl(&dir),
            force: false
        })
        .is_err()
    );
}

// 12. table drop with force returns Ok
#[test]
fn table_drop_with_force_returns_ok() {
    let dir = make_dir();
    table_run(TableCommand::Create {
        dir: tbl(&dir),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    assert!(
        table_run(TableCommand::Drop {
            dir: tbl(&dir),
            force: true
        })
        .is_ok()
    );
}

// 13. table count success returns Ok
#[test]
fn table_count_success_returns_ok() {
    let dir = make_dir();
    table_run(TableCommand::Create {
        dir: tbl(&dir),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    assert!(
        table_run(TableCommand::Count {
            dir: tbl(&dir),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
