//! Tests for Layer 296: TypedInsert value roundtrip (VV-CTL-296).

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
fn typed_insert_u64_ok() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: tdir(&d),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    assert!(
        run(TableCommand::TypedInsert {
            dir: tdir(&d),
            pk: "1".into(),
            values: vec!["42".into()]
        })
        .is_ok()
    );
}

#[test]
fn typed_insert_i64_ok() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: tdir(&d),
        columns: vec!["pk:u64".into(), "score:i64".into()],
    })
    .unwrap();
    assert!(
        run(TableCommand::TypedInsert {
            dir: tdir(&d),
            pk: "1".into(),
            values: vec!["-100".into()]
        })
        .is_ok()
    );
}

#[test]
fn typed_insert_f64_ok() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: tdir(&d),
        columns: vec!["pk:u64".into(), "ratio:f64".into()],
    })
    .unwrap();
    assert!(
        run(TableCommand::TypedInsert {
            dir: tdir(&d),
            pk: "1".into(),
            values: vec!["3.14".into()]
        })
        .is_ok()
    );
}

#[test]
fn typed_insert_bool_ok() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: tdir(&d),
        columns: vec!["pk:u64".into(), "flag:bool".into()],
    })
    .unwrap();
    assert!(
        run(TableCommand::TypedInsert {
            dir: tdir(&d),
            pk: "1".into(),
            values: vec!["true".into()]
        })
        .is_ok()
    );
}

#[test]
fn typed_insert_roundtrip_get_ok() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: tdir(&d),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    run(TableCommand::TypedInsert {
        dir: tdir(&d),
        pk: "5".into(),
        values: vec!["99".into()],
    })
    .unwrap();
    assert!(
        run(TableCommand::Get {
            dir: tdir(&d),
            pk: 5,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn typed_insert_missing_dir_errors() {
    let d = make_dir();
    assert!(
        run(TableCommand::TypedInsert {
            dir: d.path().join("nope"),
            pk: "1".into(),
            values: vec!["1".into()]
        })
        .is_err()
    );
}

#[test]
fn typed_insert_multiple_cols_ok() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: tdir(&d),
        columns: vec!["pk:u64".into(), "a:u64".into(), "b:u64".into()],
    })
    .unwrap();
    assert!(
        run(TableCommand::TypedInsert {
            dir: tdir(&d),
            pk: "1".into(),
            values: vec!["10".into(), "20".into()]
        })
        .is_ok()
    );
}

#[test]
fn typed_insert_then_count_ok() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: tdir(&d),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    run(TableCommand::TypedInsert {
        dir: tdir(&d),
        pk: "1".into(),
        values: vec!["10".into()],
    })
    .unwrap();
    run(TableCommand::TypedInsert {
        dir: tdir(&d),
        pk: "2".into(),
        values: vec!["20".into()],
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
fn typed_insert_pk0_ok() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: tdir(&d),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    assert!(
        run(TableCommand::TypedInsert {
            dir: tdir(&d),
            pk: "0".into(),
            values: vec!["0".into()]
        })
        .is_ok()
    );
}

#[test]
fn typed_insert_after_flush_ok() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: tdir(&d),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    run(TableCommand::TypedInsert {
        dir: tdir(&d),
        pk: "1".into(),
        values: vec!["10".into()],
    })
    .unwrap();
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    assert!(
        run(TableCommand::TypedInsert {
            dir: tdir(&d),
            pk: "2".into(),
            values: vec!["20".into()]
        })
        .is_ok()
    );
}

#[test]
fn typed_insert_scan_ok() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: tdir(&d),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    for i in 1u64..=5 {
        run(TableCommand::TypedInsert {
            dir: tdir(&d),
            pk: i.to_string(),
            values: vec![i.to_string()],
        })
        .unwrap();
    }
    assert!(
        run(TableCommand::Scan {
            dir: tdir(&d),
            col: 1,
            lo: 1,
            hi: 50,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn typed_insert_large_u64_ok() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: tdir(&d),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    assert!(
        run(TableCommand::TypedInsert {
            dir: tdir(&d),
            pk: "1000000000".into(),
            values: vec!["999999999".into()]
        })
        .is_ok()
    );
}

#[test]
fn typed_insert_json_output_ok() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: tdir(&d),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    run(TableCommand::TypedInsert {
        dir: tdir(&d),
        pk: "7".into(),
        values: vec!["77".into()],
    })
    .unwrap();
    assert!(
        run(TableCommand::Get {
            dir: tdir(&d),
            pk: 7,
            output: OutputFormat::Json
        })
        .is_ok()
    );
}
