//! Tests for Layer 394.
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
fn vsearch_insert_5() {
    let d = make_dir();
    setup3col(&d);
    for i in 1..=5u64 {
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
fn vsearch_top_n_3_col1() {
    let d = make_dir();
    setup3col(&d);
    for i in 1..=5u64 {
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
            n: 3,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn vsearch_top_n_3_col2() {
    let d = make_dir();
    setup3col(&d);
    for i in 1..=5u64 {
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
            col: 2,
            n: 3,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn vsearch_bottom_n_3_col1() {
    let d = make_dir();
    setup3col(&d);
    for i in 1..=5u64 {
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
            col: 1,
            n: 3,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn vsearch_count_5() {
    let d = make_dir();
    setup3col(&d);
    for i in 1..=5u64 {
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
fn vsearch_batch_get_1_2_3() {
    let d = make_dir();
    setup3col(&d);
    for i in 1..=5u64 {
        run(TableCommand::Insert {
            dir: tdir(&d),
            pk: i,
            values: vec![i * 10, i * 20],
        })
        .unwrap();
    }
    assert!(
        run(TableCommand::BatchGet {
            dir: tdir(&d),
            pks: vec![1, 2, 3],
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn vsearch_filter_scan_ok() {
    let d = make_dir();
    setup3col(&d);
    for i in 1..=5u64 {
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
            hi: 5,
            filter_col: 1,
            filter_val: 30,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn vsearch_window_rank_pk1() {
    let d = make_dir();
    setup3col(&d);
    for i in 1..=5u64 {
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
fn vsearch_window_rank_pk5() {
    let d = make_dir();
    setup3col(&d);
    for i in 1..=5u64 {
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
            pk: 5,
            col: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn vsearch_sum_product_ok() {
    let d = make_dir();
    setup3col(&d);
    for i in 1..=5u64 {
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
fn vsearch_median_col1() {
    let d = make_dir();
    setup3col(&d);
    for i in 1..=5u64 {
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
fn vsearch_percentile_pct0() {
    let d = make_dir();
    setup3col(&d);
    for i in 1..=5u64 {
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
            pct: 0,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn vsearch_percentile_pct100() {
    let d = make_dir();
    setup3col(&d);
    for i in 1..=5u64 {
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
            pct: 100,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
