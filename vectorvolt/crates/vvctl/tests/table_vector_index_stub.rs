//! Tests for Layer 395.
use std::path::PathBuf;
use tempfile::TempDir;
use vvctl::commands::table::{ExportFormat, OutputFormat, TableCommand, run};

fn make_dir() -> TempDir { TempDir::new().unwrap() }
fn tdir(d: &TempDir) -> PathBuf { d.path().join("tbl") }
fn setup4col(d: &TempDir) {
    run(TableCommand::Create { dir: tdir(d), columns: vec!["pk:u64".into(), "x:u64".into(), "y:u64".into(), "z:u64".into()] }).unwrap();
}

#[test]
fn vidx_insert_5() {
    let d = make_dir(); setup4col(&d);
    for i in 1..=5u64 { run(TableCommand::Insert { dir: tdir(&d), pk: i, values: vec![i, i*2, i*3] }).unwrap(); }
    assert!(run(TableCommand::Count { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}
#[test]
fn vidx_flush_persist() {
    let d = make_dir(); setup4col(&d);
    run(TableCommand::Insert { dir: tdir(&d), pk: 1, values: vec![1, 2, 3] }).unwrap();
    assert!(run(TableCommand::Flush { dir: tdir(&d) }).is_ok());
}
#[test]
fn vidx_compact_ok() {
    let d = make_dir(); setup4col(&d);
    run(TableCommand::Insert { dir: tdir(&d), pk: 1, values: vec![1, 2, 3] }).unwrap();
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    assert!(run(TableCommand::Compact { dir: tdir(&d) }).is_ok());
}
#[test]
fn vidx_scan_after_compact() {
    let d = make_dir(); setup4col(&d);
    run(TableCommand::Insert { dir: tdir(&d), pk: 1, values: vec![1, 2, 3] }).unwrap();
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    run(TableCommand::Compact { dir: tdir(&d) }).unwrap();
    assert!(run(TableCommand::Scan { dir: tdir(&d), col: 1, lo: 0, hi: u64::MAX, output: OutputFormat::Table }).is_ok());
}
#[test]
fn vidx_get_pk1_after_compact() {
    let d = make_dir(); setup4col(&d);
    run(TableCommand::Insert { dir: tdir(&d), pk: 1, values: vec![10, 20, 30] }).unwrap();
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    run(TableCommand::Compact { dir: tdir(&d) }).unwrap();
    assert!(run(TableCommand::Get { dir: tdir(&d), pk: 1, output: OutputFormat::Table }).is_ok());
}
#[test]
fn vidx_batch_get_ok() {
    let d = make_dir(); setup4col(&d);
    for i in 1..=3u64 { run(TableCommand::Insert { dir: tdir(&d), pk: i, values: vec![i, i*2, i*3] }).unwrap(); }
    assert!(run(TableCommand::BatchGet { dir: tdir(&d), pks: vec![1,2,3], output: OutputFormat::Table }).is_ok());
}
#[test]
fn vidx_check_ok() {
    let d = make_dir(); setup4col(&d);
    assert!(run(TableCommand::Check { dir: tdir(&d) }).is_ok());
}
#[test]
fn vidx_vacuum_ok() {
    let d = make_dir(); setup4col(&d);
    assert!(run(TableCommand::Vacuum { dir: tdir(&d) }).is_ok());
}
#[test]
fn vidx_stats_ok() {
    let d = make_dir(); setup4col(&d);
    run(TableCommand::Insert { dir: tdir(&d), pk: 1, values: vec![1, 2, 3] }).unwrap();
    assert!(run(TableCommand::Stats { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}
#[test]
fn vidx_info_ok() {
    let d = make_dir(); setup4col(&d);
    assert!(run(TableCommand::Info { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}
#[test]
fn vidx_recount_ok() {
    let d = make_dir(); setup4col(&d);
    run(TableCommand::Insert { dir: tdir(&d), pk: 1, values: vec![1, 2, 3] }).unwrap();
    assert!(run(TableCommand::Recount { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}
#[test]
fn vidx_export_ok() {
    let d = make_dir(); setup4col(&d);
    run(TableCommand::Insert { dir: tdir(&d), pk: 1, values: vec![1, 2, 3] }).unwrap();
    assert!(run(TableCommand::Export { dir: tdir(&d), format: ExportFormat::Json }).is_ok());
}
#[test]
fn vidx_size_ok() {
    let d = make_dir(); setup4col(&d);
    assert!(run(TableCommand::Size { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}
