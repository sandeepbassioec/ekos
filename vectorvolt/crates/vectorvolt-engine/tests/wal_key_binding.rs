//! PP-52 / C2: segment headers record the WAL encryption key id, a wrong or
//! missing key fails loudly before replay, and every encrypted payload is
//! bound to its sequence number (AAD), so a frame copied over another one no
//! longer decrypts.
use std::path::Path;
use std::sync::Arc;

use tempfile::TempDir;
use vectorvolt_crypto::KeyStore;
use vectorvolt_engine::{Durability, EngineError, EngineOptions, TableEngine};
use vectorvolt_txn::ops::Mutation;
use vectorvolt_wal::segment::{SegmentHeader, SegmentReader, key_id_hex};
use vectorvolt_wal::{Frame, WalConfig, WalWriter, list_segment_ids, segment_path};

fn schema() -> Vec<String> {
    vec!["pk".into(), "age".into(), "score".into()]
}

fn opts() -> EngineOptions {
    EngineOptions::new(1_000_000).with_durability(Durability::WriteOnly)
}

fn open_keyed(dir: &Path, key: &Arc<KeyStore>) -> Result<TableEngine, EngineError> {
    TableEngine::open_with_encryption(schema(), dir, opts(), Arc::clone(key))
}

fn key(byte: u8) -> Arc<KeyStore> {
    Arc::new(KeyStore::from_bytes([byte; 32]).unwrap())
}

fn headers(dir: &Path) -> Vec<SegmentHeader> {
    let wal = dir.join("wal");
    list_segment_ids(&wal)
        .unwrap()
        .into_iter()
        .map(|id| SegmentReader::open(&segment_path(&wal, id)).unwrap().header)
        .collect()
}

#[test]
fn encrypted_segment_header_records_the_key_id() {
    let dir = TempDir::new().unwrap();
    let k = key(1);
    {
        let e = open_keyed(dir.path(), &k).unwrap();
        e.insert(1, vec![1, 2, 3]).unwrap();
    }
    let hs = headers(dir.path());
    assert!(!hs.is_empty());
    assert!(hs.iter().all(|h| h.key_id == Some(k.key_id())));
}

#[test]
fn wrong_key_error_names_both_key_ids() {
    let dir = TempDir::new().unwrap();
    let (a, b) = (key(1), key(2));
    {
        let e = open_keyed(dir.path(), &a).unwrap();
        e.insert(1, vec![1, 2, 3]).unwrap();
    }
    let err = open_keyed(dir.path(), &b)
        .err()
        .expect("wrong key must fail");
    let EngineError::WalDecrypt { reason, .. } = &err else {
        panic!("expected WalDecrypt, got {err}");
    };
    assert!(reason.contains(&key_id_hex(&a.key_id())), "{reason}");
    assert!(reason.contains(&key_id_hex(&b.key_id())), "{reason}");
    assert!(reason.contains("segment"), "{reason}");
    // The failed open must not have written a segment under the wrong key.
    assert!(
        headers(dir.path())
            .iter()
            .all(|h| h.key_id != Some(b.key_id()))
    );
    // And the right key still opens it.
    let e = open_keyed(dir.path(), &a).unwrap();
    assert_eq!(e.get(1).unwrap().unwrap().values, vec![1, 2, 3]);
}

#[test]
fn missing_key_on_encrypted_segment_fails_before_replay() {
    let dir = TempDir::new().unwrap();
    let a = key(1);
    {
        let e = open_keyed(dir.path(), &a).unwrap();
        e.insert(1, vec![1, 2, 3]).unwrap();
    }
    let err = TableEngine::open_with(schema(), dir.path(), opts())
        .err()
        .expect("missing key must fail");
    let EngineError::WalDecrypt { reason, .. } = &err else {
        panic!("expected WalDecrypt, got {err}");
    };
    assert!(reason.contains("no key"), "{reason}");
    assert!(reason.contains(&key_id_hex(&a.key_id())), "{reason}");
}

#[test]
fn plaintext_table_gains_encryption() {
    let dir = TempDir::new().unwrap();
    {
        let e = TableEngine::open_with(schema(), dir.path(), opts()).unwrap();
        e.insert(1, vec![1, 10, 100]).unwrap();
    }
    let k = key(3);
    {
        let e = open_keyed(dir.path(), &k).unwrap();
        e.insert(2, vec![2, 20, 200]).unwrap();
    }
    let hs = headers(dir.path());
    assert_eq!(
        hs.first().unwrap().key_id,
        None,
        "old segment stays plaintext"
    );
    assert_eq!(hs.last().unwrap().key_id, Some(k.key_id()));
    let e = open_keyed(dir.path(), &k).unwrap();
    assert_eq!(e.get(1).unwrap().unwrap().values, vec![1, 10, 100]);
    assert_eq!(e.get(2).unwrap().unwrap().values, vec![2, 20, 200]);
}

/// Rewrite every segment, replacing frame `to`'s payload with frame `from`'s
/// and re-chaining all hashes and CRCs, so the chain still verifies.
fn copy_payload(wal: &Path, from: u64, to: u64) {
    let ids = list_segment_ids(wal).unwrap();
    let mut all: Vec<(SegmentHeader, Vec<Frame>)> = ids
        .iter()
        .map(|&id| {
            let mut r = SegmentReader::open(&segment_path(wal, id)).unwrap();
            let frames = r.all_frames().unwrap();
            (r.header, frames)
        })
        .collect();
    let src = all
        .iter()
        .flat_map(|(_, f)| f)
        .find(|f| f.sequence == from)
        .unwrap()
        .payload
        .clone();
    let mut prev = vectorvolt_wal::GENESIS_HASH;
    for (header, frames) in &mut all {
        let mut bytes = Vec::new();
        header.write(&mut bytes).unwrap();
        for f in frames.iter_mut() {
            if f.sequence == to {
                assert_eq!(f.payload.len(), src.len(), "same-length swap");
                f.payload = src.clone();
            }
            f.prev_hash = prev;
            bytes.extend(f.encode());
            prev = f.frame_hash;
        }
        std::fs::write(segment_path(wal, header.segment_id), bytes).unwrap();
    }
    assert!(
        vectorvolt_wal::WalReader::open(WalConfig::default().with_dir(wal))
            .verify()
            .is_ok()
    );
}

#[test]
fn encrypted_frame_copied_to_another_position_fails_aad() {
    let dir = TempDir::new().unwrap();
    let k = key(4);
    {
        let e = open_keyed(dir.path(), &k).unwrap();
        e.insert(1, vec![1, 10, 100]).unwrap();
        e.insert(2, vec![2, 20, 200]).unwrap();
    }
    let wal = dir.path().join("wal");
    let seqs: Vec<u64> = vectorvolt_wal::WalReader::open(WalConfig::default().with_dir(&wal))
        .read_all()
        .unwrap()
        .iter()
        .map(|f| f.sequence)
        .collect();
    let (first, second) = (seqs[seqs.len() - 2], seqs[seqs.len() - 1]);
    copy_payload(&wal, first, second);
    // The keyed tail seal also catches the rewrite, and since the table
    // marker records that the table was sealed, deleting the seal is caught
    // too (A12 follow-up): the open fails either way.
    let _ = std::fs::remove_file(wal.join(vectorvolt_wal::seal::SEAL_FILE));
    let err = open_keyed(dir.path(), &k)
        .err()
        .expect("a rewritten WAL with its seal deleted must not open");
    assert!(err.to_string().contains("seal was deleted"), "{err}");

    // The AAD check on its own: the moved payload does not decrypt at its
    // new position.
    let mut checked = false;
    vectorvolt_wal::WalReader::open(WalConfig::default().with_dir(&wal))
        .scan_with_header(vectorvolt_wal::WalCursor::beginning(), |frame, header| {
            if frame.sequence == second {
                let e = vectorvolt_txn::manager::open_wal_payload(
                    Some(&k),
                    header,
                    frame.sequence,
                    frame.payload,
                )
                .expect_err("a moved frame must not decrypt");
                assert_eq!(e.seq, second);
                assert!(e.reason.contains("moved"), "{}", e.reason);
                checked = true;
            }
            true
        })
        .unwrap();
    assert!(checked);
}

#[test]
fn legacy_vve1_wal_still_opens() {
    // A WAL exactly as written before PP-52: version-1 headers (flags = 0)
    // and unbound `VVE1` payloads.
    let dir = TempDir::new().unwrap();
    let k = key(5);
    {
        let wal = dir.path().join("wal");
        let w = WalWriter::open(WalConfig::default().with_dir(&wal).with_fsync(false)).unwrap();
        for pk in 1..=3u64 {
            let m = Mutation::Insert {
                pk,
                values: vec![pk, pk * 10, pk * 100],
            };
            w.append(vectorvolt_crypto::encrypt(&k, &m.encode()).unwrap())
                .unwrap();
        }
        w.flush().unwrap();
    }
    assert!(headers(dir.path()).iter().all(|h| h.key_id.is_none()));
    {
        let e = open_keyed(dir.path(), &k).unwrap();
        for pk in 1..=3u64 {
            assert_eq!(
                e.get(pk).unwrap().unwrap().values,
                vec![pk, pk * 10, pk * 100]
            );
        }
        e.insert(4, vec![4, 40, 400]).unwrap();
    }
    // Mixed legacy + AAD-bound history reopens too.
    let e = open_keyed(dir.path(), &k).unwrap();
    assert_eq!(e.get(1).unwrap().unwrap().values, vec![1, 10, 100]);
    assert_eq!(e.get(4).unwrap().unwrap().values, vec![4, 40, 400]);
    assert_eq!(headers(dir.path()).last().unwrap().key_id, Some(k.key_id()));
}
