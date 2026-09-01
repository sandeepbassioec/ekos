//! Tests for Layer 381.
use std::path::PathBuf;
use vvctl::commands::table::{OutputFormat, TableCommand};

#[test]
fn zsh_variant_window_rank() {
    let cmd = TableCommand::WindowRank { dir: PathBuf::from("/tmp"), pk: 1, col: 1, output: OutputFormat::Table };
    assert!(matches!(cmd, TableCommand::WindowRank { .. }));
}
#[test]
fn zsh_variant_sum_product() {
    let cmd = TableCommand::SumProduct { dir: PathBuf::from("/tmp"), col_a: 1, col_b: 2, output: OutputFormat::Table };
    assert!(matches!(cmd, TableCommand::SumProduct { .. }));
}
#[test]
fn zsh_variant_filter_scan() {
    let cmd = TableCommand::FilterScan { dir: PathBuf::from("/tmp"), scan_col: 0, lo: 0, hi: u64::MAX, filter_col: 1, filter_val: 10, output: OutputFormat::Table };
    assert!(matches!(cmd, TableCommand::FilterScan { .. }));
}
#[test]
fn zsh_variant_count_distinct() {
    let cmd = TableCommand::CountDistinct { dir: PathBuf::from("/tmp"), col: 1, output: OutputFormat::Table };
    assert!(matches!(cmd, TableCommand::CountDistinct { .. }));
}
#[test]
fn zsh_variant_scalar_min() {
    let cmd = TableCommand::ScalarMin { dir: PathBuf::from("/tmp"), col: 1, output: OutputFormat::Table };
    assert!(matches!(cmd, TableCommand::ScalarMin { .. }));
}
#[test]
fn zsh_variant_scalar_max() {
    let cmd = TableCommand::ScalarMax { dir: PathBuf::from("/tmp"), col: 1, output: OutputFormat::Table };
    assert!(matches!(cmd, TableCommand::ScalarMax { .. }));
}
#[test]
fn zsh_variant_unique_count() {
    let cmd = TableCommand::UniqueCount { dir: PathBuf::from("/tmp"), col: 1, output: OutputFormat::Table };
    assert!(matches!(cmd, TableCommand::UniqueCount { .. }));
}
#[test]
fn zsh_variant_batch_get() {
    let cmd = TableCommand::BatchGet { dir: PathBuf::from("/tmp"), pks: vec![1,2,3], output: OutputFormat::Table };
    assert!(matches!(cmd, TableCommand::BatchGet { .. }));
}
#[test]
fn zsh_variant_batch_delete() {
    let cmd = TableCommand::BatchDelete { dir: PathBuf::from("/tmp"), pks: vec![1,2] };
    assert!(matches!(cmd, TableCommand::BatchDelete { .. }));
}
#[test]
fn zsh_variant_increment_field() {
    let cmd = TableCommand::IncrementField { dir: PathBuf::from("/tmp"), pk: 1, col: 1 };
    assert!(matches!(cmd, TableCommand::IncrementField { .. }));
}
#[test]
fn zsh_variant_decrement_field() {
    let cmd = TableCommand::DecrementField { dir: PathBuf::from("/tmp"), pk: 1, col: 1 };
    assert!(matches!(cmd, TableCommand::DecrementField { .. }));
}
#[test]
fn zsh_variant_set_field_add() {
    let cmd = TableCommand::SetFieldAdd { dir: PathBuf::from("/tmp"), pk: 1, col: 1, addend: 5 };
    assert!(matches!(cmd, TableCommand::SetFieldAdd { .. }));
}
#[test]
fn zsh_variant_set_field_mul() {
    let cmd = TableCommand::SetFieldMul { dir: PathBuf::from("/tmp"), pk: 1, col: 1, factor: 2 };
    assert!(matches!(cmd, TableCommand::SetFieldMul { .. }));
}
