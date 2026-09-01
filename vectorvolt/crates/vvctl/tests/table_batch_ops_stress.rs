//! Tests for Layer 281: vvctl table batch ops under load (VV-CTL-281).

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
    for i in 1..=10u64 {
        run(TableCommand::Insert {
            dir: tdir(d),
            pk: i,
            values: vec![i * 10],
        })
        .unwrap();
    }
}

#[test]
fn batch_get_10_pks_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::BatchGet {
        dir: tdir(&d),
        pks: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn batch_get_mix_found_not_found_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::BatchGet {
        dir: tdir(&d),
        pks: vec![1, 99, 2, 100],
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn batch_get_single_pk_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::BatchGet {
        dir: tdir(&d),
        pks: vec![5],
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn batch_delete_5_pks_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::BatchDelete {
        dir: tdir(&d),
        pks: vec![1, 2, 3, 4, 5],
    })
    .unwrap();
}

#[test]
fn batch_delete_nonexistent_ok() {
    let d = make_dir();
    setup(&d);
    // May succeed or error; must not panic
    let _ = run(TableCommand::BatchDelete {
        dir: tdir(&d),
        pks: vec![99, 100],
    });
}

#[test]
fn batch_delete_then_count_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::BatchDelete {
        dir: tdir(&d),
        pks: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
    })
    .unwrap();
    run(TableCommand::Count {
        dir: tdir(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn batch_get_then_delete_then_get_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::BatchGet {
        dir: tdir(&d),
        pks: vec![1, 2, 3],
        output: OutputFormat::Table,
    })
    .unwrap();
    run(TableCommand::BatchDelete {
        dir: tdir(&d),
        pks: vec![1, 2, 3],
    })
    .unwrap();
    run(TableCommand::BatchGet {
        dir: tdir(&d),
        pks: vec![1, 2, 3],
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn batch_get_json_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::BatchGet {
        dir: tdir(&d),
        pks: vec![1, 2, 3],
        output: OutputFormat::Json,
    })
    .unwrap();
}

#[test]
fn batch_delete_then_insert_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::BatchDelete {
        dir: tdir(&d),
        pks: vec![1, 2],
    })
    .unwrap();
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 20,
        values: vec![200],
    })
    .unwrap();
}

#[test]
fn batch_get_all_pks_present() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::BatchGet {
        dir: tdir(&d),
        pks: vec![1, 3, 5, 7, 9],
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn batch_delete_idempotent() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::BatchDelete {
        dir: tdir(&d),
        pks: vec![1, 2, 3],
    })
    .unwrap();
    // Second delete of same pks: may err or ok, must not panic
    let _ = run(TableCommand::BatchDelete {
        dir: tdir(&d),
        pks: vec![1, 2, 3],
    });
}

#[test]
fn batch_after_flush_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    run(TableCommand::BatchGet {
        dir: tdir(&d),
        pks: vec![1, 5, 10],
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn batch_after_compact_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    run(TableCommand::Compact { dir: tdir(&d) }).unwrap();
    run(TableCommand::BatchGet {
        dir: tdir(&d),
        pks: vec![2, 4, 6],
        output: OutputFormat::Table,
    })
    .unwrap();
}
