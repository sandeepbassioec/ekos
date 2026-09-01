//! Tests for Layer 268: vvctl table error propagation (VV-CTL-268).

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
    run(TableCommand::Insert {
        dir: tdir(d),
        pk: 1,
        values: vec![10],
    })
    .unwrap();
}

#[test]
fn error_get_missing_dir() {
    let d = make_dir();
    let res = run(TableCommand::Get {
        dir: d.path().join("nope"),
        pk: 1,
        output: OutputFormat::Table,
    });
    assert!(res.is_err());
}

#[test]
fn error_scan_missing_dir() {
    let d = make_dir();
    let res = run(TableCommand::Scan {
        dir: d.path().join("nope"),
        col: 0,
        lo: 0,
        hi: u64::MAX,
        output: OutputFormat::Table,
    });
    assert!(res.is_err());
}

#[test]
fn error_count_missing_dir() {
    let d = make_dir();
    let res = run(TableCommand::Count {
        dir: d.path().join("nope"),
        output: OutputFormat::Table,
    });
    assert!(res.is_err());
}

#[test]
fn error_head_missing_dir() {
    let d = make_dir();
    let res = run(TableCommand::Head {
        dir: d.path().join("nope"),
        n: 5,
        output: OutputFormat::Table,
    });
    assert!(res.is_err());
}

#[test]
fn error_keys_missing_dir() {
    let d = make_dir();
    let res = run(TableCommand::Keys {
        dir: d.path().join("nope"),
        output: OutputFormat::Table,
    });
    assert!(res.is_err());
}

#[test]
fn error_info_missing_dir() {
    let d = make_dir();
    let res = run(TableCommand::Info {
        dir: d.path().join("nope"),
        output: OutputFormat::Table,
    });
    assert!(res.is_err());
}

#[test]
fn error_stats_missing_dir() {
    let d = make_dir();
    let res = run(TableCommand::Stats {
        dir: d.path().join("nope"),
        output: OutputFormat::Table,
    });
    assert!(res.is_err());
}

#[test]
fn error_schema_missing_dir() {
    let d = make_dir();
    let res = run(TableCommand::Schema {
        dir: d.path().join("nope"),
        output: OutputFormat::Table,
    });
    assert!(res.is_err());
}

#[test]
fn error_update_col0_errors() {
    let d = make_dir();
    setup(&d);
    let res = run(TableCommand::Update {
        dir: tdir(&d),
        pk: 1,
        col: 0,
        val: 99,
    });
    assert!(res.is_err());
}

#[test]
fn error_increment_col0_errors() {
    let d = make_dir();
    setup(&d);
    let res = run(TableCommand::IncrementField {
        dir: tdir(&d),
        pk: 1,
        col: 0,
    });
    assert!(res.is_err());
}

#[test]
fn error_decrement_col0_errors() {
    let d = make_dir();
    setup(&d);
    let res = run(TableCommand::DecrementField {
        dir: tdir(&d),
        pk: 1,
        col: 0,
    });
    assert!(res.is_err());
}

#[test]
fn error_cas_col0_errors() {
    let d = make_dir();
    setup(&d);
    let res = run(TableCommand::CompareAndSwap {
        dir: tdir(&d),
        pk: 1,
        col: 0,
        expected: 1,
        replacement: 99,
    });
    assert!(res.is_err());
}

#[test]
fn error_clamp_col0_errors() {
    let d = make_dir();
    setup(&d);
    let res = run(TableCommand::ClampField {
        dir: tdir(&d),
        pk: 1,
        col: 0,
        lo: 0,
        hi: 100,
    });
    assert!(res.is_err());
}
