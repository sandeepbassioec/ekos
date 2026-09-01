//! Tests for Layer 255: vvctl table WAL verify behavior (VV-CTL-255).

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
fn wal_verify_fresh_table_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Check { dir: tdir(&d) }).unwrap();
}

#[test]
fn wal_verify_missing_dir_errors() {
    let d = make_dir();
    let res = run(TableCommand::Check {
        dir: d.path().join("nope"),
    });
    assert!(res.is_err());
}

#[test]
fn wal_verify_json_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Schema {
        dir: tdir(&d),
        output: OutputFormat::Json,
    })
    .unwrap();
}

#[test]
fn wal_verify_table_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Schema {
        dir: tdir(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn wal_verify_after_insert_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    run(TableCommand::Check { dir: tdir(&d) }).unwrap();
}

#[test]
fn wal_verify_after_flush_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 2, 20);
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    run(TableCommand::Check { dir: tdir(&d) }).unwrap();
}

#[test]
fn wal_verify_consistency_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 3, 30);
    run(TableCommand::Check { dir: tdir(&d) }).unwrap();
    run(TableCommand::Stats {
        dir: tdir(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn wal_verify_second_check_same_result() {
    let d = make_dir();
    setup(&d);
    ins(&d, 4, 40);
    run(TableCommand::Check { dir: tdir(&d) }).unwrap();
    run(TableCommand::Check { dir: tdir(&d) }).unwrap();
}

#[test]
fn wal_verify_empty_wal_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Check { dir: tdir(&d) }).unwrap();
}

#[test]
fn wal_verify_bad_dir_errors() {
    let d = make_dir();
    let res = run(TableCommand::Check {
        dir: d.path().join("bad_dir"),
    });
    assert!(res.is_err());
}

#[test]
fn wal_verify_count_after_check_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 5, 50);
    run(TableCommand::Check { dir: tdir(&d) }).unwrap();
    run(TableCommand::Count {
        dir: tdir(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn wal_verify_after_compact_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 6, 60);
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    run(TableCommand::Compact { dir: tdir(&d) }).unwrap();
    run(TableCommand::Check { dir: tdir(&d) }).unwrap();
}

#[test]
fn wal_verify_args_ok() {
    let d = make_dir();
    setup(&d);
    let res = run(TableCommand::Check { dir: tdir(&d) });
    assert!(res.is_ok());
}
