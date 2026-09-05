//! `EncryptedWriter` and `EncryptedReader` — transparent AES-256-GCM I/O wrappers.
//!
//! `EncryptedWriter` buffers plaintext in memory; on `flush()` it encrypts the
//! entire buffer and writes the ciphertext blob to the inner writer.
//!
//! `EncryptedReader` reads the entire ciphertext from the inner reader, decrypts
//! it, then serves bytes from the resulting plaintext via `std::io::Cursor`.

use std::io::{self, Cursor, Read, Write};

use crate::encryptor::{decrypt, encrypt};
use crate::error::CryptoError;
use crate::key_store::KeyStore;

/// Buffers plaintext; encrypts and writes on `flush()`.
pub struct EncryptedWriter<W: Write> {
    inner: W,
    key: KeyStore,
    buf: Vec<u8>,
    flushed: bool,
}

impl<W: Write> EncryptedWriter<W> {
    pub fn new(inner: W, key: KeyStore) -> Self {
        Self {
            inner,
            key,
            buf: Vec::new(),
            flushed: false,
        }
    }

    /// How many plaintext bytes are buffered.
    pub fn buffered_len(&self) -> usize {
        self.buf.len()
    }
}

impl<W: Write> Write for EncryptedWriter<W> {
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        self.buf.extend_from_slice(data);
        Ok(data.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        if self.buf.is_empty() && self.flushed {
            return Ok(());
        }
        let blob = encrypt(&self.key, &self.buf).map_err(|e| io::Error::other(e.to_string()))?;
        self.inner.write_all(&blob)?;
        self.inner.flush()?;
        self.buf.clear();
        self.flushed = true;
        Ok(())
    }
}

/// Reads all ciphertext from inner reader, decrypts, serves plaintext.
pub struct EncryptedReader {
    inner: Cursor<Vec<u8>>,
}

impl EncryptedReader {
    pub fn new<R: Read>(mut reader: R, key: &KeyStore) -> Result<Self, CryptoError> {
        let mut blob = Vec::new();
        reader.read_to_end(&mut blob).map_err(CryptoError::Io)?;
        let plaintext = decrypt(key, &blob)?;
        Ok(Self {
            inner: Cursor::new(plaintext),
        })
    }

    pub fn plaintext_len(&self) -> usize {
        self.inner.get_ref().len()
    }
}

impl Read for EncryptedReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }
}
