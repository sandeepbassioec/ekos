use tempfile::TempDir;
use vvctl::commands::table::{OutputFormat, TableCommand, run};

fn make_dir() -> TempDir {
    TempDir::new().unwrap()
}
fn create_table(d: &TempDir) {
    run(TableCommand::Create {
        dir: d.path().to_path_buf(),
        columns: vec!["pk".into(), "val".into()],
    })
    .unwrap();
}
fn insert(d: &TempDir, pk: u64, val: u64) {
    run(TableCommand::Insert {
        dir: d.path().to_path_buf(),
        pk,
        values: vec![val],
    })
    .unwrap();
}
fn get(d: &TempDir, pk: u64) -> anyhow::Result<()> {
    run(TableCommand::Get {
        dir: d.path().to_path_buf(),
        pk,
        output: OutputFormat::Table,
    })
}

#[test]
fn get_on_empty_table_returns_ok() {
    let d = make_dir();
    create_table(&d);
    assert!(get(&d, 1).is_ok());
}

#[test]
fn get_after_insert_returns_ok() {
    let d = make_dir();
    create_table(&d);
    insert(&d, 1, 100);
    assert!(get(&d, 1).is_ok());
}

#[test]
fn get_missing_key_returns_ok_not_found() {
    let d = make_dir();
    create_table(&d);
    insert(&d, 1, 100);
    assert!(get(&d, 999).is_ok());
}

#[test]
fn get_json_format() {
    let d = make_dir();
    create_table(&d);
    insert(&d, 5, 42);
    let r = run(TableCommand::Get {
        dir: d.path().to_path_buf(),
        pk: 5,
        output: OutputFormat::Json,
    });
    assert!(r.is_ok());
}

#[test]
fn get_multiple_inserts() {
    let d = make_dir();
    create_table(&d);
    for i in 0..5u64 {
        insert(&d, i, i * 10);
    }
    for i in 0..5u64 {
        assert!(get(&d, i).is_ok());
    }
}

#[test]
fn get_missing_dir_returns_err() {
    let r = run(TableCommand::Get {
        dir: "/nonexistent/path/xyz".into(),
        pk: 1,
        output: OutputFormat::Table,
    });
    assert!(r.is_err());
}

#[test]
fn get_after_delete_returns_not_found() {
    let d = make_dir();
    create_table(&d);
    insert(&d, 10, 200);
    run(TableCommand::Delete {
        dir: d.path().to_path_buf(),
        pk: 10,
    })
    .unwrap();
    assert!(get(&d, 10).is_ok());
}

#[test]
fn get_pk_zero() {
    let d = make_dir();
    create_table(&d);
    insert(&d, 0, 1);
    assert!(get(&d, 0).is_ok());
}

#[test]
fn get_pk_high_value() {
    let d = make_dir();
    create_table(&d);
    assert!(get(&d, 1_000_000).is_ok());
}

#[test]
fn get_large_value() {
    let d = make_dir();
    create_table(&d);
    insert(&d, 1, u64::MAX);
    assert!(get(&d, 1).is_ok());
}

#[test]
fn get_multiple_columns() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: d.path().to_path_buf(),
        columns: vec!["pk".into(), "a".into(), "b".into(), "c".into()],
    })
    .unwrap();
    run(TableCommand::Insert {
        dir: d.path().to_path_buf(),
        pk: 1,
        values: vec![10, 20, 30],
    })
    .unwrap();
    assert!(get(&d, 1).is_ok());
}

#[test]
fn get_after_flush() {
    let d = make_dir();
    create_table(&d);
    insert(&d, 7, 77);
    run(TableCommand::Flush {
        dir: d.path().to_path_buf(),
    })
    .unwrap();
    assert!(get(&d, 7).is_ok());
}

#[test]
fn get_json_missing_key() {
    let d = make_dir();
    create_table(&d);
    let r = run(TableCommand::Get {
        dir: d.path().to_path_buf(),
        pk: 999,
        output: OutputFormat::Json,
    });
    assert!(r.is_ok());
}
