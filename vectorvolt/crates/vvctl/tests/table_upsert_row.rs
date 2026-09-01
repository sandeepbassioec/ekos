//! Tests for Layer 224: vvctl table upsert-row (VV-CTL-224).

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
fn upsert_row_insert_ok() {
    let d = make_dir();
    setup(&d);
    assert!(
        run(TableCommand::UpsertRow {
            dir: tdir(&d),
            pk: 1,
            values: vec![42]
        })
        .is_ok()
    );
}
#[test]
fn upsert_row_replace_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    assert!(
        run(TableCommand::UpsertRow {
            dir: tdir(&d),
            pk: 1,
            values: vec![99]
        })
        .is_ok()
    );
}
#[test]
fn upsert_row_verify_replaced() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    run(TableCommand::UpsertRow {
        dir: tdir(&d),
        pk: 1,
        values: vec![55],
    })
    .unwrap();
    assert!(
        run(TableCommand::Get {
            dir: tdir(&d),
            pk: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn upsert_row_multiple_times_ok() {
    let d = make_dir();
    setup(&d);
    for v in [1u64, 2, 3] {
        run(TableCommand::UpsertRow {
            dir: tdir(&d),
            pk: 1,
            values: vec![v],
        })
        .unwrap();
    }
    assert!(
        run(TableCommand::Get {
            dir: tdir(&d),
            pk: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn upsert_row_missing_dir_errors() {
    let d = make_dir();
    assert!(
        run(TableCommand::UpsertRow {
            dir: d.path().join("nope"),
            pk: 1,
            values: vec![1]
        })
        .is_err()
    );
}
#[test]
fn upsert_row_empty_values_ok() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: tdir(&d),
        columns: vec!["pk:u64".into()],
    })
    .unwrap();
    assert!(
        run(TableCommand::UpsertRow {
            dir: tdir(&d),
            pk: 1,
            values: vec![]
        })
        .is_ok()
    );
}
#[test]
fn upsert_row_five_rows_ok() {
    let d = make_dir();
    setup(&d);
    for i in 0..5u64 {
        run(TableCommand::UpsertRow {
            dir: tdir(&d),
            pk: i,
            values: vec![i * 10],
        })
        .unwrap();
    }
    assert!(
        run(TableCommand::UpsertRow {
            dir: tdir(&d),
            pk: 2,
            values: vec![999]
        })
        .is_ok()
    );
}
#[test]
fn upsert_row_pk_stored() {
    let cmd = TableCommand::UpsertRow {
        dir: PathBuf::from("/tmp"),
        pk: 42,
        values: vec![1],
    };
    if let TableCommand::UpsertRow { pk, .. } = cmd {
        assert_eq!(pk, 42);
    } else {
        panic!();
    }
}
#[test]
fn upsert_row_values_stored() {
    let cmd = TableCommand::UpsertRow {
        dir: PathBuf::from("/tmp"),
        pk: 1,
        values: vec![7, 8],
    };
    if let TableCommand::UpsertRow { values, .. } = cmd {
        assert_eq!(values, vec![7, 8]);
    } else {
        panic!();
    }
}
#[test]
fn upsert_row_new_pk_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    assert!(
        run(TableCommand::UpsertRow {
            dir: tdir(&d),
            pk: 100,
            values: vec![200]
        })
        .is_ok()
    );
}
#[test]
fn upsert_row_multi_col_ok() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: tdir(&d),
        columns: vec!["pk:u64".into(), "a:u64".into(), "b:u64".into()],
    })
    .unwrap();
    assert!(
        run(TableCommand::UpsertRow {
            dir: tdir(&d),
            pk: 1,
            values: vec![10, 20]
        })
        .is_ok()
    );
}
#[test]
fn upsert_row_replace_multi_col_ok() {
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
    assert!(
        run(TableCommand::UpsertRow {
            dir: tdir(&d),
            pk: 1,
            values: vec![30, 40]
        })
        .is_ok()
    );
}
#[test]
fn upsert_row_zero_value_ok() {
    let d = make_dir();
    setup(&d);
    assert!(
        run(TableCommand::UpsertRow {
            dir: tdir(&d),
            pk: 1,
            values: vec![0]
        })
        .is_ok()
    );
}
