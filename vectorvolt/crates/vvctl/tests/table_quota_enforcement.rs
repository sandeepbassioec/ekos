//! Tests for Layer 373.
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
fn quota_insert_10() {
    let d = make_dir(); setup(&d);
    for i in 1..=10u64 { ins(&d, i, i); }
    assert!(run(TableCommand::Count { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}
#[test]
fn quota_scan_10() {
    let d = make_dir(); setup(&d);
    for i in 1..=10u64 { ins(&d, i, i); }
    assert!(run(TableCommand::Scan { dir: tdir(&d), col: 0, lo: 0, hi: u64::MAX, output: OutputFormat::Table }).is_ok());
}
#[test]
fn quota_batch_delete_8_9() {
    let d = make_dir(); setup(&d);
    for i in 1..=10u64 { ins(&d, i, i); }
    assert!(run(TableCommand::BatchDelete { dir: tdir(&d), pks: vec![8,9] }).is_ok());
}
#[test]
fn quota_count_8() {
    let d = make_dir(); setup(&d);
    for i in 1..=10u64 { ins(&d, i, i); }
    run(TableCommand::BatchDelete { dir: tdir(&d), pks: vec![9,10] }).unwrap();
    assert!(run(TableCommand::Count { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}
#[test]
fn quota_insert_2_back() {
    let d = make_dir(); setup(&d);
    for i in 1..=8u64 { ins(&d, i, i); }
    ins(&d, 9, 9); ins(&d, 10, 10);
    assert!(run(TableCommand::Count { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}
#[test]
fn quota_compact_ok() {
    let d = make_dir(); setup(&d);
    for i in 1..=5u64 { ins(&d, i, i); }
    assert!(run(TableCommand::Compact { dir: tdir(&d) }).is_ok());
}
#[test]
fn quota_check_ok() {
    let d = make_dir(); setup(&d);
    ins(&d, 1, 1);
    assert!(run(TableCommand::Check { dir: tdir(&d) }).is_ok());
}
#[test]
fn quota_stats_ok() {
    let d = make_dir(); setup(&d);
    ins(&d, 1, 1);
    assert!(run(TableCommand::Stats { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}
#[test]
fn quota_info_ok() {
    let d = make_dir(); setup(&d);
    assert!(run(TableCommand::Info { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}
#[test]
fn quota_recount_ok() {
    let d = make_dir(); setup(&d);
    ins(&d, 1, 1);
    assert!(run(TableCommand::Recount { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}
#[test]
fn quota_vacuum_ok() {
    let d = make_dir(); setup(&d);
    assert!(run(TableCommand::Vacuum { dir: tdir(&d) }).is_ok());
}
#[test]
fn quota_export_ok() {
    let d = make_dir(); setup(&d);
    ins(&d, 1, 1);
    assert!(run(TableCommand::Export { dir: tdir(&d), format: ExportFormat::Json }).is_ok());
}
#[test]
fn quota_pk_range_ok() {
    let d = make_dir(); setup(&d);
    for i in 1..=5u64 { ins(&d, i, i); }
    assert!(run(TableCommand::PkRange { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}
