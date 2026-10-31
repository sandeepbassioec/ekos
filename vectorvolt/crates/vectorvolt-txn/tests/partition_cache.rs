//! B25: point reads of flushed rows must not reopen and CRC-check the
//! partition file on every call.

use std::time::Instant;

use tempfile::TempDir;
use vectorvolt_txn::TxnManager;
use vectorvolt_wal::{WalConfig, WalWriter};

fn manager(dir: &TempDir) -> TxnManager {
    let wal_dir = dir.path().join("wal");
    std::fs::create_dir_all(&wal_dir).unwrap();
    let cfg = WalConfig::default().with_dir(&wal_dir).with_fsync(false);
    TxnManager::new(
        WalWriter::open(cfg).unwrap(),
        vec!["pk".into(), "v".into()],
        dir.path().join("data"),
        1_000_000,
    )
}

/// Quick micro-benchmark: repeated point reads of flushed rows. Prints the
/// per-read cost so before/after numbers can be compared; the assertion only
/// checks the reads are correct.
#[test]
fn flushed_point_reads_microbench() {
    let dir = TempDir::new().unwrap();
    let m = manager(&dir);
    const ROWS: u64 = 4096;
    for pk in 0..ROWS {
        m.insert(pk, vec![pk, pk * 3]).unwrap();
    }
    m.flush().unwrap();

    const READS: u64 = 20_000;
    let start = Instant::now();
    for i in 0..READS {
        let pk = (i * 7919) % ROWS;
        assert_eq!(m.get(pk).unwrap(), Some(vec![pk, pk * 3]));
    }
    let per_read = start.elapsed().as_nanos() as f64 / READS as f64;
    eprintln!("B25 flushed point read: {per_read:.0} ns/read over {READS} reads");
}
