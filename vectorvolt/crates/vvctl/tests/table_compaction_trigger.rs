//! Tests for Layer 321: Compaction trigger correctness (VV-CTL-321).

use std::path::PathBuf;
use tempfile::TempDir;
use vvctl::commands::table::{ExportFormat, OutputFormat, TableCommand, run};

fn make_dir() -> TempDir {
    TempDir::new().unwrap()
}
fn tdir(d: &TempDir) -> PathBuf {
    d.path().join("tbl")
}
fn setup(d: &TempDir) {
    run(TableCommand::Create {
        dir: tdir(d),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
}
fn ins(d: &TempDir, pk: u64, val: u64) {
    run(TableCommand::Insert {
        dir: tdir(d),
        pk,
        values: vec![val],
    })
    .unwrap();
}

#[test]
fn compact_after_flush_ok() {
    let d = make_dir();
    setup(&d);
    for i in 0..5 {
        ins(&d, i, i * 10);
    }
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    assert!(run(TableCommand::Compact { dir: tdir(&d) }).is_ok());
}

#[test]
fn compact_stats_valid() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 100);
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    run(TableCommand::Compact { dir: tdir(&d) }).unwrap();
    assert!(
        run(TableCommand::Stats {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn compact_then_check_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    run(TableCommand::Compact { dir: tdir(&d) }).unwrap();
    assert!(run(TableCommand::Check { dir: tdir(&d) }).is_ok());
}

#[test]
fn compact_then_scan_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    ins(&d, 2, 20);
    run(TableCommand::Compact { dir: tdir(&d) }).unwrap();
    assert!(
        run(TableCommand::Scan {
            dir: tdir(&d),
            col: 0,
            lo: 0,
            hi: u64::MAX,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn compact_then_vacuum_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    run(TableCommand::Compact { dir: tdir(&d) }).unwrap();
    assert!(run(TableCommand::Vacuum { dir: tdir(&d) }).is_ok());
}

#[test]
fn compact_then_info_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    run(TableCommand::Compact { dir: tdir(&d) }).unwrap();
    assert!(
        run(TableCommand::Info {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn compact_then_stats_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 2, 20);
    run(TableCommand::Compact { dir: tdir(&d) }).unwrap();
    assert!(
        run(TableCommand::Stats {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn compact_twice_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    run(TableCommand::Compact { dir: tdir(&d) }).unwrap();
    assert!(run(TableCommand::Compact { dir: tdir(&d) }).is_ok());
}

#[test]
fn compact_empty_table_ok() {
    let d = make_dir();
    setup(&d);
    assert!(run(TableCommand::Compact { dir: tdir(&d) }).is_ok());
}

#[test]
fn compact_missing_dir_error() {
    assert!(
        run(TableCommand::Compact {
            dir: PathBuf::from("/nonexistent/path")
        })
        .is_err()
    );
}

#[test]
fn compact_then_recount_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    run(TableCommand::Compact { dir: tdir(&d) }).unwrap();
    assert!(
        run(TableCommand::Recount {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn compact_then_export_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    run(TableCommand::Compact { dir: tdir(&d) }).unwrap();
    assert!(
        run(TableCommand::Export {
            dir: tdir(&d),
            format: ExportFormat::Json
        })
        .is_ok()
    );
}

#[test]
fn compact_then_pkrange_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 5, 50);
    run(TableCommand::Compact { dir: tdir(&d) }).unwrap();
    assert!(
        run(TableCommand::PkRange {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
