//! Tests for Layer 363.
use std::path::PathBuf;
use tempfile::TempDir;
use vvctl::commands::table::{ExportFormat, OutputFormat, TableCommand, run};

fn make_dir() -> TempDir {
    TempDir::new().unwrap()
}
fn tdir(d: &TempDir) -> PathBuf {
    d.path().join("tbl")
}

#[test]
fn cluster_create_1col() {
    let d = make_dir();
    assert!(
        run(TableCommand::Create {
            dir: tdir(&d),
            columns: vec!["pk:u64".into()]
        })
        .is_ok()
    );
}
#[test]
fn cluster_create_2col() {
    let d = make_dir();
    assert!(
        run(TableCommand::Create {
            dir: tdir(&d),
            columns: vec!["pk:u64".into(), "v:u64".into()]
        })
        .is_ok()
    );
}
#[test]
fn cluster_create_3col() {
    let d = make_dir();
    assert!(
        run(TableCommand::Create {
            dir: tdir(&d),
            columns: vec!["pk:u64".into(), "a:u64".into(), "b:u64".into()]
        })
        .is_ok()
    );
}
#[test]
fn cluster_insert_ok() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: tdir(&d),
        columns: vec!["pk:u64".into(), "v:u64".into()],
    })
    .unwrap();
    assert!(
        run(TableCommand::Insert {
            dir: tdir(&d),
            pk: 1,
            values: vec![42]
        })
        .is_ok()
    );
}
#[test]
fn cluster_stats_ok() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: tdir(&d),
        columns: vec!["pk:u64".into(), "v:u64".into()],
    })
    .unwrap();
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![1],
    })
    .unwrap();
    assert!(
        run(TableCommand::Stats {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn cluster_info_ok() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: tdir(&d),
        columns: vec!["pk:u64".into(), "v:u64".into()],
    })
    .unwrap();
    assert!(
        run(TableCommand::Info {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn cluster_schema_ok() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: tdir(&d),
        columns: vec!["pk:u64".into(), "v:u64".into()],
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
fn cluster_count_ok() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: tdir(&d),
        columns: vec!["pk:u64".into(), "v:u64".into()],
    })
    .unwrap();
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![1],
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
fn cluster_check_ok() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: tdir(&d),
        columns: vec!["pk:u64".into(), "v:u64".into()],
    })
    .unwrap();
    assert!(run(TableCommand::Check { dir: tdir(&d) }).is_ok());
}
#[test]
fn cluster_vacuum_ok() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: tdir(&d),
        columns: vec!["pk:u64".into(), "v:u64".into()],
    })
    .unwrap();
    assert!(run(TableCommand::Vacuum { dir: tdir(&d) }).is_ok());
}
#[test]
fn cluster_export_ok() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: tdir(&d),
        columns: vec!["pk:u64".into(), "v:u64".into()],
    })
    .unwrap();
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![1],
    })
    .unwrap();
    assert!(
        run(TableCommand::Export {
            dir: tdir(&d),
            format: ExportFormat::Json
        })
        .is_ok()
    );
}
#[test]
fn cluster_snapshot_ok() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: tdir(&d),
        columns: vec!["pk:u64".into(), "v:u64".into()],
    })
    .unwrap();
    assert!(run(TableCommand::Snapshot { dir: tdir(&d) }).is_ok());
}
#[test]
fn cluster_compact_size_ok() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: tdir(&d),
        columns: vec!["pk:u64".into(), "v:u64".into()],
    })
    .unwrap();
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![1],
    })
    .unwrap();
    run(TableCommand::Compact { dir: tdir(&d) }).unwrap();
    assert!(
        run(TableCommand::Size {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
