//! Tests for Layer 379.
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
fn pkg_create_ok() {
    let d = make_dir();
    assert!(run(TableCommand::Create { dir: tdir(&d), columns: vec!["pk:u64".into(), "val:u64".into()] }).is_ok());
}
#[test]
fn pkg_insert_config_row() {
    let d = make_dir(); setup(&d);
    assert!(run(TableCommand::Insert { dir: tdir(&d), pk: 1, values: vec![42] }).is_ok());
}
#[test]
fn pkg_get_config() {
    let d = make_dir(); setup(&d); ins(&d, 1, 42);
    assert!(run(TableCommand::Get { dir: tdir(&d), pk: 1, output: OutputFormat::Table }).is_ok());
}
#[test]
fn pkg_schema_ok() {
    let d = make_dir(); setup(&d);
    assert!(run(TableCommand::Schema { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}
#[test]
fn pkg_stats_ok() {
    let d = make_dir(); setup(&d); ins(&d, 1, 1);
    assert!(run(TableCommand::Stats { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}
#[test]
fn pkg_info_ok() {
    let d = make_dir(); setup(&d);
    assert!(run(TableCommand::Info { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}
#[test]
fn pkg_check_ok() {
    let d = make_dir(); setup(&d);
    assert!(run(TableCommand::Check { dir: tdir(&d) }).is_ok());
}
#[test]
fn pkg_vacuum_ok() {
    let d = make_dir(); setup(&d);
    assert!(run(TableCommand::Vacuum { dir: tdir(&d) }).is_ok());
}
#[test]
fn pkg_compact_ok() {
    let d = make_dir(); setup(&d); ins(&d, 1, 1);
    assert!(run(TableCommand::Compact { dir: tdir(&d) }).is_ok());
}
#[test]
fn pkg_recount_ok() {
    let d = make_dir(); setup(&d); ins(&d, 1, 1);
    assert!(run(TableCommand::Recount { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}
#[test]
fn pkg_export_ok() {
    let d = make_dir(); setup(&d); ins(&d, 1, 1);
    assert!(run(TableCommand::Export { dir: tdir(&d), format: ExportFormat::Json }).is_ok());
}
#[test]
fn pkg_size_ok() {
    let d = make_dir(); setup(&d);
    assert!(run(TableCommand::Size { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}
#[test]
fn pkg_pk_range_ok() {
    let d = make_dir(); setup(&d); ins(&d, 1, 1); ins(&d, 5, 5);
    assert!(run(TableCommand::PkRange { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}
