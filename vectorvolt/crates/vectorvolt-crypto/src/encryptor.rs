//! VV-SEC-002: AES-256-GCM file encryption with per-file random nonce.
//!
//! On-disk format:
//!   [magic: 4 bytes "VVE1"][nonce: 12 bytes][ciphertext + GCM tag: N+16 bytes]
//!
//! Total overhead: 32 bytes per file.
//! Per-file random nonce ensures identical plaintexts produce different ciphertexts.

use aes_gcm::{
    Aes256Gcm, Key, Nonce,
    aead::{Aead, KeyInit},
};
use rand::RngCore;

use crate::error::{CryptoError, CryptoResult};
use crate::key_store::{KEY_LEN, KeyStore};

const MAGIC: &[u8; 4] = b"VVE1";
const NONCE_LEN: usize = 12;
const HEADER_LEN: usize = 4 + NONCE_LEN; // 16 bytes

/// True if `blob` has the shape of an [`encrypt`] output (magic prefix and room
/// for nonce + tag). Lets readers tell encrypted records from plaintext ones
/// (e.g. metadata written unencrypted) without trying a key.
pub fn is_encrypted(blob: &[u8]) -> bool {
    blob.len() >= HEADER_LEN + 16 && &blob[..4] == MAGIC
}

/// Encrypt `plaintext` with the given key, returning `[magic || nonce || ciphertext+tag]`.
pub fn encrypt(key: &KeyStore, plaintext: &[u8]) -> CryptoResult<Vec<u8>> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key.as_bytes()));

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
