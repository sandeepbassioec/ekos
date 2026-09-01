//! Tests for Layer 374.
use std::path::PathBuf;
use tempfile::TempDir;
use vvctl::commands::table::{ExportFormat, OutputFormat, TableCommand, run};

fn make_dir() -> TempDir { TempDir::new().unwrap() }
fn tdir(d: &TempDir) -> PathBuf { d.path().join("tbl") }
fn setup(d: &TempDir) {
    run(TableCommand::Create { dir: tdir(d), columns: vec!["pk:u64".into(), "action:u64".into()] }).unwrap();
}
fn ins(d: &TempDir, pk: u64, val: u64) {
    run(TableCommand::Insert { dir: tdir(d), pk, values: vec![val] }).unwrap();
}

#[test]
fn audit_insert_13() {
    let d = make_dir(); setup(&d);
    for i in 1..=13u64 { ins(&d, i, i); }
    assert!(run(TableCommand::Count { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}
#[test]
fn audit_scan_13() {
    let d = make_dir(); setup(&d);
    for i in 1..=13u64 { ins(&d, i, i); }
    assert!(run(TableCommand::Scan { dir: tdir(&d), col: 0, lo: 0, hi: u64::MAX, output: OutputFormat::Table }).is_ok());
}
#[test]
fn audit_get_pk1() {
    let d = make_dir(); setup(&d); ins(&d, 1, 1);
    assert!(run(TableCommand::Get { dir: tdir(&d), pk: 1, output: OutputFormat::Table }).is_ok());
}
#[test]
fn audit_get_pk3() {
    let d = make_dir(); setup(&d); ins(&d, 3, 3);
    assert!(run(TableCommand::Get { dir: tdir(&d), pk: 3, output: OutputFormat::Table }).is_ok());
}
#[test]
fn audit_get_pk5() {
    let d = make_dir(); setup(&d); ins(&d, 5, 5);
    assert!(run(TableCommand::Get { dir: tdir(&d), pk: 5, output: OutputFormat::Table }).is_ok());
}
#[test]
fn audit_export_json() {
    let d = make_dir(); setup(&d); ins(&d, 1, 1);
    assert!(run(TableCommand::Export { dir: tdir(&d), format: ExportFormat::Json }).is_ok());
}
#[test]
fn audit_check_integrity() {
    let d = make_dir(); setup(&d); ins(&d, 1, 1);
    assert!(run(TableCommand::Check { dir: tdir(&d) }).is_ok());
}
#[test]
fn audit_stats_ok() {
    let d = make_dir(); setup(&d); ins(&d, 1, 1);
    assert!(run(TableCommand::Stats { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}
#[test]
fn audit_info_ok() {
    let d = make_dir(); setup(&d);
    assert!(run(TableCommand::Info { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}
#[test]
fn audit_recount_ok() {
    let d = make_dir(); setup(&d); ins(&d, 1, 1);
    assert!(run(TableCommand::Recount { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}
#[test]
fn audit_vacuum_ok() {
    let d = make_dir(); setup(&d);
    assert!(run(TableCommand::Vacuum { dir: tdir(&d) }).is_ok());
}
#[test]
fn audit_compact_ok() {
    let d = make_dir(); setup(&d); ins(&d, 1, 1);
    assert!(run(TableCommand::Compact { dir: tdir(&d) }).is_ok());
}
#[test]
fn audit_batch_get_ok() {
    let d = make_dir(); setup(&d);
    for i in 1..=5u64 { ins(&d, i, i); }
    assert!(run(TableCommand::BatchGet { dir: tdir(&d), pks: vec![1,2,3,4,5], output: OutputFormat::Table }).is_ok());
}
