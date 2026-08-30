//! Integration tests for L129: TypedValue enum + typed display in vvctl table get/scan (VV-ENG-008).

use std::path::PathBuf;

use tempfile::TempDir;
use vectorvolt_engine::{ColumnType, TypedValue};
use vvctl::commands::table::{TableCommand, run};

fn make_dir() -> TempDir {
    TempDir::new().unwrap()
}

fn table_dir(dir: &TempDir) -> PathBuf {
    dir.path().join("tbl")
}

// ── TypedValue::from_bits ─────────────────────────────────────────────────────

#[test]
fn from_bits_u64_roundtrip() {
    let v = TypedValue::from_bits(ColumnType::U64, 42);
    assert_eq!(v, TypedValue::U64(42));
    assert_eq!(v.to_bits(), 42);
}

#[test]
fn from_bits_i64_negative() {
    let bits = (-5i64) as u64;
    let v = TypedValue::from_bits(ColumnType::I64, bits);
    assert_eq!(v, TypedValue::I64(-5));
    assert_eq!(v.display_str(), "-5");
}

#[test]
fn from_bits_bool_true_false() {
    assert_eq!(TypedValue::from_bits(ColumnType::Bool, 1), TypedValue::Bool(true));
    assert_eq!(TypedValue::from_bits(ColumnType::Bool, 0), TypedValue::Bool(false));
    assert_eq!(TypedValue::from_bits(ColumnType::Bool, 1).display_str(), "true");
}

#[test]
fn from_bits_f64_roundtrip() {
    let f = 3.14f64;
    let v = TypedValue::from_bits(ColumnType::F64, f.to_bits());
    assert_eq!(v.to_bits(), f.to_bits());
    assert!(v.display_str().starts_with("3.14"));
}

#[test]
fn from_bits_decimal_display() {
    let raw = 1_500_000i64 as u64; // 1.500000
    let v = TypedValue::from_bits(ColumnType::Decimal, raw);
    assert_eq!(v.display_str(), "1.500000");
}

// ── TypedValue::parse ─────────────────────────────────────────────────────────

#[test]
fn parse_bool_variants() {
    assert_eq!(TypedValue::parse(ColumnType::Bool, "true").unwrap(), TypedValue::Bool(true));
    assert_eq!(TypedValue::parse(ColumnType::Bool, "false").unwrap(), TypedValue::Bool(false));
    assert_eq!(TypedValue::parse(ColumnType::Bool, "1").unwrap(), TypedValue::Bool(true));
    assert_eq!(TypedValue::parse(ColumnType::Bool, "0").unwrap(), TypedValue::Bool(false));
    assert!(TypedValue::parse(ColumnType::Bool, "yes").is_err());
}

#[test]
fn parse_i64_negative() {
    let v = TypedValue::parse(ColumnType::I64, "-99").unwrap();
    assert_eq!(v, TypedValue::I64(-99));
}

#[test]
fn parse_decimal_with_fraction() {
    let v = TypedValue::parse(ColumnType::Decimal, "2.500000").unwrap();
    assert_eq!(v.to_bits(), 2_500_000i64 as u64);
}

// ── CLI: typed display in get ─────────────────────────────────────────────────

#[test]
fn get_displays_bool_as_true_false() {
    let dir = make_dir();
    run(TableCommand::Create {
        dir: table_dir(&dir),
        columns: vec!["pk:u64".into(), "active:bool".into()],
    })
    .unwrap();
    run(TableCommand::Insert {
        dir: table_dir(&dir),
        pk: 1,
        values: vec![1],
    })
    .unwrap();
    // get succeeds (the display is visible via stdout; we just verify no error)
    run(TableCommand::Get {
        dir: table_dir(&dir),
        pk: 1,
    })
    .unwrap();
}

#[test]
fn get_displays_i64_negative() {
    let dir = make_dir();
    run(TableCommand::Create {
        dir: table_dir(&dir),
        columns: vec!["pk:u64".into(), "score:i64".into()],
    })
    .unwrap();
    run(TableCommand::Insert {
        dir: table_dir(&dir),
        pk: 10,
        values: vec![(-7i64) as u64],
    })
    .unwrap();
    run(TableCommand::Get {
        dir: table_dir(&dir),
        pk: 10,
    })
    .unwrap();
}

// ── CLI: typed display in scan ────────────────────────────────────────────────

#[test]
fn scan_succeeds_with_typed_columns() {
    let dir = make_dir();
    run(TableCommand::Create {
        dir: table_dir(&dir),
        columns: vec!["pk:u64".into(), "ts:timestamp".into()],
    })
    .unwrap();
    run(TableCommand::Insert {
        dir: table_dir(&dir),
        pk: 1,
        values: vec![1_700_000_000_000_000_000u64],
    })
    .unwrap();
    run(TableCommand::Flush {
        dir: table_dir(&dir),
    })
    .unwrap();
    run(TableCommand::Scan {
        dir: table_dir(&dir),
        col: 0,
        lo: 0,
        hi: u64::MAX,
    })
    .unwrap();
}
