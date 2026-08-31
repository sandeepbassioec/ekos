//! Layer 175 — vvctl table drop / truncate with force flag

use std::path::PathBuf;
use tempfile::TempDir;
use vvctl::commands::table::{TableCommand, run};

fn make_dir() -> TempDir {
    TempDir::new().unwrap()
}

fn tbl(dir: &TempDir) -> PathBuf {
    dir.path().join("t")
}

fn setup(dir: &TempDir) {
    run(TableCommand::Create {
        dir: tbl(dir),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
}

// 1. drop with force=true returns Ok
#[test]
fn drop_force_true_returns_ok() {
    let dir = make_dir();
    setup(&dir);
    assert!(
        run(TableCommand::Drop {
            dir: tbl(&dir),
            force: true
        })
        .is_ok()
    );
}

// 2. drop with force=false returns Err (non-TTY)
#[test]
fn drop_force_false_returns_err() {
    let dir = make_dir();
    setup(&dir);
    assert!(
        run(TableCommand::Drop {
            dir: tbl(&dir),
            force: false
        })
        .is_err()
    );
}

// 3. drop missing dir returns Err
#[test]
fn drop_missing_dir_returns_err() {
    let dir = make_dir();
    assert!(
        run(TableCommand::Drop {
            dir: dir.path().join("gone"),
            force: true
        })
        .is_err()
    );
}

// 4. dir does not exist after drop
#[test]
fn drop_removes_dir() {
    let dir = make_dir();
    setup(&dir);
    run(TableCommand::Drop {
        dir: tbl(&dir),
        force: true,
    })
    .unwrap();
    assert!(!tbl(&dir).exists());
}

// 5. truncate on existing table returns Ok
#[test]
fn truncate_existing_table_ok() {
    let dir = make_dir();
    setup(&dir);
    assert!(
        run(TableCommand::Truncate {
            dir: tbl(&dir),
            force: true
        })
        .is_ok()
    );
}

// 6. truncate with force=false returns Err
#[test]
fn truncate_force_false_returns_err() {
    let dir = make_dir();
    setup(&dir);
    assert!(
        run(TableCommand::Truncate {
            dir: tbl(&dir),
            force: false
        })
        .is_err()
    );
}

// 7. truncate missing dir returns Err
#[test]
fn truncate_missing_dir_returns_err() {
    let dir = make_dir();
    assert!(
        run(TableCommand::Truncate {
            dir: dir.path().join("gone"),
            force: true
        })
        .is_err()
    );
}

// 8. table still exists after truncate
#[test]
fn truncate_dir_still_exists() {
    let dir = make_dir();
    setup(&dir);
    run(TableCommand::Truncate {
        dir: tbl(&dir),
        force: true,
    })
    .unwrap();
    assert!(tbl(&dir).exists());
}

// 9. insert after truncate returns Ok
#[test]
fn insert_after_truncate_ok() {
    let dir = make_dir();
    setup(&dir);
    run(TableCommand::Truncate {
        dir: tbl(&dir),
        force: true,
    })
    .unwrap();
    assert!(
        run(TableCommand::Insert {
            dir: tbl(&dir),
            pk: 1,
            values: vec![10]
        })
        .is_ok()
    );
}

// 10. count after truncate is zero
#[test]
fn count_after_truncate_is_zero() {
    use vvctl::commands::table::OutputFormat;
    let dir = make_dir();
    setup(&dir);
    for i in 0u64..5 {
        run(TableCommand::Insert {
            dir: tbl(&dir),
            pk: i,
            values: vec![i],
        })
        .unwrap();
    }
    run(TableCommand::Flush { dir: tbl(&dir) }).unwrap();
    run(TableCommand::Truncate {
        dir: tbl(&dir),
        force: true,
    })
    .unwrap();
    assert!(
        run(TableCommand::Count {
            dir: tbl(&dir),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

// 11. exists returns Err after drop
#[test]
fn exists_err_after_drop() {
    let dir = make_dir();
    setup(&dir);
    run(TableCommand::Drop {
        dir: tbl(&dir),
        force: true,
    })
    .unwrap();
    assert!(run(TableCommand::Exists { dir: tbl(&dir) }).is_err());
}

// 12. double drop returns Err
#[test]
fn double_drop_returns_err() {
    let dir = make_dir();
    setup(&dir);
    run(TableCommand::Drop {
        dir: tbl(&dir),
        force: true,
    })
    .unwrap();
    assert!(
        run(TableCommand::Drop {
            dir: tbl(&dir),
            force: true
        })
        .is_err()
    );
}

// 13. truncate twice returns Ok
#[test]
fn truncate_twice_ok() {
    let dir = make_dir();
    setup(&dir);
    run(TableCommand::Truncate {
        dir: tbl(&dir),
        force: true,
    })
    .unwrap();
    assert!(
        run(TableCommand::Truncate {
            dir: tbl(&dir),
            force: true
        })
        .is_ok()
    );
}
