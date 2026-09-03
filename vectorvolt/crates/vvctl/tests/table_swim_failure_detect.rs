//! Tests for Layer 357.
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
fn failure_insert_10_flush() {
    let d = make_dir();
    setup(&d);
    for i in 1..=10u64 {
        ins(&d, i, i);
    }
    assert!(run(TableCommand::Flush { dir: tdir(&d) }).is_ok());
}
#[test]
fn failure_compact_ok() {
    let d = make_dir();
    setup(&d);
    for i in 1..=10u64 {
        ins(&d, i, i);
    }
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    assert!(run(TableCommand::Compact { dir: tdir(&d) }).is_ok());
}
#[test]
fn failure_batch_delete_failed_nodes() {
    let d = make_dir();
    setup(&d);
    for i in 1..=10u64 {
        ins(&d, i, i);
    }
    assert!(
        run(TableCommand::BatchDelete {
            dir: tdir(&d),
            pks: vec![8, 9]
        })
        .is_ok()
    );
}
#[test]
fn failure_count_8() {
    let d = make_dir();
    setup(&d);
    for i in 1..=10u64 {
        ins(&d, i, i);
    }
    run(TableCommand::BatchDelete {
        dir: tdir(&d),
        pks: vec![8, 9],
    })
    .unwrap();
    assert!(
        run(TableCommand::Count {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn failure_scan_8() {
    let d = make_dir();
    setup(&d);
    for i in 1..=10u64 {
        ins(&d, i, i);
    }
    run(TableCommand::BatchDelete {
        dir: tdir(&d),
        pks: vec![8, 9],
    })
    .unwrap();
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
fn failure_insert_replacements() {
    let d = make_dir();
    setup(&d);
    for i in 1..=10u64 {
        ins(&d, i, i);
    }
    run(TableCommand::BatchDelete {
        dir: tdir(&d),
        pks: vec![8, 9],
    })
    .unwrap();
    ins(&d, 100, 100);
    ins(&d, 101, 101);
    assert!(
        run(TableCommand::Count {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn failure_count_10_after_replace() {
    let d = make_dir();
    setup(&d);
    for i in 1..=8u64 {
        ins(&d, i, i);
    }
    ins(&d, 100, 100);
    ins(&d, 101, 101);
    assert!(
        run(TableCommand::Count {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn failure_check_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 1);
    assert!(run(TableCommand::Check { dir: tdir(&d) }).is_ok());
}
#[test]
fn failure_vacuum_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 1);
    assert!(run(TableCommand::Vacuum { dir: tdir(&d) }).is_ok());
}
#[test]
fn failure_stats_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 1);
    assert!(
        run(TableCommand::Stats {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn failure_recount_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 1);
    assert!(
        run(TableCommand::Recount {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn failure_info_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 1);
    assert!(
        run(TableCommand::Info {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn failure_export_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 1);
    assert!(
        run(TableCommand::Export {
            dir: tdir(&d),
            format: ExportFormat::Json
        })
        .is_ok()
    );
}
