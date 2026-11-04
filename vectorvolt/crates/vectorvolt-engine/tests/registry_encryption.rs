//! PP-51 (C1): `TableRegistry` encryption at rest.
//!
//! A registry opened with `TableRegistry::open_with_encryption` must open and
//! create every table with WAL encryption, so `vvctl serve --multi
//! --encrypt-at-rest` covers every table in the catalog.
use std::sync::Arc;

use tempfile::TempDir;
use vectorvolt_crypto::KeyStore;
use vectorvolt_engine::{ColumnDef, ColumnType, EngineError, EngineOptions, TableRegistry};

/// Distinctive value that the plaintext WAL encoding writes verbatim
/// (little-endian u64), and that is vanishingly unlikely in ciphertext.
const MARKER: u64 = 0xDEAD_BEEF_CAFE_F00D;

fn opts() -> EngineOptions {
    EngineOptions::new(1_000_000)
}

fn key() -> Arc<KeyStore> {
    Arc::new(
        KeyStore::from_hex("891cbeb0ca01a43cadf845ce73c7756c5e4ce5e63e831132fe44c1803c473c35")
            .unwrap(),
    )
}

fn other_key() -> Arc<KeyStore> {
    Arc::new(
        KeyStore::from_hex("ac8795ba9b94370dbdd53f308f7c22099ae3af377a0f6433bb50855aa1f70dec")
            .unwrap(),
    )
}

fn cols(names: &[&str]) -> Vec<String> {
    names.iter().map(|s| s.to_string()).collect()
}

fn dir_bytes(dir: &std::path::Path) -> Vec<u8> {
    let mut bytes = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries {
            let path = entry.unwrap().path();
            if path.is_file() {
                bytes.extend(std::fs::read(&path).unwrap());
            }
        }
    }
    bytes
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len()).any(|w| w == needle)
}

#[test]
fn encrypted_registry_round_trips_rows_across_reopen() {
    let tmp = TempDir::new().unwrap();
    {
        let reg = TableRegistry::open_with_encryption(tmp.path(), opts(), key()).unwrap();
        assert!(reg.is_encrypted());
        let t = reg.create_table("orders", cols(&["pk", "val"])).unwrap();
        t.insert(1, vec![1, MARKER]).unwrap();
    }

    let reg = TableRegistry::open_with_encryption(tmp.path(), opts(), key()).unwrap();
    let got = reg.get("orders").unwrap().get(1).unwrap().unwrap();
    assert_eq!(got.values, vec![1, MARKER]);
}

#[test]
fn encrypted_registry_wal_bytes_are_not_plaintext() {
    let tmp = TempDir::new().unwrap();
    {
        let reg = TableRegistry::open_with_encryption(tmp.path(), opts(), key()).unwrap();
        reg.create_table("orders", cols(&["pk", "val"]))
            .unwrap()
            .insert(7, vec![7, MARKER])
            .unwrap();
    }
    let wal = dir_bytes(&tmp.path().join("orders").join("wal"));
    assert!(!wal.is_empty(), "WAL must have been written");
    assert!(
        !contains(&wal, &MARKER.to_le_bytes()),
        "marker found in plaintext in the table WAL — registry did not encrypt"
    );
}

#[test]
fn plaintext_registry_wal_contains_marker() {
    // Control for the test above: without a key the marker IS on disk.
    let tmp = TempDir::new().unwrap();
    {
        let reg = TableRegistry::open(tmp.path(), opts()).unwrap();
        assert!(!reg.is_encrypted());
        reg.create_table("orders", cols(&["pk", "val"]))
            .unwrap()
            .insert(7, vec![7, MARKER])
            .unwrap();
    }
    let wal = dir_bytes(&tmp.path().join("orders").join("wal"));
    assert!(contains(&wal, &MARKER.to_le_bytes()));
}

#[test]
fn encrypted_registry_reopened_without_key_fails() {
    let tmp = TempDir::new().unwrap();
    {
        let reg = TableRegistry::open_with_encryption(tmp.path(), opts(), key()).unwrap();
        reg.create_table("orders", cols(&["pk", "val"]))
            .unwrap()
            .insert(1, vec![1, 2])
            .unwrap();
    }
    let err = TableRegistry::open(tmp.path(), opts())
        .err()
        .expect("restoring an encrypted table without its key must fail");
    assert!(matches!(err, EngineError::WalDecrypt { .. }), "{err}");
}

#[test]
fn encrypted_registry_reopened_with_wrong_key_fails() {
    let tmp = TempDir::new().unwrap();
    {
        let reg = TableRegistry::open_with_encryption(tmp.path(), opts(), key()).unwrap();
        reg.create_table("orders", cols(&["pk", "val"]))
            .unwrap()
            .insert(1, vec![1, 2])
            .unwrap();
    }
    assert!(TableRegistry::open_with_encryption(tmp.path(), opts(), other_key()).is_err());
}

#[test]
fn typed_create_and_alter_stay_encrypted() {
    let tmp = TempDir::new().unwrap();
    {
        let reg = TableRegistry::open_with_encryption(tmp.path(), opts(), key()).unwrap();
        let schema = vectorvolt_engine::TableSchema::from_names(&cols(&["pk", "val"]));
        reg.create_table_typed("typed", schema)
            .unwrap()
            .insert(1, vec![1, MARKER])
            .unwrap();
        let altered = reg
            .alter_table("typed", &[ColumnDef::new("extra", ColumnType::U64)], &[])
            .unwrap();
        assert_eq!(
            altered.typed_schema().names(),
            cols(&["pk", "val", "extra"])
        );
        altered.insert(2, vec![2, MARKER, 5]).unwrap();
    }

    let wal = dir_bytes(&tmp.path().join("typed").join("wal"));
    assert!(!contains(&wal, &MARKER.to_le_bytes()));

    let reg = TableRegistry::open_with_encryption(tmp.path(), opts(), key()).unwrap();
    let t = reg.get("typed").unwrap();
    assert_eq!(t.typed_schema().names(), cols(&["pk", "val", "extra"]));
    assert_eq!(t.get(2).unwrap().unwrap().values, vec![2, MARKER, 5]);
    assert!(TableRegistry::open(tmp.path(), opts()).is_err());
}

#[test]
fn catalog_wal_still_logs_ddl_with_encryption() {
    let tmp = TempDir::new().unwrap();
    let wal_dir = tmp.path().join("_catalog_wal");
    {
        let mut reg = TableRegistry::open_with_encryption(tmp.path(), opts(), key()).unwrap();
        reg.enable_catalog_wal(&wal_dir).unwrap();
        assert!(reg.has_catalog_wal());
        reg.create_table("audited", cols(&["pk", "val"])).unwrap();
        reg.drop_table("audited").unwrap();
    }
    // PP-54: the catalog WAL is encrypted like a table WAL.
    assert!(!contains(&dir_bytes(&wal_dir), b"audited"));
    let ddl = vectorvolt_engine::read_catalog_wal(&wal_dir, Some(&key())).unwrap();
    assert_eq!(ddl.len(), 2);
    // `_catalog_wal` has no schema.txt, so reopening ignores it.
    let reg = TableRegistry::open_with_encryption(tmp.path(), opts(), key()).unwrap();
    assert!(reg.list_tables().is_empty());
}
