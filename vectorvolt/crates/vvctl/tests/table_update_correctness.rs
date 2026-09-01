//! Tests for Layer 252: vvctl table update deep correctness (VV-CTL-252).

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
fn update_col_value_changes() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    run(TableCommand::Update {
        dir: tdir(&d),
        pk: 1,
        col: 1,
        val: 99,
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
fn update_col0_errors() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    let res = run(TableCommand::Update {
        dir: tdir(&d),
        pk: 1,
        col: 0,
        val: 99,
    });
    assert!(res.is_err());
}

#[test]
fn update_missing_pk_errors() {
    let d = make_dir();
    setup(&d);
    let res = run(TableCommand::Update {
        dir: tdir(&d),
        pk: 42,
        col: 1,
        val: 99,
    });
    assert!(res.is_err());
}

#[test]
fn update_pk1_val99_readable() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    run(TableCommand::Update {
        dir: tdir(&d),
        pk: 1,
        col: 1,
        val: 99,
    })
    .unwrap();
    run(TableCommand::Get {
        dir: tdir(&d),
        pk: 1,
        output: OutputFormat::Json,
    })
    .unwrap();
}

#[test]
fn update_json_output_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 5, 50);
    run(TableCommand::Update {
        dir: tdir(&d),
        pk: 5,
        col: 1,
        val: 77,
    })
    .unwrap();
    run(TableCommand::Get {
        dir: tdir(&d),
        pk: 5,
        output: OutputFormat::Json,
    })
    .unwrap();
}

#[test]
fn update_then_scan_reflects_new_value() {
    let d = make_dir();
    setup(&d);
    ins(&d, 3, 30);
    run(TableCommand::Update {
        dir: tdir(&d),
        pk: 3,
        col: 1,
        val: 300,
    })
    .unwrap();
    run(TableCommand::Scan {
        dir: tdir(&d),
        col: 1,
        lo: 300,
        hi: 300,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn update_preserves_other_rows() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 100);
    ins(&d, 2, 200);
    run(TableCommand::Update {
        dir: tdir(&d),
        pk: 1,
        col: 1,
        val: 999,
    })
    .unwrap();
    run(TableCommand::Get {
        dir: tdir(&d),
        pk: 2,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn update_twice_second_wins() {
    let d = make_dir();
    setup(&d);
    ins(&d, 7, 70);
    run(TableCommand::Update {
        dir: tdir(&d),
        pk: 7,
        col: 1,
        val: 100,
    })
    .unwrap();
    run(TableCommand::Update {
        dir: tdir(&d),
        pk: 7,
        col: 1,
        val: 200,
    })
    .unwrap();
    run(TableCommand::Get {
        dir: tdir(&d),
        pk: 7,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn update_multi_col_table_ok() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: tdir(&d),
        columns: vec!["pk:u64".into(), "a:u64".into(), "b:u64".into()],
    })
    .unwrap();
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![10, 20],
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
fn update_round_trip() {
    let d = make_dir();
    setup(&d);
    ins(&d, 9, 90);
    run(TableCommand::Update {
        dir: tdir(&d),
        pk: 9,
        col: 1,
        val: 42,
    })
    .unwrap();
    run(TableCommand::Get {
        dir: tdir(&d),
        pk: 9,
        output: OutputFormat::Json,
    })
    .unwrap();
}

#[test]
fn update_after_flush_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 4, 40);
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    run(TableCommand::Update {
        dir: tdir(&d),
        pk: 4,
        col: 1,
        val: 400,
    })
    .unwrap();
}

#[test]
fn update_pk_eq_zero_updates_first_inserted() {
    let d = make_dir();
    setup(&d);
    ins(&d, 0, 5);
    run(TableCommand::Update {
        dir: tdir(&d),
        pk: 0,
        col: 1,
        val: 55,
    })
    .unwrap();
    run(TableCommand::Get {
        dir: tdir(&d),
        pk: 0,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn update_large_val_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 100, 1);
    run(TableCommand::Update {
        dir: tdir(&d),
        pk: 100,
        col: 1,
        val: u64::MAX,
    })
    .unwrap();
}
