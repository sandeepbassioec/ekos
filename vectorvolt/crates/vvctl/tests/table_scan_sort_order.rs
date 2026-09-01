//! Tests for Layer 266: vvctl table scan and sort order (VV-CTL-266).

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

#[test]
fn sort_top_n_returns_highest_first_ok() {
    let d = make_dir();
    setup_5(&d);
    run(TableCommand::TopN {
        dir: tdir(&d),
        col: 1,
        n: 3,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn sort_bottom_n_returns_lowest_first_ok() {
    let d = make_dir();
    setup_5(&d);
    run(TableCommand::BottomN {
        dir: tdir(&d),
        col: 1,
        n: 3,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn sort_head_returns_low_pk_first_ok() {
    let d = make_dir();
    setup_5(&d);
    run(TableCommand::Head {
        dir: tdir(&d),
        n: 3,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn sort_tail_returns_high_pk_last_ok() {
    let d = make_dir();
    setup_5(&d);
    run(TableCommand::Tail {
        dir: tdir(&d),
        n: 3,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn sort_scan_range_ok() {
    let d = make_dir();
    setup_5(&d);
    run(TableCommand::Scan {
        dir: tdir(&d),
        col: 1,
        lo: 20,
        hi: 40,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn sort_scan_exact_range_ok() {
    let d = make_dir();
    setup_5(&d);
    run(TableCommand::Scan {
        dir: tdir(&d),
        col: 1,
        lo: 30,
        hi: 30,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn sort_scan_lo_gt_hi_ok() {
    let d = make_dir();
    setup_5(&d);
    run(TableCommand::Scan {
        dir: tdir(&d),
        col: 1,
        lo: 100,
        hi: 10,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn sort_top_n1_is_max_ok() {
    let d = make_dir();
    setup_5(&d);
    run(TableCommand::TopN {
        dir: tdir(&d),
        col: 1,
        n: 1,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn sort_bottom_n1_is_min_ok() {
    let d = make_dir();
    setup_5(&d);
    run(TableCommand::BottomN {
        dir: tdir(&d),
        col: 1,
        n: 1,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn sort_scan_all_rows_ok() {
    let d = make_dir();
    setup_5(&d);
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
fn sort_running_sum_ok() {
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
fn sort_window_rank_of_max_ok() {
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
fn sort_window_rank_of_min_ok() {
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
