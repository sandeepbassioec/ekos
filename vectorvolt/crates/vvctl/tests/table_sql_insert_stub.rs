//! Tests for Layer 388.
use std::path::PathBuf;
use tempfile::TempDir;
use vvctl::commands::table::{OutputFormat, TableCommand, run};

fn make_dir() -> TempDir { TempDir::new().unwrap() }
fn tdir(d: &TempDir) -> PathBuf { d.path().join("tbl") }
fn setup(d: &TempDir) {
    run(TableCommand::Create { dir: tdir(d), columns: vec!["pk:u64".into(), "val:u64".into()] }).unwrap();
}

#[test]
fn sql_insert_typed_pk1() {
    let d = make_dir(); setup(&d);
    assert!(run(TableCommand::TypedInsert { dir: tdir(&d), pk: "1".into(), values: vec!["100".into()] }).is_ok());
}
#[test]
fn sql_insert_typed_pk2() {
    let d = make_dir(); setup(&d);
    assert!(run(TableCommand::TypedInsert { dir: tdir(&d), pk: "2".into(), values: vec!["200".into()] }).is_ok());
}
#[test]
fn sql_insert_typed_pk3() {
    let d = make_dir(); setup(&d);
    assert!(run(TableCommand::TypedInsert { dir: tdir(&d), pk: "3".into(), values: vec!["300".into()] }).is_ok());
}
#[test]
fn sql_insert_get_pk1() {
    let d = make_dir(); setup(&d);
    run(TableCommand::TypedInsert { dir: tdir(&d), pk: "1".into(), values: vec!["100".into()] }).unwrap();
    assert!(run(TableCommand::Get { dir: tdir(&d), pk: 1, output: OutputFormat::Table }).is_ok());
}
#[test]
fn sql_insert_get_pk2() {
    let d = make_dir(); setup(&d);
    run(TableCommand::TypedInsert { dir: tdir(&d), pk: "2".into(), values: vec!["200".into()] }).unwrap();
    assert!(run(TableCommand::Get { dir: tdir(&d), pk: 2, output: OutputFormat::Table }).is_ok());
}
#[test]
fn sql_insert_get_pk3() {
    let d = make_dir(); setup(&d);
    run(TableCommand::TypedInsert { dir: tdir(&d), pk: "3".into(), values: vec!["300".into()] }).unwrap();
    assert!(run(TableCommand::Get { dir: tdir(&d), pk: 3, output: OutputFormat::Table }).is_ok());
}
#[test]
fn sql_insert_count_3() {
    let d = make_dir(); setup(&d);
    run(TableCommand::TypedInsert { dir: tdir(&d), pk: "1".into(), values: vec!["100".into()] }).unwrap();
    run(TableCommand::TypedInsert { dir: tdir(&d), pk: "2".into(), values: vec!["200".into()] }).unwrap();
    run(TableCommand::TypedInsert { dir: tdir(&d), pk: "3".into(), values: vec!["300".into()] }).unwrap();
    assert!(run(TableCommand::Count { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}
#[test]
fn sql_insert_scan_3() {
    let d = make_dir(); setup(&d);
    run(TableCommand::TypedInsert { dir: tdir(&d), pk: "1".into(), values: vec!["100".into()] }).unwrap();
    run(TableCommand::TypedInsert { dir: tdir(&d), pk: "2".into(), values: vec!["200".into()] }).unwrap();
    run(TableCommand::TypedInsert { dir: tdir(&d), pk: "3".into(), values: vec!["300".into()] }).unwrap();
    assert!(run(TableCommand::Scan { dir: tdir(&d), col: 0, lo: 0, hi: u64::MAX, output: OutputFormat::Table }).is_ok());
}
#[test]
fn sql_insert_upsert_pk1() {
    let d = make_dir(); setup(&d);
    run(TableCommand::Insert { dir: tdir(&d), pk: 1, values: vec![100] }).unwrap();
    assert!(run(TableCommand::UpsertRow { dir: tdir(&d), pk: 1, values: vec![999] }).is_ok());
}
#[test]
fn sql_insert_get_pk1_after_upsert() {
    let d = make_dir(); setup(&d);
    run(TableCommand::Insert { dir: tdir(&d), pk: 1, values: vec![100] }).unwrap();
    run(TableCommand::UpsertRow { dir: tdir(&d), pk: 1, values: vec![999] }).unwrap();
    assert!(run(TableCommand::Get { dir: tdir(&d), pk: 1, output: OutputFormat::Table }).is_ok());
}
#[test]
fn sql_insert_count_still_3() {
    let d = make_dir(); setup(&d);
    for i in 1..=3u64 { run(TableCommand::Insert { dir: tdir(&d), pk: i, values: vec![i * 100] }).unwrap(); }
    run(TableCommand::UpsertRow { dir: tdir(&d), pk: 1, values: vec![999] }).unwrap();
    assert!(run(TableCommand::Count { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}
#[test]
fn sql_insert_check_ok() {
    let d = make_dir(); setup(&d);
    run(TableCommand::Insert { dir: tdir(&d), pk: 1, values: vec![1] }).unwrap();
    assert!(run(TableCommand::Check { dir: tdir(&d) }).is_ok());
}
#[test]
fn sql_insert_recount_ok() {
    let d = make_dir(); setup(&d);
    run(TableCommand::Insert { dir: tdir(&d), pk: 1, values: vec![1] }).unwrap();
    assert!(run(TableCommand::Recount { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}
