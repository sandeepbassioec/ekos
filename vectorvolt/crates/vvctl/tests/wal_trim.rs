//! Integration tests for Layer 159: vvctl wal trim (VV-CTL-159).

use std::path::{Path, PathBuf};
use tempfile::TempDir;
use vectorvolt_wal::{WalConfig, WalWriter};
use vvctl::commands::wal::{WalCommand, run};

fn make_wal() -> (TempDir, PathBuf) {
    let d = TempDir::new().unwrap();
    let wal_dir = d.path().join("wal");
    std::fs::create_dir_all(&wal_dir).unwrap();
    (d, wal_dir)
}

fn write_frames(dir: &Path, n: usize) {
    let cfg = WalConfig::default()
        .with_dir(dir.to_string_lossy().as_ref())
        .with_fsync(false);
    let writer = WalWriter::open(cfg).unwrap();
    for i in 0..n {
        writer.append(format!("data{i}").into_bytes()).unwrap();
    }
    writer.flush().unwrap();
}

#[test]
fn trim_empty_wal_ok() {
    let (_d, dir) = make_wal();
    assert!(
        run(WalCommand::Trim {
            dir: dir.clone(),
            before: 100
        })
        .is_ok()
    );
}

#[test]
fn trim_missing_dir_errors() {
    assert!(
        run(WalCommand::Trim {
            dir: PathBuf::from("/no/such/dir"),
            before: 100
        })
        .is_err()
    );
}

#[test]
fn trim_before_zero_ok() {
    let (_d, dir) = make_wal();
    write_frames(&dir, 3);
    assert!(
        run(WalCommand::Trim {
            dir: dir.clone(),
            before: 0
        })
        .is_ok()
    );
}

#[test]
fn trim_before_max_deletes_all_ok() {
    let (_d, dir) = make_wal();
    write_frames(&dir, 3);
    assert!(
        run(WalCommand::Trim {
            dir: dir.clone(),
            before: u64::MAX
        })
        .is_ok()
    );
}

#[test]
fn trim_with_frames_ok() {
    let (_d, dir) = make_wal();
    write_frames(&dir, 5);
    assert!(
        run(WalCommand::Trim {
            dir: dir.clone(),
            before: 1
        })
        .is_ok()
    );
}

#[test]
fn trim_repeated_calls_ok() {
    let (_d, dir) = make_wal();
    write_frames(&dir, 5);
    assert!(
        run(WalCommand::Trim {
            dir: dir.clone(),
            before: 1
        })
        .is_ok()
    );
    assert!(
        run(WalCommand::Trim {
            dir: dir.clone(),
            before: 2
        })
        .is_ok()
    );
}

#[test]
fn trim_large_before_ok() {
    let (_d, dir) = make_wal();
    write_frames(&dir, 3);
    assert!(
        run(WalCommand::Trim {
            dir: dir.clone(),
            before: 999999
        })
        .is_ok()
    );
}

#[test]
fn trim_dir_exists_ok() {
    let (_d, dir) = make_wal();
    assert!(
        run(WalCommand::Trim {
            dir: dir.clone(),
            before: 1
        })
        .is_ok()
    );
    assert!(dir.exists());
}

#[test]
fn trim_zero_frames_written_ok() {
    let (_d, dir) = make_wal();
    assert!(
        run(WalCommand::Trim {
            dir: dir.clone(),
            before: 10
        })
        .is_ok()
    );
}

#[test]
fn trim_before_one_ok() {
    let (_d, dir) = make_wal();
    write_frames(&dir, 2);
    assert!(
        run(WalCommand::Trim {
            dir: dir.clone(),
            before: 1
        })
        .is_ok()
    );
}

#[test]
fn trim_single_frame_ok() {
    let (_d, dir) = make_wal();
    write_frames(&dir, 1);
    assert!(
        run(WalCommand::Trim {
            dir: dir.clone(),
            before: u64::MAX
        })
        .is_ok()
    );
}

#[test]
fn trim_many_segments_ok() {
    let (_d, dir) = make_wal();
    write_frames(&dir, 10);
    assert!(
        run(WalCommand::Trim {
            dir: dir.clone(),
            before: 5
        })
        .is_ok()
    );
}

#[test]
fn trim_before_far_future_ok() {
    let (_d, dir) = make_wal();
    write_frames(&dir, 4);
    assert!(
        run(WalCommand::Trim {
            dir: dir.clone(),
            before: 10000
        })
        .is_ok()
    );
}
