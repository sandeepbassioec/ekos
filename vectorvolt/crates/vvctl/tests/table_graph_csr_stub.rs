//! Tests for Layer 396.
use std::path::PathBuf;
use tempfile::TempDir;
use vvctl::commands::table::{OutputFormat, TableCommand, run};

fn make_dir() -> TempDir { TempDir::new().unwrap() }
fn tdir(d: &TempDir) -> PathBuf { d.path().join("tbl") }
fn setup(d: &TempDir) {
    run(TableCommand::Create { dir: tdir(d), columns: vec!["pk:u64".into(), "target:u64".into()] }).unwrap();
}
fn ins(d: &TempDir, pk: u64, val: u64) {
    run(TableCommand::Insert { dir: tdir(d), pk, values: vec![val] }).unwrap();
}

#[test]
fn csr_insert_13_edges() {
    let d = make_dir(); setup(&d);
    for i in 1..=13u64 { ins(&d, i, (i % 5) + 1); }
    assert!(run(TableCommand::Count { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}
#[test]
fn csr_scan_13() {
    let d = make_dir(); setup(&d);
    for i in 1..=13u64 { ins(&d, i, (i % 5) + 1); }
    assert!(run(TableCommand::Scan { dir: tdir(&d), col: 0, lo: 0, hi: u64::MAX, output: OutputFormat::Table }).is_ok());
}
#[test]
fn csr_get_edge1() {
    let d = make_dir(); setup(&d); ins(&d, 1, 2);
    assert!(run(TableCommand::Get { dir: tdir(&d), pk: 1, output: OutputFormat::Table }).is_ok());
}
#[test]
fn csr_get_edge3() {
    let d = make_dir(); setup(&d); ins(&d, 3, 4);
    assert!(run(TableCommand::Get { dir: tdir(&d), pk: 3, output: OutputFormat::Table }).is_ok());
}
#[test]
fn csr_get_edge5() {
    let d = make_dir(); setup(&d); ins(&d, 5, 1);
    assert!(run(TableCommand::Get { dir: tdir(&d), pk: 5, output: OutputFormat::Table }).is_ok());
}
#[test]
fn csr_get_edge7() {
    let d = make_dir(); setup(&d); ins(&d, 7, 3);
    assert!(run(TableCommand::Get { dir: tdir(&d), pk: 7, output: OutputFormat::Table }).is_ok());
}
#[test]
fn csr_get_edge9() {
    let d = make_dir(); setup(&d); ins(&d, 9, 5);
    assert!(run(TableCommand::Get { dir: tdir(&d), pk: 9, output: OutputFormat::Table }).is_ok());
}
#[test]
fn csr_filter_scan_ok() {
    let d = make_dir(); setup(&d);
    for i in 1..=13u64 { ins(&d, i, (i % 5) + 1); }
    assert!(run(TableCommand::FilterScan { dir: tdir(&d), scan_col: 0, lo: 1, hi: 13, filter_col: 1, filter_val: 1, output: OutputFormat::Table }).is_ok());
}
#[test]
fn csr_top_n_by_target() {
    let d = make_dir(); setup(&d);
    for i in 1..=13u64 { ins(&d, i, (i % 5) + 1); }
    assert!(run(TableCommand::TopN { dir: tdir(&d), col: 1, n: 3, output: OutputFormat::Table }).is_ok());
}
#[test]
fn csr_batch_get_ok() {
    let d = make_dir(); setup(&d);
    for i in 1..=13u64 { ins(&d, i, (i % 5) + 1); }
    assert!(run(TableCommand::BatchGet { dir: tdir(&d), pks: vec![1,2,3], output: OutputFormat::Table }).is_ok());
}
#[test]
fn csr_check_ok() {
    let d = make_dir(); setup(&d); ins(&d, 1, 2);
    assert!(run(TableCommand::Check { dir: tdir(&d) }).is_ok());
}
#[test]
fn csr_recount_ok() {
    let d = make_dir(); setup(&d); ins(&d, 1, 2);
    assert!(run(TableCommand::Recount { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}
#[test]
fn csr_compact_ok() {
    let d = make_dir(); setup(&d); ins(&d, 1, 2);
    assert!(run(TableCommand::Compact { dir: tdir(&d) }).is_ok());
}
