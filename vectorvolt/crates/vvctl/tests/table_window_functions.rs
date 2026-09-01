//! Tests for Layer 284: vvctl table window functions combined (VV-CTL-284).

use std::path::PathBuf;
use tempfile::TempDir;
use vvctl::commands::table::{OutputFormat, TableCommand, run};

fn make_dir() -> TempDir {
    TempDir::new().unwrap()
}
fn tdir(d: &TempDir) -> PathBuf {
    d.path().join("tbl")
}
fn setup_5(d: &TempDir) {
    run(TableCommand::Create {
        dir: tdir(d),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    for i in 1..=5u64 {
        run(TableCommand::Insert {
            dir: tdir(d),
            pk: i,
            values: vec![i * 10],
        })
        .unwrap();
    }
}
fn setup_2col(d: &TempDir) {
    run(TableCommand::Create {
        dir: tdir(d),
        columns: vec!["pk:u64".into(), "a:u64".into(), "b:u64".into()],
    })
    .unwrap();
    for i in 1..=3u64 {
        run(TableCommand::Insert {
            dir: tdir(d),
            pk: i,
            values: vec![i, i * 2],
        })
        .unwrap();
    }
}

#[test]
fn window_running_sum_5rows_ok() {
    let d = make_dir();
    setup_5(&d);
    run(TableCommand::RunningSum {
        dir: tdir(&d),
        col: 1,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn window_sum_product_2col_ok() {
    let d = make_dir();
    setup_2col(&d);
    run(TableCommand::SumProduct {
        dir: tdir(&d),
        col_a: 1,
        col_b: 2,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn window_rank_highest_is_1_ok() {
    let d = make_dir();
    setup_5(&d);
    run(TableCommand::WindowRank {
        dir: tdir(&d),
        pk: 5,
        col: 1,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn window_rank_lowest_is_count_ok() {
    let d = make_dir();
    setup_5(&d);
    run(TableCommand::WindowRank {
        dir: tdir(&d),
        pk: 1,
        col: 1,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn window_rank_single_row_is_1() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: tdir(&d),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![42],
    })
    .unwrap();
    run(TableCommand::WindowRank {
        dir: tdir(&d),
        pk: 1,
        col: 1,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn window_rank_after_insert_ok() {
    let d = make_dir();
    setup_5(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 6,
        values: vec![100],
    })
    .unwrap();
    run(TableCommand::WindowRank {
        dir: tdir(&d),
        pk: 6,
        col: 1,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn window_sum_product_empty_ok() {
    let d = make_dir();
    setup_2col(&d);
    run(TableCommand::SumProduct {
        dir: tdir(&d),
        col_a: 1,
        col_b: 2,
        output: OutputFormat::Json,
    })
    .unwrap();
}

#[test]
fn window_running_sum_empty_ok() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: tdir(&d),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    run(TableCommand::RunningSum {
        dir: tdir(&d),
        col: 1,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn window_running_sum_single_ok() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: tdir(&d),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![42],
    })
    .unwrap();
    run(TableCommand::RunningSum {
        dir: tdir(&d),
        col: 1,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn window_rank_missing_dir_errors() {
    let d = make_dir();
    let res = run(TableCommand::WindowRank {
        dir: d.path().join("nope"),
        pk: 1,
        col: 1,
        output: OutputFormat::Table,
    });
    assert!(res.is_err());
}

#[test]
fn window_sum_product_cola_eq_colb_ok() {
    let d = make_dir();
    setup_2col(&d);
    run(TableCommand::SumProduct {
        dir: tdir(&d),
        col_a: 1,
        col_b: 1,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn window_running_sum_after_flush_ok() {
    let d = make_dir();
    setup_5(&d);
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    run(TableCommand::RunningSum {
        dir: tdir(&d),
        col: 1,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn window_sum_product_json_ok() {
    let d = make_dir();
    setup_2col(&d);
    run(TableCommand::SumProduct {
        dir: tdir(&d),
        col_a: 1,
        col_b: 2,
        output: OutputFormat::Json,
    })
    .unwrap();
}
