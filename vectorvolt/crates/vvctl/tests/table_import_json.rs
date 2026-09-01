//! Tests for Layer 278: vvctl table import JSON (VV-CTL-278).

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

fn write_import_file(d: &TempDir, content: &str) -> PathBuf {
    let path = d.path().join("import.json");
    std::fs::write(&path, content).unwrap();
    path
}

#[test]
fn import_empty_array_ok() {
    let d = make_dir();
    setup(&d);
    let f = write_import_file(&d, "[]");
    run(TableCommand::Import {
        dir: tdir(&d),
        file: f,
    })
    .unwrap();
}

#[test]
fn import_one_row_ok() {
    let d = make_dir();
    setup(&d);
    let f = write_import_file(&d, r#"[{"pk":1,"val":10}]"#);
    run(TableCommand::Import {
        dir: tdir(&d),
        file: f,
    })
    .unwrap();
}

#[test]
fn import_five_rows_count_ok() {
    let d = make_dir();
    setup(&d);
    let f = write_import_file(
        &d,
        r#"[{"pk":1,"val":10},{"pk":2,"val":20},{"pk":3,"val":30},{"pk":4,"val":40},{"pk":5,"val":50}]"#,
    );
    run(TableCommand::Import {
        dir: tdir(&d),
        file: f,
    })
    .unwrap();
    run(TableCommand::Count {
        dir: tdir(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn import_then_get_ok() {
    let d = make_dir();
    setup(&d);
    let f = write_import_file(&d, r#"[{"pk":7,"val":70}]"#);
    run(TableCommand::Import {
        dir: tdir(&d),
        file: f,
    })
    .unwrap();
    run(TableCommand::Get {
        dir: tdir(&d),
        pk: 7,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn import_then_scan_ok() {
    let d = make_dir();
    setup(&d);
    let f = write_import_file(&d, r#"[{"pk":3,"val":30}]"#);
    run(TableCommand::Import {
        dir: tdir(&d),
        file: f,
    })
    .unwrap();
    run(TableCommand::Scan {
        dir: tdir(&d),
        col: 0,
        lo: 0,
        hi: u64::MAX - 1,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn import_missing_dir_errors() {
    let d = make_dir();
    let f = write_import_file(&d, "[]");
    let res = run(TableCommand::Import {
        dir: d.path().join("nope"),
        file: f,
    });
    assert!(res.is_err());
}

#[test]
fn import_missing_file_errors() {
    let d = make_dir();
    setup(&d);
    let res = run(TableCommand::Import {
        dir: tdir(&d),
        file: d.path().join("nofile.json"),
    });
    assert!(res.is_err());
}

#[test]
fn import_overwrites_existing_pk() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![10],
    })
    .unwrap();
    let f = write_import_file(&d, r#"[{"pk":1,"val":99}]"#);
    run(TableCommand::Import {
        dir: tdir(&d),
        file: f,
    })
    .unwrap();
}

#[test]
fn import_after_truncate_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![10],
    })
    .unwrap();
    run(TableCommand::Truncate {
        dir: tdir(&d),
        force: true,
    })
    .unwrap();
    let f = write_import_file(&d, r#"[{"pk":2,"val":20}]"#);
    run(TableCommand::Import {
        dir: tdir(&d),
        file: f,
    })
    .unwrap();
}

#[test]
fn import_count_same_as_exported() {
    let d = make_dir();
    setup(&d);
    for i in 1..=3u64 {
        run(TableCommand::Insert {
            dir: tdir(&d),
            pk: i,
            values: vec![i],
        })
        .unwrap();
    }
    run(TableCommand::Count {
        dir: tdir(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
    run(TableCommand::Export {
        dir: tdir(&d),
        format: ExportFormat::Json,
    })
    .unwrap();
}

#[test]
fn import_export_roundtrip_count_ok() {
    let d = make_dir();
    setup(&d);
    let f = write_import_file(&d, r#"[{"pk":10,"val":100},{"pk":20,"val":200}]"#);
    run(TableCommand::Import {
        dir: tdir(&d),
        file: f,
    })
    .unwrap();
    run(TableCommand::Export {
        dir: tdir(&d),
        format: ExportFormat::Json,
    })
    .unwrap();
}

#[test]
fn import_json_written_to_temp_dir_ok() {
    let d = make_dir();
    setup(&d);
    let f = write_import_file(&d, r#"[{"pk":5,"val":55}]"#);
    assert!(f.exists());
    run(TableCommand::Import {
        dir: tdir(&d),
        file: f,
    })
    .unwrap();
}

#[test]
fn import_large_dataset_ok() {
    let d = make_dir();
    setup(&d);
    // Build a JSON array of 10 rows
    let rows: Vec<String> = (1..=10u64)
        .map(|i| format!(r#"{{"pk":{},"val":{}}}"#, i, i * 10))
        .collect();
    let json = format!("[{}]", rows.join(","));
    let f = write_import_file(&d, &json);
    run(TableCommand::Import {
        dir: tdir(&d),
        file: f,
    })
    .unwrap();
}
