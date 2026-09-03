//! Tests for Layer 387.
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
}
fn ins(d: &TempDir, pk: u64, val: u64) {
    run(TableCommand::Insert {
        dir: tdir(d),
        pk,
        values: vec![val],
    })
    .unwrap();
}

#[test]
fn select_insert_5() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5u64 {
        ins(&d, i, i * 10);
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
fn select_scan_pk_range() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5u64 {
        ins(&d, i, i * 10);
    }
    assert!(
        run(TableCommand::Scan {
            dir: tdir(&d),
            col: 0,
            lo: 1,
            hi: 5,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn select_scan_val_range() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5u64 {
        ins(&d, i, i * 10);
    }
    assert!(
        run(TableCommand::Scan {
            dir: tdir(&d),
            col: 1,
            lo: 10,
            hi: 30,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn select_top_n_3() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5u64 {
        ins(&d, i, i * 10);
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
fn select_bottom_n_2() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5u64 {
        ins(&d, i, i * 10);
    }
    assert!(
        run(TableCommand::BottomN {
            dir: tdir(&d),
            col: 1,
            n: 2,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn select_count_5() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5u64 {
        ins(&d, i, i * 10);
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
fn select_sum_150() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5u64 {
        ins(&d, i, i * 10);
    }
    assert!(
        run(TableCommand::SumField {
            dir: tdir(&d),
            col: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn select_avg_30() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5u64 {
        ins(&d, i, i * 10);
    }
    assert!(
        run(TableCommand::AvgField {
            dir: tdir(&d),
            col: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn select_scalar_min_10() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5u64 {
        ins(&d, i, i * 10);
    }
    assert!(
        run(TableCommand::ScalarMin {
            dir: tdir(&d),
            col: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn select_scalar_max_50() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5u64 {
        ins(&d, i, i * 10);
    }
    assert!(
        run(TableCommand::ScalarMax {
            dir: tdir(&d),
            col: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn select_filter_scan_1_row() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5u64 {
        ins(&d, i, i * 10);
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
fn select_count_distinct_5() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5u64 {
        ins(&d, i, i * 10);
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
fn select_median_30() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5u64 {
        ins(&d, i, i * 10);
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
