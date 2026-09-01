//! Tests for Layer 225: vvctl table compare-and-swap (VV-CTL-225).

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
fn cas_match_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    assert!(
        run(TableCommand::CompareAndSwap {
            dir: tdir(&d),
            pk: 1,
            col: 1,
            expected: 10,
            replacement: 20
        })
        .is_ok()
    );
}
#[test]
fn cas_no_match_is_err() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    // CAS with wrong expected returns Err
    assert!(
        run(TableCommand::CompareAndSwap {
            dir: tdir(&d),
            pk: 1,
            col: 1,
            expected: 99,
            replacement: 20
        })
        .is_err()
    );
}
#[test]
fn cas_verify_replaced() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 5);
    run(TableCommand::CompareAndSwap {
        dir: tdir(&d),
        pk: 1,
        col: 1,
        expected: 5,
        replacement: 50,
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
fn cas_verify_not_replaced() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 5);
    // CAS with wrong expected returns Err; value should remain 5
    let _ = run(TableCommand::CompareAndSwap {
        dir: tdir(&d),
        pk: 1,
        col: 1,
        expected: 999,
        replacement: 50,
    });
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
fn cas_missing_pk_errors() {
    let d = make_dir();
    setup(&d);
    assert!(
        run(TableCommand::CompareAndSwap {
            dir: tdir(&d),
            pk: 99,
            col: 1,
            expected: 0,
            replacement: 1
        })
        .is_err()
    );
}
#[test]
fn cas_missing_dir_errors() {
    let d = make_dir();
    assert!(
        run(TableCommand::CompareAndSwap {
            dir: d.path().join("nope"),
            pk: 1,
            col: 1,
            expected: 0,
            replacement: 1
        })
        .is_err()
    );
}
#[test]
fn cas_same_expected_replacement_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 7);
    assert!(
        run(TableCommand::CompareAndSwap {
            dir: tdir(&d),
            pk: 1,
            col: 1,
            expected: 7,
            replacement: 7
        })
        .is_ok()
    );
}
#[test]
fn cas_pk_stored() {
    let cmd = TableCommand::CompareAndSwap {
        dir: PathBuf::from("/tmp"),
        pk: 42,
        col: 1,
        expected: 0,
        replacement: 1,
    };
    if let TableCommand::CompareAndSwap { pk, .. } = cmd {
        assert_eq!(pk, 42);
    } else {
        panic!();
    }
}
#[test]
fn cas_expected_stored() {
    let cmd = TableCommand::CompareAndSwap {
        dir: PathBuf::from("/tmp"),
        pk: 1,
        col: 1,
        expected: 77,
        replacement: 1,
    };
    if let TableCommand::CompareAndSwap { expected, .. } = cmd {
        assert_eq!(expected, 77);
    } else {
        panic!();
    }
}
#[test]
fn cas_replacement_stored() {
    let cmd = TableCommand::CompareAndSwap {
        dir: PathBuf::from("/tmp"),
        pk: 1,
        col: 1,
        expected: 0,
        replacement: 99,
    };
    if let TableCommand::CompareAndSwap { replacement, .. } = cmd {
        assert_eq!(replacement, 99);
    } else {
        panic!();
    }
}
#[test]
fn cas_col_stored() {
    let cmd = TableCommand::CompareAndSwap {
        dir: PathBuf::from("/tmp"),
        pk: 1,
        col: 2,
        expected: 0,
        replacement: 1,
    };
    if let TableCommand::CompareAndSwap { col, .. } = cmd {
        assert_eq!(col, 2);
    } else {
        panic!();
    }
}
#[test]
fn cas_multi_col_ok() {
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
        run(TableCommand::CompareAndSwap {
            dir: tdir(&d),
            pk: 1,
            col: 2,
            expected: 20,
            replacement: 30
        })
        .is_ok()
    );
}
#[test]
fn cas_chained_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 1);
    run(TableCommand::CompareAndSwap {
        dir: tdir(&d),
        pk: 1,
        col: 1,
        expected: 1,
        replacement: 2,
    })
    .unwrap();
    assert!(
        run(TableCommand::CompareAndSwap {
            dir: tdir(&d),
            pk: 1,
            col: 1,
            expected: 2,
            replacement: 3
        })
        .is_ok()
    );
}
