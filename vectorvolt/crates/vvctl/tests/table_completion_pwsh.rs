//! Tests for Layer 383.
use std::path::PathBuf;
use vvctl::commands::table::{OutputFormat, TableCommand};

#[test]
fn pwsh_variant_stats() {
    let cmd = TableCommand::Stats { dir: PathBuf::from("/tmp"), output: OutputFormat::Table };
    assert!(matches!(cmd, TableCommand::Stats { .. }));
}
#[test]
fn pwsh_variant_flush() {
    let cmd = TableCommand::Flush { dir: PathBuf::from("/tmp") };
    assert!(matches!(cmd, TableCommand::Flush { .. }));
}
#[test]
fn pwsh_variant_compact() {
    let cmd = TableCommand::Compact { dir: PathBuf::from("/tmp") };
    assert!(matches!(cmd, TableCommand::Compact { .. }));
}
#[test]
fn pwsh_variant_schema() {
    let cmd = TableCommand::Schema { dir: PathBuf::from("/tmp"), output: OutputFormat::Table };
    assert!(matches!(cmd, TableCommand::Schema { .. }));
}
#[test]
fn pwsh_variant_drop() {
    let cmd = TableCommand::Drop { dir: PathBuf::from("/tmp"), force: false };
    assert!(matches!(cmd, TableCommand::Drop { .. }));
}
#[test]
fn pwsh_variant_truncate() {
    let cmd = TableCommand::Truncate { dir: PathBuf::from("/tmp"), force: true };
    assert!(matches!(cmd, TableCommand::Truncate { .. }));
}
#[test]
fn pwsh_variant_size() {
    let cmd = TableCommand::Size { dir: PathBuf::from("/tmp"), output: OutputFormat::Table };
    assert!(matches!(cmd, TableCommand::Size { .. }));
}
#[test]
fn pwsh_variant_info() {
    let cmd = TableCommand::Info { dir: PathBuf::from("/tmp"), output: OutputFormat::Table };
    assert!(matches!(cmd, TableCommand::Info { .. }));
}
#[test]
fn pwsh_variant_check() {
    let cmd = TableCommand::Check { dir: PathBuf::from("/tmp") };
    assert!(matches!(cmd, TableCommand::Check { .. }));
}
#[test]
fn pwsh_variant_col_stats() {
    let cmd = TableCommand::ColStats { dir: PathBuf::from("/tmp"), col: "val".into(), output: OutputFormat::Table };
    assert!(matches!(cmd, TableCommand::ColStats { .. }));
}
#[test]
fn pwsh_variant_pk_range() {
    let cmd = TableCommand::PkRange { dir: PathBuf::from("/tmp"), output: OutputFormat::Table };
    assert!(matches!(cmd, TableCommand::PkRange { .. }));
}
#[test]
fn pwsh_variant_keys() {
    let cmd = TableCommand::Keys { dir: PathBuf::from("/tmp"), output: OutputFormat::Table };
    assert!(matches!(cmd, TableCommand::Keys { .. }));
}
#[test]
fn pwsh_variant_exists() {
    let cmd = TableCommand::Exists { dir: PathBuf::from("/tmp") };
    assert!(matches!(cmd, TableCommand::Exists { .. }));
}
