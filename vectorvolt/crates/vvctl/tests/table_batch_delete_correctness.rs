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
fn batch_delete_empty_pks() {
    let d = make_dir();
    setup(&d);
    assert!(
        run(TableCommand::BatchDelete {
            dir: tdir(&d),
            pks: vec![]
        })
        .is_ok()
    );
}

#[test]
fn batch_delete_single_existing() {
    let d = make_dir();
    setup(&d);
    ins(&d, 5, 50);
    assert!(
        run(TableCommand::BatchDelete {
            dir: tdir(&d),
            pks: vec![5]
        })
        .is_ok()
    );
}

#[test]
fn batch_delete_single_missing() {
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
fn batch_delete_multiple_all_exist() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 {
        ins(&d, i, i * 10);
    }
    assert!(
        run(TableCommand::BatchDelete {
            dir: tdir(&d),
            pks: vec![1, 2, 3, 4, 5]
        })
        .is_ok()
    );
}

#[test]
fn batch_delete_partial_hit() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    ins(&d, 3, 30);
    assert!(
        run(TableCommand::BatchDelete {
            dir: tdir(&d),
            pks: vec![1, 2, 3]
        })
        .is_ok()
    );
}

#[test]
fn batch_delete_then_count() {
    let d = make_dir();
    setup(&d);
    for i in 1..=6 {
        ins(&d, i, i);
    }
    run(TableCommand::BatchDelete {
        dir: tdir(&d),
        pks: vec![1, 2, 3],
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
fn batch_delete_then_scan() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 {
        ins(&d, i, i * 4);
    }
    run(TableCommand::BatchDelete {
        dir: tdir(&d),
        pks: vec![2, 4],
    })
    .unwrap();
    assert!(
        run(TableCommand::Scan {
            dir: tdir(&d),
            col: 1,
            lo: 0,
            hi: 100,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn batch_delete_after_flush() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 {
        ins(&d, i, i);
    }
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    assert!(
        run(TableCommand::BatchDelete {
            dir: tdir(&d),
            pks: vec![1, 3, 5]
        })
        .is_ok()
    );
}

#[test]
fn batch_delete_idempotent() {
    let d = make_dir();
    setup(&d);
    ins(&d, 7, 77);
    run(TableCommand::BatchDelete {
        dir: tdir(&d),
        pks: vec![7],
    })
    .unwrap();
    assert!(
        run(TableCommand::BatchDelete {
            dir: tdir(&d),
            pks: vec![7]
        })
        .is_ok()
    );
}

#[test]
fn batch_delete_duplicate_pks() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    assert!(
        run(TableCommand::BatchDelete {
            dir: tdir(&d),
            pks: vec![1, 1, 1]
        })
        .is_ok()
    );
}

#[test]
fn batch_delete_all_rows() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 {
        ins(&d, i, i * 3);
    }
    let pks: Vec<u64> = (1..=5).collect();
    run(TableCommand::BatchDelete { dir: tdir(&d), pks }).unwrap();
    assert!(
        run(TableCommand::Count {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn batch_delete_then_stats() {
    let d = make_dir();
    setup(&d);
    for i in 1..=8 {
        ins(&d, i, i * 2);
    }
    run(TableCommand::BatchDelete {
        dir: tdir(&d),
        pks: vec![2, 4, 6, 8],
    })
    .unwrap();
    assert!(
        run(TableCommand::Stats {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn batch_delete_large_batch() {
    let d = make_dir();
    setup(&d);
    for i in 1..=20 {
        ins(&d, i, i);
    }
    let pks: Vec<u64> = (1..=20).collect();
    assert!(run(TableCommand::BatchDelete { dir: tdir(&d), pks }).is_ok());
}
