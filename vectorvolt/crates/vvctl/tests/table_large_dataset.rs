//! Tests for Layer 262: vvctl table large dataset operations (VV-CTL-262).

use std::path::PathBuf;
use tempfile::TempDir;
use vvctl::commands::table::{OutputFormat, TableCommand, run};

fn make_dir() -> TempDir {
    TempDir::new().unwrap()
}
fn tdir(d: &TempDir) -> PathBuf {
    d.path().join("tbl")
}
fn setup_100(d: &TempDir) {
    run(TableCommand::Create {
        dir: tdir(d),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    for i in 0..100u64 {
        run(TableCommand::Insert {
            dir: tdir(d),
            pk: i,
            values: vec![i * 10],
        })
        .unwrap();
    }
}

#[test]
fn large_insert_100_rows_ok() {
    let d = make_dir();
    setup_100(&d);
    run(TableCommand::Count {
        dir: tdir(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn large_count_100_ok() {
    let d = make_dir();
    setup_100(&d);
    run(TableCommand::Count {
        dir: tdir(&d),
        output: OutputFormat::Json,
    })
    .unwrap();
}

#[test]
fn large_head_10_returns_ok() {
    let d = make_dir();
    setup_100(&d);
    run(TableCommand::Head {
        dir: tdir(&d),
        n: 10,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn large_tail_10_returns_ok() {
    let d = make_dir();
    setup_100(&d);
    run(TableCommand::Tail {
        dir: tdir(&d),
        n: 10,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn large_scan_all_ok() {
    let d = make_dir();
    setup_100(&d);
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
fn large_top_n_5_ok() {
    let d = make_dir();
    setup_100(&d);
    run(TableCommand::TopN {
        dir: tdir(&d),
        col: 1,
        n: 5,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn large_bottom_n_5_ok() {
    let d = make_dir();
    setup_100(&d);
    run(TableCommand::BottomN {
        dir: tdir(&d),
        col: 1,
        n: 5,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn large_sum_ok() {
    let d = make_dir();
    setup_100(&d);
    run(TableCommand::SumField {
        dir: tdir(&d),
        col: 1,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn large_avg_ok() {
    let d = make_dir();
    setup_100(&d);
    run(TableCommand::AvgField {
        dir: tdir(&d),
        col: 1,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn large_min_ok() {
    let d = make_dir();
    setup_100(&d);
    run(TableCommand::ScalarMin {
        dir: tdir(&d),
        col: 1,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn large_max_ok() {
    let d = make_dir();
    setup_100(&d);
    run(TableCommand::ScalarMax {
        dir: tdir(&d),
        col: 1,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn large_count_distinct_ok() {
    let d = make_dir();
    setup_100(&d);
    run(TableCommand::CountDistinct {
        dir: tdir(&d),
        col: 1,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn large_batch_get_5_pks_ok() {
    let d = make_dir();
    setup_100(&d);
    run(TableCommand::BatchGet {
        dir: tdir(&d),
        pks: vec![0, 10, 20, 30, 40],
        output: OutputFormat::Table,
    })
    .unwrap();
}
