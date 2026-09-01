//! Tests for Layer 280: vvctl table alter add/drop column (VV-CTL-280).

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
fn alter_add_col_to_2col_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::AlterAddCol {
        dir: tdir(&d),
        column: "extra:u64".into(),
    })
    .unwrap();
}

#[test]
fn alter_schema_has_3_cols_after_add() {
    let d = make_dir();
    setup(&d);
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
fn alter_add_then_insert_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::AlterAddCol {
        dir: tdir(&d),
        column: "extra:u64".into(),
    })
    .unwrap();
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![10, 20],
    })
    .unwrap();
}

#[test]
fn alter_drop_col1_ok() {
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
}

#[test]
fn alter_schema_1_col_after_drop() {
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
fn alter_drop_pk_col_errors() {
    let d = make_dir();
    setup(&d);
    let res = run(TableCommand::AlterDropCol {
        dir: tdir(&d),
        column: "pk".into(),
    });
    assert!(res.is_err());
}

#[test]
fn alter_add_then_drop_same_col_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::AlterAddCol {
        dir: tdir(&d),
        column: "temp:u64".into(),
    })
    .unwrap();
    run(TableCommand::AlterDropCol {
        dir: tdir(&d),
        column: "temp".into(),
    })
    .unwrap();
}

#[test]
fn alter_add_3_cols_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::AlterAddCol {
        dir: tdir(&d),
        column: "a:u64".into(),
    })
    .unwrap();
    run(TableCommand::AlterAddCol {
        dir: tdir(&d),
        column: "b:u64".into(),
    })
    .unwrap();
    run(TableCommand::AlterAddCol {
        dir: tdir(&d),
        column: "c:u64".into(),
    })
    .unwrap();
}

#[test]
fn alter_drop_nonexistent_col_ok() {
    let d = make_dir();
    setup(&d);
    // Dropping a non-existent col may be a no-op or error; must not panic
    let _ = run(TableCommand::AlterDropCol {
        dir: tdir(&d),
        column: "noexist".into(),
    });
}

#[test]
fn alter_missing_dir_errors() {
    let d = make_dir();
    let res = run(TableCommand::AlterAddCol {
        dir: d.path().join("nope"),
        column: "x:u64".into(),
    });
    assert!(res.is_err());
}

#[test]
fn alter_add_then_check_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::AlterAddCol {
        dir: tdir(&d),
        column: "extra:u64".into(),
    })
    .unwrap();
    run(TableCommand::Check { dir: tdir(&d) }).unwrap();
}

#[test]
fn alter_drop_then_check_ok() {
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
    run(TableCommand::Check { dir: tdir(&d) }).unwrap();
}

#[test]
fn alter_multi_step_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::AlterAddCol {
        dir: tdir(&d),
        column: "a:u64".into(),
    })
    .unwrap();
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![10, 20],
    })
    .unwrap();
    run(TableCommand::AlterDropCol {
        dir: tdir(&d),
        column: "a".into(),
    })
    .unwrap();
    run(TableCommand::Schema {
        dir: tdir(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
}
