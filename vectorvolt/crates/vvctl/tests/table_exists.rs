//! Integration tests for Layer 140: vvctl table exists (VV-CTL-021).

use std::path::PathBuf;

use tempfile::TempDir;
use vvctl::commands::table::{TableCommand, run};

fn make_dir() -> TempDir {
    TempDir::new().unwrap()
}

fn table_dir(dir: &TempDir) -> PathBuf {
    dir.path().join("tbl")
}

fn create_table(dir: &TempDir) {
    run(TableCommand::Create {
        dir: table_dir(dir),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
}

// 1. exists after create succeeds
#[test]
fn exists_after_create_succeeds() {
    let dir = make_dir();
    create_table(&dir);
    assert!(
        run(TableCommand::Exists {
            dir: table_dir(&dir),
        })
        .is_ok()
    );
}

// 2. exists missing dir errors
#[test]
fn exists_missing_dir_errors() {
    let dir = make_dir();
    let result = run(TableCommand::Exists {
        dir: dir.path().join("nonexistent"),
    });
    assert!(result.is_err());
}

// 3. exists empty dir without schema errors
#[test]
fn exists_empty_dir_without_schema_errors() {
    let dir = make_dir();
    let empty = dir.path().join("empty_dir");
    std::fs::create_dir_all(&empty).unwrap();
    let result = run(TableCommand::Exists { dir: empty });
    assert!(result.is_err());
}

// 4. exists after drop errors
#[test]
fn exists_after_drop_errors() {
    let dir = make_dir();
    create_table(&dir);
    run(TableCommand::Drop {
        dir: table_dir(&dir),
        force: true,
    })
    .unwrap();
    let result = run(TableCommand::Exists {
        dir: table_dir(&dir),
    });
    assert!(result.is_err());
}

// 5. exists with data succeeds
#[test]
fn exists_with_data_succeeds() {
    let dir = make_dir();
    create_table(&dir);
    run(TableCommand::Insert {
        dir: table_dir(&dir),
        pk: 1,
        values: vec![10],
    })
    .unwrap();
    assert!(
        run(TableCommand::Exists {
            dir: table_dir(&dir),
        })
        .is_ok()
    );
}

// 6. exists after flush succeeds
#[test]
fn exists_after_flush_succeeds() {
    let dir = make_dir();
    create_table(&dir);
    run(TableCommand::Insert {
        dir: table_dir(&dir),
        pk: 1,
        values: vec![10],
    })
    .unwrap();
    run(TableCommand::Flush {
        dir: table_dir(&dir),
    })
    .unwrap();
    assert!(
        run(TableCommand::Exists {
            dir: table_dir(&dir),
        })
        .is_ok()
    );
}

// 7. exists multi-column table
#[test]
fn exists_multi_column_table() {
    let dir = make_dir();
    run(TableCommand::Create {
        dir: table_dir(&dir),
        columns: vec!["pk:u64".into(), "a:u64".into(), "b:u64".into()],
    })
    .unwrap();
    assert!(
        run(TableCommand::Exists {
            dir: table_dir(&dir),
        })
        .is_ok()
    );
}

// 8. exists typed table
#[test]
fn exists_typed_table() {
    let dir = make_dir();
    run(TableCommand::Create {
        dir: table_dir(&dir),
        columns: vec!["pk:u64".into(), "score:i64".into()],
    })
    .unwrap();
    assert!(
        run(TableCommand::Exists {
            dir: table_dir(&dir),
        })
        .is_ok()
    );
}

// 9. exists after insert succeeds
#[test]
fn exists_after_insert_succeeds() {
    let dir = make_dir();
    create_table(&dir);
    run(TableCommand::Insert {
        dir: table_dir(&dir),
        pk: 42,
        values: vec![99],
    })
    .unwrap();
    assert!(
        run(TableCommand::Exists {
            dir: table_dir(&dir),
        })
        .is_ok()
    );
}

// 10. exists nonexistent path errors
#[test]
fn exists_nonexistent_path_errors() {
    let result = run(TableCommand::Exists {
        dir: PathBuf::from("/nonexistent/path/to/table"),
    });
    assert!(result.is_err());
}

// 11. exists returns Ok not just no panic
#[test]
fn exists_returns_ok_not_just_no_panic() {
    let dir = make_dir();
    create_table(&dir);
    let result = run(TableCommand::Exists {
        dir: table_dir(&dir),
    });
    assert!(result.is_ok(), "expected Ok, got {:?}", result);
}

// 12. exists dir is file errors
#[test]
fn exists_dir_is_file_errors() {
    let dir = make_dir();
    // Create a file where the table dir would be
    let file_path = dir.path().join("tbl");
    std::fs::write(&file_path, b"not a dir").unwrap();
    // Trying to check exists on a path that is a file (not a dir with schema.txt)
    let result = run(TableCommand::Exists { dir: file_path });
    assert!(result.is_err());
}

// 13. exists nested path succeeds
#[test]
fn exists_nested_path_succeeds() {
    let dir = make_dir();
    let nested = dir.path().join("a").join("b").join("tbl");
    run(TableCommand::Create {
        dir: nested.clone(),
        columns: vec!["pk:u64".into()],
    })
    .unwrap();
    assert!(run(TableCommand::Exists { dir: nested }).is_ok());
}
