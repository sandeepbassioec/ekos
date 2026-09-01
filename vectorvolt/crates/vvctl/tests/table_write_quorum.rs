//! Tests for Layer 368.
use std::path::PathBuf;
use tempfile::TempDir;
use vvctl::commands::table::{ExportFormat, OutputFormat, TableCommand, run};

fn make_dir() -> TempDir { TempDir::new().unwrap() }
fn tdir(d: &TempDir) -> PathBuf { d.path().join("tbl") }
fn setup(d: &TempDir) {
    run(TableCommand::Create { dir: tdir(d), columns: vec!["pk:u64".into(), "val:u64".into()] }).unwrap();
}

#[test]
fn wq_insert_5_with_flush() {
    let d = make_dir(); setup(&d);
    for i in 1..=5u64 {
        run(TableCommand::Insert { dir: tdir(&d), pk: i, values: vec![i] }).unwrap();
        run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    }
    assert!(run(TableCommand::Count { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}
#[test]
fn wq_all_rows_readable() {
    let d = make_dir(); setup(&d);
    for i in 1..=5u64 { run(TableCommand::Insert { dir: tdir(&d), pk: i, values: vec![i] }).unwrap(); }
    assert!(run(TableCommand::Get { dir: tdir(&d), pk: 1, output: OutputFormat::Table }).is_ok());
}
#[test]
fn wq_count_5() {
    let d = make_dir(); setup(&d);
    for i in 1..=5u64 { run(TableCommand::Insert { dir: tdir(&d), pk: i, values: vec![i] }).unwrap(); }
    assert!(run(TableCommand::Count { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}
#[test]
fn wq_scan_5() {
    let d = make_dir(); setup(&d);
    for i in 1..=5u64 { run(TableCommand::Insert { dir: tdir(&d), pk: i, values: vec![i] }).unwrap(); }
    assert!(run(TableCommand::Scan { dir: tdir(&d), col: 0, lo: 0, hi: u64::MAX, output: OutputFormat::Table }).is_ok());
}
#[test]
fn wq_batch_get_all_5() {
    let d = make_dir(); setup(&d);
    for i in 1..=5u64 { run(TableCommand::Insert { dir: tdir(&d), pk: i, values: vec![i] }).unwrap(); }
    assert!(run(TableCommand::BatchGet { dir: tdir(&d), pks: vec![1,2,3,4,5], output: OutputFormat::Table }).is_ok());
}
#[test]
fn wq_update_2_rows() {
    let d = make_dir(); setup(&d);
    for i in 1..=5u64 { run(TableCommand::Insert { dir: tdir(&d), pk: i, values: vec![i] }).unwrap(); }
    run(TableCommand::Update { dir: tdir(&d), pk: 1, col: 1, val: 100 }).unwrap();
    assert!(run(TableCommand::Update { dir: tdir(&d), pk: 2, col: 1, val: 200 }).is_ok());
}
#[test]
fn wq_scan_after_update_5() {
    let d = make_dir(); setup(&d);
    for i in 1..=5u64 { run(TableCommand::Insert { dir: tdir(&d), pk: i, values: vec![i] }).unwrap(); }
    run(TableCommand::Update { dir: tdir(&d), pk: 1, col: 1, val: 100 }).unwrap();
    assert!(run(TableCommand::Scan { dir: tdir(&d), col: 0, lo: 0, hi: u64::MAX, output: OutputFormat::Table }).is_ok());
}
#[test]
fn wq_check_ok() {
    let d = make_dir(); setup(&d);
    run(TableCommand::Insert { dir: tdir(&d), pk: 1, values: vec![1] }).unwrap();
    assert!(run(TableCommand::Check { dir: tdir(&d) }).is_ok());
}
#[test]
fn wq_compact_ok() {
    let d = make_dir(); setup(&d);
    run(TableCommand::Insert { dir: tdir(&d), pk: 1, values: vec![1] }).unwrap();
    assert!(run(TableCommand::Compact { dir: tdir(&d) }).is_ok());
}
#[test]
fn wq_recount_ok() {
    let d = make_dir(); setup(&d);
    run(TableCommand::Insert { dir: tdir(&d), pk: 1, values: vec![1] }).unwrap();
    assert!(run(TableCommand::Recount { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}
#[test]
fn wq_vacuum_ok() {
    let d = make_dir(); setup(&d);
    run(TableCommand::Insert { dir: tdir(&d), pk: 1, values: vec![1] }).unwrap();
    assert!(run(TableCommand::Vacuum { dir: tdir(&d) }).is_ok());
}
#[test]
fn wq_stats_ok() {
    let d = make_dir(); setup(&d);
    run(TableCommand::Insert { dir: tdir(&d), pk: 1, values: vec![1] }).unwrap();
    assert!(run(TableCommand::Stats { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}
#[test]
fn wq_export_ok() {
    let d = make_dir(); setup(&d);
    run(TableCommand::Insert { dir: tdir(&d), pk: 1, values: vec![1] }).unwrap();
    assert!(run(TableCommand::Export { dir: tdir(&d), format: ExportFormat::Json }).is_ok());
}
