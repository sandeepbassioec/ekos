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
fn ins(d: &TempDir, pk: u64, val: u64) {
    run(TableCommand::Insert {
        dir: tdir(d),
        pk,
        values: vec![val],
    })
    .unwrap();
}

#[test]
fn set_field_add_basic() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    assert!(
        run(TableCommand::SetFieldAdd {
            dir: tdir(&d),
            pk: 1,
            col: 1,
            addend: 5
        })
        .is_ok()
    );
}

#[test]
fn set_field_add_zero() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    assert!(
        run(TableCommand::SetFieldAdd {
            dir: tdir(&d),
            pk: 1,
            col: 1,
            addend: 0
        })
        .is_ok()
    );
}

#[test]
fn set_field_add_large() {
    let d = make_dir();
    setup(&d);
    ins(&d, 2, 0);
    assert!(
        run(TableCommand::SetFieldAdd {
            dir: tdir(&d),
            pk: 2,
            col: 1,
            addend: 1000
        })
        .is_ok()
    );
}

#[test]
fn set_field_add_then_get() {
    let d = make_dir();
    setup(&d);
    ins(&d, 3, 50);
    run(TableCommand::SetFieldAdd {
        dir: tdir(&d),
        pk: 3,
        col: 1,
        addend: 25,
    })
    .unwrap();
    assert!(
        run(TableCommand::Get {
            dir: tdir(&d),
            pk: 3,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn set_field_mul_basic() {
    let d = make_dir();
    setup(&d);
    ins(&d, 4, 5);
    assert!(
        run(TableCommand::SetFieldMul {
            dir: tdir(&d),
            pk: 4,
            col: 1,
            factor: 3
        })
        .is_ok()
    );
}

#[test]
fn set_field_mul_by_zero() {
    let d = make_dir();
    setup(&d);
    ins(&d, 5, 100);
    assert!(
        run(TableCommand::SetFieldMul {
            dir: tdir(&d),
            pk: 5,
            col: 1,
            factor: 0
        })
        .is_ok()
    );
}

#[test]
fn set_field_mul_by_one() {
    let d = make_dir();
    setup(&d);
    ins(&d, 6, 42);
    assert!(
        run(TableCommand::SetFieldMul {
            dir: tdir(&d),
            pk: 6,
            col: 1,
            factor: 1
        })
        .is_ok()
    );
}

#[test]
fn set_field_mul_then_get() {
    let d = make_dir();
    setup(&d);
    ins(&d, 7, 10);
    run(TableCommand::SetFieldMul {
        dir: tdir(&d),
        pk: 7,
        col: 1,
        factor: 4,
    })
    .unwrap();
    assert!(
        run(TableCommand::Get {
            dir: tdir(&d),
            pk: 7,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn add_then_mul() {
    let d = make_dir();
    setup(&d);
    ins(&d, 8, 10);
    run(TableCommand::SetFieldAdd {
        dir: tdir(&d),
        pk: 8,
        col: 1,
        addend: 10,
    })
    .unwrap();
    assert!(
        run(TableCommand::SetFieldMul {
            dir: tdir(&d),
            pk: 8,
            col: 1,
            factor: 2
        })
        .is_ok()
    );
}

#[test]
fn mul_then_add() {
    let d = make_dir();
    setup(&d);
    ins(&d, 9, 5);
    run(TableCommand::SetFieldMul {
        dir: tdir(&d),
        pk: 9,
        col: 1,
        factor: 3,
    })
    .unwrap();
    assert!(
        run(TableCommand::SetFieldAdd {
            dir: tdir(&d),
            pk: 9,
            col: 1,
            addend: 1
        })
        .is_ok()
    );
}

#[test]
fn set_field_add_nonexistent_pk() {
    let d = make_dir();
    setup(&d);
    assert!(
        run(TableCommand::SetFieldAdd {
            dir: tdir(&d),
            pk: 999,
            col: 1,
            addend: 5
        })
        .is_err()
    );
}

#[test]
fn set_field_mul_nonexistent_pk() {
    let d = make_dir();
    setup(&d);
    assert!(
        run(TableCommand::SetFieldMul {
            dir: tdir(&d),
            pk: 999,
            col: 1,
            factor: 2
        })
        .is_err()
    );
}

#[test]
fn set_field_add_after_flush() {
    let d = make_dir();
    setup(&d);
    ins(&d, 10, 100);
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    assert!(
        run(TableCommand::SetFieldAdd {
            dir: tdir(&d),
            pk: 10,
            col: 1,
            addend: 50
        })
        .is_ok()
    );
}
