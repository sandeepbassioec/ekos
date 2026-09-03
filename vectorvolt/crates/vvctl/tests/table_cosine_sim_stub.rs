//! Tests for Layer 393.
use std::path::PathBuf;
use tempfile::TempDir;
use vvctl::commands::table::{OutputFormat, TableCommand, run};

fn make_dir() -> TempDir {
    TempDir::new().unwrap()
}
fn tdir(d: &TempDir) -> PathBuf {
    d.path().join("tbl")
}
fn setup3col(d: &TempDir) {
    run(TableCommand::Create {
        dir: tdir(d),
        columns: vec!["pk:u64".into(), "x:u64".into(), "y:u64".into()],
    })
    .unwrap();
}

#[test]
fn cosim_insert_4() {
    let d = make_dir();
    setup3col(&d);
    for i in 1..=4u64 {
        run(TableCommand::Insert {
            dir: tdir(&d),
            pk: i,
            values: vec![i * 10, i * 20],
        })
        .unwrap();
    }
    assert!(
        run(TableCommand::Count {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn cosim_sum_product_dot() {
    let d = make_dir();
    setup3col(&d);
    for i in 1..=4u64 {
        run(TableCommand::Insert {
            dir: tdir(&d),
            pk: i,
            values: vec![i * 10, i * 20],
        })
        .unwrap();
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
fn cosim_scan_col1() {
    let d = make_dir();
    setup3col(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![10, 20],
    })
    .unwrap();
    assert!(
        run(TableCommand::Scan {
            dir: tdir(&d),
            col: 1,
            lo: 0,
            hi: u64::MAX,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn cosim_scan_col2() {
    let d = make_dir();
    setup3col(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![10, 20],
    })
    .unwrap();
    assert!(
        run(TableCommand::Scan {
            dir: tdir(&d),
            col: 2,
            lo: 0,
            hi: u64::MAX,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn cosim_top_n_col1() {
    let d = make_dir();
    setup3col(&d);
    for i in 1..=4u64 {
        run(TableCommand::Insert {
            dir: tdir(&d),
            pk: i,
            values: vec![i * 10, i * 20],
        })
        .unwrap();
    }
    assert!(
        run(TableCommand::TopN {
            dir: tdir(&d),
            col: 1,
            n: 2,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn cosim_bottom_n_col2() {
    let d = make_dir();
    setup3col(&d);
    for i in 1..=4u64 {
        run(TableCommand::Insert {
            dir: tdir(&d),
            pk: i,
            values: vec![i * 10, i * 20],
        })
        .unwrap();
    }
    assert!(
        run(TableCommand::BottomN {
            dir: tdir(&d),
            col: 2,
            n: 2,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn cosim_filter_scan_ok() {
    let d = make_dir();
    setup3col(&d);
    for i in 1..=4u64 {
        run(TableCommand::Insert {
            dir: tdir(&d),
            pk: i,
            values: vec![i * 10, i * 20],
        })
        .unwrap();
    }
    assert!(
        run(TableCommand::FilterScan {
            dir: tdir(&d),
            scan_col: 0,
            lo: 1,
            hi: 4,
            filter_col: 1,
            filter_val: 20,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn cosim_window_rank_pk1() {
    let d = make_dir();
    setup3col(&d);
    for i in 1..=4u64 {
        run(TableCommand::Insert {
            dir: tdir(&d),
            pk: i,
            values: vec![i * 10, i * 20],
        })
        .unwrap();
    }
    assert!(
        run(TableCommand::WindowRank {
            dir: tdir(&d),
            pk: 1,
            col: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn cosim_window_rank_pk4() {
    let d = make_dir();
    setup3col(&d);
    for i in 1..=4u64 {
        run(TableCommand::Insert {
            dir: tdir(&d),
            pk: i,
            values: vec![i * 10, i * 20],
        })
        .unwrap();
    }
    assert!(
        run(TableCommand::WindowRank {
            dir: tdir(&d),
            pk: 4,
            col: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn cosim_median_col1() {
    let d = make_dir();
    setup3col(&d);
    for i in 1..=4u64 {
        run(TableCommand::Insert {
            dir: tdir(&d),
            pk: i,
            values: vec![i * 10, i * 20],
        })
        .unwrap();
    }
    assert!(
        run(TableCommand::MedianField {
            dir: tdir(&d),
            col: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn cosim_histogram_ok() {
    let d = make_dir();
    setup3col(&d);
    for i in 1..=4u64 {
        run(TableCommand::Insert {
            dir: tdir(&d),
            pk: i,
            values: vec![i * 10, i * 20],
        })
        .unwrap();
    }
    assert!(
        run(TableCommand::Histogram {
            dir: tdir(&d),
            col: 1,
            bucket_size: 10,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn cosim_zscore_ok() {
    let d = make_dir();
    setup3col(&d);
    for i in 1..=4u64 {
        run(TableCommand::Insert {
            dir: tdir(&d),
            pk: i,
            values: vec![i * 10, i * 20],
        })
        .unwrap();
    }
    assert!(
        run(TableCommand::ZScoreFilter {
            dir: tdir(&d),
            col: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn cosim_percentile_75() {
    let d = make_dir();
    setup3col(&d);
    for i in 1..=4u64 {
        run(TableCommand::Insert {
            dir: tdir(&d),
            pk: i,
            values: vec![i * 10, i * 20],
        })
        .unwrap();
    }
    assert!(
        run(TableCommand::PercentileField {
            dir: tdir(&d),
            col: 1,
            pct: 75,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
