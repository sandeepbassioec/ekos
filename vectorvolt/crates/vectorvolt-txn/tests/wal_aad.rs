//! PP-52: TxnManager binds every encrypted WAL payload to its sequence, and
//! `open_wal_payload` refuses downgrades inside keyed segments.
use std::sync::Arc;

use tempfile::TempDir;
use vectorvolt_crypto::KeyStore;
use vectorvolt_txn::manager::{TxnManager, open_wal_payload};
use vectorvolt_txn::ops::Mutation;
use vectorvolt_wal::segment::{SegmentHeader, frame_aad};
use vectorvolt_wal::{WalConfig, WalReader, WalWriter};

fn keyed_header(key: &KeyStore) -> SegmentHeader {
    SegmentHeader::with_key_id(0, 1, Some(key.key_id()))
}

#[test]
fn concurrent_encrypted_commits_are_bound_to_their_sequence() {
    let dir = TempDir::new().unwrap();
    let wal_dir = dir.path().join("wal");
    let key = Arc::new(KeyStore::generate().unwrap());
    let wal = WalWriter::open(WalConfig::default().with_dir(&wal_dir).with_fsync(false)).unwrap();
    let txn = Arc::new(
        TxnManager::new(
            wal,
            vec!["pk".into(), "v".into()],
            dir.path().join("data"),
            1_000_000,
        )
        .with_wal_encryption(Arc::clone(&key)),
    );
    let threads: Vec<_> = (0..4u64)
        .map(|t| {
            let txn = Arc::clone(&txn);
            std::thread::spawn(move || {
                for i in 0..50u64 {
                    let pk = t * 1000 + i + 1;
                    txn.insert(pk, vec![pk, i]).unwrap();
                }
            })
        })
        .collect();
    for t in threads {
        t.join().unwrap();
    }
    drop(txn);

    let reader = WalReader::open(WalConfig::default().with_dir(&wal_dir));
    let mut n = 0;
    reader
        .scan_with_header(vectorvolt_wal::WalCursor::beginning(), |frame, header| {
            assert_eq!(header.key_id, Some(key.key_id()));
            assert!(vectorvolt_crypto::encryptor::is_aad_encrypted(
                &frame.payload
            ));
            let aad = frame_aad(&key.key_id(), frame.sequence);
            vectorvolt_crypto::encryptor::decrypt_with_aad(&key, &frame.payload, &aad).unwrap();
            n += 1;
            true
        })
        .unwrap();
    assert_eq!(n, 200);
}

#[test]
fn keyed_segment_refuses_unbound_and_plaintext_mutations() {
    let key = KeyStore::generate().unwrap();
    let m = Mutation::Insert {
        pk: 1,
        values: vec![1, 2],
    }
    .encode();
    let keyed = keyed_header(&key);
    let plain = SegmentHeader::new(0, 1);

    let vve1 = vectorvolt_crypto::encrypt(&key, &m).unwrap();
    assert!(open_wal_payload(Some(&key), &keyed, 5, vve1.clone()).is_err());
    assert_eq!(open_wal_payload(Some(&key), &plain, 5, vve1).unwrap(), m);

    assert!(open_wal_payload(Some(&key), &keyed, 5, m.clone()).is_err());
    assert_eq!(open_wal_payload(None, &plain, 5, m.clone()).unwrap(), m);
    // Non-mutation plaintext (e.g. a snapshot marker) is still allowed.
    assert!(open_wal_payload(Some(&key), &keyed, 5, b"VVSM-marker".to_vec()).is_ok());

    let aad = frame_aad(&key.key_id(), 5);
    let vve2 = vectorvolt_crypto::encryptor::encrypt_with_aad(&key, &m, &aad).unwrap();
    assert_eq!(
        open_wal_payload(Some(&key), &keyed, 5, vve2.clone()).unwrap(),
        m
    );
    assert!(open_wal_payload(Some(&key), &keyed, 6, vve2.clone()).is_err());
    assert!(open_wal_payload(None, &keyed, 5, vve2).is_err());
}
