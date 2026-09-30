/// Path-to-production item #1 (row-data + WAL encryption at rest), WAL half.
///
/// Mirrors wal_recovery.rs's crash-simulation pattern (drop the engine
/// without flushing, then re-open) but additionally proves the WAL bytes on
/// disk are genuinely ciphertext — not just that the feature round-trips.
use std::sync::Arc;

use tempfile::TempDir;
use vectorvolt_crypto::KeyStore;
use vectorvolt_engine::{Durability, EngineOptions, TableEngine};

fn open_encrypted(dir: &TempDir, key: Arc<KeyStore>) -> Arc<TableEngine> {
    Arc::new(
        TableEngine::open_with_encryption(
            vec!["pk".into(), "age".into(), "score".into()],
            dir.path(),
            EngineOptions::new(1_000_000).with_durability(Durability::WriteOnly),
            key,
        )
        .unwrap(),
    )
}

fn row(pk: u64, age: u64, score: u64) -> Vec<u64> {
    vec![pk, age, score]
}

fn wal_file_bytes(dir: &TempDir) -> Vec<u8> {
    let wal_dir = dir.path().join("wal");
    let mut bytes = Vec::new();
    for entry in std::fs::read_dir(&wal_dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_file() {
            bytes.extend(std::fs::read(&path).unwrap());
        }
    }
    bytes
}

#[test]
fn unflushed_encrypted_insert_survives_restart_with_same_key() {
    let dir = TempDir::new().unwrap();
    let key = Arc::new(KeyStore::generate().unwrap());
    {
        let e = open_encrypted(&dir, Arc::clone(&key));
        e.insert(1, row(1, 30, 100)).unwrap();
        // drop without flush — simulates a crash before the WAL is superseded
        // by a compacted partition.
    }

    let e = open_encrypted(&dir, key);
    let got = e.get(1).unwrap().unwrap();
    assert_eq!(got.values, row(1, 30, 100));
}

#[test]
fn wal_bytes_on_disk_are_not_plaintext() {
    let dir = TempDir::new().unwrap();
    let key = Arc::new(KeyStore::generate().unwrap());
    // A distinctive value unlikely to appear by chance in ciphertext, and
    // guaranteed to appear in the plaintext WAL encoding of this mutation
    // (Mutation::Insert little-endian-encodes every u64 value verbatim).
    let marker: u64 = 0xDEAD_BEEF_CAFE_F00D;
    {
        let e = open_encrypted(&dir, key);
        e.insert(7, vec![7, marker, 0]).unwrap();
        // Drop (out of scope) to ensure the WAL segment writer's buffer is
        // flushed to disk, matching wal_recovery.rs's established pattern.
    }

    let bytes = wal_file_bytes(&dir);
    let marker_le = marker.to_le_bytes();
    assert!(
        !bytes.windows(marker_le.len()).any(|w| w == marker_le),
        "marker value found in plaintext in the WAL file — encryption is not \
         actually being applied to the payload"
    );
}

#[test]
fn wrong_key_does_not_recover_data_but_also_does_not_panic() {
    let dir = TempDir::new().unwrap();
    let key_a = Arc::new(KeyStore::generate().unwrap());
    {
        let e = open_encrypted(&dir, key_a);
        e.insert(1, row(1, 30, 100)).unwrap();
    }

    // Reopening with a DIFFERENT key must not crash — every WAL frame fails
    // to decrypt, so replay silently skips them all (same as a torn/corrupt
    // frame today), leaving the table empty rather than corrupted.
    let key_b = Arc::new(KeyStore::generate().unwrap());
    let e = open_encrypted(&dir, key_b);
    assert!(e.get(1).unwrap().is_none());
}

#[test]
fn unencrypted_open_still_works_unchanged() {
    // Regression check: open_with (no key) must behave exactly as before —
    // this feature is additive/opt-in only.
    let dir = TempDir::new().unwrap();
    let e = Arc::new(
        TableEngine::open_with(
            vec!["pk".into(), "age".into(), "score".into()],
            dir.path(),
            EngineOptions::new(1_000_000).with_durability(Durability::WriteOnly),
        )
        .unwrap(),
    );
    e.insert(1, row(1, 30, 100)).unwrap();
    assert_eq!(e.get(1).unwrap().unwrap().values, row(1, 30, 100));
}
