//! Tests for Layer 398.
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
        columns: vec!["pk:u64".into(), "fk:u64".into()],
    })
    .unwrap();
}
fn ins(d: &TempDir, pk: u64, fk: u64) {
    run(TableCommand::Insert {
        dir: tdir(d),
        pk,
        values: vec![fk],
    })
    .unwrap();
}

#[test]
fn cascade_insert_parent() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 0);
    assert!(
        run(TableCommand::Get {
            dir: tdir(&d),
            pk: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn cascade_insert_children_fk1() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 0);
    ins(&d, 10, 1);
    ins(&d, 11, 1);
    ins(&d, 12, 1);
    assert!(
        run(TableCommand::Count {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn cascade_insert_children_fk2() {
    let d = make_dir();
    setup(&d);
    ins(&d, 2, 0);
    ins(&d, 20, 2);
    ins(&d, 21, 2);
    assert!(
        run(TableCommand::Count {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn cascade_delete_parent1() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 0);
    assert!(
        run(TableCommand::Delete {
            dir: tdir(&d),
            pk: 1
        })
        .is_ok()
    );
}
#[test]
fn cascade_batch_delete_children() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 0);
    ins(&d, 10, 1);
    ins(&d, 11, 1);
    ins(&d, 12, 1);
    assert!(
        run(TableCommand::BatchDelete {
            dir: tdir(&d),
            pks: vec![10, 11, 12]
        })
        .is_ok()
    );
}
#[test]
fn cascade_count_3_after_cascade() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 0);
    ins(&d, 2, 0);
    ins(&d, 10, 1);
    ins(&d, 11, 1);
    ins(&d, 12, 1);
    ins(&d, 20, 2);
    ins(&d, 21, 2);
    run(TableCommand::Delete {
        dir: tdir(&d),
        pk: 1,
    })
    .unwrap();
    run(TableCommand::BatchDelete {
        dir: tdir(&d),
        pks: vec![10, 11, 12],
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
fn cascade_scan_3() {
    let d = make_dir();
    setup(&d);
    ins(&d, 2, 0);
    ins(&d, 20, 2);
    ins(&d, 21, 2);
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
fn cascade_check_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 0);
    assert!(run(TableCommand::Check { dir: tdir(&d) }).is_ok());
}
#[test]
fn cascade_vacuum_ok() {
    let d = make_dir();
    setup(&d);
    assert!(run(TableCommand::Vacuum { dir: tdir(&d) }).is_ok());
}
#[test]
fn cascade_stats_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 0);
    assert!(
        run(TableCommand::Stats {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn cascade_recount_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 0);
    assert!(
        run(TableCommand::Recount {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn cascade_compact_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 0);
    assert!(run(TableCommand::Compact { dir: tdir(&d) }).is_ok());
}
#[test]
fn cascade_export_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 0);
    assert!(
        run(TableCommand::Export {
            dir: tdir(&d),
            format: ExportFormat::Json
        })
        .is_ok()
    );
}
