//! Tests for Layer 391.
use std::path::PathBuf;
use tempfile::TempDir;
use vvctl::commands::table::{OutputFormat, TableCommand, run};

fn make_dir() -> TempDir {
    TempDir::new().unwrap()
}
fn tdir(d: &TempDir) -> PathBuf {
    d.path().join("tbl")
}
fn setup4col(d: &TempDir) {
    run(TableCommand::Create {
        dir: tdir(d),
        columns: vec![
            "pk:u64".into(),
            "x:u64".into(),
            "y:u64".into(),
            "z:u64".into(),
        ],
    })
    .unwrap();
}

#[test]
fn hnsw_insert_5_vectors() {
    let d = make_dir();
    setup4col(&d);
    for i in 1..=5u64 {
        run(TableCommand::Insert {
            dir: tdir(&d),
            pk: i,
            values: vec![i, i * 2, i * 3],
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
fn hnsw_scan_col1() {
    let d = make_dir();
    setup4col(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![10, 20, 30],
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
fn hnsw_scan_col2() {
    let d = make_dir();
    setup4col(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![10, 20, 30],
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
fn hnsw_scan_col3() {
    let d = make_dir();
    setup4col(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![10, 20, 30],
    })
    .unwrap();
    assert!(
        run(TableCommand::Scan {
            dir: tdir(&d),
            col: 3,
            lo: 0,
            hi: u64::MAX,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn hnsw_get_pk3() {
    let d = make_dir();
    setup4col(&d);
    for i in 1..=5u64 {
        run(TableCommand::Insert {
            dir: tdir(&d),
            pk: i,
            values: vec![i, i * 2, i * 3],
        })
        .unwrap();
    }
    assert!(
        run(TableCommand::Get {
            dir: tdir(&d),
            pk: 3,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn hnsw_top_n_col1() {
    let d = make_dir();
    setup4col(&d);
    for i in 1..=5u64 {
        run(TableCommand::Insert {
            dir: tdir(&d),
            pk: i,
            values: vec![i, i * 2, i * 3],
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
fn hnsw_bottom_n_col2() {
    let d = make_dir();
    setup4col(&d);
    for i in 1..=5u64 {
        run(TableCommand::Insert {
            dir: tdir(&d),
            pk: i,
            values: vec![i, i * 2, i * 3],
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
fn hnsw_sum_product_ok() {
    let d = make_dir();
    setup4col(&d);
    for i in 1..=3u64 {
        run(TableCommand::Insert {
            dir: tdir(&d),
            pk: i,
            values: vec![i, i * 2, i * 3],
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
fn hnsw_filter_scan_ok() {
    let d = make_dir();
    setup4col(&d);
    for i in 1..=5u64 {
        run(TableCommand::Insert {
            dir: tdir(&d),
            pk: i,
            values: vec![i, i * 2, i * 3],
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
            filter_val: 3,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn hnsw_count_distinct_ok() {
    let d = make_dir();
    setup4col(&d);
    for i in 1..=5u64 {
        run(TableCommand::Insert {
            dir: tdir(&d),
            pk: i,
            values: vec![i, i * 2, i * 3],
        })
        .unwrap();
    }
    assert!(
        run(TableCommand::CountDistinct {
            dir: tdir(&d),
            col: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn hnsw_window_rank_ok() {
    let d = make_dir();
    setup4col(&d);
    for i in 1..=5u64 {
        run(TableCommand::Insert {
            dir: tdir(&d),
            pk: i,
            values: vec![i, i * 2, i * 3],
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
fn hnsw_recount_ok() {
    let d = make_dir();
    setup4col(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![1, 2, 3],
    })
    .unwrap();
    assert!(
        run(TableCommand::Recount {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn hnsw_check_ok() {
    let d = make_dir();
    setup4col(&d);
    assert!(run(TableCommand::Check { dir: tdir(&d) }).is_ok());
}
