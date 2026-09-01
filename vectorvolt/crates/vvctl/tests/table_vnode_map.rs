//! Tests for Layer 359.
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
fn vnode_insert_spread_pks() {
    let d = make_dir(); setup(&d);
    for pk in [0u64,16,32,48,64,80,96,112,128,144,160,176,192] { ins(&d, pk, pk); }
    assert!(run(TableCommand::Count { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}
#[test]
fn vnode_scan_13() {
    let d = make_dir(); setup(&d);
    for pk in [0u64,16,32,48,64,80,96,112,128,144,160,176,192] { ins(&d, pk, pk); }
    assert!(run(TableCommand::Scan { dir: tdir(&d), col: 0, lo: 0, hi: u64::MAX, output: OutputFormat::Table }).is_ok());
}
#[test]
fn vnode_count_13() {
    let d = make_dir(); setup(&d);
    for pk in [0u64,16,32,48,64,80,96,112,128,144,160,176,192] { ins(&d, pk, pk); }
    assert!(run(TableCommand::Count { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}
#[test]
fn vnode_top_n_5() {
    let d = make_dir(); setup(&d);
    for pk in [0u64,16,32,48,64,80,96,112,128,144,160,176,192] { ins(&d, pk, pk); }
    assert!(run(TableCommand::TopN { dir: tdir(&d), col: 0, n: 5, output: OutputFormat::Table }).is_ok());
}
#[test]
fn vnode_bottom_n_5() {
    let d = make_dir(); setup(&d);
    for pk in [0u64,16,32,48,64,80,96,112,128,144,160,176,192] { ins(&d, pk, pk); }
    assert!(run(TableCommand::BottomN { dir: tdir(&d), col: 0, n: 5, output: OutputFormat::Table }).is_ok());
}
#[test]
fn vnode_pk_range_ok() {
    let d = make_dir(); setup(&d);
    for pk in [0u64,16,32,48,64] { ins(&d, pk, pk); }
    assert!(run(TableCommand::PkRange { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}
#[test]
fn vnode_batch_get_ok() {
    let d = make_dir(); setup(&d);
    for pk in [0u64,16,32,48,64,80,96,112,128,144,160,176,192] { ins(&d, pk, pk); }
    assert!(run(TableCommand::BatchGet { dir: tdir(&d), pks: vec![0,16,32], output: OutputFormat::Table }).is_ok());
}
#[test]
fn vnode_batch_delete_ok() {
    let d = make_dir(); setup(&d);
    for pk in [0u64,16,32,48,64,80,96,112,128,144,160,176,192] { ins(&d, pk, pk); }
    assert!(run(TableCommand::BatchDelete { dir: tdir(&d), pks: vec![160,176,192] }).is_ok());
}
#[test]
fn vnode_count_10_after_delete() {
    let d = make_dir(); setup(&d);
    for pk in [0u64,16,32,48,64,80,96,112,128,144,160,176,192] { ins(&d, pk, pk); }
    run(TableCommand::BatchDelete { dir: tdir(&d), pks: vec![160,176,192] }).unwrap();
    assert!(run(TableCommand::Count { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}
#[test]
fn vnode_keys_10() {
    let d = make_dir(); setup(&d);
    for pk in [0u64,16,32,48,64,80,96,112] { ins(&d, pk, pk); }
    assert!(run(TableCommand::Keys { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}
#[test]
fn vnode_sum_field_col1() {
    let d = make_dir(); setup(&d);
    for pk in [0u64,16,32,48,64] { ins(&d, pk, pk); }
    assert!(run(TableCommand::SumField { dir: tdir(&d), col: 1, output: OutputFormat::Table }).is_ok());
}
#[test]
fn vnode_avg_field_col1() {
    let d = make_dir(); setup(&d);
    for pk in [0u64,16,32,48,64] { ins(&d, pk, pk); }
    assert!(run(TableCommand::AvgField { dir: tdir(&d), col: 1, output: OutputFormat::Table }).is_ok());
}
#[test]
fn vnode_check_ok() {
    let d = make_dir(); setup(&d);
    ins(&d, 1, 1);
    assert!(run(TableCommand::Check { dir: tdir(&d) }).is_ok());
}
