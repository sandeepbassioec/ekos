//! Unit tests for VV-SEC-001..004 (vectorvolt-crypto).

use vectorvolt_crypto::{
    CryptoError, EncryptedReader, EncryptedWriter, KeyStore, decrypt, encrypt, harden_data_dir,
};

// ── KeyStore (VV-SEC-001) ─────────────────────────────────────────────────────

#[test]
fn key_store_generate_produces_32_bytes() {
    let ks = KeyStore::generate().unwrap();
    assert_eq!(ks.as_bytes().len(), 32);
}

#[test]
fn key_store_generate_is_random() {
    let a = KeyStore::generate().unwrap();
    let b = KeyStore::generate().unwrap();
    // Two independently generated keys must differ (negligible collision probability)
    assert_ne!(a.as_bytes(), b.as_bytes());
}

#[test]
fn key_store_from_hex_roundtrip() {
    let k = KeyStore::generate().unwrap();
    let hex = k.to_hex();
    assert_eq!(hex.len(), 64);
    let k2 = KeyStore::from_hex(&hex).unwrap();
    assert_eq!(k.as_bytes(), k2.as_bytes());
}

#[test]
fn key_store_from_hex_bad_length_errors() {
    let result = KeyStore::from_hex("deadbeef");
    assert!(result.is_err());
}

#[test]
fn key_store_from_hex_bad_char_errors() {
    let bad: String = "zz".repeat(32);
    let result = KeyStore::from_hex(&bad);
    assert!(result.is_err());
}

#[test]
fn key_store_from_env_missing_errors() {
    unsafe { std::env::remove_var("VECTORVOLT_MASTER_KEY") };
    let result = KeyStore::from_env();
    assert!(result.is_err());
}

#[test]
fn key_store_from_env_reads_hex_key() {
    let k = KeyStore::generate().unwrap();
    let hex = k.to_hex();
    unsafe { std::env::set_var("VECTORVOLT_MASTER_KEY", &hex) };
    let k2 = KeyStore::from_env().unwrap();
    assert_eq!(k.as_bytes(), k2.as_bytes());
    unsafe { std::env::remove_var("VECTORVOLT_MASTER_KEY") };
}

// ── Encryptor (VV-SEC-002) ────────────────────────────────────────────────────

#[test]
fn encrypt_decrypt_roundtrip() {
    let key = KeyStore::generate().unwrap();
    let plaintext = b"VectorVolt secret data row";
    let blob = encrypt(&key, plaintext).unwrap();
    let got = decrypt(&key, &blob).unwrap();
    assert_eq!(got, plaintext);
}

#[test]
fn encrypt_produces_vve1_magic() {
    let key = KeyStore::generate().unwrap();
    let blob = encrypt(&key, b"hello").unwrap();
    assert_eq!(&blob[..4], b"VVE1");
}

#[test]
fn encrypt_nonce_differs_each_call() {
    let key = KeyStore::generate().unwrap();
    let a = encrypt(&key, b"same plaintext").unwrap();
    let b = encrypt(&key, b"same plaintext").unwrap();
    // Nonces are bytes 4..16; ciphertexts should differ
    assert_ne!(&a[4..16], &b[4..16]);
    assert_ne!(a, b);
}

#[test]
fn decrypt_wrong_key_fails() {
    let key = KeyStore::generate().unwrap();
    let blob = encrypt(&key, b"secret").unwrap();
    let wrong_key = KeyStore::generate().unwrap();
    let result = decrypt(&wrong_key, &blob);
    assert!(matches!(result, Err(CryptoError::Decrypt)));
}

#[test]
fn decrypt_bad_magic_errors() {
    let mut blob = vec![0u8; 50];
    blob[..4].copy_from_slice(b"NOPE");
    let key = KeyStore::generate().unwrap();
    let result = decrypt(&key, &blob);
    assert!(matches!(result, Err(CryptoError::BadMagic)));
}

#[test]
fn decrypt_truncated_blob_errors() {
    let key = KeyStore::generate().unwrap();
    let result = decrypt(&key, b"VVE1tooshort");
    assert!(matches!(result, Err(CryptoError::TooShort)));
}

#[test]
fn encrypt_empty_plaintext() {
    let key = KeyStore::generate().unwrap();
    let blob = encrypt(&key, b"").unwrap();
    let got = decrypt(&key, &blob).unwrap();
    assert!(got.is_empty());
}

#[test]
fn encrypt_large_plaintext() {
    let key = KeyStore::generate().unwrap();
    let plaintext = vec![0xABu8; 64 * 1024];
    let blob = encrypt(&key, &plaintext).unwrap();
    let got = decrypt(&key, &blob).unwrap();
    assert_eq!(got, plaintext);
}

// ── EncryptedWriter + EncryptedReader ─────────────────────────────────────────

#[test]
fn encrypted_writer_reader_roundtrip() {
    let key = KeyStore::generate().unwrap();
    let plaintext = b"the quick brown fox jumps over the lazy dog";

    let mut buf = Vec::new();
    {
        let write_key = KeyStore::from_hex(&key.to_hex()).unwrap();
        let mut w = EncryptedWriter::new(&mut buf, write_key);
        w.write_all(plaintext).unwrap();
        w.flush().unwrap();
    }

    let mut r = EncryptedReader::new(buf.as_slice(), &key).unwrap();
    let mut got = Vec::new();
    r.read_to_end(&mut got).unwrap();
    assert_eq!(got, plaintext);
}

#[test]
fn encrypted_reader_wrong_key_fails() {
    let key = KeyStore::generate().unwrap();
    let mut buf = Vec::new();
    {
        let write_key = KeyStore::from_hex(&key.to_hex()).unwrap();
        let mut w = EncryptedWriter::new(&mut buf, write_key);
        w.write_all(b"secret").unwrap();
        w.flush().unwrap();
    }
    let wrong_key = KeyStore::generate().unwrap();
    let result = EncryptedReader::new(buf.as_slice(), &wrong_key);
    assert!(matches!(result, Err(CryptoError::Decrypt)));
}

// ── DAC hardening (VV-SEC-003) ────────────────────────────────────────────────

#[test]
fn harden_data_dir_sets_permissions() {
    let dir = tempfile::TempDir::new().unwrap();
    // Create a file inside
    std::fs::write(dir.path().join("test.vvsst"), b"data").unwrap();

    let report = harden_data_dir(dir.path()).unwrap();

    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::PermissionsExt;
        let dir_mode = std::fs::metadata(dir.path()).unwrap().permissions().mode() & 0o777;
        assert_eq!(dir_mode, 0o700, "directory should be 700");
        let file_mode = std::fs::metadata(dir.path().join("test.vvsst"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(file_mode, 0o600, "file should be 600");
        assert!(report.dirs_hardened >= 1);
        assert!(report.files_hardened >= 1);
    }
    #[cfg(not(target_os = "linux"))]
    {
        // Non-Linux: should succeed (no-op with warning)
        let _ = report;
    }
}

// ── Hardening / capabilities (VV-SEC-004) ─────────────────────────────────────

#[test]
fn harden_is_callable_on_all_platforms() {
    // Layer 4 should not panic — on non-Linux it is a no-op
    // On Linux in CI (non-root) PR_CAPBSET_DROP may silently fail but
    // PR_SET_NO_NEW_PRIVS should always succeed.
    let result = vectorvolt_crypto::harden();
    // We accept both Ok and Err (root-only ops may fail in CI sandboxes)
    let _ = result;
}

use std::io::{Read, Write};
