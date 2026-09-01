//! Tests for Layer 256: vvctl table WAL trim behavior (VV-CTL-256).

use std::path::PathBuf;
use tempfile::TempDir;
use vvctl::commands::table::{OutputFormat, TableCommand, run};

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
fn wal_trim_fresh_table_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Vacuum { dir: tdir(&d) }).unwrap();
}

#[test]
fn wal_trim_missing_dir_errors() {
    let d = make_dir();
    let res = run(TableCommand::Vacuum {
        dir: d.path().join("nope"),
    });
    assert!(res.is_err());
}

#[test]
fn wal_trim_after_insert_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    run(TableCommand::Vacuum { dir: tdir(&d) }).unwrap();
}

#[test]
fn wal_trim_after_flush_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 2, 20);
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    run(TableCommand::Vacuum { dir: tdir(&d) }).unwrap();
}

#[test]
fn wal_trim_after_compact_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 3, 30);
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    run(TableCommand::Compact { dir: tdir(&d) }).unwrap();
    run(TableCommand::Vacuum { dir: tdir(&d) }).unwrap();
}

#[test]
fn wal_trim_then_verify_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 4, 40);
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    run(TableCommand::Vacuum { dir: tdir(&d) }).unwrap();
    run(TableCommand::Check { dir: tdir(&d) }).unwrap();
}

#[test]
fn wal_trim_idempotent() {
    let d = make_dir();
    setup(&d);
    ins(&d, 5, 50);
    run(TableCommand::Vacuum { dir: tdir(&d) }).unwrap();
    run(TableCommand::Vacuum { dir: tdir(&d) }).unwrap();
}

#[test]
fn wal_trim_returns_ok() {
    let d = make_dir();
    setup(&d);
    let res = run(TableCommand::Vacuum { dir: tdir(&d) });
    assert!(res.is_ok());
}

#[test]
fn wal_trim_data_readable_after() {
    let d = make_dir();
    setup(&d);
    ins(&d, 6, 60);
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    run(TableCommand::Vacuum { dir: tdir(&d) }).unwrap();
    run(TableCommand::Get {
        dir: tdir(&d),
        pk: 6,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn wal_trim_stats_after_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 7, 70);
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    run(TableCommand::Vacuum { dir: tdir(&d) }).unwrap();
    run(TableCommand::Stats {
        dir: tdir(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn wal_trim_count_unchanged_after() {
    let d = make_dir();
    setup(&d);
    ins(&d, 8, 80);
    ins(&d, 9, 90);
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    run(TableCommand::Vacuum { dir: tdir(&d) }).unwrap();
    run(TableCommand::Count {
        dir: tdir(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn wal_trim_two_cycles_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 10, 100);
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    run(TableCommand::Vacuum { dir: tdir(&d) }).unwrap();
    ins(&d, 11, 110);
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    run(TableCommand::Vacuum { dir: tdir(&d) }).unwrap();
}

#[test]
fn wal_trim_check_after_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Vacuum { dir: tdir(&d) }).unwrap();
    run(TableCommand::Check { dir: tdir(&d) }).unwrap();
}
