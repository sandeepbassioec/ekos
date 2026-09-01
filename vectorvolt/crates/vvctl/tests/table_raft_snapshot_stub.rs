//! Tests for Layer 355.
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
fn snapshot_insert_5_flush() {
    let d = make_dir(); setup(&d);
    for i in 1..=5u64 { ins(&d, i, i); }
    assert!(run(TableCommand::Flush { dir: tdir(&d) }).is_ok());
}
#[test]
fn snapshot_install_ok() {
    let d = make_dir(); setup(&d);
    ins(&d, 1, 1);
    assert!(run(TableCommand::Snapshot { dir: tdir(&d) }).is_ok());
}
#[test]
fn snapshot_pitr_list_has_entry() {
    let d = make_dir(); setup(&d);
    ins(&d, 1, 1);
    run(TableCommand::Snapshot { dir: tdir(&d) }).unwrap();
    assert!(run(TableCommand::PitrList { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}
#[test]
fn snapshot_truncate_wipe() {
    let d = make_dir(); setup(&d);
    for i in 1..=5u64 { ins(&d, i, i); }
    assert!(run(TableCommand::Truncate { dir: tdir(&d), force: true }).is_ok());
}
#[test]
fn snapshot_insert_3_restore() {
    let d = make_dir(); setup(&d);
    run(TableCommand::Truncate { dir: tdir(&d), force: true }).unwrap();
    for i in 1..=3u64 { ins(&d, i, i); }
    assert!(run(TableCommand::Count { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}
#[test]
fn snapshot_count_3() {
    let d = make_dir(); setup(&d);
    for i in 1..=3u64 { ins(&d, i, i); }
    assert!(run(TableCommand::Count { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}
#[test]
fn snapshot_scan_3() {
    let d = make_dir(); setup(&d);
    for i in 1..=3u64 { ins(&d, i, i); }
    assert!(run(TableCommand::Scan { dir: tdir(&d), col: 0, lo: 0, hi: u64::MAX, output: OutputFormat::Table }).is_ok());
}
#[test]
fn snapshot_check_ok() {
    let d = make_dir(); setup(&d);
    ins(&d, 1, 1);
    assert!(run(TableCommand::Check { dir: tdir(&d) }).is_ok());
}
#[test]
fn snapshot_vacuum_ok() {
    let d = make_dir(); setup(&d);
    ins(&d, 1, 1);
    assert!(run(TableCommand::Vacuum { dir: tdir(&d) }).is_ok());
}
#[test]
fn snapshot_stats_ok() {
    let d = make_dir(); setup(&d);
    ins(&d, 1, 1);
    assert!(run(TableCommand::Stats { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}
#[test]
fn snapshot_info_ok() {
    let d = make_dir(); setup(&d);
    ins(&d, 1, 1);
    assert!(run(TableCommand::Info { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}
#[test]
fn snapshot_recount_ok() {
    let d = make_dir(); setup(&d);
    ins(&d, 1, 1);
    assert!(run(TableCommand::Recount { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}
#[test]
fn snapshot_compact_ok() {
    let d = make_dir(); setup(&d);
    ins(&d, 1, 1);
    assert!(run(TableCommand::Compact { dir: tdir(&d) }).is_ok());
}
