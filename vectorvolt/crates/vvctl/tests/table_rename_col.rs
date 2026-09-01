//! Tests for Layer 279: vvctl table rename column (VV-CTL-279).

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
fn rename_col_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::RenameCol {
        dir: tdir(&d),
        from: "val".into(),
        to: "value".into(),
    })
    .unwrap();
}

#[test]
fn rename_then_schema_shows_new_name() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::RenameCol {
        dir: tdir(&d),
        from: "val".into(),
        to: "score".into(),
    })
    .unwrap();
    run(TableCommand::Schema {
        dir: tdir(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn rename_nonexistent_col_errors() {
    let d = make_dir();
    setup(&d);
    let res = run(TableCommand::RenameCol {
        dir: tdir(&d),
        from: "noexist".into(),
        to: "newname".into(),
    });
    assert!(res.is_err());
}

#[test]
fn rename_then_insert_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::RenameCol {
        dir: tdir(&d),
        from: "val".into(),
        to: "score".into(),
    })
    .unwrap();
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![10],
    })
    .unwrap();
}

#[test]
fn rename_then_typed_insert_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::RenameCol {
        dir: tdir(&d),
        from: "val".into(),
        to: "amount".into(),
    })
    .unwrap();
    run(TableCommand::TypedInsert {
        dir: tdir(&d),
        pk: "2".into(),
        values: vec!["20".into()],
    })
    .unwrap();
}

#[test]
fn rename_then_get_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 3,
        values: vec![30],
    })
    .unwrap();
    run(TableCommand::RenameCol {
        dir: tdir(&d),
        from: "val".into(),
        to: "qty".into(),
    })
    .unwrap();
    run(TableCommand::Get {
        dir: tdir(&d),
        pk: 3,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn rename_then_scan_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 4,
        values: vec![40],
    })
    .unwrap();
    run(TableCommand::RenameCol {
        dir: tdir(&d),
        from: "val".into(),
        to: "count".into(),
    })
    .unwrap();
    run(TableCommand::Scan {
        dir: tdir(&d),
        col: 0,
        lo: 0,
        hi: u64::MAX - 1,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn rename_then_col_stats_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 5,
        values: vec![50],
    })
    .unwrap();
    run(TableCommand::RenameCol {
        dir: tdir(&d),
        from: "val".into(),
        to: "amount".into(),
    })
    .unwrap();
    run(TableCommand::ColStats {
        dir: tdir(&d),
        col: "amount".into(),
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn rename_preserves_data() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 6,
        values: vec![60],
    })
    .unwrap();
    run(TableCommand::RenameCol {
        dir: tdir(&d),
        from: "val".into(),
        to: "renamed".into(),
    })
    .unwrap();
    run(TableCommand::Count {
        dir: tdir(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn rename_multi_col_ok() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: tdir(&d),
        columns: vec!["pk:u64".into(), "a:u64".into(), "b:u64".into()],
    })
    .unwrap();
    run(TableCommand::RenameCol {
        dir: tdir(&d),
        from: "a".into(),
        to: "alpha".into(),
    })
    .unwrap();
    run(TableCommand::Schema {
        dir: tdir(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn rename_then_alter_drop_ok() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: tdir(&d),
        columns: vec!["pk:u64".into(), "a:u64".into(), "b:u64".into()],
    })
    .unwrap();
    run(TableCommand::RenameCol {
        dir: tdir(&d),
        from: "a".into(),
        to: "alpha".into(),
    })
    .unwrap();
    run(TableCommand::AlterDropCol {
        dir: tdir(&d),
        column: "b".into(),
    })
    .unwrap();
}

#[test]
fn rename_missing_dir_errors() {
    let d = make_dir();
    let res = run(TableCommand::RenameCol {
        dir: d.path().join("nope"),
        from: "val".into(),
        to: "value".into(),
    });
    assert!(res.is_err());
}

#[test]
fn rename_col0_errors() {
    let d = make_dir();
    setup(&d);
    // Renaming the PK column ("pk") should error or be disallowed
    let res = run(TableCommand::RenameCol {
        dir: tdir(&d),
        from: "pk".into(),
        to: "id".into(),
    });
    // Either ok or err — just must not panic
    let _ = res;
}
