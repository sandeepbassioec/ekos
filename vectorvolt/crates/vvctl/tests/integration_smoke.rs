//! Layer 201: End-to-end integration smoke tests for vvctl table commands.

use tempfile::TempDir;
use vvctl::commands::table::{OutputFormat, TableCommand, run};

fn tempdir() -> TempDir {
    tempfile::tempdir().expect("tempdir")
}

#[test]
fn smoke_create_table() {
    let d = tempdir();
    let cmd = TableCommand::Create {
        dir: d.path().to_path_buf(),
        columns: vec!["pk".into(), "score".into()],
    };
    assert!(run(cmd).is_ok());
}

#[test]
fn smoke_insert_row() {
    let d = tempdir();
    run(TableCommand::Create {
        dir: d.path().to_path_buf(),
        columns: vec!["pk".into(), "val".into()],
    })
    .unwrap();
    let cmd = TableCommand::Insert {
        dir: d.path().to_path_buf(),
        pk: 1,
        values: vec![42],
    };
    assert!(run(cmd).is_ok());
}

#[test]
fn smoke_get_row() {
    let d = tempdir();
    run(TableCommand::Create {
        dir: d.path().to_path_buf(),
        columns: vec!["pk".into(), "val".into()],
    })
    .unwrap();
    run(TableCommand::Insert {
        dir: d.path().to_path_buf(),
        pk: 1,
        values: vec![10],
    })
    .unwrap();
    let cmd = TableCommand::Get {
        dir: d.path().to_path_buf(),
        pk: 1,
        output: OutputFormat::Table,
    };
    assert!(run(cmd).is_ok());
}

#[test]
fn smoke_scan() {
    let d = tempdir();
    run(TableCommand::Create {
        dir: d.path().to_path_buf(),
        columns: vec!["pk".into(), "score".into()],
    })
    .unwrap();
    run(TableCommand::Insert {
        dir: d.path().to_path_buf(),
        pk: 1,
        values: vec![100],
    })
    .unwrap();
    run(TableCommand::Insert {
        dir: d.path().to_path_buf(),
        pk: 2,
        values: vec![200],
    })
    .unwrap();
    let cmd = TableCommand::Scan {
        dir: d.path().to_path_buf(),
        col: 0,
        lo: 1,
        hi: u64::MAX,
        output: OutputFormat::Table,
    };
    assert!(run(cmd).is_ok());
}

#[test]
fn smoke_stats() {
    let d = tempdir();
    run(TableCommand::Create {
        dir: d.path().to_path_buf(),
        columns: vec!["pk".into(), "val".into()],
    })
    .unwrap();
    let cmd = TableCommand::Stats {
        dir: d.path().to_path_buf(),
        output: OutputFormat::Table,
    };
    assert!(run(cmd).is_ok());
}

#[test]
fn smoke_delete_row() {
    let d = tempdir();
    run(TableCommand::Create {
        dir: d.path().to_path_buf(),
        columns: vec!["pk".into(), "val".into()],
    })
    .unwrap();
    run(TableCommand::Insert {
        dir: d.path().to_path_buf(),
        pk: 5,
        values: vec![99],
    })
    .unwrap();
    let cmd = TableCommand::Delete {
        dir: d.path().to_path_buf(),
        pk: 5,
    };
    assert!(run(cmd).is_ok());
}

#[test]
fn smoke_flush() {
    let d = tempdir();
    run(TableCommand::Create {
        dir: d.path().to_path_buf(),
        columns: vec!["pk".into()],
    })
    .unwrap();
    let cmd = TableCommand::Flush {
        dir: d.path().to_path_buf(),
    };
    assert!(run(cmd).is_ok());
}

#[test]
fn smoke_compact() {
    let d = tempdir();
    run(TableCommand::Create {
        dir: d.path().to_path_buf(),
        columns: vec!["pk".into()],
    })
    .unwrap();
    let cmd = TableCommand::Compact {
        dir: d.path().to_path_buf(),
    };
    assert!(run(cmd).is_ok());
}

#[test]
fn smoke_schema() {
    let d = tempdir();
    run(TableCommand::Create {
        dir: d.path().to_path_buf(),
        columns: vec!["pk".into(), "score".into()],
    })
    .unwrap();
    let cmd = TableCommand::Schema {
        dir: d.path().to_path_buf(),
        output: OutputFormat::Table,
    };
    assert!(run(cmd).is_ok());
}

#[test]
fn smoke_json_output() {
    let d = tempdir();
    run(TableCommand::Create {
        dir: d.path().to_path_buf(),
        columns: vec!["pk".into(), "val".into()],
    })
    .unwrap();
    run(TableCommand::Insert {
        dir: d.path().to_path_buf(),
        pk: 1,
        values: vec![10],
    })
    .unwrap();
    let cmd = TableCommand::Get {
        dir: d.path().to_path_buf(),
        pk: 1,
        output: OutputFormat::Json,
    };
    assert!(run(cmd).is_ok());
}

#[test]
fn smoke_multiple_inserts() {
    let d = tempdir();
    run(TableCommand::Create {
        dir: d.path().to_path_buf(),
        columns: vec!["pk".into(), "val".into()],
    })
    .unwrap();
    for i in 1u64..=5 {
        run(TableCommand::Insert {
            dir: d.path().to_path_buf(),
            pk: i,
            values: vec![i * 10],
        })
        .unwrap();
    }
    let cmd = TableCommand::Scan {
        dir: d.path().to_path_buf(),
        col: 0,
        lo: 1,
        hi: u64::MAX,
        output: OutputFormat::Table,
    };
    assert!(run(cmd).is_ok());
}

#[test]
fn smoke_create_two_col_table() {
    let d = tempdir();
    let cmd = TableCommand::Create {
        dir: d.path().to_path_buf(),
        columns: vec!["pk".into(), "a".into(), "b".into()],
    };
    assert!(run(cmd).is_ok());
}

#[test]
fn smoke_scan_json() {
    let d = tempdir();
    run(TableCommand::Create {
        dir: d.path().to_path_buf(),
        columns: vec!["pk".into(), "val".into()],
    })
    .unwrap();
    run(TableCommand::Insert {
        dir: d.path().to_path_buf(),
        pk: 10,
        values: vec![55],
    })
    .unwrap();
    let cmd = TableCommand::Scan {
        dir: d.path().to_path_buf(),
        col: 0,
        lo: 0,
        hi: u64::MAX,
        output: OutputFormat::Json,
    };
    assert!(run(cmd).is_ok());
}
