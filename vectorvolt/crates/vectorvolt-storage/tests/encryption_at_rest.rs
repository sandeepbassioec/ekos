//! PP-53: encrypted `.vvsst` files and encrypted manifests at the storage
//! layer. End-to-end table behaviour is covered in
//! `vectorvolt-txn/tests/partition_encryption.rs`.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use tempfile::TempDir;
use vectorvolt_crypto::KeyStore;
use vectorvolt_storage::lifecycle::{CompactionJob, CompactionReason, PartitionMeta};
use vectorvolt_storage::{
    ColumnValues, CompactionOptions, KeepAll, Manifest, ManifestEdit, MicroPartition,
    PartitionReader, StorageError, WriteOptions, execute_job, is_encrypted_partition,
    register_partition_key, write_partition_keyed,
};

fn key() -> Arc<KeyStore> {
    Arc::new(KeyStore::generate().unwrap())
}

fn partition(file_id: u16, pid: u16, pks: std::ops::Range<u64>) -> MicroPartition {
    let mut p = MicroPartition::new(file_id, pid);
    let pk: Vec<u64> = pks.clone().collect();
    let v: Vec<u64> = pks.map(|k| k.wrapping_mul(0x9E37_79B9_7F4A_7C15)).collect();
    p.push_column("pk", ColumnValues::Raw(pk)).unwrap();
    p.push_column("v", ColumnValues::auto_encode(v)).unwrap();
    p
}

fn write(dir: &Path, p: &MicroPartition, opts: WriteOptions, key: Option<&KeyStore>) -> PathBuf {
    let path = dir.join(format!("{:05}_{:05}.vvsst", p.file_id, p.partition_id));
    let mut f = std::fs::File::create(&path).unwrap();
    write_partition_keyed(&mut f, p, opts, key).unwrap();
    f.flush().unwrap();
    path
}

fn meta(path: &Path, file_id: u16, pid: u16, rows: u32, lo: u64, hi: u64) -> PartitionMeta {
    PartitionMeta {
        file_id,
        partition_id: pid,
        path: path.to_path_buf(),
        level: 0,
        size_bytes: std::fs::metadata(path).unwrap().len(),
        row_count: rows,
        created_at_millis: 1,
        min_key: lo,
        max_key: hi,
        tombstones: 0,
        worm_locked: false,
    }
}

#[test]
fn encrypted_round_trip_including_compressed_columns() {
    let dir = TempDir::new().unwrap();
    let k = key();
    for opts in [WriteOptions::new(), WriteOptions::new().with_lz4()] {
        let p = partition(1, 2, 0..500);
        let path = write(dir.path(), &p, opts, Some(&k));
        assert!(is_encrypted_partition(&std::fs::read(&path).unwrap()));
        let r = PartitionReader::open_with_key(&path, Some(&k)).unwrap();
        assert!(r.is_encrypted());
        assert_eq!((r.file_id(), r.partition_id(), r.row_count()), (1, 2, 500));
        for c in 0..2 {
            assert_eq!(
                r.read_column(c).unwrap().to_raw_vec(),
                p.columns[c].to_raw_vec()
            );
        }
        assert_eq!(
            r.read_value(1, 77).unwrap(),
            77u64.wrapping_mul(0x9E37_79B9_7F4A_7C15)
        );
    }
}

#[test]
fn plaintext_files_open_with_or_without_a_key() {
    let dir = TempDir::new().unwrap();
    let p = partition(0, 1, 0..10);
    let path = write(dir.path(), &p, WriteOptions::new(), None);
    let k = key();
    assert!(
        !PartitionReader::open_with_key(&path, Some(&k))
            .unwrap()
            .is_encrypted()
    );
    assert!(!PartitionReader::open(&path).unwrap().is_encrypted());
}

#[test]
fn zone_peek_on_an_encrypted_file_uses_the_registered_key() {
    let dir = TempDir::new().unwrap();
    let k = key();
    let path = write(
        dir.path(),
        &partition(0, 3, 40..90),
        WriteOptions::new(),
        Some(&k),
    );
    let err = PartitionReader::peek_zone_maps(&path).unwrap_err();
    assert!(
        matches!(err, StorageError::PartitionKeyMissing { .. }),
        "{err}"
    );
    register_partition_key(&k);
    let peek = PartitionReader::peek_zone_maps(&path).unwrap();
    assert_eq!((peek.partition_id, peek.row_count), (3, 50));
    assert_eq!(peek.zones[0], (40, 89));
}

#[test]
fn compaction_migrates_mixed_inputs_and_never_downgrades() {
    let dir = TempDir::new().unwrap();
    let k = key();
    let a = write(
        dir.path(),
        &partition(0, 1, 0..100),
        WriteOptions::new(),
        None,
    );
    let b = write(
        dir.path(),
        &partition(0, 2, 100..200),
        WriteOptions::new(),
        Some(&k),
    );
    let job = CompactionJob {
        inputs: vec![meta(&a, 0, 1, 100, 0, 99), meta(&b, 0, 2, 100, 100, 199)],
        output_level: 1,
        reason: CompactionReason::SizeRun,
    };
    let schema = vec!["pk".to_string(), "v".to_string()];
    let mut opts = CompactionOptions {
        output_dir: dir.path().to_path_buf(),
        output_file_id: 0,
        output_partition_id: 9,
        write_options: WriteOptions::new(),
        now_millis: 2,
        encryption_key: None,
    };
    // No key: the encrypted input is an error, not rewritten as plaintext.
    let err = execute_job(&job, &schema, &KeepAll, &opts).unwrap_err();
    assert!(
        matches!(err, StorageError::PartitionKeyMissing { .. }),
        "{err}"
    );

    opts.encryption_key = Some(Arc::clone(&k));
    let out = execute_job(&job, &schema, &KeepAll, &opts).unwrap();
    assert_eq!(out.rows_out, 200);
    assert!(is_encrypted_partition(
        &std::fs::read(&out.output.path).unwrap()
    ));
    let r = PartitionReader::open_with_key(&out.output.path, Some(&k)).unwrap();
    assert_eq!(
        r.read_column(0).unwrap().to_raw_vec(),
        (0..200).collect::<Vec<_>>()
    );
}

fn add(m: &mut Manifest, path: &Path, pid: u16) {
    let p = PartitionMeta {
        file_id: 0,
        partition_id: pid,
        path: path.to_path_buf(),
        level: 0,
        size_bytes: 10,
        row_count: 5,
        created_at_millis: 1,
        min_key: 0xABCD_EF01_2345_6789,
        max_key: 0xABCD_EF01_2345_6799,
        tombstones: 0,
        worm_locked: false,
    };
    m.apply(ManifestEdit::Add(Box::new(p))).unwrap();
}

#[test]
fn encrypted_manifest_round_trips_and_hides_metadata() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("MANIFEST");
    let k = key();
    {
        let (mut m, _) = Manifest::open_with_key(&path, Some(Arc::clone(&k))).unwrap();
        assert!(m.is_encrypted());
        for pid in 0..5 {
            add(
                &mut m,
                &dir.path().join(format!("00000_{pid:05}.vvsst")),
                pid,
            );
        }
        m.apply(ManifestEdit::Remove {
            file_id: 0,
            partition_id: 2,
        })
        .unwrap();
        m.rewrite().unwrap();
        add(&mut m, &dir.path().join("00000_00009.vvsst"), 9);
    }
    let bytes = std::fs::read(&path).unwrap();
    assert_eq!(&bytes[..8], b"VVMANIF2");
    assert!(
        !bytes
            .windows(8)
            .any(|w| w == 0xABCD_EF01_2345_6789u64.to_le_bytes())
    );
    assert!(!bytes.windows(5).any(|w| w == b"vvsst"));

    let (m, rec) = Manifest::open_with_key(&path, Some(Arc::clone(&k))).unwrap();
    assert!(rec.is_clean());
    let ids: Vec<u16> = m.partitions().iter().map(|p| p.partition_id).collect();
    assert_eq!(ids, vec![0, 1, 3, 4, 9]);

    let err = Manifest::open(&path).err().unwrap();
    assert!(err.to_string().contains("no key was supplied"), "{err}");
    let err = Manifest::open_with_key(&path, Some(key())).err().unwrap();
    assert!(err.to_string().contains("supplied key has id"), "{err}");
}

#[test]
fn plaintext_manifest_is_migrated_when_a_key_appears() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("MANIFEST");
    {
        let (mut m, _) = Manifest::open(&path).unwrap();
        add(&mut m, &dir.path().join("00000_00001.vvsst"), 1);
    }
    let k = key();
    {
        let (mut m, _) = Manifest::open_with_key(&path, Some(Arc::clone(&k))).unwrap();
        assert!(m.is_encrypted());
        add(&mut m, &dir.path().join("00000_00002.vvsst"), 2);
    }
    assert_eq!(&std::fs::read(&path).unwrap()[..8], b"VVMANIF2");
    let (m, _) = Manifest::open_with_key(&path, Some(k)).unwrap();
    assert_eq!(m.len(), 2);
}

#[test]
fn tampered_manifest_record_fails_but_torn_tail_truncates() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("MANIFEST");
    let k = key();
    {
        let (mut m, _) = Manifest::open_with_key(&path, Some(Arc::clone(&k))).unwrap();
        add(&mut m, &dir.path().join("00000_00001.vvsst"), 1);
        add(&mut m, &dir.path().join("00000_00002.vvsst"), 2);
    }
    let good = std::fs::read(&path).unwrap();

    // Torn tail: a partial record after the last good one.
    let mut torn = good.clone();
    torn.extend_from_slice(&[9, 0, 0, 0, 1, 2]);
    std::fs::write(&path, &torn).unwrap();
    let (m, rec) = Manifest::open_with_key(&path, Some(Arc::clone(&k))).unwrap();
    assert_eq!((m.len(), rec.discarded_bytes), (2, 6));
    drop(m);

    // Tamper inside the first record's ciphertext and fix up its CRC, so only
    // the AES-GCM tag can notice.
    let mut bad = good.clone();
    let len = u32::from_le_bytes(bad[32..36].try_into().unwrap()) as usize;
    bad[40 + len / 2] ^= 1;
    let crc = crc32fast::hash(&bad[40..40 + len]);
    bad[36..40].copy_from_slice(&crc.to_le_bytes());
    std::fs::write(&path, &bad).unwrap();
    let err = Manifest::open_with_key(&path, Some(Arc::clone(&k)))
        .err()
        .unwrap();
    assert!(
        err.to_string().contains("authenticated decryption"),
        "{err}"
    );

    // Swapping the two records is caught by the index in the AAD.
    let mut swapped = good[..32].to_vec();
    let first_end = 40 + len;
    swapped.extend_from_slice(&good[first_end..]);
    swapped.extend_from_slice(&good[32..first_end]);
    std::fs::write(&path, &swapped).unwrap();
    assert!(Manifest::open_with_key(&path, Some(k)).is_err());
}
