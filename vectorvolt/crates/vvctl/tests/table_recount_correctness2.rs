//! Tests for Layer 295: Recount deep correctness (VV-CTL-295).

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
fn recount_fresh_ok() {
    let d = make_dir();
    setup(&d);
    assert!(
        run(TableCommand::Recount {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn recount_after_5_inserts_ok() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 {
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

#[test]
fn recount_after_flush_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    assert!(
        run(TableCommand::Recount {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn recount_json_ok() {
    let d = make_dir();
    setup(&d);
    assert!(
        run(TableCommand::Recount {
            dir: tdir(&d),
            output: OutputFormat::Json
        })
        .is_ok()
    );
}

#[test]
fn recount_table_format_ok() {
    let d = make_dir();
    setup(&d);
    assert!(
        run(TableCommand::Recount {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn recount_missing_dir_errors() {
    let d = make_dir();
    let bad = d.path().join("no_such");
    assert!(
        run(TableCommand::Recount {
            dir: bad,
            output: OutputFormat::Table
        })
        .is_err()
    );
}

#[test]
fn recount_after_compact_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
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
fn recount_after_vacuum_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    run(TableCommand::Vacuum { dir: tdir(&d) }).unwrap();
    assert!(
        run(TableCommand::Recount {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn recount_after_truncate_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    run(TableCommand::Truncate {
        dir: tdir(&d),
        force: true,
    })
    .unwrap();
    assert!(
        run(TableCommand::Recount {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn recount_after_insert_delete_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    run(TableCommand::Delete {
        dir: tdir(&d),
        pk: 1,
    })
    .unwrap();
    assert!(
        run(TableCommand::Recount {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn recount_after_batch_delete_ok() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 {
        ins(&d, i, i * 10);
    }
    run(TableCommand::BatchDelete {
        dir: tdir(&d),
        pks: vec![1, 2, 3],
    })
    .unwrap();
    assert!(
        run(TableCommand::Recount {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn recount_after_upsert_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::UpsertRow {
        dir: tdir(&d),
        pk: 99,
        values: vec![99],
    })
    .unwrap();
    assert!(
        run(TableCommand::Recount {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn recount_after_copy_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    let dst = d.path().join("dst");
    run(TableCommand::Copy {
        src: tdir(&d),
        dst: dst.clone(),
    })
    .unwrap();
    assert!(
        run(TableCommand::Recount {
            dir: dst,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
