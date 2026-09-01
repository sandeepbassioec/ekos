//! Tests for Layer 375.
use std::path::PathBuf;
use tempfile::TempDir;
use vvctl::commands::table::{ExportFormat, OutputFormat, TableCommand, run};

fn make_dir() -> TempDir { TempDir::new().unwrap() }
fn tdir(d: &TempDir) -> PathBuf { d.path().join("tbl") }
fn setup(d: &TempDir) {
    run(TableCommand::Create { dir: tdir(d), columns: vec!["pk:u64".into(), "val:u64".into()] }).unwrap();
}
fn ins(d: &TempDir, pk: u64, val: u64) {
    run(TableCommand::Insert { dir: tdir(d), pk, values: vec![val] }).unwrap();
}

#[test]
fn pysdh_create_ok() {
    let d = make_dir();
    assert!(run(TableCommand::Create { dir: tdir(&d), columns: vec!["pk:u64".into(), "val:u64".into()] }).is_ok());
}
#[test]
fn pysdk_insert_ok() {
    let d = make_dir(); setup(&d);
    assert!(run(TableCommand::Insert { dir: tdir(&d), pk: 1, values: vec![10] }).is_ok());
}
#[test]
fn pysdk_get_ok() {
    let d = make_dir(); setup(&d); ins(&d, 1, 10);
    assert!(run(TableCommand::Get { dir: tdir(&d), pk: 1, output: OutputFormat::Table }).is_ok());
}
#[test]
fn pysdk_scan_ok() {
    let d = make_dir(); setup(&d); ins(&d, 1, 10);
    assert!(run(TableCommand::Scan { dir: tdir(&d), col: 0, lo: 0, hi: u64::MAX, output: OutputFormat::Table }).is_ok());
}
#[test]
fn pysdk_delete_ok() {
    let d = make_dir(); setup(&d); ins(&d, 1, 10);
    assert!(run(TableCommand::Delete { dir: tdir(&d), pk: 1 }).is_ok());
}
#[test]
fn pysdk_update_ok() {
    let d = make_dir(); setup(&d); ins(&d, 1, 10);
    assert!(run(TableCommand::Update { dir: tdir(&d), pk: 1, col: 1, val: 99 }).is_ok());
}
#[test]
fn pysdk_count_ok() {
    let d = make_dir(); setup(&d); ins(&d, 1, 10);
    assert!(run(TableCommand::Count { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}
#[test]
fn pysdk_schema_ok() {
    let d = make_dir(); setup(&d);
    assert!(run(TableCommand::Schema { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}
#[test]
fn pysdk_export_ok() {
    let d = make_dir(); setup(&d); ins(&d, 1, 10);
    assert!(run(TableCommand::Export { dir: tdir(&d), format: ExportFormat::Json }).is_ok());
}
#[test]
fn pysdk_import_ok() {
    let d = make_dir(); setup(&d);
    let json = d.path().join("data.json");
    std::fs::write(&json, r#"[{"pk":1,"val":99}]"#).unwrap();
    assert!(run(TableCommand::Import { dir: tdir(&d), file: json }).is_ok());
}
#[test]
fn pysdk_check_ok() {
    let d = make_dir(); setup(&d);
    assert!(run(TableCommand::Check { dir: tdir(&d) }).is_ok());
}
#[test]
fn pysdk_stats_ok() {
    let d = make_dir(); setup(&d); ins(&d, 1, 1);
    assert!(run(TableCommand::Stats { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}
#[test]
fn pysdk_vacuum_ok() {
    let d = make_dir(); setup(&d);
    assert!(run(TableCommand::Vacuum { dir: tdir(&d) }).is_ok());
}
