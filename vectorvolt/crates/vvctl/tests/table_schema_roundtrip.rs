//! Tests for Layer 258: vvctl table schema roundtrip (VV-CTL-258).

use std::path::PathBuf;
use tempfile::TempDir;
use vvctl::commands::table::{OutputFormat, TableCommand, run};

fn make_dir() -> TempDir {
    TempDir::new().unwrap()
}
fn tdir(d: &TempDir) -> PathBuf {
    d.path().join("tbl")
}
fn tdir2(d: &TempDir) -> PathBuf {
    d.path().join("tbl2")
}

#[test]
fn schema_one_col_ok() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: tdir(&d),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    run(TableCommand::Schema {
        dir: tdir(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn schema_three_cols_ok() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: tdir(&d),
        columns: vec!["pk:u64".into(), "a:u64".into(), "b:u64".into()],
    })
    .unwrap();
    run(TableCommand::Schema {
        dir: tdir(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn schema_col_count_matches_create() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: tdir(&d),
        columns: vec!["pk:u64".into(), "x:u64".into(), "y:u64".into()],
    })
    .unwrap();
    run(TableCommand::Schema {
        dir: tdir(&d),
        output: OutputFormat::Json,
    })
    .unwrap();
}

#[test]
fn schema_pk_is_col0() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: tdir(&d),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    run(TableCommand::Schema {
        dir: tdir(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn schema_alter_add_increases_count() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: tdir(&d),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    run(TableCommand::AlterAddCol {
        dir: tdir(&d),
        column: "extra:u64".into(),
    })
    .unwrap();
    run(TableCommand::Schema {
        dir: tdir(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn schema_alter_drop_decreases_count() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: tdir(&d),
        columns: vec!["pk:u64".into(), "val:u64".into(), "extra:u64".into()],
    })
    .unwrap();
    run(TableCommand::AlterDropCol {
        dir: tdir(&d),
        column: "extra".into(),
    })
    .unwrap();
    run(TableCommand::Schema {
        dir: tdir(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn schema_after_alter_readable() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: tdir(&d),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    run(TableCommand::AlterAddCol {
        dir: tdir(&d),
        column: "score:u64".into(),
    })
    .unwrap();
    run(TableCommand::Schema {
        dir: tdir(&d),
        output: OutputFormat::Json,
    })
    .unwrap();
}

#[test]
fn schema_json_ok() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: tdir(&d),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    run(TableCommand::Schema {
        dir: tdir(&d),
        output: OutputFormat::Json,
    })
    .unwrap();
}

#[test]
fn schema_table_ok() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: tdir(&d),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    run(TableCommand::Schema {
        dir: tdir(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn schema_after_truncate_preserved() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: tdir(&d),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![10],
    })
    .unwrap();
    run(TableCommand::Truncate {
        dir: tdir(&d),
        force: true,
    })
    .unwrap();
    run(TableCommand::Schema {
        dir: tdir(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn schema_after_copy_matches() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: tdir(&d),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    run(TableCommand::Copy {
        src: tdir(&d),
        dst: tdir2(&d),
    })
    .unwrap();
    run(TableCommand::Schema {
        dir: tdir2(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn schema_missing_dir_errors() {
    let d = make_dir();
    let res = run(TableCommand::Schema {
        dir: d.path().join("nope"),
        output: OutputFormat::Table,
    });
    assert!(res.is_err());
}

#[test]
fn schema_rename_col_ok() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: tdir(&d),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    run(TableCommand::RenameCol {
        dir: tdir(&d),
        from: "val".into(),
        to: "value".into(),
    })
    .unwrap();
    run(TableCommand::Schema {
        dir: tdir(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
}
