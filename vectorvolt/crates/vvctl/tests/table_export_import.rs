use std::path::PathBuf;
use tempfile::TempDir;
use vvctl::commands::table::{ExportFormat, TableCommand, run};

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
fn export_json_empty_table() {
    let d = make_dir();
    setup(&d);
    assert!(
        run(TableCommand::Export {
            dir: tdir(&d),
            format: ExportFormat::Json
        })
        .is_ok()
    );
}

#[test]
fn export_csv_empty_table() {
    let d = make_dir();
    setup(&d);
    assert!(
        run(TableCommand::Export {
            dir: tdir(&d),
            format: ExportFormat::Csv
        })
        .is_ok()
    );
}

#[test]
fn export_json_with_rows() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 {
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
fn export_csv_with_rows() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 {
        ins(&d, i, i * 10);
    }
    assert!(
        run(TableCommand::Export {
            dir: tdir(&d),
            format: ExportFormat::Csv
        })
        .is_ok()
    );
}

#[test]
fn export_after_flush() {
    let d = make_dir();
    setup(&d);
    for i in 1..=4 {
        ins(&d, i, i * 3);
    }
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    assert!(
        run(TableCommand::Export {
            dir: tdir(&d),
            format: ExportFormat::Json
        })
        .is_ok()
    );
}

#[test]
fn export_after_compact() {
    let d = make_dir();
    setup(&d);
    for i in 1..=4 {
        ins(&d, i, i * 3);
    }
    run(TableCommand::Compact { dir: tdir(&d) }).unwrap();
    assert!(
        run(TableCommand::Export {
            dir: tdir(&d),
            format: ExportFormat::Csv
        })
        .is_ok()
    );
}

#[test]
fn import_from_file() {
    let d = make_dir();
    setup(&d);
    let file = d.path().join("data.json");
    std::fs::write(&file, "[]").unwrap();
    assert!(
        run(TableCommand::Import {
            dir: tdir(&d),
            file
        })
        .is_ok()
    );
}

#[test]
fn export_large_table_json() {
    let d = make_dir();
    setup(&d);
    for i in 1..=25 {
        ins(&d, i, i * 11);
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
fn export_large_table_csv() {
    let d = make_dir();
    setup(&d);
    for i in 1..=25 {
        ins(&d, i, i * 11);
    }
    assert!(
        run(TableCommand::Export {
            dir: tdir(&d),
            format: ExportFormat::Csv
        })
        .is_ok()
    );
}

#[test]
fn export_single_row_json() {
    let d = make_dir();
    setup(&d);
    ins(&d, 42, 420);
    assert!(
        run(TableCommand::Export {
            dir: tdir(&d),
            format: ExportFormat::Json
        })
        .is_ok()
    );
}

#[test]
fn export_after_delete() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 {
        ins(&d, i, i);
    }
    run(TableCommand::Delete {
        dir: tdir(&d),
        pk: 3,
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
fn export_after_update() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    run(TableCommand::Update {
        dir: tdir(&d),
        pk: 1,
        col: 1,
        val: 999,
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
fn import_empty_json() {
    let d = make_dir();
    setup(&d);
    let file = d.path().join("data.json");
    std::fs::write(&file, "[]").unwrap();
    assert!(
        run(TableCommand::Import {
            dir: tdir(&d),
            file
        })
        .is_ok()
    );
}
