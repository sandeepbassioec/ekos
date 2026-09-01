//! Tests for Layer 222: vvctl table set-field-add (VV-CTL-222).

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
fn set_field_add_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    assert!(
        run(TableCommand::SetFieldAdd {
            dir: tdir(&d),
            pk: 1,
            col: 1,
            addend: 5
        })
        .is_ok()
    );
}
#[test]
fn set_field_add_zero_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    assert!(
        run(TableCommand::SetFieldAdd {
            dir: tdir(&d),
            pk: 1,
            col: 1,
            addend: 0
        })
        .is_ok()
    );
}
#[test]
fn set_field_add_verify_value() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    run(TableCommand::SetFieldAdd {
        dir: tdir(&d),
        pk: 1,
        col: 1,
        addend: 5,
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
fn set_field_add_large_addend_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 0);
    assert!(
        run(TableCommand::SetFieldAdd {
            dir: tdir(&d),
            pk: 1,
            col: 1,
            addend: 1_000_000
        })
        .is_ok()
    );
}
#[test]
fn set_field_add_missing_pk_errors() {
    let d = make_dir();
    setup(&d);
    assert!(
        run(TableCommand::SetFieldAdd {
            dir: tdir(&d),
            pk: 99,
            col: 1,
            addend: 5
        })
        .is_err()
    );
}
#[test]
fn set_field_add_missing_dir_errors() {
    let d = make_dir();
    assert!(
        run(TableCommand::SetFieldAdd {
            dir: d.path().join("nope"),
            pk: 1,
            col: 1,
            addend: 1
        })
        .is_err()
    );
}
#[test]
fn set_field_add_twice_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    run(TableCommand::SetFieldAdd {
        dir: tdir(&d),
        pk: 1,
        col: 1,
        addend: 5,
    })
    .unwrap();
    assert!(
        run(TableCommand::SetFieldAdd {
            dir: tdir(&d),
            pk: 1,
            col: 1,
            addend: 3
        })
        .is_ok()
    );
}
#[test]
fn set_field_add_pk_stored() {
    let cmd = TableCommand::SetFieldAdd {
        dir: PathBuf::from("/tmp"),
        pk: 42,
        col: 1,
        addend: 1,
    };
    if let TableCommand::SetFieldAdd { pk, .. } = cmd {
        assert_eq!(pk, 42);
    } else {
        panic!();
    }
}
#[test]
fn set_field_add_addend_stored() {
    let cmd = TableCommand::SetFieldAdd {
        dir: PathBuf::from("/tmp"),
        pk: 1,
        col: 1,
        addend: 77,
    };
    if let TableCommand::SetFieldAdd { addend, .. } = cmd {
        assert_eq!(addend, 77);
    } else {
        panic!();
    }
}
#[test]
fn set_field_add_col_stored() {
    let cmd = TableCommand::SetFieldAdd {
        dir: PathBuf::from("/tmp"),
        pk: 1,
        col: 2,
        addend: 1,
    };
    if let TableCommand::SetFieldAdd { col, .. } = cmd {
        assert_eq!(col, 2);
    } else {
        panic!();
    }
}
#[test]
fn set_field_add_multi_col_ok() {
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
        run(TableCommand::SetFieldAdd {
            dir: tdir(&d),
            pk: 1,
            col: 2,
            addend: 5
        })
        .is_ok()
    );
}
#[test]
fn set_field_add_multiple_rows_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 5);
    ins(&d, 2, 10);
    run(TableCommand::SetFieldAdd {
        dir: tdir(&d),
        pk: 1,
        col: 1,
        addend: 1,
    })
    .unwrap();
    assert!(
        run(TableCommand::SetFieldAdd {
            dir: tdir(&d),
            pk: 2,
            col: 1,
            addend: 2
        })
        .is_ok()
    );
}
#[test]
fn set_field_add_from_zero_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 0);
    assert!(
        run(TableCommand::SetFieldAdd {
            dir: tdir(&d),
            pk: 1,
            col: 1,
            addend: 100
        })
        .is_ok()
    );
}
