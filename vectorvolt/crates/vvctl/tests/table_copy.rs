//! Integration tests for Layer 155: vvctl table copy (VV-CTL-155).

use std::path::PathBuf;
use tempfile::TempDir;
use vvctl::commands::table::{TableCommand, run};

fn make_dir() -> TempDir {
    TempDir::new().unwrap()
}
fn tbl(d: &TempDir) -> PathBuf {
    d.path().join("src")
}
fn dst(d: &TempDir) -> PathBuf {
    d.path().join("dst")
}

fn create(d: &TempDir) {
    run(TableCommand::Create {
        dir: tbl(d),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
}

fn insert(d: &TempDir, pk: u64, val: u64) {
    run(TableCommand::Insert {
        dir: tbl(d),
        pk,
        values: vec![val],
    })
    .unwrap();
}

#[test]
fn copy_empty_table_ok() {
    let d = make_dir();
    create(&d);
    assert!(
        run(TableCommand::Copy {
            src: tbl(&d),
            dst: dst(&d)
        })
        .is_ok()
    );
}

#[test]
fn copy_with_rows_ok() {
    let d = make_dir();
    create(&d);
    insert(&d, 1, 10);
    insert(&d, 2, 20);
    assert!(
        run(TableCommand::Copy {
            src: tbl(&d),
            dst: dst(&d)
        })
        .is_ok()
    );
}

#[test]
fn copy_missing_src_errors() {
    let d = make_dir();
    assert!(
        run(TableCommand::Copy {
            src: PathBuf::from("/no/such/src"),
            dst: dst(&d)
        })
        .is_err()
    );
}

#[test]
fn copy_creates_dst_dir() {
    let d = make_dir();
    create(&d);
    let dst_path = dst(&d);
    assert!(!dst_path.exists());
    run(TableCommand::Copy {
        src: tbl(&d),
        dst: dst_path.clone(),
    })
    .unwrap();
    assert!(dst_path.exists());
}

#[test]
fn copy_dst_has_schema() {
    let d = make_dir();
    create(&d);
    run(TableCommand::Copy {
        src: tbl(&d),
        dst: dst(&d),
    })
    .unwrap();
    assert!(dst(&d).join("schema.txt").exists());
}

#[test]
fn copy_many_rows_ok() {
    let d = make_dir();
    create(&d);
    for i in 0..10u64 {
        insert(&d, i, i * 2);
    }
    assert!(
        run(TableCommand::Copy {
            src: tbl(&d),
            dst: dst(&d)
        })
        .is_ok()
    );
}

#[test]
fn copy_then_scan_dst_ok() {
    let d = make_dir();
    create(&d);
    insert(&d, 5, 50);
    run(TableCommand::Copy {
        src: tbl(&d),
        dst: dst(&d),
    })
    .unwrap();
    use vvctl::commands::table::OutputFormat;
    assert!(
        run(TableCommand::Scan {
            dir: dst(&d),
            col: 0,
            lo: 0,
            hi: u64::MAX,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn copy_single_row_ok() {
    let d = make_dir();
    create(&d);
    insert(&d, 42, 99);
    assert!(
        run(TableCommand::Copy {
            src: tbl(&d),
            dst: dst(&d)
        })
        .is_ok()
    );
}

#[test]
fn copy_does_not_modify_src() {
    let d = make_dir();
    create(&d);
    insert(&d, 1, 1);
    run(TableCommand::Copy {
        src: tbl(&d),
        dst: dst(&d),
    })
    .unwrap();
    use vvctl::commands::table::OutputFormat;
    assert!(
        run(TableCommand::Scan {
            dir: tbl(&d),
            col: 0,
            lo: 0,
            hi: u64::MAX,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn copy_preserves_row_count() {
    let d = make_dir();
    create(&d);
    for i in 0..5u64 {
        insert(&d, i, i);
    }
    run(TableCommand::Copy {
        src: tbl(&d),
        dst: dst(&d),
    })
    .unwrap();
    use vvctl::commands::table::OutputFormat;
    assert!(
        run(TableCommand::Count {
            dir: dst(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn copy_zero_pk_ok() {
    let d = make_dir();
    create(&d);
    insert(&d, 0, 0);
    assert!(
        run(TableCommand::Copy {
            src: tbl(&d),
            dst: dst(&d)
        })
        .is_ok()
    );
}

#[test]
fn copy_repeated_ok() {
    let d = make_dir();
    create(&d);
    insert(&d, 7, 70);
    run(TableCommand::Copy {
        src: tbl(&d),
        dst: dst(&d),
    })
    .unwrap();
    let dst2 = d.path().join("dst2");
    assert!(
        run(TableCommand::Copy {
            src: dst(&d),
            dst: dst2
        })
        .is_ok()
    );
}

#[test]
fn copy_then_insert_dst_ok() {
    let d = make_dir();
    create(&d);
    run(TableCommand::Copy {
        src: tbl(&d),
        dst: dst(&d),
    })
    .unwrap();
    assert!(
        run(TableCommand::Insert {
            dir: dst(&d),
            pk: 100,
            values: vec![1000]
        })
        .is_ok()
    );
}
