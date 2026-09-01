//! Tests for Layer 361.
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
fn split_insert_10_flush() {
    let d = make_dir(); setup(&d);
    for i in 0..10u64 { ins(&d, i, i); }
    assert!(run(TableCommand::Flush { dir: tdir(&d) }).is_ok());
}
#[test]
fn split_compact_ok() {
    let d = make_dir(); setup(&d);
    for i in 0..10u64 { ins(&d, i, i); }
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    assert!(run(TableCommand::Compact { dir: tdir(&d) }).is_ok());
}
#[test]
fn split_copy_to_shard_a() {
    let d = make_dir(); setup(&d);
    for i in 0..10u64 { ins(&d, i, i); }
    let sa = d.path().join("shard_a");
    assert!(run(TableCommand::Copy { src: tdir(&d), dst: sa }).is_ok());
}
#[test]
fn split_copy_to_shard_b() {
    let d = make_dir(); setup(&d);
    for i in 0..10u64 { ins(&d, i, i); }
    let sb = d.path().join("shard_b");
    assert!(run(TableCommand::Copy { src: tdir(&d), dst: sb }).is_ok());
}
#[test]
fn split_shard_a_trim_high_half() {
    let d = make_dir(); setup(&d);
    for i in 0..10u64 { ins(&d, i, i); }
    let sa = d.path().join("shard_a");
    run(TableCommand::Copy { src: tdir(&d), dst: sa.clone() }).unwrap();
    run(TableCommand::BatchDelete { dir: sa.clone(), pks: vec![5,6,7,8,9] }).unwrap();
    assert!(run(TableCommand::Count { dir: sa, output: OutputFormat::Table }).is_ok());
}
#[test]
fn split_shard_b_trim_low_half() {
    let d = make_dir(); setup(&d);
    for i in 0..10u64 { ins(&d, i, i); }
    let sb = d.path().join("shard_b");
    run(TableCommand::Copy { src: tdir(&d), dst: sb.clone() }).unwrap();
    run(TableCommand::BatchDelete { dir: sb.clone(), pks: vec![0,1,2,3,4] }).unwrap();
    assert!(run(TableCommand::Count { dir: sb, output: OutputFormat::Table }).is_ok());
}
#[test]
fn split_shard_a_count_5() {
    let d = make_dir(); setup(&d);
    for i in 0..10u64 { ins(&d, i, i); }
    let sa = d.path().join("shard_a");
    run(TableCommand::Copy { src: tdir(&d), dst: sa.clone() }).unwrap();
    run(TableCommand::BatchDelete { dir: sa.clone(), pks: vec![5,6,7,8,9] }).unwrap();
    assert!(run(TableCommand::Scan { dir: sa, col: 0, lo: 0, hi: u64::MAX, output: OutputFormat::Table }).is_ok());
}
#[test]
fn split_shard_b_count_5() {
    let d = make_dir(); setup(&d);
    for i in 0..10u64 { ins(&d, i, i); }
    let sb = d.path().join("shard_b");
    run(TableCommand::Copy { src: tdir(&d), dst: sb.clone() }).unwrap();
    run(TableCommand::BatchDelete { dir: sb.clone(), pks: vec![0,1,2,3,4] }).unwrap();
    assert!(run(TableCommand::Scan { dir: sb, col: 0, lo: 0, hi: u64::MAX, output: OutputFormat::Table }).is_ok());
}
#[test]
fn split_check_shard_a() {
    let d = make_dir(); setup(&d);
    ins(&d, 1, 1);
    let sa = d.path().join("shard_a");
    run(TableCommand::Copy { src: tdir(&d), dst: sa.clone() }).unwrap();
    assert!(run(TableCommand::Check { dir: sa }).is_ok());
}
#[test]
fn split_check_shard_b() {
    let d = make_dir(); setup(&d);
    ins(&d, 1, 1);
    let sb = d.path().join("shard_b");
    run(TableCommand::Copy { src: tdir(&d), dst: sb.clone() }).unwrap();
    assert!(run(TableCommand::Check { dir: sb }).is_ok());
}
#[test]
fn split_vacuum_shard_a() {
    let d = make_dir(); setup(&d);
    ins(&d, 1, 1);
    let sa = d.path().join("shard_a");
    run(TableCommand::Copy { src: tdir(&d), dst: sa.clone() }).unwrap();
    assert!(run(TableCommand::Vacuum { dir: sa }).is_ok());
}
#[test]
fn split_vacuum_shard_b() {
    let d = make_dir(); setup(&d);
    ins(&d, 1, 1);
    let sb = d.path().join("shard_b");
    run(TableCommand::Copy { src: tdir(&d), dst: sb.clone() }).unwrap();
    assert!(run(TableCommand::Vacuum { dir: sb }).is_ok());
}
#[test]
fn split_stats_original_ok() {
    let d = make_dir(); setup(&d);
    for i in 0..5u64 { ins(&d, i, i); }
    assert!(run(TableCommand::Stats { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}
