//! Tests for Layer 212: vvctl table sum-product (VV-CTL-212).

use std::path::PathBuf;
use tempfile::TempDir;
use vvctl::commands::table::{OutputFormat, TableCommand, run};

fn make_dir() -> TempDir {
    TempDir::new().unwrap()
}
fn tdir(d: &TempDir) -> PathBuf {
    d.path().join("tbl")
}

fn setup2(d: &TempDir) {
    run(TableCommand::Create {
        dir: tdir(d),
        columns: vec!["pk:u64".into(), "a:u64".into(), "b:u64".into()],
    })
    .unwrap();
}
fn ins2(d: &TempDir, pk: u64, a: u64, b: u64) {
    run(TableCommand::Insert {
        dir: tdir(d),
        pk,
        values: vec![a, b],
    })
    .unwrap();
}

#[test]
fn sum_product_empty_ok() {
    let d = make_dir();
    setup2(&d);
    assert!(
        run(TableCommand::SumProduct {
            dir: tdir(&d),
            col_a: 1,
            col_b: 2,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn sum_product_single_row_ok() {
    let d = make_dir();
    setup2(&d);
    ins2(&d, 1, 3, 4);
    assert!(
        run(TableCommand::SumProduct {
            dir: tdir(&d),
            col_a: 1,
            col_b: 2,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn sum_product_three_rows_ok() {
    let d = make_dir();
    setup2(&d);
    for i in 1..=3u64 {
        ins2(&d, i, i, i * 2);
    }
    assert!(
        run(TableCommand::SumProduct {
            dir: tdir(&d),
            col_a: 1,
            col_b: 2,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn sum_product_json_ok() {
    let d = make_dir();
    setup2(&d);
    ins2(&d, 1, 5, 10);
    assert!(
        run(TableCommand::SumProduct {
            dir: tdir(&d),
            col_a: 1,
            col_b: 2,
            output: OutputFormat::Json
        })
        .is_ok()
    );
}
#[test]
fn sum_product_missing_dir_errors() {
    let d = make_dir();
    assert!(
        run(TableCommand::SumProduct {
            dir: d.path().join("nope"),
            col_a: 1,
            col_b: 2,
            output: OutputFormat::Table
        })
        .is_err()
    );
}
#[test]
fn sum_product_zero_values_ok() {
    let d = make_dir();
    setup2(&d);
    for i in 0..5u64 {
        ins2(&d, i, 0, i);
    }
    assert!(
        run(TableCommand::SumProduct {
            dir: tdir(&d),
            col_a: 1,
            col_b: 2,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn sum_product_ten_rows_ok() {
    let d = make_dir();
    setup2(&d);
    for i in 0..10u64 {
        ins2(&d, i, i, i * 3);
    }
    assert!(
        run(TableCommand::SumProduct {
            dir: tdir(&d),
            col_a: 1,
            col_b: 2,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn sum_product_json_ten_rows() {
    let d = make_dir();
    setup2(&d);
    for i in 0..10u64 {
        ins2(&d, i, i * 2, i * 3);
    }
    assert!(
        run(TableCommand::SumProduct {
            dir: tdir(&d),
            col_a: 1,
            col_b: 2,
            output: OutputFormat::Json
        })
        .is_ok()
    );
}
#[test]
fn sum_product_same_col_ok() {
    let d = make_dir();
    setup2(&d);
    ins2(&d, 1, 4, 4);
    assert!(
        run(TableCommand::SumProduct {
            dir: tdir(&d),
            col_a: 1,
            col_b: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn sum_product_col_a_stored() {
    let cmd = TableCommand::SumProduct {
        dir: PathBuf::from("/tmp"),
        col_a: 1,
        col_b: 2,
        output: OutputFormat::Table,
    };
    if let TableCommand::SumProduct { col_a, .. } = cmd {
        assert_eq!(col_a, 1);
    } else {
        panic!();
    }
}
#[test]
fn sum_product_col_b_stored() {
    let cmd = TableCommand::SumProduct {
        dir: PathBuf::from("/tmp"),
        col_a: 1,
        col_b: 3,
        output: OutputFormat::Table,
    };
    if let TableCommand::SumProduct { col_b, .. } = cmd {
        assert_eq!(col_b, 3);
    } else {
        panic!();
    }
}
#[test]
fn sum_product_large_values_ok() {
    let d = make_dir();
    setup2(&d);
    ins2(&d, 1, 1000, 2000);
    assert!(
        run(TableCommand::SumProduct {
            dir: tdir(&d),
            col_a: 1,
            col_b: 2,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn sum_product_five_rows_json() {
    let d = make_dir();
    setup2(&d);
    for i in 1..=5u64 {
        ins2(&d, i, i * 4, i * 5);
    }
    assert!(
        run(TableCommand::SumProduct {
            dir: tdir(&d),
            col_a: 1,
            col_b: 2,
            output: OutputFormat::Json
        })
        .is_ok()
    );
}
