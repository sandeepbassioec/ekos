//! Tests for Layer 331: Sink trait stub (VV-CTL-331).

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
fn ins(d: &TempDir, pk: u64, val: u64) {
    run(TableCommand::Insert {
        dir: tdir(d),
        pk,
        values: vec![val],
    })
    .unwrap();
}

#[test]
fn insert_3_rows_ok() {
    let d = make_dir();
    setup(&d);
    for i in 0..3 {
        ins(&d, i, i * 10);
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
fn export_json_ok() {
    let d = make_dir();
    setup(&d);
    for i in 0..3 {
        ins(&d, i, i * 10);
    }
    assert!(
        run(TableCommand::Export {
            dir: tdir(&d),
            format: ExportFormat::Json
        })
        .is_ok()
    );
}

#[test]
fn export_then_import_ok() {
    let d = make_dir();
    setup(&d);
    for i in 0..3 {
        ins(&d, i, i * 10);
    }
    let tmp = TempDir::new().unwrap();
    let json_path = tmp.path().join("export.json");
    // Write a simple JSON import file
    let json = r#"[{"pk":100,"val":1000},{"pk":101,"val":1010}]"#;
    std::fs::write(&json_path, json).unwrap();
    assert!(
        run(TableCommand::Import {
            dir: tdir(&d),
            file: json_path
        })
        .is_ok()
    );
}

#[test]
fn count_after_import_correct() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    let tmp = TempDir::new().unwrap();
    let json_path = tmp.path().join("import.json");
    let json = r#"[{"pk":200,"val":2000}]"#;
    std::fs::write(&json_path, json).unwrap();
    run(TableCommand::Import {
        dir: tdir(&d),
        file: json_path,
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
fn sum_after_import_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    let tmp = TempDir::new().unwrap();
    let json_path = tmp.path().join("sum_import.json");
    let json = r#"[{"pk":300,"val":3000}]"#;
    std::fs::write(&json_path, json).unwrap();
    run(TableCommand::Import {
        dir: tdir(&d),
        file: json_path,
    })
    .unwrap();
    assert!(
        run(TableCommand::SumField {
            dir: tdir(&d),
            col: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn scan_after_import_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    let tmp = TempDir::new().unwrap();
    let json_path = tmp.path().join("scan_import.json");
    let json = r#"[{"pk":400,"val":4000}]"#;
    std::fs::write(&json_path, json).unwrap();
    run(TableCommand::Import {
        dir: tdir(&d),
        file: json_path,
    })
    .unwrap();
    assert!(
        run(TableCommand::Scan {
            dir: tdir(&d),
            col: 0,
            lo: 0,
            hi: u64::MAX,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn keys_after_import_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    let tmp = TempDir::new().unwrap();
    let json_path = tmp.path().join("keys_import.json");
    let json = r#"[{"pk":500,"val":5000}]"#;
    std::fs::write(&json_path, json).unwrap();
    run(TableCommand::Import {
        dir: tdir(&d),
        file: json_path,
    })
    .unwrap();
    assert!(
        run(TableCommand::Keys {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn get_each_pk_ok() {
    let d = make_dir();
    setup(&d);
    for i in 0..3 {
        ins(&d, i, i * 10);
    }
    for pk in 0..3 {
        assert!(
            run(TableCommand::Get {
                dir: tdir(&d),
                pk,
                output: OutputFormat::Table
            })
            .is_ok()
        );
    }
}

#[test]
fn info_ok() {
    let d = make_dir();
    setup(&d);
    assert!(
        run(TableCommand::Info {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn check_ok() {
    let d = make_dir();
    setup(&d);
    assert!(run(TableCommand::Check { dir: tdir(&d) }).is_ok());
}

#[test]
fn stats_ok() {
    let d = make_dir();
    setup(&d);
    assert!(
        run(TableCommand::Stats {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn vacuum_ok() {
    let d = make_dir();
    setup(&d);
    assert!(run(TableCommand::Vacuum { dir: tdir(&d) }).is_ok());
}

#[test]
fn export_import_roundtrip() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 100);
    ins(&d, 2, 200);
    let tmp = TempDir::new().unwrap();
    let json_path = tmp.path().join("roundtrip.json");
    let json = r#"[{"pk":1,"val":100},{"pk":2,"val":200}]"#;
    std::fs::write(&json_path, json).unwrap();
    run(TableCommand::Import {
        dir: tdir(&d),
        file: json_path,
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
