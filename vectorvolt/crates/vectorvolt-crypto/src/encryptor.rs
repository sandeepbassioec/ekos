//! VV-SEC-002: AES-256-GCM file encryption with per-file random nonce.
//!
//! On-disk format:
//!   [magic: 4 bytes "VVE1"][nonce: 12 bytes][ciphertext + GCM tag: N+16 bytes]
//!
//! Total overhead: 32 bytes per file.
//! Per-file random nonce ensures identical plaintexts produce different ciphertexts.
//!
//! [`encrypt_with_aad`] writes the same layout under magic `"VVE2"`, with the
//! caller's additional authenticated data bound into the GCM tag (PP-52: a WAL
//! frame payload is bound to its sequence number, so a ciphertext moved to
//! another position no longer decrypts). `VVE1` blobs remain readable through
//! [`decrypt`]; [`decrypt_with_aad`] accepts only `VVE2`, so an AAD-bound
//! position can never be satisfied by an unbound `VVE1` blob.
//!
//! Every encryption calls [`KeyStore::record_use`] (one per random nonce);
//! callers can check [`KeyStore::needs_rotation`].

use aes_gcm::{
    Aes256Gcm, Key, Nonce,
    aead::{Aead, KeyInit, Payload},
};
use rand::RngCore;

use crate::error::{CryptoError, CryptoResult};
use crate::key_store::{KEY_LEN, KeyStore};

const MAGIC: &[u8; 4] = b"VVE1";
const MAGIC_AAD: &[u8; 4] = b"VVE2";
const NONCE_LEN: usize = 12;
const HEADER_LEN: usize = 4 + NONCE_LEN; // 16 bytes

/// True if `blob` has the shape of an [`encrypt`] or [`encrypt_with_aad`]
/// output (magic prefix and room for nonce + tag). Lets readers tell encrypted
/// records from plaintext ones (e.g. metadata written unencrypted) without
/// trying a key.
pub fn is_encrypted(blob: &[u8]) -> bool {
    blob.len() >= HEADER_LEN + 16 && (&blob[..4] == MAGIC || &blob[..4] == MAGIC_AAD)
}

/// True if `blob` has the shape of an [`encrypt_with_aad`] (`VVE2`) output.
pub fn is_aad_encrypted(blob: &[u8]) -> bool {
    blob.len() >= HEADER_LEN + 16 && &blob[..4] == MAGIC_AAD
}

/// Encrypt `plaintext` with `aad` bound into the tag, returning
/// `["VVE2" || nonce || ciphertext+tag]`. The same `aad` must be supplied to
/// [`decrypt_with_aad`]; it is not stored.
pub fn encrypt_with_aad(key: &KeyStore, plaintext: &[u8], aad: &[u8]) -> CryptoResult<Vec<u8>> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key.as_bytes()));
    let mut nonce_bytes = [0u8; NONCE_LEN];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    key.record_use();
    let ciphertext = cipher
        .encrypt(
            Nonce::from_slice(&nonce_bytes),
            Payload {
                msg: plaintext,
                aad,
            },
        )
        .map_err(|_| CryptoError::Encrypt)?;
    let mut out = Vec::with_capacity(HEADER_LEN + ciphertext.len());
    out.extend_from_slice(MAGIC_AAD);
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(&ciphertext);
    Ok(out)
}

/// Decrypt a `VVE2` blob produced by [`encrypt_with_aad`] with the same
/// `aad`. A different key, altered bytes or a different `aad` (e.g. the blob
/// was moved to another WAL position) all fail with [`CryptoError::Decrypt`].
/// A `VVE1` blob is rejected with [`CryptoError::BadMagic`].
pub fn decrypt_with_aad(key: &KeyStore, blob: &[u8], aad: &[u8]) -> CryptoResult<Vec<u8>> {
    if blob.len() < HEADER_LEN + 16 {
        return Err(CryptoError::TooShort);
    }
    if &blob[..4] != MAGIC_AAD {
        return Err(CryptoError::BadMagic);
    }
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key.as_bytes()));
    cipher
        .decrypt(
            Nonce::from_slice(&blob[4..HEADER_LEN]),
            Payload {
                msg: &blob[HEADER_LEN..],
                aad,
            },
        )
        .map_err(|_| CryptoError::Decrypt)
}

/// Encrypt `plaintext` with the given key, returning `[magic || nonce || ciphertext+tag]`.
pub fn encrypt(key: &KeyStore, plaintext: &[u8]) -> CryptoResult<Vec<u8>> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key.as_bytes()));

    let mut nonce_bytes = [0u8; NONCE_LEN];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    key.record_use();

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|_| CryptoError::Encrypt)?;

    let mut out = Vec::with_capacity(HEADER_LEN + ciphertext.len());
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(&ciphertext);
    Ok(out)
}

/// Decrypt a blob produced by [`encrypt`].
pub fn decrypt(key: &KeyStore, blob: &[u8]) -> CryptoResult<Vec<u8>> {
    if blob.len() < HEADER_LEN + 16 {
        return Err(CryptoError::TooShort);
    }
    if &blob[..4] != MAGIC {
        return Err(CryptoError::BadMagic);
    }
    let nonce = Nonce::from_slice(&blob[4..4 + NONCE_LEN]);
    let ciphertext = &blob[HEADER_LEN..];

    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key.as_bytes()));
    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| CryptoError::Decrypt)
}

/// Encrypt `plaintext` using a raw 32-byte key slice (convenience for tests).
pub fn encrypt_raw(key_bytes: &[u8; KEY_LEN], plaintext: &[u8]) -> CryptoResult<Vec<u8>> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key_bytes));
    let mut nonce_bytes = [0u8; NONCE_LEN];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|_| CryptoError::Encrypt)?;
    let mut out = Vec::with_capacity(HEADER_LEN + ciphertext.len());
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(&ciphertext);
    Ok(out)
}

/// Decrypt using a raw key slice.
pub fn decrypt_raw(key_bytes: &[u8; KEY_LEN], blob: &[u8]) -> CryptoResult<Vec<u8>> {
    if blob.len() < HEADER_LEN + 16 {
        return Err(CryptoError::TooShort);
    }
    if &blob[..4] != MAGIC {
        return Err(CryptoError::BadMagic);
    }
    let nonce = Nonce::from_slice(&blob[4..4 + NONCE_LEN]);
    let ciphertext = &blob[HEADER_LEN..];
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key_bytes));
    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| CryptoError::Decrypt)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aad_round_trip_and_binding() {
        let key = KeyStore::generate().unwrap();
        let blob = encrypt_with_aad(&key, b"row", b"pos-1").unwrap();
        assert!(is_encrypted(&blob));
        assert!(is_aad_encrypted(&blob));
        assert_eq!(decrypt_with_aad(&key, &blob, b"pos-1").unwrap(), b"row");
        assert!(matches!(
            decrypt_with_aad(&key, &blob, b"pos-2"),
            Err(CryptoError::Decrypt)
        ));
        let other = KeyStore::generate().unwrap();
        assert!(decrypt_with_aad(&other, &blob, b"pos-1").is_err());
        // The unbound decoder refuses VVE2, and the bound one refuses VVE1.
        assert!(matches!(decrypt(&key, &blob), Err(CryptoError::BadMagic)));
        let legacy = encrypt(&key, b"row").unwrap();
        assert!(is_encrypted(&legacy) && !is_aad_encrypted(&legacy));
        assert!(matches!(
            decrypt_with_aad(&key, &legacy, b"pos-1"),
            Err(CryptoError::BadMagic)
        ));
        assert_eq!(decrypt(&key, &legacy).unwrap(), b"row");
    }

    #[test]
    fn every_encryption_is_counted() {
        let key = KeyStore::generate().unwrap();
        let before = key.uses();
        encrypt(&key, b"a").unwrap();
        encrypt_with_aad(&key, b"b", b"x").unwrap();
        assert_eq!(key.uses(), before + 2);
    }
}
