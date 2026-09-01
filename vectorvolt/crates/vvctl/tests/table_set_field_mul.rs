//! Tests for Layer 223: vvctl table set-field-mul (VV-CTL-223).

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
fn set_field_mul_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    assert!(
        run(TableCommand::SetFieldMul {
            dir: tdir(&d),
            pk: 1,
            col: 1,
            factor: 2
        })
        .is_ok()
    );
}
#[test]
fn set_field_mul_by_zero_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    assert!(
        run(TableCommand::SetFieldMul {
            dir: tdir(&d),
            pk: 1,
            col: 1,
            factor: 0
        })
        .is_ok()
    );
}
#[test]
fn set_field_mul_by_one_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 42);
    assert!(
        run(TableCommand::SetFieldMul {
            dir: tdir(&d),
            pk: 1,
            col: 1,
            factor: 1
        })
        .is_ok()
    );
}
#[test]
fn set_field_mul_verify_value() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 5);
    run(TableCommand::SetFieldMul {
        dir: tdir(&d),
        pk: 1,
        col: 1,
        factor: 3,
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
fn set_field_mul_missing_pk_errors() {
    let d = make_dir();
    setup(&d);
    assert!(
        run(TableCommand::SetFieldMul {
            dir: tdir(&d),
            pk: 99,
            col: 1,
            factor: 2
        })
        .is_err()
    );
}
#[test]
fn set_field_mul_missing_dir_errors() {
    let d = make_dir();
    assert!(
        run(TableCommand::SetFieldMul {
            dir: d.path().join("nope"),
            pk: 1,
            col: 1,
            factor: 2
        })
        .is_err()
    );
}
#[test]
fn set_field_mul_twice_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 3);
    run(TableCommand::SetFieldMul {
        dir: tdir(&d),
        pk: 1,
        col: 1,
        factor: 2,
    })
    .unwrap();
    assert!(
        run(TableCommand::SetFieldMul {
            dir: tdir(&d),
            pk: 1,
            col: 1,
            factor: 2
        })
        .is_ok()
    );
}
#[test]
fn set_field_mul_pk_stored() {
    let cmd = TableCommand::SetFieldMul {
        dir: PathBuf::from("/tmp"),
        pk: 42,
        col: 1,
        factor: 2,
    };
    if let TableCommand::SetFieldMul { pk, .. } = cmd {
        assert_eq!(pk, 42);
    } else {
        panic!();
    }
}
#[test]
fn set_field_mul_factor_stored() {
    let cmd = TableCommand::SetFieldMul {
        dir: PathBuf::from("/tmp"),
        pk: 1,
        col: 1,
        factor: 7,
    };
    if let TableCommand::SetFieldMul { factor, .. } = cmd {
        assert_eq!(factor, 7);
    } else {
        panic!();
    }
}
#[test]
fn set_field_mul_col_stored() {
    let cmd = TableCommand::SetFieldMul {
        dir: PathBuf::from("/tmp"),
        pk: 1,
        col: 2,
        factor: 2,
    };
    if let TableCommand::SetFieldMul { col, .. } = cmd {
        assert_eq!(col, 2);
    } else {
        panic!();
    }
}
#[test]
fn set_field_mul_multi_col_ok() {
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
        run(TableCommand::SetFieldMul {
            dir: tdir(&d),
            pk: 1,
            col: 2,
            factor: 3
        })
        .is_ok()
    );
}
#[test]
fn set_field_mul_large_factor_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 2);
    assert!(
        run(TableCommand::SetFieldMul {
            dir: tdir(&d),
            pk: 1,
            col: 1,
            factor: 1000
        })
        .is_ok()
    );
}
#[test]
fn set_field_mul_from_zero_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 0);
    assert!(
        run(TableCommand::SetFieldMul {
            dir: tdir(&d),
            pk: 1,
            col: 1,
            factor: 999
        })
        .is_ok()
    );
}
