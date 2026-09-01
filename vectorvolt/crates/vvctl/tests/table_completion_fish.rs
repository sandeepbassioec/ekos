//! Tests for Layer 382.
use std::path::PathBuf;
use vvctl::commands::table::{ExportFormat, OutputFormat, TableCommand};

#[test]
fn fish_variant_upsert_row() {
    let cmd = TableCommand::UpsertRow { dir: PathBuf::from("/tmp"), pk: 1, values: vec![99] };
    assert!(matches!(cmd, TableCommand::UpsertRow { .. }));
}
#[test]
fn fish_variant_compare_and_swap() {
    let cmd = TableCommand::CompareAndSwap { dir: PathBuf::from("/tmp"), pk: 1, col: 1, expected: 0, replacement: 1 };
    assert!(matches!(cmd, TableCommand::CompareAndSwap { .. }));
}
#[test]
fn fish_variant_clamp_field() {
    let cmd = TableCommand::ClampField { dir: PathBuf::from("/tmp"), pk: 1, col: 1, lo: 0, hi: 100 };
    assert!(matches!(cmd, TableCommand::ClampField { .. }));
}
#[test]
fn fish_variant_typed_insert() {
    let cmd = TableCommand::TypedInsert { dir: PathBuf::from("/tmp"), pk: "1".into(), values: vec!["10".into()] };
    assert!(matches!(cmd, TableCommand::TypedInsert { .. }));
}
#[test]
fn fish_variant_alter_add_col() {
    let cmd = TableCommand::AlterAddCol { dir: PathBuf::from("/tmp"), column: "x:u64".into() };
    assert!(matches!(cmd, TableCommand::AlterAddCol { .. }));
}
#[test]
fn fish_variant_alter_drop_col() {
    let cmd = TableCommand::AlterDropCol { dir: PathBuf::from("/tmp"), column: "x".into() };
    assert!(matches!(cmd, TableCommand::AlterDropCol { .. }));
}
#[test]
fn fish_variant_rename_col() {
    let cmd = TableCommand::RenameCol { dir: PathBuf::from("/tmp"), from: "a".into(), to: "b".into() };
    assert!(matches!(cmd, TableCommand::RenameCol { .. }));
}
#[test]
fn fish_variant_copy() {
    let cmd = TableCommand::Copy { src: PathBuf::from("/tmp/src"), dst: PathBuf::from("/tmp/dst") };
    assert!(matches!(cmd, TableCommand::Copy { .. }));
}
#[test]
fn fish_variant_diff() {
    let cmd = TableCommand::Diff { a: PathBuf::from("/tmp/a"), b: PathBuf::from("/tmp/b"), output: OutputFormat::Table };
    assert!(matches!(cmd, TableCommand::Diff { .. }));
}
#[test]
fn fish_variant_merge() {
    let cmd = TableCommand::Merge { src: PathBuf::from("/tmp/src"), dst: PathBuf::from("/tmp/dst") };
    assert!(matches!(cmd, TableCommand::Merge { .. }));
}
#[test]
fn fish_variant_recount() {
    let cmd = TableCommand::Recount { dir: PathBuf::from("/tmp"), output: OutputFormat::Table };
    assert!(matches!(cmd, TableCommand::Recount { .. }));
}
#[test]
fn fish_variant_export() {
    let cmd = TableCommand::Export { dir: PathBuf::from("/tmp"), format: ExportFormat::Json };
    assert!(matches!(cmd, TableCommand::Export { .. }));
}
#[test]
fn fish_variant_import() {
    let cmd = TableCommand::Import { dir: PathBuf::from("/tmp"), file: PathBuf::from("/tmp/f.json") };
    assert!(matches!(cmd, TableCommand::Import { .. }));
}
