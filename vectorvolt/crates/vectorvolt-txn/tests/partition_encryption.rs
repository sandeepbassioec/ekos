//! PP-53 / C3: partition files and the manifest are encrypted at rest when the
//! manager has a key, old plaintext files stay readable, and every way of
//! presenting the wrong bytes or the wrong key fails loudly.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use tempfile::TempDir;
use vectorvolt_crypto::KeyStore;
use vectorvolt_storage::{
    CompactionStrategy, PartitionReader, StorageError, WriteOptions, is_encrypted_partition,
    verify_partitions,
};
use vectorvolt_txn::TxnManager;
use vectorvolt_wal::{WalConfig, WalWriter};

/// A value that cannot occur by accident in a header or offset table.
const MARKER: u64 = 0x5EC2_E7C0_FFEE_D00D;

fn key() -> Arc<KeyStore> {
    Arc::new(KeyStore::generate().unwrap())
}

/// A manager over `data`, with a fresh WAL each time (the WAL is not under
/// test here; the partition files are).
fn manager(root: &Path, key: Option<&Arc<KeyStore>>, start_pid: u16) -> TxnManager {
    static N: AtomicUsize = AtomicUsize::new(0);
    let wal_dir = root.join(format!("wal-{}", N.fetch_add(1, Ordering::Relaxed)));
    std::fs::create_dir_all(&wal_dir).unwrap();
    let cfg = WalConfig::default().with_dir(&wal_dir).with_fsync(false);
    let m = TxnManager::with_starting_partition(
        WalWriter::open(cfg).unwrap(),
        vec!["pk".into(), "secret".into()],
        root.join("data"),
        1_000_000,
        start_pid,
    );
    match key {
        Some(k) => m.with_wal_encryption(Arc::clone(k)),
        None => m,
    }
}

fn vvsst_files(root: &Path) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = std::fs::read_dir(root.join("data"))
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "vvsst"))
        .collect();
    v.sort();
    v
}

fn contains(hay: &[u8], needle: &[u8]) -> bool {
    hay.windows(needle.len()).any(|w| w == needle)
}

/// The secret column: spread values so the column is stored raw and a
/// plaintext file holds them verbatim (an encoded column would not, which
/// would make the absence check below vacuous).
fn secret(pk: u64) -> u64 {
    MARKER ^ pk.wrapping_mul(0x9E37_79B9_7F4A_7C15)
}

fn row(pk: u64) -> Vec<u64> {
    vec![pk, secret(pk)]
}

fn insert_and_flush(m: &TxnManager, pks: std::ops::Range<u64>) {
    for pk in pks {
        m.insert(pk, row(pk)).unwrap();
    }
    m.flush().unwrap();
}

/// Reopen the table the way `TableEngine` does: manifest, readers, pk index.
fn reopen(root: &Path, key: Option<&Arc<KeyStore>>) -> TxnManager {
    let max_pid = vvsst_files(root)
        .iter()
        .map(|p| PartitionReader::open_with_key(p, key.map(|k| &**k)).unwrap())
        .map(|r| r.partition_id())
        .max()
        .map_or(0, |p| p + 1);
    let m = manager(root, key, max_pid);
    m.enable_manifest().unwrap();
    let mut readers = HashMap::new();
    for p in m.partitions() {
        let r = PartitionReader::open_with_key(&p.path, key.map(|k| &**k)).unwrap();
        readers.insert((p.file_id, p.partition_id), r);
    }
    m.rebuild_pk_index_from_partitions(&readers).unwrap();
    m
}

#[test]
fn encrypted_flush_hides_row_values_and_reopens_with_the_key() {
    let dir = TempDir::new().unwrap();
    let k = key();
    {
        let m = manager(dir.path(), Some(&k), 0);
        m.enable_manifest().unwrap();
        insert_and_flush(&m, 1..200);
    }
    let files = vvsst_files(dir.path());
    assert_eq!(files.len(), 1);
    let bytes = std::fs::read(&files[0]).unwrap();
    assert!(
        is_encrypted_partition(&bytes),
        "flush wrote a plaintext file"
    );
    for pk in [1u64, 100, 199] {
        assert!(
            !contains(&bytes, &secret(pk).to_le_bytes()),
            "row value of pk {pk} visible in the encrypted file"
        );
    }
    let manifest = std::fs::read(dir.path().join("data/MANIFEST")).unwrap();
    assert_eq!(&manifest[..8], b"VVMANIF2", "manifest not encrypted");
    // A plaintext manifest carries each partition's path and key range.
    assert!(!contains(&manifest, b"00000_00000.vvsst"));
    assert!(!contains(&manifest, &199u64.to_le_bytes()));

    let m = reopen(dir.path(), Some(&k));
    for pk in 1..200 {
        assert_eq!(m.get(pk).unwrap(), Some(row(pk)), "pk {pk}");
    }
}

/// Control for the test above: without a key the marker IS on disk, so the
/// absence check is meaningful.
#[test]
fn plaintext_flush_still_writes_v3_and_shows_the_marker() {
    let dir = TempDir::new().unwrap();
    let m = manager(dir.path(), None, 0);
    m.enable_manifest().unwrap();
    insert_and_flush(&m, 1..50);
    let bytes = std::fs::read(&vvsst_files(dir.path())[0]).unwrap();
    assert!(!is_encrypted_partition(&bytes));
    assert_eq!(u32::from_le_bytes(bytes[8..12].try_into().unwrap()), 3);
    assert!(contains(&bytes, &secret(7).to_le_bytes()));
    let manifest = std::fs::read(dir.path().join("data/MANIFEST")).unwrap();
    assert_eq!(&manifest[..8], b"VVMANIF1");
}

#[test]
fn wrong_key_fails_with_a_clear_error() {
    let dir = TempDir::new().unwrap();
    let k = key();
    {
        let m = manager(dir.path(), Some(&k), 0);
        m.enable_manifest().unwrap();
        insert_and_flush(&m, 1..20);
    }
    let path = vvsst_files(dir.path()).remove(0);
    let other = key();
    let err = PartitionReader::open_with_key(&path, Some(&other))
        .err()
        .expect("wrong key opened the file");
    assert!(
        matches!(err, StorageError::PartitionKeyMismatch { .. }),
        "{err}"
    );
    let msg = err.to_string();
    assert!(msg.contains("00000_00000.vvsst"), "{msg}");
    assert!(
        msg.contains(&hex(&k.key_id())) && msg.contains(&hex(&other.key_id())),
        "{msg}"
    );

    // The manifest refuses the wrong key too.
    let m = manager(dir.path(), Some(&other), 10);
    let err = m.enable_manifest().expect_err("manifest opened");
    assert!(err.to_string().contains("key id"), "{err}");
}

#[test]
fn missing_key_on_an_encrypted_file_fails() {
    let dir = TempDir::new().unwrap();
    let path;
    {
        let k = key();
        let m = manager(dir.path(), Some(&k), 0);
        m.enable_manifest().unwrap();
        insert_and_flush(&m, 1..20);
        path = vvsst_files(dir.path()).remove(0);
        let err = PartitionReader::open_with_key(&path, None)
            .err()
            .expect("opened without a key");
        assert!(
            matches!(err, StorageError::PartitionKeyMissing { .. }),
            "{err}"
        );
        assert!(err.to_string().contains(&hex(&k.key_id())), "{err}");
        // While the key is alive it is resolvable by id for path-only opens.
        assert!(PartitionReader::open(&path).unwrap().is_encrypted());
    }
    // Key and manager dropped: the registry no longer resolves it.
    let err = PartitionReader::open(&path).err().expect("opened");
    assert!(
        matches!(err, StorageError::PartitionKeyMissing { .. }),
        "{err}"
    );

    // A keyless manager cannot open the encrypted manifest.
    let m = manager(dir.path(), None, 10);
    let err = m.enable_manifest().expect_err("manifest opened");
    assert!(err.to_string().contains("no key was supplied"), "{err}");
}

/// The migration path: a plaintext table gains a key, flushes encrypted
/// partitions next to its plaintext ones, and compaction folds both into one
/// encrypted output.
#[test]
fn mixed_plaintext_and_encrypted_partitions_compact_to_encrypted() {
    let dir = TempDir::new().unwrap();
    {
        let m = manager(dir.path(), None, 0);
        m.enable_manifest().unwrap();
        insert_and_flush(&m, 0..100);
        insert_and_flush(&m, 100..200);
        insert_and_flush(&m, 200..300);
    }
    let k = key();
    let m = reopen(dir.path(), Some(&k));
    assert_eq!(
        &std::fs::read(dir.path().join("data/MANIFEST")).unwrap()[..8],
        b"VVMANIF2",
        "plaintext manifest was not migrated"
    );
    insert_and_flush(&m, 300..400);
    insert_and_flush(&m, 400..500);
    // Overwrite some old plaintext rows from the encrypted side.
    for pk in 0..10 {
        m.upsert(pk, vec![pk, MARKER]).unwrap();
    }
    m.flush().unwrap();

    let files = vvsst_files(dir.path());
    let enc = files
        .iter()
        .filter(|p| is_encrypted_partition(&std::fs::read(p).unwrap()))
        .count();
    assert_eq!((files.len(), enc), (6, 3), "expected 3 plain + 3 encrypted");
    let expect = |pk: u64| {
        if pk < 10 { vec![pk, MARKER] } else { row(pk) }
    };
    for pk in 0..500 {
        assert_eq!(m.get(pk).unwrap(), Some(expect(pk)), "pre-compaction {pk}");
    }

    let out = m
        .run_compaction(CompactionStrategy::default(), WriteOptions::default())
        .unwrap();
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].rows_in, 510);
    assert_eq!(out[0].rows_out, 500);
    let files = vvsst_files(dir.path());
    assert_eq!(files.len(), 1, "inputs left behind: {files:?}");
    let bytes = std::fs::read(&files[0]).unwrap();
    assert!(
        is_encrypted_partition(&bytes),
        "compaction output plaintext"
    );
    assert!(!contains(&bytes, &secret(250).to_le_bytes()));
    for pk in 0..500 {
        assert_eq!(m.get(pk).unwrap(), Some(expect(pk)), "post-compaction {pk}");
    }
    // Path-only verification resolves the key through the registry.
    assert!(verify_partitions(&m.partitions()).unwrap().is_clean());
    drop(m);

    let m = reopen(dir.path(), Some(&k));
    for pk in 0..500 {
        assert_eq!(m.get(pk).unwrap(), Some(expect(pk)), "after reopen {pk}");
    }
}

#[test]
fn a_flipped_byte_is_an_error_not_garbage() {
    let dir = TempDir::new().unwrap();
    let k = key();
    let m = manager(dir.path(), Some(&k), 0);
    insert_and_flush(&m, 1..100);
    let path = vvsst_files(dir.path()).remove(0);
    let good = std::fs::read(&path).unwrap();
    // A ciphertext byte, the last tag byte, and a header byte.
    for at in [good.len() / 2, good.len() - 1, 20] {
        let mut bad = good.clone();
        bad[at] ^= 0x01;
        std::fs::write(&path, &bad).unwrap();
        let err = PartitionReader::open_with_key(&path, Some(&k))
            .err()
            .unwrap_or_else(|| panic!("flip at {at} went unnoticed"));
        assert!(
            matches!(
                err,
                StorageError::PartitionDecrypt { .. } | StorageError::HeaderCrcMismatch { .. }
            ),
            "flip at {at}: {err}"
        );
    }
    std::fs::write(&path, &good).unwrap();
    assert_eq!(
        PartitionReader::open_with_key(&path, Some(&k))
            .unwrap()
            .row_count(),
        99
    );
}

#[test]
fn a_partition_copied_over_another_fails_authentication() {
    let dir = TempDir::new().unwrap();
    let k = key();
    let m = manager(dir.path(), Some(&k), 0);
    insert_and_flush(&m, 1..50);
    insert_and_flush(&m, 50..100);
    drop(m);
    let files = vvsst_files(dir.path());
    assert_eq!(files.len(), 2);
    let (a, b) = (&files[0], &files[1]);
    std::fs::copy(a, b).unwrap();
    let err = PartitionReader::open_with_key(b, Some(&k))
        .err()
        .expect("A's bytes were accepted as partition B");
    assert!(
        matches!(err, StorageError::PartitionDecrypt { .. }),
        "{err}"
    );
    assert!(PartitionReader::open_with_key(a, Some(&k)).is_ok());
}

/// Rough cost of encryption on the read path. Prints numbers; asserts only
/// correctness (timings in a debug test build are not a benchmark — run with
/// `--release -- --nocapture` for the figures quoted in the PR).
#[test]
fn point_read_cost_encrypted_vs_plaintext() {
    const ROWS: u64 = 20_000;
    const READS: u64 = 50_000;
    let mut report = Vec::new();
    for encrypted in [false, true] {
        let dir = TempDir::new().unwrap();
        let k = key();
        let m = manager(dir.path(), encrypted.then_some(&k), 0);
        insert_and_flush(&m, 0..ROWS);

        let t = Instant::now();
        assert_eq!(m.get(1).unwrap(), Some(row(1))); // opens (and decrypts) the file
        let first = t.elapsed();

        let t = Instant::now();
        for i in 0..READS {
            let pk = (i * 7919) % ROWS;
            assert_eq!(m.get(pk).unwrap(), Some(row(pk)));
        }
        let warm = t.elapsed() / READS as u32;
        report.push((encrypted, first, warm));
    }
    for (enc, first, warm) in &report {
        eprintln!(
            "{:>9}: first read (open) {:?}, warm point read {:?}",
            if *enc { "encrypted" } else { "plaintext" },
            first,
            warm
        );
    }
}

fn hex(id: &[u8]) -> String {
    id.iter().map(|b| format!("{b:02x}")).collect()
}
