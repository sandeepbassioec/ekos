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
fn batch_get_empty_pks() {
    let d = make_dir();
    setup(&d);
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
fn batch_get_single_pk_exists() {
    let d = make_dir();
    setup(&d);
    ins(&d, 5, 50);
    assert!(
        run(TableCommand::BatchGet {
            dir: tdir(&d),
            pks: vec![5],
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn batch_get_single_pk_missing() {
    let d = make_dir();
    setup(&d);
    assert!(
        run(TableCommand::BatchGet {
            dir: tdir(&d),
            pks: vec![999],
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn batch_get_multiple_all_exist() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 {
        ins(&d, i, i * 10);
    }
    assert!(
        run(TableCommand::BatchGet {
            dir: tdir(&d),
            pks: vec![1, 2, 3, 4, 5],
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn batch_get_partial_hit() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    ins(&d, 3, 30);
    assert!(
        run(TableCommand::BatchGet {
            dir: tdir(&d),
            pks: vec![1, 2, 3],
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn batch_get_all_missing() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 {
        ins(&d, i, i);
    }
    assert!(
        run(TableCommand::BatchGet {
            dir: tdir(&d),
            pks: vec![100, 200, 300],
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn batch_get_json_output() {
    let d = make_dir();
    setup(&d);
    for i in 1..=3 {
        ins(&d, i, i * 5);
    }
    assert!(
        run(TableCommand::BatchGet {
            dir: tdir(&d),
            pks: vec![1, 2, 3],
            output: OutputFormat::Json
        })
        .is_ok()
    );
}

#[test]
fn batch_get_after_flush() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 {
        ins(&d, i, i);
    }
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
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
fn batch_get_after_compact() {
    let d = make_dir();
    setup(&d);
    for i in 1..=8 {
        ins(&d, i, i * 3);
    }
    run(TableCommand::Compact { dir: tdir(&d) }).unwrap();
    assert!(
        run(TableCommand::BatchGet {
            dir: tdir(&d),
            pks: vec![2, 4, 6, 8],
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn batch_get_after_delete() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 {
        ins(&d, i, i * 2);
    }
    run(TableCommand::Delete {
        dir: tdir(&d),
        pk: 3,
    })
    .unwrap();
    assert!(
        run(TableCommand::BatchGet {
            dir: tdir(&d),
            pks: vec![1, 2, 3, 4, 5],
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn batch_get_duplicate_pks() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
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
fn batch_get_large_batch() {
    let d = make_dir();
    setup(&d);
    for i in 1..=20 {
        ins(&d, i, i * 4);
    }
    let pks: Vec<u64> = (1..=20).collect();
    assert!(
        run(TableCommand::BatchGet {
            dir: tdir(&d),
            pks,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn batch_get_unsorted_pks() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 {
        ins(&d, i, i * 6);
    }
    assert!(
        run(TableCommand::BatchGet {
            dir: tdir(&d),
            pks: vec![5, 3, 1, 4, 2],
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
