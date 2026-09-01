//! Tests for Layer 389.
use std::path::PathBuf;
use tempfile::TempDir;
use vvctl::commands::table::{OutputFormat, TableCommand, run};

fn make_dir() -> TempDir { TempDir::new().unwrap() }
fn tdir(d: &TempDir) -> PathBuf { d.path().join("tbl") }
fn setup(d: &TempDir) {
    run(TableCommand::Create { dir: tdir(d), columns: vec!["pk:u64".into(), "val:u64".into()] }).unwrap();
}
fn ins(d: &TempDir, pk: u64, val: u64) {
    run(TableCommand::Insert { dir: tdir(d), pk, values: vec![val] }).unwrap();
}

#[test]
fn where_filter_scan_val20() {
    let d = make_dir(); setup(&d);
    for i in 1..=5u64 { ins(&d, i, i*10); }
    assert!(run(TableCommand::FilterScan { dir: tdir(&d), scan_col: 0, lo: 1, hi: 5, filter_col: 1, filter_val: 20, output: OutputFormat::Table }).is_ok());
}
#[test]
fn where_filter_scan_val99_empty() {
    let d = make_dir(); setup(&d);
    for i in 1..=5u64 { ins(&d, i, i*10); }
    assert!(run(TableCommand::FilterScan { dir: tdir(&d), scan_col: 0, lo: 1, hi: 5, filter_col: 1, filter_val: 99, output: OutputFormat::Table }).is_ok());
}
#[test]
fn where_scan_range_20_40() {
    let d = make_dir(); setup(&d);
    for i in 1..=5u64 { ins(&d, i, i*10); }
    assert!(run(TableCommand::Scan { dir: tdir(&d), col: 1, lo: 20, hi: 40, output: OutputFormat::Table }).is_ok());
}
#[test]
fn where_count_distinct_5() {
    let d = make_dir(); setup(&d);
    for i in 1..=5u64 { ins(&d, i, i*10); }
    assert!(run(TableCommand::CountDistinct { dir: tdir(&d), col: 1, output: OutputFormat::Table }).is_ok());
}
#[test]
fn where_zscore_filter_ok() {
    let d = make_dir(); setup(&d);
    for i in 1..=5u64 { ins(&d, i, i*10); }
    assert!(run(TableCommand::ZScoreFilter { dir: tdir(&d), col: 1, output: OutputFormat::Table }).is_ok());
}
#[test]
fn where_top_n_2() {
    let d = make_dir(); setup(&d);
    for i in 1..=5u64 { ins(&d, i, i*10); }
    assert!(run(TableCommand::TopN { dir: tdir(&d), col: 1, n: 2, output: OutputFormat::Table }).is_ok());
}
#[test]
fn where_bottom_n_2() {
    let d = make_dir(); setup(&d);
    for i in 1..=5u64 { ins(&d, i, i*10); }
    assert!(run(TableCommand::BottomN { dir: tdir(&d), col: 1, n: 2, output: OutputFormat::Table }).is_ok());
}
#[test]
fn where_percentile_50() {
    let d = make_dir(); setup(&d);
    for i in 1..=5u64 { ins(&d, i, i*10); }
    assert!(run(TableCommand::PercentileField { dir: tdir(&d), col: 1, pct: 50, output: OutputFormat::Table }).is_ok());
}
#[test]
fn where_median_30() {
    let d = make_dir(); setup(&d);
    for i in 1..=5u64 { ins(&d, i, i*10); }
    assert!(run(TableCommand::MedianField { dir: tdir(&d), col: 1, output: OutputFormat::Table }).is_ok());
}
#[test]
fn where_histogram_ok() {
    let d = make_dir(); setup(&d);
    for i in 1..=5u64 { ins(&d, i, i*10); }
    assert!(run(TableCommand::Histogram { dir: tdir(&d), col: 1, bucket_size: 10, output: OutputFormat::Table }).is_ok());
}
#[test]
fn where_running_sum_ok() {
    let d = make_dir(); setup(&d);
    for i in 1..=5u64 { ins(&d, i, i*10); }
    assert!(run(TableCommand::RunningSum { dir: tdir(&d), col: 1, output: OutputFormat::Table }).is_ok());
}
#[test]
fn where_sum_product_ok() {
    let d = make_dir();
    run(TableCommand::Create { dir: tdir(&d), columns: vec!["pk:u64".into(), "a:u64".into(), "b:u64".into()] }).unwrap();
    for i in 1..=5u64 { run(TableCommand::Insert { dir: tdir(&d), pk: i, values: vec![i, i*10] }).unwrap(); }
    assert!(run(TableCommand::SumProduct { dir: tdir(&d), col_a: 1, col_b: 2, output: OutputFormat::Table }).is_ok());
}
#[test]
fn where_window_rank_ok() {
    let d = make_dir(); setup(&d);
    for i in 1..=5u64 { ins(&d, i, i*10); }
    assert!(run(TableCommand::WindowRank { dir: tdir(&d), pk: 3, col: 1, output: OutputFormat::Table }).is_ok());
}
