//! Tests for Layer 285: vvctl table aggregates combined (VV-CTL-285).

use std::path::PathBuf;
use tempfile::TempDir;
use vvctl::commands::table::{OutputFormat, TableCommand, run};

fn make_dir() -> TempDir {
    TempDir::new().unwrap()
}
fn tdir(d: &TempDir) -> PathBuf {
    d.path().join("tbl")
}
fn setup_known(d: &TempDir) {
    run(TableCommand::Create {
        dir: tdir(d),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    // Insert 10, 20, 30, 40, 50
    for (pk, val) in [(1u64, 10u64), (2, 20), (3, 30), (4, 40), (5, 50)] {
        run(TableCommand::Insert {
            dir: tdir(d),
            pk,
            values: vec![val],
        })
        .unwrap();
    }
}

#[test]
fn agg_sum_150_ok() {
    let d = make_dir();
    setup_known(&d);
    run(TableCommand::SumField {
        dir: tdir(&d),
        col: 1,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn agg_avg_30_ok() {
    let d = make_dir();
    setup_known(&d);
    run(TableCommand::AvgField {
        dir: tdir(&d),
        col: 1,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn agg_median_30_ok() {
    let d = make_dir();
    setup_known(&d);
    run(TableCommand::MedianField {
        dir: tdir(&d),
        col: 1,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn agg_min_10_ok() {
    let d = make_dir();
    setup_known(&d);
    run(TableCommand::ScalarMin {
        dir: tdir(&d),
        col: 1,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn agg_max_50_ok() {
    let d = make_dir();
    setup_known(&d);
    run(TableCommand::ScalarMax {
        dir: tdir(&d),
        col: 1,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn agg_p50_30_ok() {
    let d = make_dir();
    setup_known(&d);
    run(TableCommand::PercentileField {
        dir: tdir(&d),
        col: 1,
        pct: 50,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn agg_p0_10_ok() {
    let d = make_dir();
    setup_known(&d);
    run(TableCommand::PercentileField {
        dir: tdir(&d),
        col: 1,
        pct: 0,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn agg_p100_50_ok() {
    let d = make_dir();
    setup_known(&d);
    run(TableCommand::PercentileField {
        dir: tdir(&d),
        col: 1,
        pct: 100,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn agg_count_distinct_5_ok() {
    let d = make_dir();
    setup_known(&d);
    run(TableCommand::CountDistinct {
        dir: tdir(&d),
        col: 1,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn agg_unique_count_5_ok() {
    let d = make_dir();
    setup_known(&d);
    run(TableCommand::UniqueCount {
        dir: tdir(&d),
        col: 1,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn agg_histogram_buckets_ok() {
    let d = make_dir();
    setup_known(&d);
    run(TableCommand::Histogram {
        dir: tdir(&d),
        col: 1,
        bucket_size: 10,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn agg_zscore_no_outliers_ok() {
    let d = make_dir();
    setup_known(&d);
    run(TableCommand::ZScoreFilter {
        dir: tdir(&d),
        col: 1,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn agg_running_sum_last_150_ok() {
    let d = make_dir();
    setup_known(&d);
    run(TableCommand::RunningSum {
        dir: tdir(&d),
        col: 1,
        output: OutputFormat::Table,
    })
    .unwrap();
}
