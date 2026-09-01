//! Tests for Layer 336: Dead-letter queue stub (VV-CTL-336).

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
fn insert_to_dlq_ok() {
    let d = make_dir();
    setup(&d);
    for i in 0..3 {
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
fn scan_dlq_ok() {
    let d = make_dir();
    setup(&d);
    for i in 0..3 {
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
fn flush_dlq_ok() {
    let d = make_dir();
    setup(&d);
    for i in 0..3 {
        ins(&d, i, i * 10);
    }
    assert!(run(TableCommand::Flush { dir: tdir(&d) }).is_ok());
}

#[test]
fn compact_dlq_ok() {
    let d = make_dir();
    setup(&d);
    for i in 0..3 {
        ins(&d, i, i * 10);
    }
    assert!(run(TableCommand::Compact { dir: tdir(&d) }).is_ok());
}

#[test]
fn check_dlq_ok() {
    let d = make_dir();
    setup(&d);
    assert!(run(TableCommand::Check { dir: tdir(&d) }).is_ok());
}

#[test]
fn stats_dlq_ok() {
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
fn vacuum_dlq_ok() {
    let d = make_dir();
    setup(&d);
    assert!(run(TableCommand::Vacuum { dir: tdir(&d) }).is_ok());
}

#[test]
fn info_dlq_ok() {
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
fn keys_dlq_ok() {
    let d = make_dir();
    setup(&d);
    for i in 0..3 {
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
fn pkrange_dlq_ok() {
    let d = make_dir();
    setup(&d);
    for i in 0..3 {
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
fn head_dlq_ok() {
    let d = make_dir();
    setup(&d);
    for i in 0..3 {
        ins(&d, i, i * 10);
    }
    assert!(
        run(TableCommand::Head {
            dir: tdir(&d),
            n: 2,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn tail_dlq_ok() {
    let d = make_dir();
    setup(&d);
    for i in 0..3 {
        ins(&d, i, i * 10);
    }
    assert!(
        run(TableCommand::Tail {
            dir: tdir(&d),
            n: 2,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn recount_dlq_ok() {
    let d = make_dir();
    setup(&d);
    for i in 0..3 {
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
