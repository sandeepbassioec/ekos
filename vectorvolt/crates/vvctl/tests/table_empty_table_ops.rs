//! Tests for Layer 261: vvctl table empty table operations (VV-CTL-261).

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

#[test]
fn empty_scan_ok() {
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
fn empty_get_missing_ok() {
    let d = make_dir();
    setup(&d);
    // Get on non-existent pk returns Ok (no row found)
    run(TableCommand::Get {
        dir: tdir(&d),
        pk: 99,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn empty_count_zero_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Count {
        dir: tdir(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn empty_head_n5_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Head {
        dir: tdir(&d),
        n: 5,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn empty_tail_n5_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Tail {
        dir: tdir(&d),
        n: 5,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn empty_keys_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Keys {
        dir: tdir(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn empty_pk_range_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::PkRange {
        dir: tdir(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn empty_col_stats_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::ColStats {
        dir: tdir(&d),
        col: "val".into(),
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn empty_info_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Info {
        dir: tdir(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn empty_size_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Size {
        dir: tdir(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn empty_top_n_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::TopN {
        dir: tdir(&d),
        col: 1,
        n: 5,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn empty_bottom_n_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::BottomN {
        dir: tdir(&d),
        col: 1,
        n: 5,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn empty_sum_field_zero() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::SumField {
        dir: tdir(&d),
        col: 1,
        output: OutputFormat::Table,
    })
    .unwrap();
}
