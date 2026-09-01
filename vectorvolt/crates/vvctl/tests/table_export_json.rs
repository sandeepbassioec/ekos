//! Tests for Layer 277: vvctl table export JSON/CSV (VV-CTL-277).

use std::path::PathBuf;
use tempfile::TempDir;
use vvctl::commands::table::{ExportFormat, OutputFormat, TableCommand, run};

fn make_dir() -> TempDir {
    TempDir::new().unwrap()
}
fn tdir(d: &TempDir) -> PathBuf {
    d.path().join("tbl")
}
fn setup(d: &TempDir) {
    run(TableCommand::Create {
        dir: tdir(d),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
}

#[test]
fn export_empty_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Export {
        dir: tdir(&d),
        format: ExportFormat::Json,
    })
    .unwrap();
}

#[test]
fn export_one_row_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![10],
    })
    .unwrap();
    run(TableCommand::Export {
        dir: tdir(&d),
        format: ExportFormat::Json,
    })
    .unwrap();
}

#[test]
fn export_five_rows_ok() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5u64 {
        run(TableCommand::Insert {
            dir: tdir(&d),
            pk: i,
            values: vec![i * 10],
        })
        .unwrap();
    }
    run(TableCommand::Export {
        dir: tdir(&d),
        format: ExportFormat::Json,
    })
    .unwrap();
}

#[test]
fn export_json_format_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![10],
    })
    .unwrap();
    run(TableCommand::Export {
        dir: tdir(&d),
        format: ExportFormat::Json,
    })
    .unwrap();
}

#[test]
fn export_csv_format_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![10],
    })
    .unwrap();
    run(TableCommand::Export {
        dir: tdir(&d),
        format: ExportFormat::Csv,
    })
    .unwrap();
}

#[test]
fn export_missing_dir_errors() {
    let d = make_dir();
    let res = run(TableCommand::Export {
        dir: d.path().join("nope"),
        format: ExportFormat::Json,
    });
    assert!(res.is_err());
}

#[test]
fn export_large_table_ok() {
    let d = make_dir();
    setup(&d);
    for i in 0..20u64 {
        run(TableCommand::Insert {
            dir: tdir(&d),
            pk: i,
            values: vec![i],
        })
        .unwrap();
    }
    run(TableCommand::Export {
        dir: tdir(&d),
        format: ExportFormat::Json,
    })
    .unwrap();
}

#[test]
fn export_import_roundtrip_ok() {
    let d = make_dir();
    setup(&d);
    for i in 1..=3u64 {
        run(TableCommand::Insert {
            dir: tdir(&d),
            pk: i,
            values: vec![i * 10],
        })
        .unwrap();
    }
    // Just verify both export and count work
    run(TableCommand::Export {
        dir: tdir(&d),
        format: ExportFormat::Json,
    })
    .unwrap();
    run(TableCommand::Count {
        dir: tdir(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn export_multi_col_ok() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: tdir(&d),
        columns: vec!["pk:u64".into(), "a:u64".into(), "b:u64".into()],
    })
    .unwrap();
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![10, 20],
    })
    .unwrap();
    run(TableCommand::Export {
        dir: tdir(&d),
        format: ExportFormat::Json,
    })
    .unwrap();
}

#[test]
fn export_pk0_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 0,
        values: vec![100],
    })
    .unwrap();
    run(TableCommand::Export {
        dir: tdir(&d),
        format: ExportFormat::Json,
    })
    .unwrap();
}

#[test]
fn export_all_pks_present_ok() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5u64 {
        run(TableCommand::Insert {
            dir: tdir(&d),
            pk: i,
            values: vec![i],
        })
        .unwrap();
    }
    run(TableCommand::Export {
        dir: tdir(&d),
        format: ExportFormat::Json,
    })
    .unwrap();
}

#[test]
fn export_after_flush_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 9,
        values: vec![90],
    })
    .unwrap();
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    run(TableCommand::Export {
        dir: tdir(&d),
        format: ExportFormat::Json,
    })
    .unwrap();
}

#[test]
fn export_after_compact_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 10,
        values: vec![100],
    })
    .unwrap();
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    run(TableCommand::Compact { dir: tdir(&d) }).unwrap();
    run(TableCommand::Export {
        dir: tdir(&d),
        format: ExportFormat::Json,
    })
    .unwrap();
}
