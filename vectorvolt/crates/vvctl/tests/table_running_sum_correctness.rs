use std::path::PathBuf;
use tempfile::TempDir;
use vvctl::commands::table::{OutputFormat, TableCommand, run};

fn make_dir() -> TempDir {
    TempDir::new().unwrap()
}
fn tdir(d: &TempDir) -> PathBuf {
    d.path().join("tbl")
}
fn tdir_3col(d: &TempDir) -> PathBuf {
    d.path().join("tbl3")
}
fn setup(d: &TempDir) {
    run(TableCommand::Create {
        dir: tdir(d),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
}
fn setup_3col(d: &TempDir) {
    run(TableCommand::Create {
        dir: tdir_3col(d),
        columns: vec!["pk:u64".into(), "val:u64".into(), "val2:u64".into()],
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
fn ins3(d: &TempDir, pk: u64, v1: u64, v2: u64) {
    run(TableCommand::Insert {
        dir: tdir_3col(d),
        pk,
        values: vec![v1, v2],
    })
    .unwrap();
}

#[test]
fn running_sum_single_row() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    assert!(
        run(TableCommand::RunningSum {
            dir: tdir(&d),
            col: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn running_sum_multiple_rows() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 {
        ins(&d, i, i * 10);
    }
    assert!(
        run(TableCommand::RunningSum {
            dir: tdir(&d),
            col: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn running_sum_empty_table() {
    let d = make_dir();
    setup(&d);
    assert!(
        run(TableCommand::RunningSum {
            dir: tdir(&d),
            col: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn running_sum_json_output() {
    let d = make_dir();
    setup(&d);
    for i in 1..=3 {
        ins(&d, i, i * 5);
    }
    assert!(
        run(TableCommand::RunningSum {
            dir: tdir(&d),
            col: 1,
            output: OutputFormat::Json
        })
        .is_ok()
    );
}

#[test]
fn running_sum_after_flush() {
    let d = make_dir();
    setup(&d);
    for i in 1..=4 {
        ins(&d, i, i * 3);
    }
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    assert!(
        run(TableCommand::RunningSum {
            dir: tdir(&d),
            col: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn running_sum_after_compact() {
    let d = make_dir();
    setup(&d);
    for i in 1..=6 {
        ins(&d, i, i * 2);
    }
    run(TableCommand::Compact { dir: tdir(&d) }).unwrap();
    assert!(
        run(TableCommand::RunningSum {
            dir: tdir(&d),
            col: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn sum_product_basic() {
    let d = make_dir();
    setup_3col(&d);
    ins3(&d, 1, 2, 3);
    ins3(&d, 2, 4, 5);
    assert!(
        run(TableCommand::SumProduct {
            dir: tdir_3col(&d),
            col_a: 1,
            col_b: 2,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn sum_product_single_row() {
    let d = make_dir();
    setup_3col(&d);
    ins3(&d, 1, 7, 8);
    assert!(
        run(TableCommand::SumProduct {
            dir: tdir_3col(&d),
            col_a: 1,
            col_b: 2,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn sum_product_empty_table() {
    let d = make_dir();
    setup_3col(&d);
    assert!(
        run(TableCommand::SumProduct {
            dir: tdir_3col(&d),
            col_a: 1,
            col_b: 2,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn sum_product_json_output() {
    let d = make_dir();
    setup_3col(&d);
    ins3(&d, 1, 3, 4);
    ins3(&d, 2, 5, 6);
    assert!(
        run(TableCommand::SumProduct {
            dir: tdir_3col(&d),
            col_a: 1,
            col_b: 2,
            output: OutputFormat::Json
        })
        .is_ok()
    );
}

#[test]
fn sum_product_zeros() {
    let d = make_dir();
    setup_3col(&d);
    ins3(&d, 1, 0, 100);
    ins3(&d, 2, 10, 0);
    assert!(
        run(TableCommand::SumProduct {
            dir: tdir_3col(&d),
            col_a: 1,
            col_b: 2,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn sum_product_large_values() {
    let d = make_dir();
    setup_3col(&d);
    for i in 1..=5 {
        ins3(&d, i, i * 100, i * 200);
    }
    assert!(
        run(TableCommand::SumProduct {
            dir: tdir_3col(&d),
            col_a: 1,
            col_b: 2,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn running_sum_large_dataset() {
    let d = make_dir();
    setup(&d);
    for i in 1..=20 {
        ins(&d, i, i * 7);
    }
    assert!(
        run(TableCommand::RunningSum {
            dir: tdir(&d),
            col: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
