//! Tests for Layer 218: vvctl table batch-get (VV-CTL-218).

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
fn batch_get_single_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    assert!(
        run(TableCommand::BatchGet {
            dir: tdir(&d),
            pks: vec![1],
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn batch_get_multiple_ok() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5u64 {
        ins(&d, i, i * 10);
    }
    assert!(
        run(TableCommand::BatchGet {
            dir: tdir(&d),
            pks: vec![1, 3, 5],
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn batch_get_missing_pks_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    assert!(
        run(TableCommand::BatchGet {
            dir: tdir(&d),
            pks: vec![1, 99, 100],
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn batch_get_json_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 42);
    assert!(
        run(TableCommand::BatchGet {
            dir: tdir(&d),
            pks: vec![1],
            output: OutputFormat::Json
        })
        .is_ok()
    );
}
#[test]
fn batch_get_all_missing_ok() {
    let d = make_dir();
    setup(&d);
    assert!(
        run(TableCommand::BatchGet {
            dir: tdir(&d),
            pks: vec![10, 20, 30],
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn batch_get_missing_dir_errors() {
    let d = make_dir();
    assert!(
        run(TableCommand::BatchGet {
            dir: d.path().join("nope"),
            pks: vec![1],
            output: OutputFormat::Table
        })
        .is_err()
    );
}
#[test]
fn batch_get_ten_pks_ok() {
    let d = make_dir();
    setup(&d);
    for i in 0..10u64 {
        ins(&d, i, i * 7);
    }
    assert!(
        run(TableCommand::BatchGet {
            dir: tdir(&d),
            pks: (0..10).collect(),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn batch_get_json_multiple() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 5);
    ins(&d, 2, 10);
    assert!(
        run(TableCommand::BatchGet {
            dir: tdir(&d),
            pks: vec![1, 2],
            output: OutputFormat::Json
        })
        .is_ok()
    );
}
#[test]
fn batch_get_pks_stored() {
    let cmd = TableCommand::BatchGet {
        dir: PathBuf::from("/tmp"),
        pks: vec![1, 2, 3],
        output: OutputFormat::Table,
    };
    if let TableCommand::BatchGet { pks, .. } = cmd {
        assert_eq!(pks, vec![1, 2, 3]);
    } else {
        panic!();
    }
}
#[test]
fn batch_get_output_stored() {
    let cmd = TableCommand::BatchGet {
        dir: PathBuf::from("/tmp"),
        pks: vec![1],
        output: OutputFormat::Json,
    };
    if let TableCommand::BatchGet { output, .. } = cmd {
        assert!(matches!(output, OutputFormat::Json));
    } else {
        panic!();
    }
}
#[test]
fn batch_get_empty_pks_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 1);
    assert!(
        run(TableCommand::BatchGet {
            dir: tdir(&d),
            pks: vec![],
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn batch_get_duplicate_pks_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 99);
    assert!(
        run(TableCommand::BatchGet {
            dir: tdir(&d),
            pks: vec![1, 1, 1],
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn batch_get_five_rows_json() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5u64 {
        ins(&d, i, i * 3);
    }
    assert!(
        run(TableCommand::BatchGet {
            dir: tdir(&d),
            pks: vec![1, 2, 3, 4, 5],
            output: OutputFormat::Json
        })
        .is_ok()
    );
}
