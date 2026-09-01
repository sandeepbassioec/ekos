//! Tests for Layer 352.
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
fn leader_insert_5_rows() {
    let d = make_dir(); setup(&d);
    for i in 1..=5u64 { ins(&d, i, i * 10); }
    assert!(run(TableCommand::Count { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}
#[test]
fn leader_flush_ok() {
    let d = make_dir(); setup(&d);
    for i in 1..=5u64 { ins(&d, i, i); }
    assert!(run(TableCommand::Flush { dir: tdir(&d) }).is_ok());
}
#[test]
fn leader_compact_ok() {
    let d = make_dir(); setup(&d);
    for i in 1..=5u64 { ins(&d, i, i); }
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    assert!(run(TableCommand::Compact { dir: tdir(&d) }).is_ok());
}
#[test]
fn leader_scan_5() {
    let d = make_dir(); setup(&d);
    for i in 1..=5u64 { ins(&d, i, i); }
    assert!(run(TableCommand::Scan { dir: tdir(&d), col: 0, lo: 0, hi: u64::MAX, output: OutputFormat::Table }).is_ok());
}
#[test]
fn leader_count_5() {
    let d = make_dir(); setup(&d);
    for i in 1..=5u64 { ins(&d, i, i); }
    assert!(run(TableCommand::Count { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}
#[test]
fn leader_update_pk1_ok() {
    let d = make_dir(); setup(&d);
    ins(&d, 1, 100);
    assert!(run(TableCommand::Update { dir: tdir(&d), pk: 1, col: 1, val: 999 }).is_ok());
}
#[test]
fn leader_delete_pk2() {
    let d = make_dir(); setup(&d);
    ins(&d, 1, 1); ins(&d, 2, 2);
    assert!(run(TableCommand::Delete { dir: tdir(&d), pk: 2 }).is_ok());
}
#[test]
fn leader_count_after_delete() {
    let d = make_dir(); setup(&d);
    for i in 1..=5u64 { ins(&d, i, i); }
    run(TableCommand::Delete { dir: tdir(&d), pk: 2 }).unwrap();
    assert!(run(TableCommand::Count { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}
#[test]
fn leader_snapshot_ok() {
    let d = make_dir(); setup(&d);
    ins(&d, 1, 1);
    assert!(run(TableCommand::Snapshot { dir: tdir(&d) }).is_ok());
}
#[test]
fn leader_check_ok() {
    let d = make_dir(); setup(&d);
    ins(&d, 1, 1);
    assert!(run(TableCommand::Check { dir: tdir(&d) }).is_ok());
}
#[test]
fn leader_stats_ok() {
    let d = make_dir(); setup(&d);
    ins(&d, 1, 1);
    assert!(run(TableCommand::Stats { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}
#[test]
fn leader_recount_ok() {
    let d = make_dir(); setup(&d);
    ins(&d, 1, 1);
    assert!(run(TableCommand::Recount { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}
#[test]
fn leader_vacuum_info_ok() {
    let d = make_dir(); setup(&d);
    ins(&d, 1, 1);
    run(TableCommand::Vacuum { dir: tdir(&d) }).unwrap();
    assert!(run(TableCommand::Info { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}
