//! Tests for Layer 271: vvctl table TypedInsert command (VV-CTL-271).

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

#[test]
fn typed_insert_u64_col_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::TypedInsert {
        dir: tdir(&d),
        pk: "1".into(),
        values: vec!["100".into()],
    })
    .unwrap();
}

#[test]
fn typed_insert_two_values_ok() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: tdir(&d),
        columns: vec!["pk:u64".into(), "a:u64".into(), "b:u64".into()],
    })
    .unwrap();
    run(TableCommand::TypedInsert {
        dir: tdir(&d),
        pk: "1".into(),
        values: vec!["10".into(), "20".into()],
    })
    .unwrap();
}

#[test]
fn typed_insert_pk0_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::TypedInsert {
        dir: tdir(&d),
        pk: "0".into(),
        values: vec!["50".into()],
    })
    .unwrap();
}

#[test]
fn typed_insert_pk_max_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::TypedInsert {
        dir: tdir(&d),
        pk: (u64::MAX - 1).to_string(),
        values: vec!["1".into()],
    })
    .unwrap();
}

#[test]
fn typed_insert_then_get_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::TypedInsert {
        dir: tdir(&d),
        pk: "5".into(),
        values: vec!["55".into()],
    })
    .unwrap();
    run(TableCommand::Get {
        dir: tdir(&d),
        pk: 5,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn typed_insert_then_scan_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::TypedInsert {
        dir: tdir(&d),
        pk: "3".into(),
        values: vec!["33".into()],
    })
    .unwrap();
    run(TableCommand::Scan {
        dir: tdir(&d),
        col: 0,
        lo: 0,
        hi: u64::MAX,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn typed_insert_then_count_1() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::TypedInsert {
        dir: tdir(&d),
        pk: "7".into(),
        values: vec!["77".into()],
    })
    .unwrap();
    run(TableCommand::Count {
        dir: tdir(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn typed_insert_two_rows_count_2() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::TypedInsert {
        dir: tdir(&d),
        pk: "1".into(),
        values: vec!["10".into()],
    })
    .unwrap();
    run(TableCommand::TypedInsert {
        dir: tdir(&d),
        pk: "2".into(),
        values: vec!["20".into()],
    })
    .unwrap();
    run(TableCommand::Count {
        dir: tdir(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn typed_insert_missing_dir_errors() {
    let d = make_dir();
    let res = run(TableCommand::TypedInsert {
        dir: d.path().join("nope"),
        pk: "1".into(),
        values: vec!["10".into()],
    });
    assert!(res.is_err());
}

#[test]
fn typed_insert_and_update_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::TypedInsert {
        dir: tdir(&d),
        pk: "9".into(),
        values: vec!["90".into()],
    })
    .unwrap();
    run(TableCommand::Update {
        dir: tdir(&d),
        pk: 9,
        col: 1,
        val: 99,
    })
    .unwrap();
}

#[test]
fn typed_insert_after_flush_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    run(TableCommand::TypedInsert {
        dir: tdir(&d),
        pk: "11".into(),
        values: vec!["110".into()],
    })
    .unwrap();
}

#[test]
fn typed_insert_json_output_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::TypedInsert {
        dir: tdir(&d),
        pk: "4".into(),
        values: vec!["44".into()],
    })
    .unwrap();
    run(TableCommand::Get {
        dir: tdir(&d),
        pk: 4,
        output: OutputFormat::Json,
    })
    .unwrap();
}

#[test]
fn typed_insert_large_value_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::TypedInsert {
        dir: tdir(&d),
        pk: "99".into(),
        values: vec!["9999999999".into()],
    })
    .unwrap();
}
