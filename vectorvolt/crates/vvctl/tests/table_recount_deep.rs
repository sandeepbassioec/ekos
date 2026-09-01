//! Tests for Layer 295: Recount command correctness (VV-CTL-295).

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
fn recount_fresh_table_ok() {
    let d = make_dir();
    setup(&d);
    assert!(run(TableCommand::Recount { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}

#[test]
fn recount_after_1_insert_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    assert!(run(TableCommand::Recount { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}

#[test]
fn recount_after_5_inserts_ok() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 { ins(&d, i, i * 10); }
    assert!(run(TableCommand::Recount { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}

#[test]
fn recount_json_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    assert!(run(TableCommand::Recount { dir: tdir(&d), output: OutputFormat::Json }).is_ok());
}

#[test]
fn recount_missing_dir_errors() {
    let d = make_dir();
    assert!(run(TableCommand::Recount { dir: d.path().join("nope"), output: OutputFormat::Table }).is_err());
}

#[test]
fn recount_after_delete_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    ins(&d, 2, 20);
    run(TableCommand::Delete { dir: tdir(&d), pk: 1 }).unwrap();
    assert!(run(TableCommand::Recount { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}

#[test]
fn recount_after_truncate_ok() {
    let d = make_dir();
    setup(&d);
    for i in 1..=3 { ins(&d, i, i * 10); }
    run(TableCommand::Truncate { dir: tdir(&d), force: true }).unwrap();
    assert!(run(TableCommand::Recount { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}

#[test]
fn recount_after_flush_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    assert!(run(TableCommand::Recount { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}

#[test]
fn recount_after_compact_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    run(TableCommand::Compact { dir: tdir(&d) }).unwrap();
    assert!(run(TableCommand::Recount { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}

#[test]
fn recount_after_vacuum_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    run(TableCommand::Vacuum { dir: tdir(&d) }).unwrap();
    assert!(run(TableCommand::Recount { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}

#[test]
fn recount_mixed_flushed_buffered_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    ins(&d, 2, 20);
    assert!(run(TableCommand::Recount { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}

#[test]
fn recount_after_update_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    run(TableCommand::Update { dir: tdir(&d), pk: 1, col: 1, val: 99 }).unwrap();
    assert!(run(TableCommand::Recount { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}

#[test]
fn recount_10_rows_ok() {
    let d = make_dir();
    setup(&d);
    for i in 1..=10 { ins(&d, i, i * 10); }
    assert!(run(TableCommand::Recount { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}
