//! Tests for Layer 354.
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
fn log_insert_13_entries() {
    let d = make_dir();
    setup(&d);
    for i in 1..=13u64 {
        ins(&d, i, i);
    }
    assert!(
        run(TableCommand::Count {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn log_flush_commit() {
    let d = make_dir();
    setup(&d);
    for i in 1..=13u64 {
        ins(&d, i, i);
    }
    assert!(run(TableCommand::Flush { dir: tdir(&d) }).is_ok());
}
#[test]
fn log_compact_checkpoint() {
    let d = make_dir();
    setup(&d);
    for i in 1..=13u64 {
        ins(&d, i, i);
    }
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    assert!(run(TableCommand::Compact { dir: tdir(&d) }).is_ok());
}
#[test]
fn log_scan_13() {
    let d = make_dir();
    setup(&d);
    for i in 1..=13u64 {
        ins(&d, i, i);
    }
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
fn log_count_13() {
    let d = make_dir();
    setup(&d);
    for i in 1..=13u64 {
        ins(&d, i, i);
    }
    assert!(
        run(TableCommand::Count {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn log_recount_ok() {
    let d = make_dir();
    setup(&d);
    for i in 1..=13u64 {
        ins(&d, i, i);
    }
    assert!(
        run(TableCommand::Recount {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn log_stats_ok() {
    let d = make_dir();
    setup(&d);
    for i in 1..=13u64 {
        ins(&d, i, i);
    }
    assert!(
        run(TableCommand::Stats {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn log_check_ok() {
    let d = make_dir();
    setup(&d);
    for i in 1..=13u64 {
        ins(&d, i, i);
    }
    assert!(run(TableCommand::Check { dir: tdir(&d) }).is_ok());
}
#[test]
fn log_info_ok() {
    let d = make_dir();
    setup(&d);
    for i in 1..=13u64 {
        ins(&d, i, i);
    }
    assert!(
        run(TableCommand::Info {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn log_vacuum_ok() {
    let d = make_dir();
    setup(&d);
    for i in 1..=13u64 {
        ins(&d, i, i);
    }
    assert!(run(TableCommand::Vacuum { dir: tdir(&d) }).is_ok());
}
#[test]
fn log_export_ok() {
    let d = make_dir();
    setup(&d);
    for i in 1..=13u64 {
        ins(&d, i, i);
    }
    assert!(
        run(TableCommand::Export {
            dir: tdir(&d),
            format: ExportFormat::Json
        })
        .is_ok()
    );
}
#[test]
fn log_keys_13() {
    let d = make_dir();
    setup(&d);
    for i in 1..=13u64 {
        ins(&d, i, i);
    }
    assert!(
        run(TableCommand::Keys {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn log_pk_range_ok() {
    let d = make_dir();
    setup(&d);
    for i in 1..=13u64 {
        ins(&d, i, i);
    }
    assert!(
        run(TableCommand::PkRange {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
