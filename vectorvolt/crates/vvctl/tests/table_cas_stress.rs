//! Tests for Layer 283: vvctl table CAS under load (VV-CTL-283).

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
fn cas_correct_expected_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![42],
    })
    .unwrap();
    run(TableCommand::CompareAndSwap {
        dir: tdir(&d),
        pk: 1,
        col: 1,
        expected: 42,
        replacement: 99,
    })
    .unwrap();
}

#[test]
fn cas_wrong_expected_errors() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 2,
        values: vec![10],
    })
    .unwrap();
    let res = run(TableCommand::CompareAndSwap {
        dir: tdir(&d),
        pk: 2,
        col: 1,
        expected: 999,
        replacement: 0,
    });
    assert!(res.is_err());
}

#[test]
fn cas_then_get_replacement() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 3,
        values: vec![30],
    })
    .unwrap();
    run(TableCommand::CompareAndSwap {
        dir: tdir(&d),
        pk: 3,
        col: 1,
        expected: 30,
        replacement: 300,
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
fn cas_then_cas_again_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 4,
        values: vec![10],
    })
    .unwrap();
    run(TableCommand::CompareAndSwap {
        dir: tdir(&d),
        pk: 4,
        col: 1,
        expected: 10,
        replacement: 20,
    })
    .unwrap();
    run(TableCommand::CompareAndSwap {
        dir: tdir(&d),
        pk: 4,
        col: 1,
        expected: 20,
        replacement: 30,
    })
    .unwrap();
}

#[test]
fn cas_chain_5_steps_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 5,
        values: vec![0],
    })
    .unwrap();
    for i in 0..5u64 {
        run(TableCommand::CompareAndSwap {
            dir: tdir(&d),
            pk: 5,
            col: 1,
            expected: i,
            replacement: i + 1,
        })
        .unwrap();
    }
}

#[test]
fn cas_col0_errors() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 6,
        values: vec![10],
    })
    .unwrap();
    let res = run(TableCommand::CompareAndSwap {
        dir: tdir(&d),
        pk: 6,
        col: 0,
        expected: 6,
        replacement: 0,
    });
    assert!(res.is_err());
}

#[test]
fn cas_missing_pk_errors() {
    let d = make_dir();
    setup(&d);
    let res = run(TableCommand::CompareAndSwap {
        dir: tdir(&d),
        pk: 999,
        col: 1,
        expected: 0,
        replacement: 99,
    });
    assert!(res.is_err());
}

#[test]
fn cas_missing_dir_errors() {
    let d = make_dir();
    let res = run(TableCommand::CompareAndSwap {
        dir: d.path().join("nope"),
        pk: 1,
        col: 1,
        expected: 0,
        replacement: 1,
    });
    assert!(res.is_err());
}

#[test]
fn cas_expected_zero_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 7,
        values: vec![0],
    })
    .unwrap();
    run(TableCommand::CompareAndSwap {
        dir: tdir(&d),
        pk: 7,
        col: 1,
        expected: 0,
        replacement: 1,
    })
    .unwrap();
}

#[test]
fn cas_replacement_zero_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 8,
        values: vec![5],
    })
    .unwrap();
    run(TableCommand::CompareAndSwap {
        dir: tdir(&d),
        pk: 8,
        col: 1,
        expected: 5,
        replacement: 0,
    })
    .unwrap();
}

#[test]
fn cas_large_values_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 9,
        values: vec![1_000_000],
    })
    .unwrap();
    run(TableCommand::CompareAndSwap {
        dir: tdir(&d),
        pk: 9,
        col: 1,
        expected: 1_000_000,
        replacement: 2_000_000,
    })
    .unwrap();
}

#[test]
fn cas_then_increment_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 10,
        values: vec![5],
    })
    .unwrap();
    run(TableCommand::CompareAndSwap {
        dir: tdir(&d),
        pk: 10,
        col: 1,
        expected: 5,
        replacement: 10,
    })
    .unwrap();
    run(TableCommand::IncrementField {
        dir: tdir(&d),
        pk: 10,
        col: 1,
    })
    .unwrap();
}

#[test]
fn cas_then_decrement_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 11,
        values: vec![100],
    })
    .unwrap();
    run(TableCommand::CompareAndSwap {
        dir: tdir(&d),
        pk: 11,
        col: 1,
        expected: 100,
        replacement: 50,
    })
    .unwrap();
    run(TableCommand::DecrementField {
        dir: tdir(&d),
        pk: 11,
        col: 1,
    })
    .unwrap();
}
