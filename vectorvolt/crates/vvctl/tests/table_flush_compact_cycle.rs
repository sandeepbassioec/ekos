//! Tests for Layer 297: Flush/Compact cycle correctness (VV-CTL-297).

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
fn flush_then_compact_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    assert!(run(TableCommand::Compact { dir: tdir(&d) }).is_ok());
}

#[test]
fn flush_compact_vacuum_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    run(TableCommand::Compact { dir: tdir(&d) }).unwrap();
    assert!(run(TableCommand::Vacuum { dir: tdir(&d) }).is_ok());
}

#[test]
fn flush_compact_count_ok() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 { ins(&d, i, i * 10); }
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    run(TableCommand::Compact { dir: tdir(&d) }).unwrap();
    assert!(run(TableCommand::Count { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}

#[test]
fn flush_compact_get_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 42, 420);
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    run(TableCommand::Compact { dir: tdir(&d) }).unwrap();
    assert!(run(TableCommand::Get { dir: tdir(&d), pk: 42, output: OutputFormat::Table }).is_ok());
}

#[test]
fn flush_empty_table_ok() {
    let d = make_dir();
    setup(&d);
    assert!(run(TableCommand::Flush { dir: tdir(&d) }).is_ok());
}

#[test]
fn compact_empty_table_ok() {
    let d = make_dir();
    setup(&d);
    assert!(run(TableCommand::Compact { dir: tdir(&d) }).is_ok());
}

#[test]
fn double_flush_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    ins(&d, 2, 20);
    assert!(run(TableCommand::Flush { dir: tdir(&d) }).is_ok());
}

#[test]
fn flush_compact_scan_ok() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 { ins(&d, i, i * 10); }
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    run(TableCommand::Compact { dir: tdir(&d) }).unwrap();
    assert!(run(TableCommand::Scan { dir: tdir(&d), col: 1, lo: 10, hi: 50, output: OutputFormat::Table }).is_ok());
}

#[test]
fn flush_compact_after_delete_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    ins(&d, 2, 20);
    run(TableCommand::Delete { dir: tdir(&d), pk: 1 }).unwrap();
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    assert!(run(TableCommand::Compact { dir: tdir(&d) }).is_ok());
}

#[test]
fn flush_compact_stats_ok() {
    let d = make_dir();
    setup(&d);
    for i in 1..=3 { ins(&d, i, i * 10); }
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    run(TableCommand::Compact { dir: tdir(&d) }).unwrap();
    assert!(run(TableCommand::Stats { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}

#[test]
fn flush_missing_dir_errors() {
    let d = make_dir();
    assert!(run(TableCommand::Flush { dir: d.path().join("nope") }).is_err());
}

#[test]
fn compact_missing_dir_errors() {
    let d = make_dir();
    assert!(run(TableCommand::Compact { dir: d.path().join("nope") }).is_err());
}

#[test]
fn flush_compact_keys_ok() {
    let d = make_dir();
    setup(&d);
    for i in 1..=4 { ins(&d, i, i * 10); }
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    run(TableCommand::Compact { dir: tdir(&d) }).unwrap();
    assert!(run(TableCommand::Keys { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}
