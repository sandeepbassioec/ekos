//! Tests for Layer 264: vvctl table column index boundary (VV-CTL-264).

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
    run(TableCommand::Insert {
        dir: tdir(d),
        pk: 1,
        values: vec![100],
    })
    .unwrap();
}

#[test]
fn col_boundary_scan_col0_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Scan {
        dir: tdir(&d),
        col: 0,
        lo: 0,
        hi: u64::MAX,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn col_boundary_scan_col1_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Scan {
        dir: tdir(&d),
        col: 1,
        lo: 0,
        hi: u64::MAX,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn col_boundary_top_n_col0_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::TopN {
        dir: tdir(&d),
        col: 0,
        n: 1,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn col_boundary_sum_col1_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::SumField {
        dir: tdir(&d),
        col: 1,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn col_boundary_avg_col1_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::AvgField {
        dir: tdir(&d),
        col: 1,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn col_boundary_median_col1_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::MedianField {
        dir: tdir(&d),
        col: 1,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn col_boundary_min_col1_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::ScalarMin {
        dir: tdir(&d),
        col: 1,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn col_boundary_max_col1_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::ScalarMax {
        dir: tdir(&d),
        col: 1,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn col_boundary_distinct_col1_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::CountDistinct {
        dir: tdir(&d),
        col: 1,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn col_boundary_unique_col1_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::UniqueCount {
        dir: tdir(&d),
        col: 1,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn col_boundary_window_rank_col0_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::WindowRank {
        dir: tdir(&d),
        pk: 1,
        col: 0,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn col_boundary_histogram_col1_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Histogram {
        dir: tdir(&d),
        col: 1,
        bucket_size: 10,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn col_boundary_percentile_col1_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::PercentileField {
        dir: tdir(&d),
        col: 1,
        pct: 50,
        output: OutputFormat::Table,
    })
    .unwrap();
}
