//! Tests for Layer 360.
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
fn chash_insert_prime_pks() {
    let d = make_dir(); setup(&d);
    for pk in [1u64,7,13,17,23,29,31,37,41,43,47,53,59] { ins(&d, pk, pk); }
    assert!(run(TableCommand::Count { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}
#[test]
fn chash_scan_13() {
    let d = make_dir(); setup(&d);
    for pk in [1u64,7,13,17,23,29,31,37,41,43,47,53,59] { ins(&d, pk, pk); }
    assert!(run(TableCommand::Scan { dir: tdir(&d), col: 0, lo: 0, hi: u64::MAX, output: OutputFormat::Table }).is_ok());
}
#[test]
fn chash_count_13() {
    let d = make_dir(); setup(&d);
    for pk in [1u64,7,13,17,23,29,31,37,41,43,47,53,59] { ins(&d, pk, pk); }
    assert!(run(TableCommand::Count { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}
#[test]
fn chash_top_n_3() {
    let d = make_dir(); setup(&d);
    for pk in [1u64,7,13,17,23,29,31,37,41,43,47,53,59] { ins(&d, pk, pk); }
    assert!(run(TableCommand::TopN { dir: tdir(&d), col: 0, n: 3, output: OutputFormat::Table }).is_ok());
}
#[test]
fn chash_bottom_n_3() {
    let d = make_dir(); setup(&d);
    for pk in [1u64,7,13,17,23,29,31,37,41,43,47,53,59] { ins(&d, pk, pk); }
    assert!(run(TableCommand::BottomN { dir: tdir(&d), col: 0, n: 3, output: OutputFormat::Table }).is_ok());
}
#[test]
fn chash_pk_range_ok() {
    let d = make_dir(); setup(&d);
    for pk in [1u64,7,13,17,23] { ins(&d, pk, pk); }
    assert!(run(TableCommand::PkRange { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}
#[test]
fn chash_batch_get_ok() {
    let d = make_dir(); setup(&d);
    for pk in [1u64,7,13,17,23,29,31,37,41,43,47,53,59] { ins(&d, pk, pk); }
    assert!(run(TableCommand::BatchGet { dir: tdir(&d), pks: vec![1,7,13,17,23], output: OutputFormat::Table }).is_ok());
}
#[test]
fn chash_batch_delete_ok() {
    let d = make_dir(); setup(&d);
    for pk in [1u64,7,13,17,23,29,31,37,41,43,47,53,59] { ins(&d, pk, pk); }
    assert!(run(TableCommand::BatchDelete { dir: tdir(&d), pks: vec![53,59] }).is_ok());
}
#[test]
fn chash_count_11() {
    let d = make_dir(); setup(&d);
    for pk in [1u64,7,13,17,23,29,31,37,41,43,47,53,59] { ins(&d, pk, pk); }
    run(TableCommand::BatchDelete { dir: tdir(&d), pks: vec![53,59] }).unwrap();
    assert!(run(TableCommand::Count { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}
#[test]
fn chash_keys_11() {
    let d = make_dir(); setup(&d);
    for pk in [1u64,7,13,17,23,29,31,37,41,43,47,53,59] { ins(&d, pk, pk); }
    run(TableCommand::BatchDelete { dir: tdir(&d), pks: vec![53,59] }).unwrap();
    assert!(run(TableCommand::Keys { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}
#[test]
fn chash_sum_ok() {
    let d = make_dir(); setup(&d);
    for pk in [1u64,7,13,17,23] { ins(&d, pk, pk); }
    assert!(run(TableCommand::SumField { dir: tdir(&d), col: 1, output: OutputFormat::Table }).is_ok());
}
#[test]
fn chash_avg_ok() {
    let d = make_dir(); setup(&d);
    for pk in [1u64,7,13,17,23] { ins(&d, pk, pk); }
    assert!(run(TableCommand::AvgField { dir: tdir(&d), col: 1, output: OutputFormat::Table }).is_ok());
}
#[test]
fn chash_compact_ok() {
    let d = make_dir(); setup(&d);
    ins(&d, 1, 1);
    assert!(run(TableCommand::Compact { dir: tdir(&d) }).is_ok());
}
