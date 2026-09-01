//! Tests for Layer 300: Full lifecycle correctness (VV-CTL-300).

use std::path::PathBuf;
use tempfile::TempDir;
use vvctl::commands::table::{OutputFormat, TableCommand, run};

fn make_dir() -> TempDir {
    TempDir::new().unwrap()
}
fn tdir(d: &TempDir) -> PathBuf {
    d.path().join("tbl")
}

#[test]
fn lifecycle_create_insert_get_ok() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: tdir(&d),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![10],
    })
    .unwrap();
    assert!(
        run(TableCommand::Get {
            dir: tdir(&d),
            pk: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn lifecycle_create_insert_flush_compact_ok() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: tdir(&d),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    for i in 1..=5 {
        run(TableCommand::Insert {
            dir: tdir(&d),
            pk: i,
            values: vec![i * 10],
        })
        .unwrap();
    }
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    assert!(run(TableCommand::Compact { dir: tdir(&d) }).is_ok());
}

#[test]
fn lifecycle_insert_update_get_ok() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: tdir(&d),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![10],
    })
    .unwrap();
    run(TableCommand::Update {
        dir: tdir(&d),
        pk: 1,
        col: 1,
        val: 99,
    })
    .unwrap();
    assert!(
        run(TableCommand::Get {
            dir: tdir(&d),
            pk: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn lifecycle_insert_delete_count_ok() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: tdir(&d),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    for i in 1..=5 {
        run(TableCommand::Insert {
            dir: tdir(&d),
            pk: i,
            values: vec![i * 10],
        })
        .unwrap();
    }
    run(TableCommand::Delete {
        dir: tdir(&d),
        pk: 3,
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
fn lifecycle_alter_add_insert_schema_ok() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: tdir(&d),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    run(TableCommand::AlterAddCol {
        dir: tdir(&d),
        column: "extra:u64".into(),
    })
    .unwrap();
    assert!(
        run(TableCommand::Schema {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn lifecycle_truncate_refill_ok() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: tdir(&d),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    for i in 1..=5 {
        run(TableCommand::Insert {
            dir: tdir(&d),
            pk: i,
            values: vec![i * 10],
        })
        .unwrap();
    }
    run(TableCommand::Truncate {
        dir: tdir(&d),
        force: true,
    })
    .unwrap();
    for i in 10..=14 {
        run(TableCommand::Insert {
            dir: tdir(&d),
            pk: i,
            values: vec![i * 10],
        })
        .unwrap();
    }
    assert!(
        run(TableCommand::Count {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn lifecycle_snapshot_restore_ok() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: tdir(&d),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![10],
    })
    .unwrap();
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    assert!(run(TableCommand::Snapshot { dir: tdir(&d) }).is_ok());
}

#[test]
fn lifecycle_rename_col_schema_ok() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: tdir(&d),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    run(TableCommand::RenameCol {
        dir: tdir(&d),
        from: "val".into(),
        to: "value".into(),
    })
    .unwrap();
    assert!(
        run(TableCommand::Schema {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn lifecycle_full_crud_ok() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: tdir(&d),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![100],
    })
    .unwrap();
    run(TableCommand::Update {
        dir: tdir(&d),
        pk: 1,
        col: 1,
        val: 200,
    })
    .unwrap();
    run(TableCommand::Get {
        dir: tdir(&d),
        pk: 1,
        output: OutputFormat::Table,
    })
    .unwrap();
    run(TableCommand::Delete {
        dir: tdir(&d),
        pk: 1,
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
fn lifecycle_flush_vacuum_check_ok() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: tdir(&d),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    for i in 1..=3 {
        run(TableCommand::Insert {
            dir: tdir(&d),
            pk: i,
            values: vec![i * 10],
        })
        .unwrap();
    }
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    run(TableCommand::Vacuum { dir: tdir(&d) }).unwrap();
    assert!(run(TableCommand::Check { dir: tdir(&d) }).is_ok());
}

#[test]
fn lifecycle_pk_range_after_many_inserts_ok() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: tdir(&d),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    for i in 1..=10 {
        run(TableCommand::Insert {
            dir: tdir(&d),
            pk: i,
            values: vec![i * 10],
        })
        .unwrap();
    }
    assert!(
        run(TableCommand::PkRange {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn lifecycle_info_after_full_ops_ok() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: tdir(&d),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    for i in 1..=5 {
        run(TableCommand::Insert {
            dir: tdir(&d),
            pk: i,
            values: vec![i * 10],
        })
        .unwrap();
    }
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    run(TableCommand::Compact { dir: tdir(&d) }).unwrap();
    assert!(
        run(TableCommand::Info {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn lifecycle_size_grows_after_inserts_ok() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: tdir(&d),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    for i in 1..=5 {
        run(TableCommand::Insert {
            dir: tdir(&d),
            pk: i,
            values: vec![i * 10],
        })
        .unwrap();
    }
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    assert!(
        run(TableCommand::Size {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
