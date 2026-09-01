//! Tests for Layer 384.
use std::path::PathBuf;
use tempfile::TempDir;
use vvctl::commands::table::{OutputFormat, TableCommand, run};

fn make_dir() -> TempDir { TempDir::new().unwrap() }
fn tdir(d: &TempDir) -> PathBuf { d.path().join("tbl") }
fn setup(d: &TempDir) {
    run(TableCommand::Create { dir: tdir(d), columns: vec!["pk:u64".into(), "val:u64".into()] }).unwrap();
    run(TableCommand::Insert { dir: tdir(d), pk: 1, values: vec![10] }).unwrap();
}

#[test]
fn color_get_json() {
    let d = make_dir(); setup(&d);
    assert!(run(TableCommand::Get { dir: tdir(&d), pk: 1, output: OutputFormat::Json }).is_ok());
}
#[test]
fn color_get_table() {
    let d = make_dir(); setup(&d);
    assert!(run(TableCommand::Get { dir: tdir(&d), pk: 1, output: OutputFormat::Table }).is_ok());
}
#[test]
fn color_scan_json() {
    let d = make_dir(); setup(&d);
    assert!(run(TableCommand::Scan { dir: tdir(&d), col: 0, lo: 0, hi: u64::MAX, output: OutputFormat::Json }).is_ok());
}
#[test]
fn color_scan_table() {
    let d = make_dir(); setup(&d);
    assert!(run(TableCommand::Scan { dir: tdir(&d), col: 0, lo: 0, hi: u64::MAX, output: OutputFormat::Table }).is_ok());
}
#[test]
fn color_count_json() {
    let d = make_dir(); setup(&d);
    assert!(run(TableCommand::Count { dir: tdir(&d), output: OutputFormat::Json }).is_ok());
}
#[test]
fn color_count_table() {
    let d = make_dir(); setup(&d);
    assert!(run(TableCommand::Count { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}
#[test]
fn color_head_json() {
    let d = make_dir(); setup(&d);
    assert!(run(TableCommand::Head { dir: tdir(&d), n: 1, output: OutputFormat::Json }).is_ok());
}
#[test]
fn color_head_table() {
    let d = make_dir(); setup(&d);
    assert!(run(TableCommand::Head { dir: tdir(&d), n: 1, output: OutputFormat::Table }).is_ok());
}
#[test]
fn color_stats_json() {
    let d = make_dir(); setup(&d);
    assert!(run(TableCommand::Stats { dir: tdir(&d), output: OutputFormat::Json }).is_ok());
}
#[test]
fn color_stats_table() {
    let d = make_dir(); setup(&d);
    assert!(run(TableCommand::Stats { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}
#[test]
fn color_schema_json() {
    let d = make_dir(); setup(&d);
    assert!(run(TableCommand::Schema { dir: tdir(&d), output: OutputFormat::Json }).is_ok());
}
#[test]
fn color_schema_table() {
    let d = make_dir(); setup(&d);
    assert!(run(TableCommand::Schema { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}
#[test]
fn color_info_json() {
    let d = make_dir(); setup(&d);
    assert!(run(TableCommand::Info { dir: tdir(&d), output: OutputFormat::Json }).is_ok());
}
