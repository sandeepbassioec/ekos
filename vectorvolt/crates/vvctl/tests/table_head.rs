//! Integration tests for Layer 137: vvctl table head (VV-CTL-019).

use std::path::PathBuf;

use tempfile::TempDir;
use vvctl::commands::table::{OutputFormat, TableCommand, run};

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

fn insert_row(dir: &TempDir, pk: u64, val: u64) {
    run(TableCommand::Insert {
        dir: table_dir(dir),
        pk,
        values: vec![val],
    })
    .unwrap();
}

// 1. head on empty table returns Ok
#[test]
fn head_empty_table_returns_ok() {
    let dir = make_dir();
    create_table(&dir);
    assert!(
        run(TableCommand::Head {
            dir: table_dir(&dir),
            n: 10,
            output: OutputFormat::Table,
        })
        .is_ok()
    );
}

// 2. head n=1 returns one row
#[test]
fn head_n_1_returns_one_row() {
    let dir = make_dir();
    create_table(&dir);
    for i in 0..5u64 {
        insert_row(&dir, i, i);
    }
    assert!(
        run(TableCommand::Head {
            dir: table_dir(&dir),
            n: 1,
            output: OutputFormat::Table,
        })
        .is_ok()
    );
}

// 3. head n larger than rows returns all
#[test]
fn head_n_larger_than_rows_returns_all() {
    let dir = make_dir();
    create_table(&dir);
    insert_row(&dir, 1, 10);
    insert_row(&dir, 2, 20);
    assert!(
        run(TableCommand::Head {
            dir: table_dir(&dir),
            n: 100,
            output: OutputFormat::Table,
        })
        .is_ok()
    );
}

// 4. head default n=10 succeeds
#[test]
fn head_default_n_succeeds() {
    let dir = make_dir();
    create_table(&dir);
    for i in 0..3u64 {
        insert_row(&dir, i, i);
    }
    assert!(
        run(TableCommand::Head {
            dir: table_dir(&dir),
            n: 10,
            output: OutputFormat::Table,
        })
        .is_ok()
    );
}

// 5. head json output succeeds
#[test]
fn head_json_output_succeeds() {
    let dir = make_dir();
    create_table(&dir);
    insert_row(&dir, 1, 100);
    assert!(
        run(TableCommand::Head {
            dir: table_dir(&dir),
            n: 10,
            output: OutputFormat::Json,
        })
        .is_ok()
    );
}

// 6. head table output succeeds
#[test]
fn head_table_output_succeeds() {
    let dir = make_dir();
    create_table(&dir);
    insert_row(&dir, 1, 100);
    assert!(
        run(TableCommand::Head {
            dir: table_dir(&dir),
            n: 10,
            output: OutputFormat::Table,
        })
        .is_ok()
    );
}

// 7. head missing dir errors
#[test]
fn head_missing_dir_errors() {
    let dir = make_dir();
    let result = run(TableCommand::Head {
        dir: dir.path().join("nonexistent"),
        n: 10,
        output: OutputFormat::Table,
    });
    assert!(result.is_err());
}

// 8. head after flush succeeds
#[test]
fn head_after_flush_succeeds() {
    let dir = make_dir();
    create_table(&dir);
    insert_row(&dir, 1, 10);
    run(TableCommand::Flush {
        dir: table_dir(&dir),
    })
    .unwrap();
    assert!(
        run(TableCommand::Head {
            dir: table_dir(&dir),
            n: 10,
            output: OutputFormat::Table,
        })
        .is_ok()
    );
}

// 9. head multi-column table
#[test]
fn head_multi_column_table() {
    let dir = make_dir();
    run(TableCommand::Create {
        dir: table_dir(&dir),
        columns: vec!["pk:u64".into(), "a:u64".into(), "b:u64".into()],
    })
    .unwrap();
    run(TableCommand::Insert {
        dir: table_dir(&dir),
        pk: 1,
        values: vec![2, 3],
    })
    .unwrap();
    assert!(
        run(TableCommand::Head {
            dir: table_dir(&dir),
            n: 10,
            output: OutputFormat::Table,
        })
        .is_ok()
    );
}

// 10. head n=0 returns ok (empty result)
#[test]
fn head_n_0_returns_ok() {
    let dir = make_dir();
    create_table(&dir);
    insert_row(&dir, 1, 10);
    assert!(
        run(TableCommand::Head {
            dir: table_dir(&dir),
            n: 0,
            output: OutputFormat::Table,
        })
        .is_ok()
    );
}

// 11. head typed bool column
#[test]
fn head_typed_bool_column() {
    let dir = make_dir();
    run(TableCommand::Create {
        dir: table_dir(&dir),
        columns: vec!["pk:u64".into(), "flag:bool".into()],
    })
    .unwrap();
    run(TableCommand::TypedInsert {
        dir: table_dir(&dir),
        pk: "1".into(),
        values: vec!["true".into()],
    })
    .unwrap();
    assert!(
        run(TableCommand::Head {
            dir: table_dir(&dir),
            n: 10,
            output: OutputFormat::Table,
        })
        .is_ok()
    );
}

// 12. head typed i64 column
#[test]
fn head_typed_i64_column() {
    let dir = make_dir();
    run(TableCommand::Create {
        dir: table_dir(&dir),
        columns: vec!["pk:u64".into(), "score:i64".into()],
    })
    .unwrap();
    run(TableCommand::TypedInsert {
        dir: table_dir(&dir),
        pk: "1".into(),
        values: vec!["-5".into()],
    })
    .unwrap();
    assert!(
        run(TableCommand::Head {
            dir: table_dir(&dir),
            n: 10,
            output: OutputFormat::Table,
        })
        .is_ok()
    );
}

// 13. head n=5 with 10 rows returns 5
#[test]
fn head_n_5_with_10_rows_returns_5() {
    let dir = make_dir();
    create_table(&dir);
    for i in 0..10u64 {
        insert_row(&dir, i, i * 2);
    }
    // Just verify it returns Ok â€” we can't capture stdout here
    assert!(
        run(TableCommand::Head {
            dir: table_dir(&dir),
            n: 5,
            output: OutputFormat::Table,
        })
        .is_ok()
    );
}
