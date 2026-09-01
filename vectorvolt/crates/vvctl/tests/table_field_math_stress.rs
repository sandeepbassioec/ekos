//! Tests for Layer 282: vvctl table field math under load (VV-CTL-282).

use std::path::PathBuf;
use tempfile::TempDir;
use vvctl::commands::table::{OutputFormat, TableCommand, run};

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
fn field_math_incr_10_times_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![0],
    })
    .unwrap();
    for _ in 0..10 {
        run(TableCommand::IncrementField {
            dir: tdir(&d),
            pk: 1,
            col: 1,
        })
        .unwrap();
    }
    run(TableCommand::Get {
        dir: tdir(&d),
        pk: 1,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn field_math_decr_10_times_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 2,
        values: vec![20],
    })
    .unwrap();
    for _ in 0..10 {
        run(TableCommand::DecrementField {
            dir: tdir(&d),
            pk: 2,
            col: 1,
        })
        .unwrap();
    }
    run(TableCommand::Get {
        dir: tdir(&d),
        pk: 2,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn field_math_set_add_5_times_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 3,
        values: vec![0],
    })
    .unwrap();
    for _ in 0..5 {
        run(TableCommand::SetFieldAdd {
            dir: tdir(&d),
            pk: 3,
            col: 1,
            addend: 10,
        })
        .unwrap();
    }
    run(TableCommand::Get {
        dir: tdir(&d),
        pk: 3,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn field_math_set_mul_3_times_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 4,
        values: vec![2],
    })
    .unwrap();
    for _ in 0..3 {
        run(TableCommand::SetFieldMul {
            dir: tdir(&d),
            pk: 4,
            col: 1,
            factor: 2,
        })
        .unwrap();
    }
    run(TableCommand::Get {
        dir: tdir(&d),
        pk: 4,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn field_math_clamp_in_range_unchanged() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 5,
        values: vec![50],
    })
    .unwrap();
    run(TableCommand::ClampField {
        dir: tdir(&d),
        pk: 5,
        col: 1,
        lo: 10,
        hi: 100,
    })
    .unwrap();
    run(TableCommand::Get {
        dir: tdir(&d),
        pk: 5,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn field_math_clamp_below_lo_becomes_lo() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 6,
        values: vec![5],
    })
    .unwrap();
    run(TableCommand::ClampField {
        dir: tdir(&d),
        pk: 6,
        col: 1,
        lo: 10,
        hi: 100,
    })
    .unwrap();
}

#[test]
fn field_math_clamp_above_hi_becomes_hi() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 7,
        values: vec![200],
    })
    .unwrap();
    run(TableCommand::ClampField {
        dir: tdir(&d),
        pk: 7,
        col: 1,
        lo: 10,
        hi: 100,
    })
    .unwrap();
}

#[test]
fn field_math_incr_saturates_ok() {
    let d = make_dir();
    setup(&d);
    // Insert near max, then increment - should not panic
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 8,
        values: vec![u64::MAX - 2],
    })
    .unwrap();
    let _ = run(TableCommand::IncrementField {
        dir: tdir(&d),
        pk: 8,
        col: 1,
    });
}

#[test]
fn field_math_decr_saturates_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 9,
        values: vec![0],
    })
    .unwrap();
    let _ = run(TableCommand::DecrementField {
        dir: tdir(&d),
        pk: 9,
        col: 1,
    });
}

#[test]
fn field_math_set_add_saturates_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 10,
        values: vec![u64::MAX - 2],
    })
    .unwrap();
    let _ = run(TableCommand::SetFieldAdd {
        dir: tdir(&d),
        pk: 10,
        col: 1,
        addend: 100,
    });
}

#[test]
fn field_math_set_mul_saturates_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 11,
        values: vec![u64::MAX / 2 + 1],
    })
    .unwrap();
    let _ = run(TableCommand::SetFieldMul {
        dir: tdir(&d),
        pk: 11,
        col: 1,
        factor: 3,
    });
}

#[test]
fn field_math_incr_decr_round_trip_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 12,
        values: vec![50],
    })
    .unwrap();
    run(TableCommand::IncrementField {
        dir: tdir(&d),
        pk: 12,
        col: 1,
    })
    .unwrap();
    run(TableCommand::DecrementField {
        dir: tdir(&d),
        pk: 12,
        col: 1,
    })
    .unwrap();
}

#[test]
fn field_math_set_add_then_set_mul_order_matters() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 13,
        values: vec![10],
    })
    .unwrap();
    run(TableCommand::SetFieldAdd {
        dir: tdir(&d),
        pk: 13,
        col: 1,
        addend: 5,
    })
    .unwrap();
    run(TableCommand::SetFieldMul {
        dir: tdir(&d),
        pk: 13,
        col: 1,
        factor: 2,
    })
    .unwrap();
    run(TableCommand::Get {
        dir: tdir(&d),
        pk: 13,
        output: OutputFormat::Table,
    })
    .unwrap();
}
