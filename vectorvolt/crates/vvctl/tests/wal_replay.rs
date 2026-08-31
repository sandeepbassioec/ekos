//! Integration tests for Layer 136: vvctl wal replay (VV-WAL-012).

use std::path::PathBuf;
use tempfile::TempDir;
use vectorvolt_wal::{WalConfig, WalWriter};
use vvctl::commands::wal::{ReplayOutput, WalCommand, cmd_replay};

fn make_dir() -> TempDir {
    TempDir::new().unwrap()
}

/// Create a WAL with `n` frames in `dir` and return the wal subdir path.
fn write_wal_frames(dir: &TempDir, n: usize) -> PathBuf {
    let wal_dir = dir.path().join("wal");
    std::fs::create_dir_all(&wal_dir).unwrap();
    let cfg = WalConfig::default()
        .with_dir(wal_dir.to_str().unwrap())
        .with_fsync(false);
    let writer = WalWriter::open(cfg).unwrap();
    for i in 0..n {
        writer.append(format!("record-{}", i).into_bytes()).unwrap();
    }
    writer.flush().unwrap();
    wal_dir
}

// â”€â”€ 1. Missing dir errors â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn replay_missing_dir_errors() {
    let dir = PathBuf::from("/nonexistent/wal/path/that/does/not/exist");
    let result = cmd_replay(&dir, 0, &ReplayOutput::Table);
    assert!(result.is_err(), "expected error for missing dir");
}

// â”€â”€ 2. Empty dir succeeds with 0 records â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn replay_empty_dir_succeeds() {
    let tmp = make_dir();
    let wal_dir = tmp.path().join("wal");
    std::fs::create_dir_all(&wal_dir).unwrap();
    cmd_replay(&wal_dir, 0, &ReplayOutput::Table).unwrap();
}

// â”€â”€ 3. Table output format succeeds â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn replay_table_output_succeeds() {
    let tmp = make_dir();
    let wal_dir = write_wal_frames(&tmp, 5);
    cmd_replay(&wal_dir, 0, &ReplayOutput::Table).unwrap();
}

// â”€â”€ 4. JSON output format succeeds â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn replay_json_output_succeeds() {
    let tmp = make_dir();
    let wal_dir = write_wal_frames(&tmp, 5);
    cmd_replay(&wal_dir, 0, &ReplayOutput::Json).unwrap();
}

// â”€â”€ 5. Limit 0 reads all records â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn replay_limit_zero_reads_all() {
    let tmp = make_dir();
    let wal_dir = write_wal_frames(&tmp, 10);
    cmd_replay(&wal_dir, 0, &ReplayOutput::Table).unwrap();
}

// â”€â”€ 6. With segments after writes returns Ok â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn replay_with_segments_after_writes() {
    let tmp = make_dir();
    let wal_dir = write_wal_frames(&tmp, 3);
    cmd_replay(&wal_dir, 0, &ReplayOutput::Table).unwrap();
}

// â”€â”€ 7. Single record insert, table output â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn replay_one_record_table() {
    let tmp = make_dir();
    let wal_dir = write_wal_frames(&tmp, 1);
    cmd_replay(&wal_dir, 0, &ReplayOutput::Table).unwrap();
}

// â”€â”€ 8. Single record insert, json output â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn replay_one_record_json() {
    let tmp = make_dir();
    let wal_dir = write_wal_frames(&tmp, 1);
    cmd_replay(&wal_dir, 0, &ReplayOutput::Json).unwrap();
}

// â”€â”€ 9. Limit = 1 caps reads â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn replay_limit_one_succeeds() {
    let tmp = make_dir();
    let wal_dir = write_wal_frames(&tmp, 20);
    cmd_replay(&wal_dir, 1, &ReplayOutput::Table).unwrap();
}

// â”€â”€ 10. Limit = 100, fewer records available â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn replay_limit_100_with_fewer_records() {
    let tmp = make_dir();
    let wal_dir = write_wal_frames(&tmp, 5);
    cmd_replay(&wal_dir, 100, &ReplayOutput::Table).unwrap();
}

// â”€â”€ 11. Many records, table output â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn replay_many_records_table() {
    let tmp = make_dir();
    let wal_dir = write_wal_frames(&tmp, 50);
    cmd_replay(&wal_dir, 0, &ReplayOutput::Table).unwrap();
}

// â”€â”€ 12. Many records, json output â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn replay_many_records_json() {
    let tmp = make_dir();
    let wal_dir = write_wal_frames(&tmp, 50);
    cmd_replay(&wal_dir, 0, &ReplayOutput::Json).unwrap();
}

// â”€â”€ 13. WalCommand::Replay variant dispatches correctly â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn replay_via_run_dispatch() {
    let tmp = make_dir();
    let wal_dir = write_wal_frames(&tmp, 4);
    let cmd = WalCommand::Replay {
        dir: wal_dir,
        limit: 0,
        output: ReplayOutput::Table,
    };
    vvctl::commands::wal::run(cmd).unwrap();
}
