//! Tests for Layer 401.
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
fn prod_create_insert_flush_compact_check() {
    let d = make_dir(); setup(&d);
    for i in 1..=5u64 { ins(&d, i, i); }
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    run(TableCommand::Compact { dir: tdir(&d) }).unwrap();
    assert!(run(TableCommand::Check { dir: tdir(&d) }).is_ok());
}
#[test]
fn prod_alter_add_schema_shows_new_col() {
    let d = make_dir(); setup(&d);
    run(TableCommand::AlterAddCol { dir: tdir(&d), column: "extra:u64".into() }).unwrap();
    assert!(run(TableCommand::Schema { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}
#[test]
fn prod_rename_col_roundtrip() {
    let d = make_dir(); setup(&d);
    run(TableCommand::RenameCol { dir: tdir(&d), from: "val".into(), to: "value".into() }).unwrap();
    assert!(run(TableCommand::RenameCol { dir: tdir(&d), from: "value".into(), to: "val".into() }).is_ok());
}
#[test]
fn prod_full_crud() {
    let d = make_dir(); setup(&d);
    ins(&d, 1, 100);
    run(TableCommand::Get { dir: tdir(&d), pk: 1, output: OutputFormat::Table }).unwrap();
    run(TableCommand::Update { dir: tdir(&d), pk: 1, col: 1, val: 200 }).unwrap();
    assert!(run(TableCommand::Delete { dir: tdir(&d), pk: 1 }).is_ok());
}
#[test]
fn prod_batch_get_delete() {
    let d = make_dir(); setup(&d);
    for i in 1..=5u64 { ins(&d, i, i); }
    run(TableCommand::BatchGet { dir: tdir(&d), pks: vec![1,2,3], output: OutputFormat::Table }).unwrap();
    assert!(run(TableCommand::BatchDelete { dir: tdir(&d), pks: vec![4,5] }).is_ok());
}
#[test]
fn prod_cas_sequence() {
    let d = make_dir(); setup(&d); ins(&d, 1, 10);
    run(TableCommand::CompareAndSwap { dir: tdir(&d), pk: 1, col: 1, expected: 10, replacement: 20 }).unwrap();
    assert!(run(TableCommand::CompareAndSwap { dir: tdir(&d), pk: 1, col: 1, expected: 20, replacement: 30 }).is_ok());
}
#[test]
fn prod_incr_decr_add_mul_clamp() {
    let d = make_dir(); setup(&d); ins(&d, 1, 5);
    run(TableCommand::IncrementField { dir: tdir(&d), pk: 1, col: 1 }).unwrap();
    run(TableCommand::DecrementField { dir: tdir(&d), pk: 1, col: 1 }).unwrap();
    run(TableCommand::SetFieldAdd { dir: tdir(&d), pk: 1, col: 1, addend: 10 }).unwrap();
    run(TableCommand::SetFieldMul { dir: tdir(&d), pk: 1, col: 1, factor: 2 }).unwrap();
    assert!(run(TableCommand::ClampField { dir: tdir(&d), pk: 1, col: 1, lo: 0, hi: 1000 }).is_ok());
}
#[test]
fn prod_upsert_get() {
    let d = make_dir(); setup(&d);
    run(TableCommand::UpsertRow { dir: tdir(&d), pk: 42, values: vec![999] }).unwrap();
    assert!(run(TableCommand::Get { dir: tdir(&d), pk: 42, output: OutputFormat::Table }).is_ok());
}
#[test]
fn prod_copy_diff_merge() {
    let d = make_dir(); setup(&d);
    ins(&d, 1, 1); ins(&d, 2, 2);
    let dst = d.path().join("copy");
    run(TableCommand::Copy { src: tdir(&d), dst: dst.clone() }).unwrap();
    run(TableCommand::Diff { a: tdir(&d), b: dst.clone(), output: OutputFormat::Table }).unwrap();
    assert!(run(TableCommand::Merge { src: tdir(&d), dst }).is_ok());
}
#[test]
fn prod_export_import_roundtrip() {
    let d = make_dir(); setup(&d); ins(&d, 1, 100);
    run(TableCommand::Export { dir: tdir(&d), format: ExportFormat::Json }).unwrap();
    let json = d.path().join("data.json");
    std::fs::write(&json, r#"[{"pk":2,"val":200}]"#).unwrap();
    assert!(run(TableCommand::Import { dir: tdir(&d), file: json }).is_ok());
}
#[test]
fn prod_snapshot_pitr_list() {
    let d = make_dir(); setup(&d); ins(&d, 1, 1);
    run(TableCommand::Snapshot { dir: tdir(&d) }).unwrap();
    assert!(run(TableCommand::PitrList { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}
#[test]
fn prod_vacuum_recount_check() {
    let d = make_dir(); setup(&d); ins(&d, 1, 1);
    run(TableCommand::Vacuum { dir: tdir(&d) }).unwrap();
    run(TableCommand::Recount { dir: tdir(&d), output: OutputFormat::Table }).unwrap();
    assert!(run(TableCommand::Check { dir: tdir(&d) }).is_ok());
}
#[test]
fn prod_stats_info_size_schema() {
    let d = make_dir(); setup(&d); ins(&d, 1, 1);
    run(TableCommand::Stats { dir: tdir(&d), output: OutputFormat::Table }).unwrap();
    run(TableCommand::Info { dir: tdir(&d), output: OutputFormat::Table }).unwrap();
    run(TableCommand::Size { dir: tdir(&d), output: OutputFormat::Table }).unwrap();
    assert!(run(TableCommand::Schema { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}
