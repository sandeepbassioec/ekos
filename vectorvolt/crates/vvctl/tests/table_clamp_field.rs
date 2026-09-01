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
fn clamp_value_above_hi() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 200);
    assert!(
        run(TableCommand::ClampField {
            dir: tdir(&d),
            pk: 1,
            col: 1,
            lo: 0,
            hi: 100
        })
        .is_ok()
    );
}

#[test]
fn clamp_value_below_lo() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 5);
    assert!(
        run(TableCommand::ClampField {
            dir: tdir(&d),
            pk: 1,
            col: 1,
            lo: 10,
            hi: 100
        })
        .is_ok()
    );
}

#[test]
fn clamp_value_within_range() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 50);
    assert!(
        run(TableCommand::ClampField {
            dir: tdir(&d),
            pk: 1,
            col: 1,
            lo: 0,
            hi: 100
        })
        .is_ok()
    );
}

#[test]
fn clamp_to_exact_lo() {
    let d = make_dir();
    setup(&d);
    ins(&d, 2, 0);
    assert!(
        run(TableCommand::ClampField {
            dir: tdir(&d),
            pk: 2,
            col: 1,
            lo: 10,
            hi: 100
        })
        .is_ok()
    );
}

#[test]
fn clamp_to_exact_hi() {
    let d = make_dir();
    setup(&d);
    ins(&d, 3, 999);
    assert!(
        run(TableCommand::ClampField {
            dir: tdir(&d),
            pk: 3,
            col: 1,
            lo: 0,
            hi: 100
        })
        .is_ok()
    );
}

#[test]
fn clamp_nonexistent_pk() {
    let d = make_dir();
    setup(&d);
    assert!(
        run(TableCommand::ClampField {
            dir: tdir(&d),
            pk: 999,
            col: 1,
            lo: 0,
            hi: 100
        })
        .is_err()
    );
}

#[test]
fn clamp_then_get() {
    let d = make_dir();
    setup(&d);
    ins(&d, 5, 500);
    run(TableCommand::ClampField {
        dir: tdir(&d),
        pk: 5,
        col: 1,
        lo: 0,
        hi: 100,
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
fn clamp_multiple_rows() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 1000);
    ins(&d, 2, 5);
    ins(&d, 3, 50);
    for pk in 1..=3 {
        assert!(
            run(TableCommand::ClampField {
                dir: tdir(&d),
                pk,
                col: 1,
                lo: 10,
                hi: 100
            })
            .is_ok()
        );
    }
}

#[test]
fn clamp_zero_range() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 50);
    assert!(
        run(TableCommand::ClampField {
            dir: tdir(&d),
            pk: 1,
            col: 1,
            lo: 50,
            hi: 50
        })
        .is_ok()
    );
}

#[test]
fn clamp_after_update() {
    let d = make_dir();
    setup(&d);
    ins(&d, 4, 40);
    run(TableCommand::Update {
        dir: tdir(&d),
        pk: 4,
        col: 1,
        val: 400,
    })
    .unwrap();
    assert!(
        run(TableCommand::ClampField {
            dir: tdir(&d),
            pk: 4,
            col: 1,
            lo: 0,
            hi: 100
        })
        .is_ok()
    );
}

#[test]
fn clamp_then_scan() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 {
        ins(&d, i, i * 50);
    }
    for i in 1..=5 {
        run(TableCommand::ClampField {
            dir: tdir(&d),
            pk: i,
            col: 1,
            lo: 0,
            hi: 100,
        })
        .unwrap();
    }
    assert!(
        run(TableCommand::Scan {
            dir: tdir(&d),
            col: 1,
            lo: 0,
            hi: 100,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn clamp_pk_column() {
    let d = make_dir();
    setup(&d);
    ins(&d, 10, 100);
    assert!(
        run(TableCommand::ClampField {
            dir: tdir(&d),
            pk: 10,
            col: 0,
            lo: 5,
            hi: 15
        })
        .is_err()
    );
}

#[test]
fn clamp_after_flush() {
    let d = make_dir();
    setup(&d);
    ins(&d, 6, 600);
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    assert!(
        run(TableCommand::ClampField {
            dir: tdir(&d),
            pk: 6,
            col: 1,
            lo: 0,
            hi: 100
        })
        .is_ok()
    );
}
