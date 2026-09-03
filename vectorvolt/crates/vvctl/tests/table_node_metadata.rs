//! Tests for Layer 364.
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
        columns: vec!["pk:u64".into(), "ts:u64".into()],
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
fn node_meta_insert_5() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5u64 {
        ins(&d, i, i * 1000);
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
fn node_meta_get_each_ok() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5u64 {
        ins(&d, i, i * 1000);
    }
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
fn node_meta_scan_all() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5u64 {
        ins(&d, i, i * 1000);
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
fn node_meta_update_pk1() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 1000);
    assert!(
        run(TableCommand::Update {
            dir: tdir(&d),
            pk: 1,
            col: 1,
            val: 9999
        })
        .is_ok()
    );
}
#[test]
fn node_meta_delete_pk5() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5u64 {
        ins(&d, i, i * 1000);
    }
    assert!(
        run(TableCommand::Delete {
            dir: tdir(&d),
            pk: 5
        })
        .is_ok()
    );
}
#[test]
fn node_meta_count_4() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5u64 {
        ins(&d, i, i * 1000);
    }
    run(TableCommand::Delete {
        dir: tdir(&d),
        pk: 5,
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
fn node_meta_batch_get_ok() {
    let d = make_dir();
    setup(&d);
    for i in 1..=4u64 {
        ins(&d, i, i * 1000);
    }
    assert!(
        run(TableCommand::BatchGet {
            dir: tdir(&d),
            pks: vec![1, 2, 3, 4],
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn node_meta_export_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 1000);
    assert!(
        run(TableCommand::Export {
            dir: tdir(&d),
            format: ExportFormat::Json
        })
        .is_ok()
    );
}
#[test]
fn node_meta_check_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 1000);
    assert!(run(TableCommand::Check { dir: tdir(&d) }).is_ok());
}
#[test]
fn node_meta_recount_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 1000);
    assert!(
        run(TableCommand::Recount {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn node_meta_stats_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 1000);
    assert!(
        run(TableCommand::Stats {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn node_meta_info_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 1000);
    assert!(
        run(TableCommand::Info {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn node_meta_pk_range_ok() {
    let d = make_dir();
    setup(&d);
    for i in 1..=4u64 {
        ins(&d, i, i * 1000);
    }
    assert!(
        run(TableCommand::PkRange {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
