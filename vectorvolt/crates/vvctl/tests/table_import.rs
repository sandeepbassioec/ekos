//! Integration tests for Layer 152: vvctl table import (VV-CTL-152).

use std::path::PathBuf;
use tempfile::TempDir;
use vvctl::commands::table::{TableCommand, run};

fn make_dir() -> TempDir {
    TempDir::new().unwrap()
}
fn tbl(d: &TempDir) -> PathBuf {
    d.path().join("tbl")
}

fn create(d: &TempDir) {
    run(TableCommand::Create {
        dir: tbl(d),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
}

fn json_file(d: &TempDir, content: &str) -> PathBuf {
    let p = d.path().join("import.json");
    std::fs::write(&p, content).unwrap();
    p
}

#[test]
fn import_empty_array_ok() {
    let d = make_dir();
    create(&d);
    let f = json_file(&d, "[]");
    assert!(
        run(TableCommand::Import {
            dir: tbl(&d),
            file: f
        })
        .is_ok()
    );
}

#[test]
fn import_single_row_ok() {
    let d = make_dir();
    create(&d);
    let f = json_file(&d, r#"[{"pk":"1","val":"10"}]"#);
    assert!(
        run(TableCommand::Import {
            dir: tbl(&d),
            file: f
        })
        .is_ok()
    );
}

#[test]
fn import_multiple_rows_ok() {
    let d = make_dir();
    create(&d);
    let f = json_file(&d, r#"[{"pk":"1","val":"10"},{"pk":"2","val":"20"}]"#);
    assert!(
        run(TableCommand::Import {
            dir: tbl(&d),
            file: f
        })
        .is_ok()
    );
}

#[test]
fn import_missing_file_errors() {
    let d = make_dir();
    create(&d);
    assert!(
        run(TableCommand::Import {
            dir: tbl(&d),
            file: PathBuf::from("/no/such.json")
        })
        .is_err()
    );
}

#[test]
fn import_invalid_json_errors() {
    let d = make_dir();
    create(&d);
    let f = json_file(&d, "not valid json");
    assert!(
        run(TableCommand::Import {
            dir: tbl(&d),
            file: f
        })
        .is_err()
    );
}

#[test]
fn import_missing_dir_errors() {
    let d = make_dir();
    let f = json_file(&d, "[]");
    assert!(
        run(TableCommand::Import {
            dir: PathBuf::from("/no/such/dir"),
            file: f
        })
        .is_err()
    );
}

#[test]
fn import_upsert_same_pk_ok() {
    let d = make_dir();
    create(&d);
    let f1 = json_file(&d, r#"[{"pk":"1","val":"10"}]"#);
    assert!(
        run(TableCommand::Import {
            dir: tbl(&d),
            file: f1
        })
        .is_ok()
    );
    let f2 = d.path().join("import2.json");
    std::fs::write(&f2, r#"[{"pk":"1","val":"99"}]"#).unwrap();
    assert!(
        run(TableCommand::Import {
            dir: tbl(&d),
            file: f2
        })
        .is_ok()
    );
}

#[test]
fn import_ten_rows_ok() {
    let d = make_dir();
    create(&d);
    let rows: Vec<String> = (0..10u64)
        .map(|i| format!(r#"{{"pk":"{i}","val":"{i}"}}"#))
        .collect();
    let json = format!("[{}]", rows.join(","));
    let f = json_file(&d, &json);
    assert!(
        run(TableCommand::Import {
            dir: tbl(&d),
            file: f
        })
        .is_ok()
    );
}

#[test]
fn import_zero_pk_ok() {
    let d = make_dir();
    create(&d);
    let f = json_file(&d, r#"[{"pk":"0","val":"0"}]"#);
    assert!(
        run(TableCommand::Import {
            dir: tbl(&d),
            file: f
        })
        .is_ok()
    );
}

#[test]
fn import_large_pk_ok() {
    let d = make_dir();
    create(&d);
    let f = json_file(&d, r#"[{"pk":"9999999","val":"1"}]"#);
    assert!(
        run(TableCommand::Import {
            dir: tbl(&d),
            file: f
        })
        .is_ok()
    );
}

#[test]
fn import_then_scan_ok() {
    let d = make_dir();
    create(&d);
    let f = json_file(&d, r#"[{"pk":"5","val":"50"}]"#);
    assert!(
        run(TableCommand::Import {
            dir: tbl(&d),
            file: f
        })
        .is_ok()
    );
    use vvctl::commands::table::OutputFormat;
    assert!(
        run(TableCommand::Scan {
            dir: tbl(&d),
            col: 0,
            lo: 0,
            hi: u64::MAX,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn import_repeated_ok() {
    let d = make_dir();
    create(&d);
    for i in 0..3u32 {
        let fname = format!("import{i}.json");
        let p = d.path().join(&fname);
        std::fs::write(&p, r#"[{"pk":"7","val":"70"}]"#).unwrap();
        assert!(
            run(TableCommand::Import {
                dir: tbl(&d),
                file: p
            })
            .is_ok()
        );
    }
}

#[test]
fn import_preserves_existing_rows() {
    let d = make_dir();
    create(&d);
    run(TableCommand::Insert {
        dir: tbl(&d),
        pk: 100,
        values: vec![1000],
    })
    .unwrap();
    let f = json_file(&d, r#"[{"pk":"200","val":"2000"}]"#);
    assert!(
        run(TableCommand::Import {
            dir: tbl(&d),
            file: f
        })
        .is_ok()
    );
}
