//! Tests for Layer 345: Cluster leave stub (VV-CTL-345).

use std::path::PathBuf;
use tempfile::TempDir;
use vvctl::commands::table::{OutputFormat, TableCommand, run};

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
fn leave_create_ok() {
    let d = make_dir();
    setup(&d);
    assert!(
        run(TableCommand::Count {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn leave_insert_ok() {
    let d = make_dir();
    setup(&d);
    for i in 0..5 {
        ins(&d, i, i * 10);
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
fn leave_flush_ok() {
    let d = make_dir();
    setup(&d);
    assert!(run(TableCommand::Flush { dir: tdir(&d) }).is_ok());
}

#[test]
fn leave_compact_ok() {
    let d = make_dir();
    setup(&d);
    assert!(run(TableCommand::Compact { dir: tdir(&d) }).is_ok());
}

#[test]
fn leave_scan_ok() {
    let d = make_dir();
    setup(&d);
    for i in 0..5 {
        ins(&d, i, i * 10);
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
fn leave_check_ok() {
    let d = make_dir();
    setup(&d);
    assert!(run(TableCommand::Check { dir: tdir(&d) }).is_ok());
}

#[test]
fn leave_stats_ok() {
    let d = make_dir();
    setup(&d);
    assert!(
        run(TableCommand::Stats {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn leave_info_ok() {
    let d = make_dir();
    setup(&d);
    assert!(
        run(TableCommand::Info {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn leave_vacuum_ok() {
    let d = make_dir();
    setup(&d);
    assert!(run(TableCommand::Vacuum { dir: tdir(&d) }).is_ok());
}

#[test]
fn leave_keys_ok() {
    let d = make_dir();
    setup(&d);
    for i in 0..5 {
        ins(&d, i, i * 10);
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
fn leave_head_ok() {
    let d = make_dir();
    setup(&d);
    for i in 0..5 {
        ins(&d, i, i * 10);
    }
    assert!(
        run(TableCommand::Head {
            dir: tdir(&d),
            n: 3,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn leave_pkrange_ok() {
    let d = make_dir();
    setup(&d);
    for i in 0..5 {
        ins(&d, i, i * 10);
    }
    assert!(
        run(TableCommand::PkRange {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn leave_recount_ok() {
    let d = make_dir();
    setup(&d);
    for i in 0..5 {
        ins(&d, i, i * 10);
    }
    assert!(
        run(TableCommand::Recount {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
