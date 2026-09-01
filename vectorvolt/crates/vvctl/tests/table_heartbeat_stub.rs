//! Tests for Layer 365.
use std::path::PathBuf;
use tempfile::TempDir;
use vvctl::commands::table::{OutputFormat, TableCommand, run};

fn make_dir() -> TempDir { TempDir::new().unwrap() }
fn tdir(d: &TempDir) -> PathBuf { d.path().join("tbl") }
fn setup(d: &TempDir) {
    run(TableCommand::Create { dir: tdir(d), columns: vec!["pk:u64".into(), "hb:u64".into()] }).unwrap();
}

#[test]
fn hb_insert_pk1() {
    let d = make_dir(); setup(&d);
    assert!(run(TableCommand::Insert { dir: tdir(&d), pk: 1, values: vec![1] }).is_ok());
}
#[test]
fn hb_incr_to_2() {
    let d = make_dir(); setup(&d);
    run(TableCommand::Insert { dir: tdir(&d), pk: 1, values: vec![1] }).unwrap();
    assert!(run(TableCommand::IncrementField { dir: tdir(&d), pk: 1, col: 1 }).is_ok());
}
#[test]
fn hb_incr_to_3() {
    let d = make_dir(); setup(&d);
    run(TableCommand::Insert { dir: tdir(&d), pk: 1, values: vec![1] }).unwrap();
    run(TableCommand::IncrementField { dir: tdir(&d), pk: 1, col: 1 }).unwrap();
    assert!(run(TableCommand::IncrementField { dir: tdir(&d), pk: 1, col: 1 }).is_ok());
}
#[test]
fn hb_incr_5_more() {
    let d = make_dir(); setup(&d);
    run(TableCommand::Insert { dir: tdir(&d), pk: 1, values: vec![1] }).unwrap();
    for _ in 0..5 { run(TableCommand::IncrementField { dir: tdir(&d), pk: 1, col: 1 }).unwrap(); }
    assert!(run(TableCommand::Get { dir: tdir(&d), pk: 1, output: OutputFormat::Table }).is_ok());
}
#[test]
fn hb_get_pk1_ok() {
    let d = make_dir(); setup(&d);
    run(TableCommand::Insert { dir: tdir(&d), pk: 1, values: vec![8] }).unwrap();
    assert!(run(TableCommand::Get { dir: tdir(&d), pk: 1, output: OutputFormat::Table }).is_ok());
}
#[test]
fn hb_decr_ok() {
    let d = make_dir(); setup(&d);
    run(TableCommand::Insert { dir: tdir(&d), pk: 1, values: vec![8] }).unwrap();
    assert!(run(TableCommand::DecrementField { dir: tdir(&d), pk: 1, col: 1 }).is_ok());
}
#[test]
fn hb_set_add_ok() {
    let d = make_dir(); setup(&d);
    run(TableCommand::Insert { dir: tdir(&d), pk: 1, values: vec![7] }).unwrap();
    assert!(run(TableCommand::SetFieldAdd { dir: tdir(&d), pk: 1, col: 1, addend: 10 }).is_ok());
}
#[test]
fn hb_clamp_ok() {
    let d = make_dir(); setup(&d);
    run(TableCommand::Insert { dir: tdir(&d), pk: 1, values: vec![17] }).unwrap();
    assert!(run(TableCommand::ClampField { dir: tdir(&d), pk: 1, col: 1, lo: 0, hi: 100 }).is_ok());
}
#[test]
fn hb_check_ok() {
    let d = make_dir(); setup(&d);
    run(TableCommand::Insert { dir: tdir(&d), pk: 1, values: vec![1] }).unwrap();
    assert!(run(TableCommand::Check { dir: tdir(&d) }).is_ok());
}
#[test]
fn hb_stats_ok() {
    let d = make_dir(); setup(&d);
    run(TableCommand::Insert { dir: tdir(&d), pk: 1, values: vec![1] }).unwrap();
    assert!(run(TableCommand::Stats { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}
#[test]
fn hb_recount_ok() {
    let d = make_dir(); setup(&d);
    run(TableCommand::Insert { dir: tdir(&d), pk: 1, values: vec![1] }).unwrap();
    assert!(run(TableCommand::Recount { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}
#[test]
fn hb_vacuum_ok() {
    let d = make_dir(); setup(&d);
    run(TableCommand::Insert { dir: tdir(&d), pk: 1, values: vec![1] }).unwrap();
    assert!(run(TableCommand::Vacuum { dir: tdir(&d) }).is_ok());
}
#[test]
fn hb_incr_col0_errors() {
    let d = make_dir(); setup(&d);
    run(TableCommand::Insert { dir: tdir(&d), pk: 1, values: vec![1] }).unwrap();
    assert!(run(TableCommand::IncrementField { dir: tdir(&d), pk: 1, col: 0 }).is_err());
}
