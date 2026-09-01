//! Tests for Layer 259: vvctl table multi-column operations (VV-CTL-259).

use std::path::PathBuf;
use tempfile::TempDir;
use vvctl::commands::table::{OutputFormat, TableCommand, run};

fn make_dir() -> TempDir {
    TempDir::new().unwrap()
}
fn tdir(d: &TempDir) -> PathBuf {
    d.path().join("tbl")
}
fn setup4(d: &TempDir) {
    run(TableCommand::Create {
        dir: tdir(d),
        columns: vec![
            "pk:u64".into(),
            "a:u64".into(),
            "b:u64".into(),
            "c:u64".into(),
        ],
    })
    .unwrap();
}

#[test]
fn multi_col_create_4col_ok() {
    let d = make_dir();
    setup4(&d);
    run(TableCommand::Schema {
        dir: tdir(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn multi_col_insert_3vals_ok() {
    let d = make_dir();
    setup4(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![10, 20, 30],
    })
    .unwrap();
}

#[test]
fn multi_col_get_returns_values() {
    let d = make_dir();
    setup4(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![10, 20, 30],
    })
    .unwrap();
    run(TableCommand::Get {
        dir: tdir(&d),
        pk: 1,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn multi_col_scan_col2_ok() {
    let d = make_dir();
    setup4(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![10, 20, 30],
    })
    .unwrap();
    run(TableCommand::Scan {
        dir: tdir(&d),
        col: 2,
        lo: 0,
        hi: u64::MAX,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn multi_col_scan_col3_ok() {
    let d = make_dir();
    setup4(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![10, 20, 30],
    })
    .unwrap();
    run(TableCommand::Scan {
        dir: tdir(&d),
        col: 3,
        lo: 0,
        hi: u64::MAX,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn multi_col_update_col2_ok() {
    let d = make_dir();
    setup4(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![10, 20, 30],
    })
    .unwrap();
    run(TableCommand::Update {
        dir: tdir(&d),
        pk: 1,
        col: 2,
        val: 99,
    })
    .unwrap();
}

#[test]
fn multi_col_update_col3_ok() {
    let d = make_dir();
    setup4(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![10, 20, 30],
    })
    .unwrap();
    run(TableCommand::Update {
        dir: tdir(&d),
        pk: 1,
        col: 3,
        val: 88,
    })
    .unwrap();
}

#[test]
fn multi_col_colstats_col2_ok() {
    let d = make_dir();
    setup4(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![10, 20, 30],
    })
    .unwrap();
    run(TableCommand::ColStats {
        dir: tdir(&d),
        col: "b".into(),
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn multi_col_colstats_col3_ok() {
    let d = make_dir();
    setup4(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![10, 20, 30],
    })
    .unwrap();
    run(TableCommand::ColStats {
        dir: tdir(&d),
        col: "c".into(),
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn multi_col_topn_col2_ok() {
    let d = make_dir();
    setup4(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![10, 20, 30],
    })
    .unwrap();
    run(TableCommand::TopN {
        dir: tdir(&d),
        col: 2,
        n: 1,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn multi_col_bottomn_col3_ok() {
    let d = make_dir();
    setup4(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![10, 20, 30],
    })
    .unwrap();
    run(TableCommand::BottomN {
        dir: tdir(&d),
        col: 3,
        n: 1,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn multi_col_sumfield_col2_ok() {
    let d = make_dir();
    setup4(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![10, 20, 30],
    })
    .unwrap();
    run(TableCommand::SumField {
        dir: tdir(&d),
        col: 2,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn multi_col_avgfield_col3_ok() {
    let d = make_dir();
    setup4(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![10, 20, 30],
    })
    .unwrap();
    run(TableCommand::AvgField {
        dir: tdir(&d),
        col: 3,
        output: OutputFormat::Table,
    })
    .unwrap();
}
