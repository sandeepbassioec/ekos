//! Tests for Layer 289: Size deep correctness (VV-CTL-289).

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
fn size_empty_table_ok() {
    let d = make_dir();
    setup(&d);
    assert!(run(TableCommand::Size { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}

#[test]
fn size_after_insert_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 100);
    assert!(run(TableCommand::Size { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}

#[test]
fn size_after_flush_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    assert!(run(TableCommand::Size { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}

#[test]
fn size_json_ok() {
    let d = make_dir();
    setup(&d);
    assert!(run(TableCommand::Size { dir: tdir(&d), output: OutputFormat::Json }).is_ok());
}

#[test]
fn size_table_format_ok() {
    let d = make_dir();
    setup(&d);
    assert!(run(TableCommand::Size { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}

#[test]
fn size_missing_dir_errors() {
    let d = make_dir();
    let bad = d.path().join("no_such");
    assert!(run(TableCommand::Size { dir: bad, output: OutputFormat::Table }).is_err());
}

#[test]
fn size_two_separate_tables_ok() {
    let d = make_dir();
    let tbl1 = d.path().join("tbl1");
    let tbl2 = d.path().join("tbl2");
    run(TableCommand::Create { dir: tbl1.clone(), columns: vec!["pk:u64".into(), "val:u64".into()] }).unwrap();
    run(TableCommand::Create { dir: tbl2.clone(), columns: vec!["pk:u64".into(), "val:u64".into()] }).unwrap();
    assert!(run(TableCommand::Size { dir: tbl1, output: OutputFormat::Table }).is_ok());
    assert!(run(TableCommand::Size { dir: tbl2, output: OutputFormat::Table }).is_ok());
}

#[test]
fn size_after_compact_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    run(TableCommand::Compact { dir: tdir(&d) }).unwrap();
    assert!(run(TableCommand::Size { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}

#[test]
fn size_after_vacuum_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    run(TableCommand::Vacuum { dir: tdir(&d) }).unwrap();
    assert!(run(TableCommand::Size { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}

#[test]
fn size_after_truncate_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    run(TableCommand::Truncate { dir: tdir(&d), force: true }).unwrap();
    assert!(run(TableCommand::Size { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}

#[test]
fn size_multi_col_ok() {
    let d = make_dir();
    run(TableCommand::Create { dir: tdir(&d), columns: vec!["pk:u64".into(), "a:u64".into(), "b:u64".into()] }).unwrap();
    run(TableCommand::Insert { dir: tdir(&d), pk: 1, values: vec![10, 20] }).unwrap();
    assert!(run(TableCommand::Size { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}

#[test]
fn size_after_snapshot_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    run(TableCommand::Snapshot { dir: tdir(&d) }).unwrap();
    assert!(run(TableCommand::Size { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}

#[test]
fn size_after_copy_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    let dst = d.path().join("dst");
    run(TableCommand::Copy { src: tdir(&d), dst: dst.clone() }).unwrap();
    assert!(run(TableCommand::Size { dir: dst, output: OutputFormat::Table }).is_ok());
}
