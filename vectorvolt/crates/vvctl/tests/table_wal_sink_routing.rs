//! Tests for Layer 332: WAL sink routing (VV-CTL-332).

use std::path::PathBuf;
use tempfile::TempDir;
use vvctl::commands::table::{OutputFormat, TableCommand, run};
use vvctl::commands::wal::{ReplayOutput, WalCommand, run as wal_run};

fn make_dir() -> TempDir {
    TempDir::new().unwrap()
}
fn tdir(d: &TempDir) -> PathBuf {
    d.path().join("tbl")
}
fn setup(d: &TempDir) {
    run(TableCommand::Create {
        dir: tdir(d),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
}
fn ins(d: &TempDir, pk: u64, val: u64) {
    run(TableCommand::Insert {
        dir: tdir(d),
        pk,
        values: vec![val],
    })
    .unwrap();
}

#[test]
fn wal_stats_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    assert!(
        wal_run(WalCommand::Stats {
            dir: tdir(&d),
            output: ReplayOutput::Table
        })
        .is_ok()
    );
}

#[test]
fn wal_verify_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    assert!(
        wal_run(WalCommand::Verify {
            dir: tdir(&d).to_string_lossy().to_string(),
            output: ReplayOutput::Table
        })
        .is_ok()
    );
}

#[test]
fn wal_export_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    assert!(
        wal_run(WalCommand::Export {
            dir: tdir(&d),
            limit: None,
            output: ReplayOutput::Table
        })
        .is_ok()
    );
}

#[test]
fn wal_trim_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    assert!(
        wal_run(WalCommand::Trim {
            dir: tdir(&d),
            before: 0
        })
        .is_ok()
    );
}

#[test]
fn after_trim_verify_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    wal_run(WalCommand::Trim { dir: tdir(&d), before: 0 }).unwrap();
    assert!(
        wal_run(WalCommand::Verify {
            dir: tdir(&d).to_string_lossy().to_string(),
            output: ReplayOutput::Table
        })
        .is_ok()
    );
}

#[test]
fn after_trim_scan_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    wal_run(WalCommand::Trim { dir: tdir(&d), before: 0 }).unwrap();
    assert!(
        run(TableCommand::Scan {
            dir: tdir(&d),
            col: 0,
            lo: 0,
            hi: u64::MAX,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn after_trim_stats_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    wal_run(WalCommand::Trim { dir: tdir(&d), before: 0 }).unwrap();
    assert!(
        wal_run(WalCommand::Stats {
            dir: tdir(&d),
            output: ReplayOutput::Table
        })
        .is_ok()
    );
}

#[test]
fn after_trim_info_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    wal_run(WalCommand::Trim { dir: tdir(&d), before: 0 }).unwrap();
    assert!(
        run(TableCommand::Info {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn after_trim_check_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    wal_run(WalCommand::Trim { dir: tdir(&d), before: 0 }).unwrap();
    assert!(run(TableCommand::Check { dir: tdir(&d) }).is_ok());
}

#[test]
fn after_trim_compact_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    wal_run(WalCommand::Trim { dir: tdir(&d), before: 0 }).unwrap();
    assert!(run(TableCommand::Compact { dir: tdir(&d) }).is_ok());
}

#[test]
fn after_trim_vacuum_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    wal_run(WalCommand::Trim { dir: tdir(&d), before: 0 }).unwrap();
    assert!(run(TableCommand::Vacuum { dir: tdir(&d) }).is_ok());
}

#[test]
fn after_trim_recount_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    wal_run(WalCommand::Trim { dir: tdir(&d), before: 0 }).unwrap();
    assert!(
        run(TableCommand::Recount {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn insert_and_wal_stats() {
    let d = make_dir();
    setup(&d);
    for i in 0..3 {
        ins(&d, i, i * 10);
    }
    assert!(
        wal_run(WalCommand::Stats {
            dir: tdir(&d),
            output: ReplayOutput::Table
        })
        .is_ok()
    );
}
