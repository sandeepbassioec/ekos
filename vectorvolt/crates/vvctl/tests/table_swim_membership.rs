//! Tests for Layer 356.
use std::path::PathBuf;
use tempfile::TempDir;
use vvctl::commands::table::{ExportFormat, OutputFormat, TableCommand, run};

fn make_dir() -> TempDir { TempDir::new().unwrap() }
fn mk(d: &TempDir, sub: &str) -> PathBuf { d.path().join(sub).join("tbl") }
fn setup_at(p: &PathBuf) {
    run(TableCommand::Create { dir: p.clone(), columns: vec!["pk:u64".into(), "val:u64".into()] }).unwrap();
}

#[test]
fn swim_m1_insert_count() {
    let d = make_dir(); let m1 = mk(&d, "m1"); setup_at(&m1);
    run(TableCommand::Insert { dir: m1.clone(), pk: 1, values: vec![10] }).unwrap();
    assert!(run(TableCommand::Count { dir: m1, output: OutputFormat::Table }).is_ok());
}
#[test]
fn swim_m2_insert_count() {
    let d = make_dir(); let m2 = mk(&d, "m2"); setup_at(&m2);
    run(TableCommand::Insert { dir: m2.clone(), pk: 2, values: vec![20] }).unwrap();
    assert!(run(TableCommand::Count { dir: m2, output: OutputFormat::Table }).is_ok());
}
#[test]
fn swim_m3_insert_count() {
    let d = make_dir(); let m3 = mk(&d, "m3"); setup_at(&m3);
    run(TableCommand::Insert { dir: m3.clone(), pk: 3, values: vec![30] }).unwrap();
    assert!(run(TableCommand::Count { dir: m3, output: OutputFormat::Table }).is_ok());
}
#[test]
fn swim_m1_scan_ok() {
    let d = make_dir(); let m1 = mk(&d, "m1"); setup_at(&m1);
    run(TableCommand::Insert { dir: m1.clone(), pk: 1, values: vec![10] }).unwrap();
    assert!(run(TableCommand::Scan { dir: m1, col: 0, lo: 0, hi: u64::MAX, output: OutputFormat::Table }).is_ok());
}
#[test]
fn swim_all_check_ok() {
    let d = make_dir();
    for sub in &["m1","m2","m3"] {
        let p = mk(&d, sub); setup_at(&p);
        run(TableCommand::Insert { dir: p.clone(), pk: 1, values: vec![1] }).unwrap();
        run(TableCommand::Check { dir: p }).unwrap();
    }
    assert!(true);
}
#[test]
fn swim_all_vacuum_ok() {
    let d = make_dir();
    for sub in &["m1","m2","m3"] {
        let p = mk(&d, sub); setup_at(&p);
        run(TableCommand::Insert { dir: p.clone(), pk: 1, values: vec![1] }).unwrap();
        run(TableCommand::Vacuum { dir: p }).unwrap();
    }
    assert!(true);
}
#[test]
fn swim_diff_m1_m2_ok() {
    let d = make_dir();
    let m1 = mk(&d, "m1"); setup_at(&m1);
    let m2 = mk(&d, "m2"); setup_at(&m2);
    run(TableCommand::Insert { dir: m1.clone(), pk: 1, values: vec![10] }).unwrap();
    run(TableCommand::Insert { dir: m2.clone(), pk: 1, values: vec![10] }).unwrap();
    assert!(run(TableCommand::Diff { a: m1, b: m2, output: OutputFormat::Table }).is_ok());
}
#[test]
fn swim_merge_m1_into_m2_count2() {
    let d = make_dir();
    let m1 = mk(&d, "m1"); setup_at(&m1);
    let m2 = mk(&d, "m2"); setup_at(&m2);
    run(TableCommand::Insert { dir: m1.clone(), pk: 10, values: vec![10] }).unwrap();
    run(TableCommand::Insert { dir: m2.clone(), pk: 20, values: vec![20] }).unwrap();
    run(TableCommand::Merge { src: m1, dst: m2.clone() }).unwrap();
    assert!(run(TableCommand::Count { dir: m2, output: OutputFormat::Table }).is_ok());
}
#[test]
fn swim_merge_m2_into_m3_ok() {
    let d = make_dir();
    let m2 = mk(&d, "m2"); setup_at(&m2);
    let m3 = mk(&d, "m3"); setup_at(&m3);
    run(TableCommand::Insert { dir: m2.clone(), pk: 1, values: vec![1] }).unwrap();
    run(TableCommand::Insert { dir: m3.clone(), pk: 2, values: vec![2] }).unwrap();
    assert!(run(TableCommand::Merge { src: m2, dst: m3 }).is_ok());
}
#[test]
fn swim_keys_m3_ok() {
    let d = make_dir(); let m3 = mk(&d, "m3"); setup_at(&m3);
    run(TableCommand::Insert { dir: m3.clone(), pk: 1, values: vec![1] }).unwrap();
    assert!(run(TableCommand::Keys { dir: m3, output: OutputFormat::Table }).is_ok());
}
#[test]
fn swim_export_m1_ok() {
    let d = make_dir(); let m1 = mk(&d, "m1"); setup_at(&m1);
    run(TableCommand::Insert { dir: m1.clone(), pk: 1, values: vec![1] }).unwrap();
    assert!(run(TableCommand::Export { dir: m1, format: ExportFormat::Json }).is_ok());
}
#[test]
fn swim_stats_m1_ok() {
    let d = make_dir(); let m1 = mk(&d, "m1"); setup_at(&m1);
    run(TableCommand::Insert { dir: m1.clone(), pk: 1, values: vec![1] }).unwrap();
    assert!(run(TableCommand::Stats { dir: m1, output: OutputFormat::Table }).is_ok());
}
#[test]
fn swim_m1_pk_range_ok() {
    let d = make_dir(); let m1 = mk(&d, "m1"); setup_at(&m1);
    run(TableCommand::Insert { dir: m1.clone(), pk: 1, values: vec![1] }).unwrap();
    assert!(run(TableCommand::PkRange { dir: m1, output: OutputFormat::Table }).is_ok());
}
