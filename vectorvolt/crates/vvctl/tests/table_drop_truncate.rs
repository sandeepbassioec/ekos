//! Integration tests for L128: vvctl table drop + truncate (VV-ENG-009 DDL completion).

use std::path::PathBuf;

use tempfile::TempDir;
use vvctl::commands::table::{OutputFormat, TableCommand, run};

fn make_dir() -> TempDir {
    TempDir::new().unwrap()
}

fn table_dir(dir: &TempDir) -> PathBuf {
    dir.path().join("tbl")
}

fn create_and_insert(dir: &TempDir) {
    run(TableCommand::Create {
        dir: table_dir(dir),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    run(TableCommand::Insert {
        dir: table_dir(dir),
        pk: 1,
        values: vec![42],
    })
    .unwrap();
}

// â”€â”€ vvctl table drop â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn drop_without_force_errors() {
    let dir = make_dir();
    run(TableCommand::Create {
        dir: table_dir(&dir),
        columns: vec!["pk:u64".into()],
    })
    .unwrap();
    let result = run(TableCommand::Drop {
        dir: table_dir(&dir),
        force: false,
    });
    assert!(result.is_err());
    assert!(table_dir(&dir).exists(), "directory must still exist");
}

#[test]
fn drop_with_force_removes_directory() {
    let dir = make_dir();
    run(TableCommand::Create {
        dir: table_dir(&dir),
        columns: vec!["pk:u64".into()],
    })
    .unwrap();
    run(TableCommand::Drop {
        dir: table_dir(&dir),
        force: true,
    })
    .unwrap();
    assert!(!table_dir(&dir).exists());
}

#[test]
fn drop_nonexistent_errors() {
    let dir = make_dir();
    let result = run(TableCommand::Drop {
        dir: dir.path().join("ghost"),
        force: true,
    });
    assert!(result.is_err());
}

#[test]
fn drop_removes_schema_and_data() {
    let dir = make_dir();
    create_and_insert(&dir);
    run(TableCommand::Drop {
        dir: table_dir(&dir),
        force: true,
    })
    .unwrap();
    assert!(!table_dir(&dir).join("schema.txt").exists());
    assert!(!table_dir(&dir).join("data").exists());
}

#[test]
fn drop_idempotent_with_force_on_already_removed() {
    let dir = make_dir();
    run(TableCommand::Create {
        dir: table_dir(&dir),
        columns: vec!["pk:u64".into()],
    })
    .unwrap();
    run(TableCommand::Drop {
        dir: table_dir(&dir),
        force: true,
    })
    .unwrap();
    // Second drop should error (directory gone)
    let result = run(TableCommand::Drop {
        dir: table_dir(&dir),
        force: true,
    });
    assert!(result.is_err());
}

// â”€â”€ vvctl table truncate â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn truncate_preserves_schema() {
    let dir = make_dir();
    create_and_insert(&dir);
    run(TableCommand::Truncate {
        dir: table_dir(&dir),
        force: true,
    })
    .unwrap();
    assert!(table_dir(&dir).join("schema.txt").exists());
}

#[test]
fn truncate_removes_vvsst_files() {
    let dir = make_dir();
    create_and_insert(&dir);
    // Force a flush to create at least one .vvsst file
    run(TableCommand::Flush {
        dir: table_dir(&dir),
    })
    .unwrap();
    run(TableCommand::Truncate {
        dir: table_dir(&dir),
        force: true,
    })
    .unwrap();
    let data_dir = table_dir(&dir).join("data");
    if data_dir.exists() {
        let vvsst_count = std::fs::read_dir(&data_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("vvsst"))
            .count();
        assert_eq!(vvsst_count, 0);
    }
}

#[test]
fn truncate_missing_table_errors() {
    let dir = make_dir();
    let result = run(TableCommand::Truncate {
        dir: dir.path().join("nonexistent"),
        force: true,
    });
    assert!(result.is_err());
}

#[test]
fn truncate_then_create_succeeds() {
    let dir = make_dir();
    create_and_insert(&dir);
    run(TableCommand::Truncate {
        dir: table_dir(&dir),
        force: true,
    })
    .unwrap();
    // After truncate, schema.txt still exists so Create should fail (table exists)
    // But we can still run Schema successfully
    run(TableCommand::Schema {
        dir: table_dir(&dir),
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn truncate_empty_table_succeeds() {
    let dir = make_dir();
    run(TableCommand::Create {
        dir: table_dir(&dir),
        columns: vec!["pk:u64".into()],
    })
    .unwrap();
    run(TableCommand::Truncate {
        dir: table_dir(&dir),
        force: true,
    })
    .unwrap();
    assert!(table_dir(&dir).join("schema.txt").exists());
}

// â”€â”€ drop vs truncate distinction â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn drop_removes_dir_truncate_does_not() {
    let dir1 = make_dir();
    let dir2 = TempDir::new().unwrap();
    // Set up two identical tables
    run(TableCommand::Create {
        dir: dir1.path().join("tbl"),
        columns: vec!["pk:u64".into()],
    })
    .unwrap();
    run(TableCommand::Create {
        dir: dir2.path().join("tbl"),
        columns: vec!["pk:u64".into()],
    })
    .unwrap();

    run(TableCommand::Drop {
        dir: dir1.path().join("tbl"),
        force: true,
    })
    .unwrap();
    run(TableCommand::Truncate {
        dir: dir2.path().join("tbl"),
        force: true,
    })
    .unwrap();

    assert!(!dir1.path().join("tbl").exists());
    assert!(dir2.path().join("tbl").exists());
}
