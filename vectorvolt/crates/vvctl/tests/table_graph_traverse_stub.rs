//! Tests for Layer 397.
use std::path::PathBuf;
use tempfile::TempDir;
use vvctl::commands::table::{OutputFormat, TableCommand, run};

fn make_dir() -> TempDir { TempDir::new().unwrap() }
fn nodes(d: &TempDir) -> PathBuf { d.path().join("nodes") }
fn edges(d: &TempDir) -> PathBuf { d.path().join("edges") }

#[test]
fn traverse_insert_5_nodes() {
    let d = make_dir();
    run(TableCommand::Create { dir: nodes(&d), columns: vec!["pk:u64".into(), "val:u64".into()] }).unwrap();
    for i in 1..=5u64 { run(TableCommand::Insert { dir: nodes(&d), pk: i, values: vec![i] }).unwrap(); }
    assert!(run(TableCommand::Count { dir: nodes(&d), output: OutputFormat::Table }).is_ok());
}
#[test]
fn traverse_insert_5_edges() {
    let d = make_dir();
    run(TableCommand::Create { dir: edges(&d), columns: vec!["pk:u64".into(), "target:u64".into()] }).unwrap();
    for i in 1..=5u64 { run(TableCommand::Insert { dir: edges(&d), pk: i, values: vec![(i%5)+1] }).unwrap(); }
    assert!(run(TableCommand::Count { dir: edges(&d), output: OutputFormat::Table }).is_ok());
}
#[test]
fn traverse_scan_nodes_5() {
    let d = make_dir();
    run(TableCommand::Create { dir: nodes(&d), columns: vec!["pk:u64".into(), "val:u64".into()] }).unwrap();
    for i in 1..=5u64 { run(TableCommand::Insert { dir: nodes(&d), pk: i, values: vec![i] }).unwrap(); }
    assert!(run(TableCommand::Scan { dir: nodes(&d), col: 0, lo: 0, hi: u64::MAX, output: OutputFormat::Table }).is_ok());
}
#[test]
fn traverse_scan_edges_5() {
    let d = make_dir();
    run(TableCommand::Create { dir: edges(&d), columns: vec!["pk:u64".into(), "target:u64".into()] }).unwrap();
    for i in 1..=5u64 { run(TableCommand::Insert { dir: edges(&d), pk: i, values: vec![(i%5)+1] }).unwrap(); }
    assert!(run(TableCommand::Scan { dir: edges(&d), col: 0, lo: 0, hi: u64::MAX, output: OutputFormat::Table }).is_ok());
}
#[test]
fn traverse_get_node1() {
    let d = make_dir();
    run(TableCommand::Create { dir: nodes(&d), columns: vec!["pk:u64".into(), "val:u64".into()] }).unwrap();
    run(TableCommand::Insert { dir: nodes(&d), pk: 1, values: vec![100] }).unwrap();
    assert!(run(TableCommand::Get { dir: nodes(&d), pk: 1, output: OutputFormat::Table }).is_ok());
}
#[test]
fn traverse_get_edge1() {
    let d = make_dir();
    run(TableCommand::Create { dir: edges(&d), columns: vec!["pk:u64".into(), "target:u64".into()] }).unwrap();
    run(TableCommand::Insert { dir: edges(&d), pk: 1, values: vec![2] }).unwrap();
    assert!(run(TableCommand::Get { dir: edges(&d), pk: 1, output: OutputFormat::Table }).is_ok());
}
#[test]
fn traverse_filter_edges_by_target() {
    let d = make_dir();
    run(TableCommand::Create { dir: edges(&d), columns: vec!["pk:u64".into(), "target:u64".into()] }).unwrap();
    for i in 1..=5u64 { run(TableCommand::Insert { dir: edges(&d), pk: i, values: vec![(i%5)+1] }).unwrap(); }
    assert!(run(TableCommand::FilterScan { dir: edges(&d), scan_col: 0, lo: 1, hi: 5, filter_col: 1, filter_val: 2, output: OutputFormat::Table }).is_ok());
}
#[test]
fn traverse_top_n_nodes() {
    let d = make_dir();
    run(TableCommand::Create { dir: nodes(&d), columns: vec!["pk:u64".into(), "val:u64".into()] }).unwrap();
    for i in 1..=5u64 { run(TableCommand::Insert { dir: nodes(&d), pk: i, values: vec![i] }).unwrap(); }
    assert!(run(TableCommand::TopN { dir: nodes(&d), col: 1, n: 3, output: OutputFormat::Table }).is_ok());
}
#[test]
fn traverse_bottom_n_nodes() {
    let d = make_dir();
    run(TableCommand::Create { dir: nodes(&d), columns: vec!["pk:u64".into(), "val:u64".into()] }).unwrap();
    for i in 1..=5u64 { run(TableCommand::Insert { dir: nodes(&d), pk: i, values: vec![i] }).unwrap(); }
    assert!(run(TableCommand::BottomN { dir: nodes(&d), col: 1, n: 3, output: OutputFormat::Table }).is_ok());
}
#[test]
fn traverse_check_nodes() {
    let d = make_dir();
    run(TableCommand::Create { dir: nodes(&d), columns: vec!["pk:u64".into(), "val:u64".into()] }).unwrap();
    assert!(run(TableCommand::Check { dir: nodes(&d) }).is_ok());
}
#[test]
fn traverse_check_edges() {
    let d = make_dir();
    run(TableCommand::Create { dir: edges(&d), columns: vec!["pk:u64".into(), "target:u64".into()] }).unwrap();
    assert!(run(TableCommand::Check { dir: edges(&d) }).is_ok());
}
#[test]
fn traverse_recount_nodes() {
    let d = make_dir();
    run(TableCommand::Create { dir: nodes(&d), columns: vec!["pk:u64".into(), "val:u64".into()] }).unwrap();
    run(TableCommand::Insert { dir: nodes(&d), pk: 1, values: vec![1] }).unwrap();
    assert!(run(TableCommand::Recount { dir: nodes(&d), output: OutputFormat::Table }).is_ok());
}
#[test]
fn traverse_recount_edges() {
    let d = make_dir();
    run(TableCommand::Create { dir: edges(&d), columns: vec!["pk:u64".into(), "target:u64".into()] }).unwrap();
    run(TableCommand::Insert { dir: edges(&d), pk: 1, values: vec![2] }).unwrap();
    assert!(run(TableCommand::Recount { dir: edges(&d), output: OutputFormat::Table }).is_ok());
}
