//! End-to-end WAL integration tests.

use tempfile::TempDir;
use vectorvolt_wal::{WalConfig, WalReader, WalWriter, GroupCommitBatcher, WalCursor};
use tokio::time::Duration;

fn cfg(dir: &std::path::Path) -> WalConfig {
    WalConfig::default().with_dir(dir).with_fsync(false)
}

#[test]
fn write_read_verify() {
    let dir = TempDir::new().unwrap();
    let writer = WalWriter::open(cfg(dir.path())).unwrap();

    let payloads: Vec<Vec<u8>> = (0u32..50)
        .map(|i| format!("integration-payload-{:04}", i).into_bytes())
        .collect();

    for p in &payloads {
        writer.append(p.clone()).unwrap();
    }
    writer.flush().unwrap();

    let reader = WalReader::open(cfg(dir.path()));
    let frames = reader.read_all().unwrap();

    assert_eq!(frames.len(), 50);
    for (i, f) in frames.iter().enumerate() {
        assert_eq!(f.sequence, (i + 1) as u64);
        assert_eq!(f.payload, payloads[i]);
    }

    // Chain must verify clean.
    let verified = reader.verify().unwrap();
    assert_eq!(verified, 50);
}

#[test]
fn crash_recovery() {
    let dir = TempDir::new().unwrap();

    // First writer: write 30 frames, simulate crash (no explicit close).
    {
        let writer = WalWriter::open(cfg(dir.path())).unwrap();
        for i in 0u32..30 {
            writer.append(format!("before-crash-{}", i).into_bytes()).unwrap();
        }
        writer.flush().unwrap();
        // drop without explicit close = simulated crash
    }

    // Second writer: opens same directory, should recover and continue from seq=31.
    {
        let writer = WalWriter::open(cfg(dir.path())).unwrap();
        let tip = writer.tip();
        assert_eq!(tip.sequence, 30, "recovery should find seq 30 as last written");

        writer.append(b"after-crash".to_vec()).unwrap();
        writer.flush().unwrap();

        let tip2 = writer.tip();
        assert_eq!(tip2.sequence, 31);
    }

    // Read everything: should have 31 frames with unbroken chain.
    let reader = WalReader::open(cfg(dir.path()));
    let frames = reader.read_all().unwrap();
    assert_eq!(frames.len(), 31);
    assert_eq!(frames[30].payload, b"after-crash");

    let verified = reader.verify().unwrap();
    assert_eq!(verified, 31);
}

#[test]
fn multi_segment_verify() {
    let dir = TempDir::new().unwrap();
    // Force rotation every ~2 frames.
    let cfg_small = WalConfig::default()
        .with_dir(dir.path())
        .with_fsync(false)
        .with_segment_max_bytes(512);

    let writer = WalWriter::open(cfg_small.clone()).unwrap();
    for i in 0u32..100 {
        writer.append(format!("payload-{:04}", i).into_bytes()).unwrap();
    }
    writer.flush().unwrap();

    let reader = WalReader::open(cfg_small);
    let verified = reader.verify().unwrap();
    assert_eq!(verified, 100);
}

#[tokio::test]
async fn group_commit_throughput() {
    let dir = TempDir::new().unwrap();
    let writer = WalWriter::open(cfg(dir.path())).unwrap();

    let batcher = GroupCommitBatcher::start(
        writer.clone(),
        Duration::from_micros(500),
        4 * 1024 * 1024,
        8192,
    );

    let n = 1000u32;
    let mut handles = Vec::with_capacity(n as usize);
    for i in 0..n {
        let f = batcher.submit(i.to_le_bytes().to_vec()).unwrap();
        handles.push(tokio::spawn(async move { f.wait().await.unwrap() }));
    }

    let mut results = Vec::with_capacity(n as usize);
    for h in handles {
        results.push(h.await.unwrap());
    }
    assert_eq!(results.len(), n as usize);

    // All sequences are unique and in 1..=1000.
    let seqs: std::collections::BTreeSet<u64> = results.iter().map(|t| t.sequence).collect();
    assert_eq!(seqs.len(), n as usize);
    assert_eq!(*seqs.iter().next().unwrap(), 1);
    assert_eq!(*seqs.iter().next_back().unwrap(), n as u64);
}
