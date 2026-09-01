//! Tests for Layer 293: Head/Tail deep correctness (VV-CTL-293).

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
fn head_fresh_table_ok() {
    let d = make_dir();
    setup(&d);
    assert!(run(TableCommand::Head { dir: tdir(&d), n: 5, output: OutputFormat::Table }).is_ok());
}

#[test]
fn tail_fresh_table_ok() {
    let d = make_dir();
    setup(&d);
    assert!(run(TableCommand::Tail { dir: tdir(&d), n: 5, output: OutputFormat::Table }).is_ok());
}

#[test]
fn head_after_3_inserts_ok() {
    let d = make_dir();
    setup(&d);
    for i in 1..=3 { ins(&d, i, i * 10); }
    assert!(run(TableCommand::Head { dir: tdir(&d), n: 2, output: OutputFormat::Table }).is_ok());
}

#[test]
fn tail_after_3_inserts_ok() {
    let d = make_dir();
    setup(&d);
    for i in 1..=3 { ins(&d, i, i * 10); }
    assert!(run(TableCommand::Tail { dir: tdir(&d), n: 2, output: OutputFormat::Table }).is_ok());
}

#[test]
fn head_json_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    assert!(run(TableCommand::Head { dir: tdir(&d), n: 1, output: OutputFormat::Json }).is_ok());
}

#[test]
fn tail_json_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    assert!(run(TableCommand::Tail { dir: tdir(&d), n: 1, output: OutputFormat::Json }).is_ok());
}

#[test]
fn head_missing_dir_errors() {
    let d = make_dir();
    assert!(run(TableCommand::Head { dir: d.path().join("nope"), n: 5, output: OutputFormat::Table }).is_err());
}

#[test]
fn tail_missing_dir_errors() {
    let d = make_dir();
    assert!(run(TableCommand::Tail { dir: d.path().join("nope"), n: 5, output: OutputFormat::Table }).is_err());
}

#[test]
fn head_n_larger_than_rows_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    ins(&d, 2, 20);
    assert!(run(TableCommand::Head { dir: tdir(&d), n: 100, output: OutputFormat::Table }).is_ok());
}

#[test]
fn tail_n_larger_than_rows_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    ins(&d, 2, 20);
    assert!(run(TableCommand::Tail { dir: tdir(&d), n: 100, output: OutputFormat::Table }).is_ok());
}

#[test]
fn head_after_flush_ok() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 { ins(&d, i, i * 10); }
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    assert!(run(TableCommand::Head { dir: tdir(&d), n: 3, output: OutputFormat::Table }).is_ok());
}

#[test]
fn tail_after_flush_ok() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 { ins(&d, i, i * 10); }
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    assert!(run(TableCommand::Tail { dir: tdir(&d), n: 3, output: OutputFormat::Table }).is_ok());
}

#[test]
fn head_n1_ok() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 { ins(&d, i, i * 10); }
    assert!(run(TableCommand::Head { dir: tdir(&d), n: 1, output: OutputFormat::Table }).is_ok());
}
