use thiserror::Error;

#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("AES-GCM encryption failed")]
    Encrypt,
    #[error("AES-GCM decryption failed (wrong key or corrupted data)")]
    Decrypt,
    #[error("invalid file magic — not a VectorVolt encrypted file")]
    BadMagic,
    #[error("unsupported encryption version: {0}")]
    BadVersion(u8),
    #[error("file too short to contain encryption header")]
    TooShort,
    #[error("key store error: {0}")]
    KeyStore(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

pub type CryptoResult<T> = Result<T, CryptoError>;
