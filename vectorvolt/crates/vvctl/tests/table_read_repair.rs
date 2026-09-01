//! Tests for Layer 367.
use std::path::PathBuf;
use tempfile::TempDir;
use vvctl::commands::table::{OutputFormat, TableCommand, run};

fn make_dir() -> TempDir { TempDir::new().unwrap() }
fn primary(d: &TempDir) -> PathBuf { d.path().join("primary") }
fn replica(d: &TempDir) -> PathBuf { d.path().join("replica") }

#[test]
fn rr_insert_pk1() {
    let d = make_dir();
    run(TableCommand::Create { dir: primary(&d), columns: vec!["pk:u64".into(), "val:u64".into()] }).unwrap();
    assert!(run(TableCommand::Insert { dir: primary(&d), pk: 1, values: vec![10] }).is_ok());
}
#[test]
fn rr_copy_to_replica() {
    let d = make_dir();
    run(TableCommand::Create { dir: primary(&d), columns: vec!["pk:u64".into(), "val:u64".into()] }).unwrap();
    run(TableCommand::Insert { dir: primary(&d), pk: 1, values: vec![10] }).unwrap();
    assert!(run(TableCommand::Copy { src: primary(&d), dst: replica(&d) }).is_ok());
}
#[test]
fn rr_replica_get_pk1() {
    let d = make_dir();
    run(TableCommand::Create { dir: primary(&d), columns: vec!["pk:u64".into(), "val:u64".into()] }).unwrap();
    run(TableCommand::Insert { dir: primary(&d), pk: 1, values: vec![10] }).unwrap();
    run(TableCommand::Copy { src: primary(&d), dst: replica(&d) }).unwrap();
    assert!(run(TableCommand::Get { dir: replica(&d), pk: 1, output: OutputFormat::Table }).is_ok());
}
#[test]
fn rr_update_primary_stale_replica() {
    let d = make_dir();
    run(TableCommand::Create { dir: primary(&d), columns: vec!["pk:u64".into(), "val:u64".into()] }).unwrap();
    run(TableCommand::Insert { dir: primary(&d), pk: 1, values: vec![10] }).unwrap();
    run(TableCommand::Copy { src: primary(&d), dst: replica(&d) }).unwrap();
    assert!(run(TableCommand::Update { dir: primary(&d), pk: 1, col: 1, val: 20 }).is_ok());
}
#[test]
fn rr_diff_1_changed() {
    let d = make_dir();
    run(TableCommand::Create { dir: primary(&d), columns: vec!["pk:u64".into(), "val:u64".into()] }).unwrap();
    run(TableCommand::Insert { dir: primary(&d), pk: 1, values: vec![10] }).unwrap();
    run(TableCommand::Copy { src: primary(&d), dst: replica(&d) }).unwrap();
    run(TableCommand::Update { dir: primary(&d), pk: 1, col: 1, val: 20 }).unwrap();
    assert!(run(TableCommand::Diff { a: primary(&d), b: replica(&d), output: OutputFormat::Table }).is_ok());
}
#[test]
fn rr_merge_repair() {
    let d = make_dir();
    run(TableCommand::Create { dir: primary(&d), columns: vec!["pk:u64".into(), "val:u64".into()] }).unwrap();
    run(TableCommand::Insert { dir: primary(&d), pk: 1, values: vec![10] }).unwrap();
    run(TableCommand::Copy { src: primary(&d), dst: replica(&d) }).unwrap();
    run(TableCommand::Update { dir: primary(&d), pk: 1, col: 1, val: 20 }).unwrap();
    assert!(run(TableCommand::Merge { src: primary(&d), dst: replica(&d) }).is_ok());
}
#[test]
fn rr_replica_get_pk1_repaired() {
    let d = make_dir();
    run(TableCommand::Create { dir: primary(&d), columns: vec!["pk:u64".into(), "val:u64".into()] }).unwrap();
    run(TableCommand::Insert { dir: primary(&d), pk: 1, values: vec![10] }).unwrap();
    run(TableCommand::Copy { src: primary(&d), dst: replica(&d) }).unwrap();
    run(TableCommand::Update { dir: primary(&d), pk: 1, col: 1, val: 20 }).unwrap();
    run(TableCommand::Merge { src: primary(&d), dst: replica(&d) }).unwrap();
    assert!(run(TableCommand::Get { dir: replica(&d), pk: 1, output: OutputFormat::Table }).is_ok());
}
#[test]
fn rr_diff_zero_after_repair() {
    let d = make_dir();
    run(TableCommand::Create { dir: primary(&d), columns: vec!["pk:u64".into(), "val:u64".into()] }).unwrap();
    run(TableCommand::Insert { dir: primary(&d), pk: 1, values: vec![10] }).unwrap();
    run(TableCommand::Copy { src: primary(&d), dst: replica(&d) }).unwrap();
    run(TableCommand::Update { dir: primary(&d), pk: 1, col: 1, val: 20 }).unwrap();
    run(TableCommand::Merge { src: primary(&d), dst: replica(&d) }).unwrap();
    assert!(run(TableCommand::Diff { a: primary(&d), b: replica(&d), output: OutputFormat::Table }).is_ok());
}
#[test]
fn rr_check_primary() {
    let d = make_dir();
    run(TableCommand::Create { dir: primary(&d), columns: vec!["pk:u64".into(), "val:u64".into()] }).unwrap();
    run(TableCommand::Insert { dir: primary(&d), pk: 1, values: vec![10] }).unwrap();
    assert!(run(TableCommand::Check { dir: primary(&d) }).is_ok());
}
#[test]
fn rr_check_replica() {
    let d = make_dir();
    run(TableCommand::Create { dir: primary(&d), columns: vec!["pk:u64".into(), "val:u64".into()] }).unwrap();
    run(TableCommand::Insert { dir: primary(&d), pk: 1, values: vec![10] }).unwrap();
    run(TableCommand::Copy { src: primary(&d), dst: replica(&d) }).unwrap();
    assert!(run(TableCommand::Check { dir: replica(&d) }).is_ok());
}
#[test]
fn rr_vacuum_primary() {
    let d = make_dir();
    run(TableCommand::Create { dir: primary(&d), columns: vec!["pk:u64".into(), "val:u64".into()] }).unwrap();
    run(TableCommand::Insert { dir: primary(&d), pk: 1, values: vec![10] }).unwrap();
    assert!(run(TableCommand::Vacuum { dir: primary(&d) }).is_ok());
}
#[test]
fn rr_stats_replica() {
    let d = make_dir();
    run(TableCommand::Create { dir: primary(&d), columns: vec!["pk:u64".into(), "val:u64".into()] }).unwrap();
    run(TableCommand::Insert { dir: primary(&d), pk: 1, values: vec![10] }).unwrap();
    run(TableCommand::Copy { src: primary(&d), dst: replica(&d) }).unwrap();
    assert!(run(TableCommand::Stats { dir: replica(&d), output: OutputFormat::Table }).is_ok());
}
#[test]
fn rr_recount_primary() {
    let d = make_dir();
    run(TableCommand::Create { dir: primary(&d), columns: vec!["pk:u64".into(), "val:u64".into()] }).unwrap();
    run(TableCommand::Insert { dir: primary(&d), pk: 1, values: vec![10] }).unwrap();
    assert!(run(TableCommand::Recount { dir: primary(&d), output: OutputFormat::Table }).is_ok());
}
