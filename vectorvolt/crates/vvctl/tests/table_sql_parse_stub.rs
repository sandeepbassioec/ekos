//! Tests for Layer 386.
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
fn sql_parse_insert_3() {
    let d = make_dir(); setup(&d);
    ins(&d, 1, 10); ins(&d, 2, 20); ins(&d, 3, 30);
    assert!(run(TableCommand::Count { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}
#[test]
fn sql_parse_scan_all_3() {
    let d = make_dir(); setup(&d);
    ins(&d, 1, 10); ins(&d, 2, 20); ins(&d, 3, 30);
    assert!(run(TableCommand::Scan { dir: tdir(&d), col: 0, lo: 1, hi: 3, output: OutputFormat::Table }).is_ok());
}
#[test]
fn sql_parse_range_select() {
    let d = make_dir(); setup(&d);
    ins(&d, 1, 10); ins(&d, 2, 20); ins(&d, 3, 30);
    assert!(run(TableCommand::Scan { dir: tdir(&d), col: 0, lo: 1, hi: 3, output: OutputFormat::Table }).is_ok());
}
#[test]
fn sql_parse_point_select() {
    let d = make_dir(); setup(&d); ins(&d, 2, 20);
    assert!(run(TableCommand::Get { dir: tdir(&d), pk: 2, output: OutputFormat::Table }).is_ok());
}
#[test]
fn sql_parse_count() {
    let d = make_dir(); setup(&d);
    ins(&d, 1, 10); ins(&d, 2, 20); ins(&d, 3, 30);
    assert!(run(TableCommand::Count { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}
#[test]
fn sql_parse_sum() {
    let d = make_dir(); setup(&d);
    ins(&d, 1, 10); ins(&d, 2, 20); ins(&d, 3, 30);
    assert!(run(TableCommand::SumField { dir: tdir(&d), col: 1, output: OutputFormat::Table }).is_ok());
}
#[test]
fn sql_parse_avg() {
    let d = make_dir(); setup(&d);
    ins(&d, 1, 10); ins(&d, 2, 20); ins(&d, 3, 30);
    assert!(run(TableCommand::AvgField { dir: tdir(&d), col: 1, output: OutputFormat::Table }).is_ok());
}
#[test]
fn sql_parse_min() {
    let d = make_dir(); setup(&d);
    ins(&d, 1, 10); ins(&d, 2, 20); ins(&d, 3, 30);
    assert!(run(TableCommand::ScalarMin { dir: tdir(&d), col: 1, output: OutputFormat::Table }).is_ok());
}
#[test]
fn sql_parse_max() {
    let d = make_dir(); setup(&d);
    ins(&d, 1, 10); ins(&d, 2, 20); ins(&d, 3, 30);
    assert!(run(TableCommand::ScalarMax { dir: tdir(&d), col: 1, output: OutputFormat::Table }).is_ok());
}
#[test]
fn sql_parse_top_n_3() {
    let d = make_dir(); setup(&d);
    ins(&d, 1, 10); ins(&d, 2, 20); ins(&d, 3, 30);
    assert!(run(TableCommand::TopN { dir: tdir(&d), col: 1, n: 3, output: OutputFormat::Table }).is_ok());
}
#[test]
fn sql_parse_bottom_n_3() {
    let d = make_dir(); setup(&d);
    ins(&d, 1, 10); ins(&d, 2, 20); ins(&d, 3, 30);
    assert!(run(TableCommand::BottomN { dir: tdir(&d), col: 1, n: 3, output: OutputFormat::Table }).is_ok());
}
#[test]
fn sql_parse_filter_scan() {
    let d = make_dir(); setup(&d);
    ins(&d, 1, 10); ins(&d, 2, 20); ins(&d, 3, 30);
    assert!(run(TableCommand::FilterScan { dir: tdir(&d), scan_col: 0, lo: 1, hi: 3, filter_col: 1, filter_val: 20, output: OutputFormat::Table }).is_ok());
}
#[test]
fn sql_parse_count_distinct() {
    let d = make_dir(); setup(&d);
    ins(&d, 1, 10); ins(&d, 2, 20); ins(&d, 3, 30);
    assert!(run(TableCommand::CountDistinct { dir: tdir(&d), col: 1, output: OutputFormat::Table }).is_ok());
}
