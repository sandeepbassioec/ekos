//! Integration tests for Layer 153: vvctl wal export (VV-CTL-153).

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
fn export_empty_wal_table_ok() {
    let (_d, dir) = make_wal();
    assert!(
        run(WalCommand::Export {
            dir: dir.clone(),
            limit: None,
            output: ReplayOutput::Table
        })
        .is_ok()
    );
}

#[test]
fn export_empty_wal_json_ok() {
    let (_d, dir) = make_wal();
    assert!(
        run(WalCommand::Export {
            dir: dir.clone(),
            limit: None,
            output: ReplayOutput::Json
        })
        .is_ok()
    );
}

#[test]
fn export_frames_table_ok() {
    let (_d, dir) = make_wal();
    write_frames(&dir, 5);
    assert!(
        run(WalCommand::Export {
            dir: dir.clone(),
            limit: None,
            output: ReplayOutput::Table
        })
        .is_ok()
    );
}

#[test]
fn export_frames_json_ok() {
    let (_d, dir) = make_wal();
    write_frames(&dir, 5);
    assert!(
        run(WalCommand::Export {
            dir: dir.clone(),
            limit: None,
            output: ReplayOutput::Json
        })
        .is_ok()
    );
}

#[test]
fn export_with_limit_ok() {
    let (_d, dir) = make_wal();
    write_frames(&dir, 10);
    assert!(
        run(WalCommand::Export {
            dir: dir.clone(),
            limit: Some(3),
            output: ReplayOutput::Table
        })
        .is_ok()
    );
}

#[test]
fn export_limit_zero_ok() {
    let (_d, dir) = make_wal();
    write_frames(&dir, 5);
    assert!(
        run(WalCommand::Export {
            dir: dir.clone(),
            limit: Some(0),
            output: ReplayOutput::Table
        })
        .is_ok()
    );
}

#[test]
fn export_limit_exceeds_frames_ok() {
    let (_d, dir) = make_wal();
    write_frames(&dir, 3);
    assert!(
        run(WalCommand::Export {
            dir: dir.clone(),
            limit: Some(100),
            output: ReplayOutput::Table
        })
        .is_ok()
    );
}

#[test]
fn export_json_with_limit_ok() {
    let (_d, dir) = make_wal();
    write_frames(&dir, 8);
    assert!(
        run(WalCommand::Export {
            dir: dir.clone(),
            limit: Some(4),
            output: ReplayOutput::Json
        })
        .is_ok()
    );
}

#[test]
fn export_single_frame_ok() {
    let (_d, dir) = make_wal();
    write_frames(&dir, 1);
    assert!(
        run(WalCommand::Export {
            dir: dir.clone(),
            limit: None,
            output: ReplayOutput::Table
        })
        .is_ok()
    );
}

#[test]
fn export_missing_dir_errors() {
    assert!(
        run(WalCommand::Export {
            dir: PathBuf::from("/no/such/dir"),
            limit: None,
            output: ReplayOutput::Table
        })
        .is_err()
    );
}

#[test]
fn export_many_frames_ok() {
    let (_d, dir) = make_wal();
    write_frames(&dir, 20);
    assert!(
        run(WalCommand::Export {
            dir: dir.clone(),
            limit: None,
            output: ReplayOutput::Table
        })
        .is_ok()
    );
}

#[test]
fn export_repeated_calls_ok() {
    let (_d, dir) = make_wal();
    write_frames(&dir, 3);
    assert!(
        run(WalCommand::Export {
            dir: dir.clone(),
            limit: None,
            output: ReplayOutput::Table
        })
        .is_ok()
    );
    assert!(
        run(WalCommand::Export {
            dir: dir.clone(),
            limit: None,
            output: ReplayOutput::Json
        })
        .is_ok()
    );
}

#[test]
fn export_limit_one_ok() {
    let (_d, dir) = make_wal();
    write_frames(&dir, 5);
    assert!(
        run(WalCommand::Export {
            dir: dir.clone(),
            limit: Some(1),
            output: ReplayOutput::Json
        })
        .is_ok()
    );
}
