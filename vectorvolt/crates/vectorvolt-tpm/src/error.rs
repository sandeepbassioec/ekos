use thiserror::Error;

#[derive(Debug, Error)]
pub enum TpmError {
    #[error("TPM is not available on this platform or the 'tpm' feature is disabled")]
    NotSupported,
    #[error("TPM device error: {0}")]
    Device(String),
    #[error("sealing failed: {0}")]
    Seal(String),
    #[error("unsealing failed — PCR policy mismatch or tampered boot chain: {0}")]
    Unseal(String),
    #[error("sealed blob is malformed")]
    BadBlob,
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

pub type TpmResult<T> = Result<T, TpmError>;
