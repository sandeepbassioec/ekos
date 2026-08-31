//! Integration tests for Layer 148: vvctl wal verify with output flag (VV-CTL-148).

use std::path::{Path, PathBuf};
use tempfile::TempDir;
use vectorvolt_wal::{WalConfig, WalWriter};
use vvctl::commands::wal::{ReplayOutput, WalCommand, run};

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
}

#[test]
fn verify_empty_wal_table_ok() {
    let (_d, dir) = make_wal();
    assert!(
        run(WalCommand::Verify {
            dir: dir.to_string_lossy().into(),
            output: ReplayOutput::Table
        })
        .is_ok()
    );
}

#[test]
fn verify_empty_wal_json_ok() {
    let (_d, dir) = make_wal();
    assert!(
        run(WalCommand::Verify {
            dir: dir.to_string_lossy().into(),
            output: ReplayOutput::Json
        })
        .is_ok()
    );
}

#[test]
fn verify_with_frames_table_ok() {
    let (_d, dir) = make_wal();
    write_frames(&dir, 5);
    assert!(
        run(WalCommand::Verify {
            dir: dir.to_string_lossy().into(),
            output: ReplayOutput::Table
        })
        .is_ok()
    );
}

#[test]
fn verify_with_frames_json_ok() {
    let (_d, dir) = make_wal();
    write_frames(&dir, 5);
    assert!(
        run(WalCommand::Verify {
            dir: dir.to_string_lossy().into(),
            output: ReplayOutput::Json
        })
        .is_ok()
    );
}

#[test]
fn verify_single_frame_ok() {
    let (_d, dir) = make_wal();
    write_frames(&dir, 1);
    assert!(
        run(WalCommand::Verify {
            dir: dir.to_string_lossy().into(),
            output: ReplayOutput::Table
        })
        .is_ok()
    );
}

#[test]
fn verify_many_frames_ok() {
    let (_d, dir) = make_wal();
    write_frames(&dir, 20);
    assert!(
        run(WalCommand::Verify {
            dir: dir.to_string_lossy().into(),
            output: ReplayOutput::Table
        })
        .is_ok()
    );
}

#[test]
fn verify_json_many_frames_ok() {
    let (_d, dir) = make_wal();
    write_frames(&dir, 10);
    assert!(
        run(WalCommand::Verify {
            dir: dir.to_string_lossy().into(),
            output: ReplayOutput::Json
        })
        .is_ok()
    );
}

#[test]
fn verify_repeated_calls_ok() {
    let (_d, dir) = make_wal();
    write_frames(&dir, 3);
    assert!(
        run(WalCommand::Verify {
            dir: dir.to_string_lossy().into(),
            output: ReplayOutput::Table
        })
        .is_ok()
    );
    assert!(
        run(WalCommand::Verify {
            dir: dir.to_string_lossy().into(),
            output: ReplayOutput::Table
        })
        .is_ok()
    );
}

#[test]
fn verify_table_output_default() {
    let (_d, dir) = make_wal();
    write_frames(&dir, 2);
    let result = run(WalCommand::Verify {
        dir: dir.to_string_lossy().into(),
        output: ReplayOutput::Table,
    });
    assert!(result.is_ok());
}

#[test]
fn verify_json_output() {
    let (_d, dir) = make_wal();
    write_frames(&dir, 2);
    let result = run(WalCommand::Verify {
        dir: dir.to_string_lossy().into(),
        output: ReplayOutput::Json,
    });
    assert!(result.is_ok());
}

#[test]
fn verify_fresh_wal_no_frames_ok() {
    let (_d, dir) = make_wal();
    assert!(
        run(WalCommand::Verify {
            dir: dir.to_string_lossy().into(),
            output: ReplayOutput::Table
        })
        .is_ok()
    );
}

#[test]
fn verify_large_payload_frames_ok() {
    let (_d, dir) = make_wal();
    let cfg = WalConfig::default()
        .with_dir(dir.to_string_lossy().as_ref())
        .with_fsync(false);
    let writer = WalWriter::open(cfg).unwrap();
    writer.append(vec![0u8; 1024]).unwrap();
    assert!(
        run(WalCommand::Verify {
            dir: dir.to_string_lossy().into(),
            output: ReplayOutput::Table
        })
        .is_ok()
    );
}

#[test]
fn verify_table_succeeds_clean() {
    let (_d, dir) = make_wal();
    write_frames(&dir, 7);
    assert!(
        run(WalCommand::Verify {
            dir: dir.to_string_lossy().into(),
            output: ReplayOutput::Json
        })
        .is_ok()
    );
}
