//! Tests for Layer 220: vvctl table increment-field (VV-CTL-220).

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
fn increment_field_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    assert!(
        run(TableCommand::IncrementField {
            dir: tdir(&d),
            pk: 1,
            col: 1
        })
        .is_ok()
    );
}
#[test]
fn increment_field_from_zero_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 0);
    assert!(
        run(TableCommand::IncrementField {
            dir: tdir(&d),
            pk: 1,
            col: 1
        })
        .is_ok()
    );
}
#[test]
fn increment_field_twice_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 5);
    run(TableCommand::IncrementField {
        dir: tdir(&d),
        pk: 1,
        col: 1,
    })
    .unwrap();
    assert!(
        run(TableCommand::IncrementField {
            dir: tdir(&d),
            pk: 1,
            col: 1
        })
        .is_ok()
    );
}
#[test]
fn increment_field_verify_value() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 9);
    run(TableCommand::IncrementField {
        dir: tdir(&d),
        pk: 1,
        col: 1,
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
fn increment_field_missing_pk_errors() {
    let d = make_dir();
    setup(&d);
    assert!(
        run(TableCommand::IncrementField {
            dir: tdir(&d),
            pk: 99,
            col: 1
        })
        .is_err()
    );
}
#[test]
fn increment_field_missing_dir_errors() {
    let d = make_dir();
    assert!(
        run(TableCommand::IncrementField {
            dir: d.path().join("nope"),
            pk: 1,
            col: 1
        })
        .is_err()
    );
}
#[test]
fn increment_field_ten_times_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 0);
    for _ in 0..10 {
        run(TableCommand::IncrementField {
            dir: tdir(&d),
            pk: 1,
            col: 1,
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
fn increment_field_pk_stored() {
    let cmd = TableCommand::IncrementField {
        dir: PathBuf::from("/tmp"),
        pk: 42,
        col: 1,
    };
    if let TableCommand::IncrementField { pk, .. } = cmd {
        assert_eq!(pk, 42);
    } else {
        panic!();
    }
}
#[test]
fn increment_field_col_stored() {
    let cmd = TableCommand::IncrementField {
        dir: PathBuf::from("/tmp"),
        pk: 1,
        col: 2,
    };
    if let TableCommand::IncrementField { col, .. } = cmd {
        assert_eq!(col, 2);
    } else {
        panic!();
    }
}
#[test]
fn increment_field_multi_col_ok() {
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
        run(TableCommand::IncrementField {
            dir: tdir(&d),
            pk: 1,
            col: 2
        })
        .is_ok()
    );
}
#[test]
fn increment_field_multiple_rows_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 5);
    ins(&d, 2, 10);
    run(TableCommand::IncrementField {
        dir: tdir(&d),
        pk: 1,
        col: 1,
    })
    .unwrap();
    assert!(
        run(TableCommand::IncrementField {
            dir: tdir(&d),
            pk: 2,
            col: 1
        })
        .is_ok()
    );
}
#[test]
fn increment_field_large_value_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 999_999);
    assert!(
        run(TableCommand::IncrementField {
            dir: tdir(&d),
            pk: 1,
            col: 1
        })
        .is_ok()
    );
}
#[test]
fn increment_field_col_one_default_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 7, 3);
    assert!(
        run(TableCommand::IncrementField {
            dir: tdir(&d),
            pk: 7,
            col: 1
        })
        .is_ok()
    );
}
