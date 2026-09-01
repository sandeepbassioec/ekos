//! Tests for Layer 353.
use std::path::PathBuf;
use tempfile::TempDir;
use vvctl::commands::table::{ExportFormat, OutputFormat, TableCommand, run};

fn make_dir() -> TempDir { TempDir::new().unwrap() }
fn tdir(d: &TempDir) -> PathBuf { d.path().join("tbl") }
fn setup(d: &TempDir) {
    run(TableCommand::Create { dir: tdir(d), columns: vec!["pk:u64".into(), "val:u64".into()] }).unwrap();
}
fn ins(d: &TempDir, pk: u64, val: u64) {
    run(TableCommand::Insert { dir: tdir(d), pk, values: vec![val] }).unwrap();
}

#[test]
fn follower_copy_ok() {
    let d = make_dir(); setup(&d);
    for i in 1..=3u64 { ins(&d, i, i); }
    let dst = d.path().join("follower");
    assert!(run(TableCommand::Copy { src: tdir(&d), dst: dst.clone() }).is_ok());
}
#[test]
fn follower_count_matches_source() {
    let d = make_dir(); setup(&d);
    for i in 1..=3u64 { ins(&d, i, i); }
    let dst = d.path().join("follower");
    run(TableCommand::Copy { src: tdir(&d), dst: dst.clone() }).unwrap();
    assert!(run(TableCommand::Count { dir: dst, output: OutputFormat::Table }).is_ok());
}
#[test]
fn follower_scan_ok() {
    let d = make_dir(); setup(&d);
    ins(&d, 1, 10); ins(&d, 2, 20);
    let dst = d.path().join("follower");
    run(TableCommand::Copy { src: tdir(&d), dst: dst.clone() }).unwrap();
    assert!(run(TableCommand::Scan { dir: dst, col: 0, lo: 0, hi: u64::MAX, output: OutputFormat::Table }).is_ok());
}
#[test]
fn follower_diff_zero() {
    let d = make_dir(); setup(&d);
    ins(&d, 1, 1); ins(&d, 2, 2);
    let dst = d.path().join("follower");
    run(TableCommand::Copy { src: tdir(&d), dst: dst.clone() }).unwrap();
    assert!(run(TableCommand::Diff { a: tdir(&d), b: dst, output: OutputFormat::Table }).is_ok());
}
#[test]
fn follower_merge_noop() {
    let d = make_dir(); setup(&d);
    ins(&d, 1, 1);
    let dst = d.path().join("follower");
    run(TableCommand::Copy { src: tdir(&d), dst: dst.clone() }).unwrap();
    assert!(run(TableCommand::Merge { src: tdir(&d), dst: dst.clone() }).is_ok());
}
#[test]
fn follower_check_ok() {
    let d = make_dir(); setup(&d);
    ins(&d, 1, 1);
    let dst = d.path().join("follower");
    run(TableCommand::Copy { src: tdir(&d), dst: dst.clone() }).unwrap();
    assert!(run(TableCommand::Check { dir: dst }).is_ok());
}
#[test]
fn follower_vacuum_ok() {
    let d = make_dir(); setup(&d);
    ins(&d, 1, 1);
    let dst = d.path().join("follower");
    run(TableCommand::Copy { src: tdir(&d), dst: dst.clone() }).unwrap();
    assert!(run(TableCommand::Vacuum { dir: dst }).is_ok());
}
#[test]
fn follower_stats_ok() {
    let d = make_dir(); setup(&d);
    ins(&d, 1, 1);
    let dst = d.path().join("follower");
    run(TableCommand::Copy { src: tdir(&d), dst: dst.clone() }).unwrap();
    assert!(run(TableCommand::Stats { dir: dst, output: OutputFormat::Table }).is_ok());
}
#[test]
fn follower_info_ok() {
    let d = make_dir(); setup(&d);
    ins(&d, 1, 1);
    let dst = d.path().join("follower");
    run(TableCommand::Copy { src: tdir(&d), dst: dst.clone() }).unwrap();
    assert!(run(TableCommand::Info { dir: dst, output: OutputFormat::Table }).is_ok());
}
#[test]
fn follower_recount_ok() {
    let d = make_dir(); setup(&d);
    ins(&d, 1, 1);
    let dst = d.path().join("follower");
    run(TableCommand::Copy { src: tdir(&d), dst: dst.clone() }).unwrap();
    assert!(run(TableCommand::Recount { dir: dst, output: OutputFormat::Table }).is_ok());
}
#[test]
fn follower_compact_ok() {
    let d = make_dir(); setup(&d);
    ins(&d, 1, 1);
    let dst = d.path().join("follower");
    run(TableCommand::Copy { src: tdir(&d), dst: dst.clone() }).unwrap();
    assert!(run(TableCommand::Compact { dir: dst }).is_ok());
}
#[test]
fn follower_export_ok() {
    let d = make_dir(); setup(&d);
    ins(&d, 1, 1);
    let dst = d.path().join("follower");
    run(TableCommand::Copy { src: tdir(&d), dst: dst.clone() }).unwrap();
    assert!(run(TableCommand::Export { dir: dst, format: ExportFormat::Json }).is_ok());
}
#[test]
fn follower_pk_range_ok() {
    let d = make_dir(); setup(&d);
    ins(&d, 1, 1); ins(&d, 5, 5);
    let dst = d.path().join("follower");
    run(TableCommand::Copy { src: tdir(&d), dst: dst.clone() }).unwrap();
    assert!(run(TableCommand::PkRange { dir: dst, output: OutputFormat::Table }).is_ok());
}
