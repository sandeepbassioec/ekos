//! Tests for Layer 219: vvctl table batch-delete (VV-CTL-219).

use std::path::PathBuf;
use tempfile::TempDir;
use vvctl::commands::table::{TableCommand, run};

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
fn batch_delete_single_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    assert!(
        run(TableCommand::BatchDelete {
            dir: tdir(&d),
            pks: vec![1]
        })
        .is_ok()
    );
}
#[test]
fn batch_delete_multiple_ok() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5u64 {
        ins(&d, i, i * 10);
    }
    assert!(
        run(TableCommand::BatchDelete {
            dir: tdir(&d),
            pks: vec![1, 3, 5]
        })
        .is_ok()
    );
}
#[test]
fn batch_delete_missing_pks_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    assert!(
        run(TableCommand::BatchDelete {
            dir: tdir(&d),
            pks: vec![1, 99]
        })
        .is_ok()
    );
}
#[test]
fn batch_delete_all_ok() {
    let d = make_dir();
    setup(&d);
    for i in 1..=3u64 {
        ins(&d, i, i);
    }
    assert!(
        run(TableCommand::BatchDelete {
            dir: tdir(&d),
            pks: vec![1, 2, 3]
        })
        .is_ok()
    );
}
#[test]
fn batch_delete_missing_dir_errors() {
    let d = make_dir();
    assert!(
        run(TableCommand::BatchDelete {
            dir: d.path().join("nope"),
            pks: vec![1]
        })
        .is_err()
    );
}
#[test]
fn batch_delete_empty_pks_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 1);
    assert!(
        run(TableCommand::BatchDelete {
            dir: tdir(&d),
            pks: vec![]
        })
        .is_ok()
    );
}
#[test]
fn batch_delete_ten_pks_ok() {
    let d = make_dir();
    setup(&d);
    for i in 0..10u64 {
        ins(&d, i, i);
    }
    assert!(
        run(TableCommand::BatchDelete {
            dir: tdir(&d),
            pks: (0..10).collect()
        })
        .is_ok()
    );
}
#[test]
fn batch_delete_then_verify_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    ins(&d, 2, 20);
    run(TableCommand::BatchDelete {
        dir: tdir(&d),
        pks: vec![1],
    })
    .unwrap();
    use vvctl::commands::table::OutputFormat;
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
fn batch_delete_pks_stored() {
    let cmd = TableCommand::BatchDelete {
        dir: PathBuf::from("/tmp"),
        pks: vec![1, 2, 3],
    };
    if let TableCommand::BatchDelete { pks, .. } = cmd {
        assert_eq!(pks, vec![1, 2, 3]);
    } else {
        panic!();
    }
}
#[test]
fn batch_delete_idempotent_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 5);
    run(TableCommand::BatchDelete {
        dir: tdir(&d),
        pks: vec![1],
    })
    .unwrap();
    assert!(
        run(TableCommand::BatchDelete {
            dir: tdir(&d),
            pks: vec![1]
        })
        .is_ok()
    );
}
#[test]
fn batch_delete_nonexistent_ok() {
    let d = make_dir();
    setup(&d);
    assert!(
        run(TableCommand::BatchDelete {
            dir: tdir(&d),
            pks: vec![999]
        })
        .is_ok()
    );
}
#[test]
fn batch_delete_five_rows_ok() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5u64 {
        ins(&d, i, i * 2);
    }
    assert!(
        run(TableCommand::BatchDelete {
            dir: tdir(&d),
            pks: vec![2, 4]
        })
        .is_ok()
    );
}
#[test]
fn batch_delete_dir_stored() {
    let dir = PathBuf::from("/some/path");
    let cmd = TableCommand::BatchDelete {
        dir: dir.clone(),
        pks: vec![1],
    };
    if let TableCommand::BatchDelete { dir: d, .. } = cmd {
        assert_eq!(d, dir);
    } else {
        panic!();
    }
}
