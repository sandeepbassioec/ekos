//! Integration tests for Layer 175: table drop --force and truncate (VV-CTL-175).

use std::path::PathBuf;
use tempfile::TempDir;
use vvctl::commands::table::{TableCommand, run};

fn make_dir() -> TempDir {
    TempDir::new().unwrap()
}
fn tbl(d: &TempDir) -> PathBuf {
    d.path().join("tbl")
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
fn drop_force_removes_dir() {
    let d = make_dir();
    create(&d);
    assert!(tbl(&d).exists());
    run(TableCommand::Drop {
        dir: tbl(&d),
        force: true,
    })
    .unwrap();
    assert!(!tbl(&d).exists());
}

#[test]
fn drop_without_force_errors() {
    let d = make_dir();
    create(&d);
    assert!(
        run(TableCommand::Drop {
            dir: tbl(&d),
            force: false
        })
        .is_err()
    );
}

#[test]
fn drop_missing_dir_errors() {
    assert!(
        run(TableCommand::Drop {
            dir: PathBuf::from("/no/such/path"),
            force: true
        })
        .is_err()
    );
}

#[test]
fn drop_after_inserts_removes_all_data() {
    let d = make_dir();
    create(&d);
    for i in 0..5u64 {
        insert(&d, i, i * 10);
    }
    run(TableCommand::Drop {
        dir: tbl(&d),
        force: true,
    })
    .unwrap();
    assert!(!tbl(&d).exists());
}

#[test]
fn truncate_clears_rows_keeps_schema() {
    let d = make_dir();
    create(&d);
    insert(&d, 1, 100);
    run(TableCommand::Truncate {
        dir: tbl(&d),
        force: true,
    })
    .unwrap();
    let schema_path = tbl(&d).join("schema.txt");
    assert!(schema_path.exists());
}

#[test]
fn truncate_empty_table_ok() {
    let d = make_dir();
    create(&d);
    assert!(
        run(TableCommand::Truncate {
            dir: tbl(&d),
            force: true
        })
        .is_ok()
    );
}

#[test]
fn truncate_missing_dir_errors() {
    assert!(
        run(TableCommand::Truncate {
            dir: PathBuf::from("/no/such/path"),
            force: true,
        })
        .is_err()
    );
}

#[test]
fn drop_double_errors() {
    let d = make_dir();
    create(&d);
    run(TableCommand::Drop {
        dir: tbl(&d),
        force: true,
    })
    .unwrap();
    assert!(
        run(TableCommand::Drop {
            dir: tbl(&d),
            force: true
        })
        .is_err()
    );
}

#[test]
fn truncate_after_many_inserts_ok() {
    let d = make_dir();
    create(&d);
    for i in 0..20u64 {
        insert(&d, i, i);
    }
    assert!(
        run(TableCommand::Truncate {
            dir: tbl(&d),
            force: true
        })
        .is_ok()
    );
}

#[test]
fn drop_recreate_ok() {
    let d = make_dir();
    create(&d);
    run(TableCommand::Drop {
        dir: tbl(&d),
        force: true,
    })
    .unwrap();
    assert!(
        run(TableCommand::Create {
            dir: tbl(&d),
            columns: vec!["pk:u64".into(), "v:u64".into()],
        })
        .is_ok()
    );
}

#[test]
fn truncate_repeated_ok() {
    let d = make_dir();
    create(&d);
    insert(&d, 1, 1);
    run(TableCommand::Truncate {
        dir: tbl(&d),
        force: true,
    })
    .unwrap();
    run(TableCommand::Truncate {
        dir: tbl(&d),
        force: true,
    })
    .unwrap();
}

#[test]
fn drop_force_true_always_ok() {
    let d = make_dir();
    create(&d);
    assert!(
        run(TableCommand::Drop {
            dir: tbl(&d),
            force: true
        })
        .is_ok()
    );
}

#[test]
fn truncate_schema_preserved_after_insert() {
    let d = make_dir();
    create(&d);
    insert(&d, 42, 99);
    run(TableCommand::Truncate {
        dir: tbl(&d),
        force: true,
    })
    .unwrap();
    assert!(
        run(TableCommand::Create {
            dir: tbl(&d).join("should_not_matter"),
            columns: vec!["pk:u64".into()],
        })
        .is_ok()
    );
}
