//! Tests for Layer 380.
use std::path::PathBuf;
use vvctl::commands::table::{OutputFormat, TableCommand};

#[test]
fn bash_variant_update() {
    let cmd = TableCommand::Update {
        dir: PathBuf::from("/tmp"),
        pk: 1,
        col: 1,
        val: 10,
    };
    assert!(matches!(cmd, TableCommand::Update { .. }));
}
#[test]
fn bash_variant_vacuum() {
    let cmd = TableCommand::Vacuum {
        dir: PathBuf::from("/tmp"),
    };
    assert!(matches!(cmd, TableCommand::Vacuum { .. }));
}
#[test]
fn bash_variant_snapshot() {
    let cmd = TableCommand::Snapshot {
        dir: PathBuf::from("/tmp"),
    };
    assert!(matches!(cmd, TableCommand::Snapshot { .. }));
}
#[test]
fn bash_variant_pitr_list() {
    let cmd = TableCommand::PitrList {
        dir: PathBuf::from("/tmp"),
        output: OutputFormat::Table,
    };
    assert!(matches!(cmd, TableCommand::PitrList { .. }));
}
#[test]
fn bash_variant_top_n() {
    let cmd = TableCommand::TopN {
        dir: PathBuf::from("/tmp"),
        col: 1,
        n: 5,
        output: OutputFormat::Table,
    };
    assert!(matches!(cmd, TableCommand::TopN { .. }));
}
#[test]
fn bash_variant_bottom_n() {
    let cmd = TableCommand::BottomN {
        dir: PathBuf::from("/tmp"),
        col: 1,
        n: 5,
        output: OutputFormat::Table,
    };
    assert!(matches!(cmd, TableCommand::BottomN { .. }));
}
#[test]
fn bash_variant_sum_field() {
    let cmd = TableCommand::SumField {
        dir: PathBuf::from("/tmp"),
        col: 1,
        output: OutputFormat::Table,
    };
    assert!(matches!(cmd, TableCommand::SumField { .. }));
}
#[test]
fn bash_variant_avg_field() {
    let cmd = TableCommand::AvgField {
        dir: PathBuf::from("/tmp"),
        col: 1,
        output: OutputFormat::Table,
    };
    assert!(matches!(cmd, TableCommand::AvgField { .. }));
}
#[test]
fn bash_variant_median_field() {
    let cmd = TableCommand::MedianField {
        dir: PathBuf::from("/tmp"),
        col: 1,
        output: OutputFormat::Table,
    };
    assert!(matches!(cmd, TableCommand::MedianField { .. }));
}
#[test]
fn bash_variant_percentile_field() {
    let cmd = TableCommand::PercentileField {
        dir: PathBuf::from("/tmp"),
        col: 1,
        pct: 50,
        output: OutputFormat::Table,
    };
    assert!(matches!(cmd, TableCommand::PercentileField { .. }));
}
#[test]
fn bash_variant_histogram() {
    let cmd = TableCommand::Histogram {
        dir: PathBuf::from("/tmp"),
        col: 1,
        bucket_size: 10,
        output: OutputFormat::Table,
    };
    assert!(matches!(cmd, TableCommand::Histogram { .. }));
}
#[test]
fn bash_variant_zscore_filter() {
    let cmd = TableCommand::ZScoreFilter {
        dir: PathBuf::from("/tmp"),
        col: 1,
        output: OutputFormat::Table,
    };
    assert!(matches!(cmd, TableCommand::ZScoreFilter { .. }));
}
#[test]
fn bash_variant_running_sum() {
    let cmd = TableCommand::RunningSum {
        dir: PathBuf::from("/tmp"),
        col: 1,
        output: OutputFormat::Table,
    };
    assert!(matches!(cmd, TableCommand::RunningSum { .. }));
}
