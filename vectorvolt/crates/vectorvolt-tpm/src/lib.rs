pub mod error;

#[cfg(all(target_os = "linux", feature = "tpm"))]
mod linux;

#[cfg(not(all(target_os = "linux", feature = "tpm")))]
mod stub;

use std::path::Path;

pub use error::{TpmError, TpmResult};

/// Returns true if a TPM 2.0 device is accessible on the current system.
pub fn is_tpm_present() -> bool {
    #[cfg(all(target_os = "linux", feature = "tpm"))]
    return linux::is_tpm_present();
    #[cfg(not(all(target_os = "linux", feature = "tpm")))]
    return stub::is_tpm_present();
}

/// Seal `data` into a TPM-bound blob at `output_path`.
/// On non-Linux or without the `tpm` feature, returns `TpmError::NotSupported`.
pub fn seal(data: &[u8], output_path: &Path) -> TpmResult<()> {
    #[cfg(all(target_os = "linux", feature = "tpm"))]
    return linux::seal(data, output_path);
    #[cfg(not(all(target_os = "linux", feature = "tpm")))]
    return stub::seal(data, output_path);
}

/// Unseal a blob written by `seal`. Fails if the PCR policy (boot chain) has changed.
/// On non-Linux or without the `tpm` feature, returns `TpmError::NotSupported`.
pub fn unseal(input_path: &Path) -> TpmResult<Vec<u8>> {
    #[cfg(all(target_os = "linux", feature = "tpm"))]
    return linux::unseal(input_path);
    #[cfg(not(all(target_os = "linux", feature = "tpm")))]
    return stub::unseal(input_path);
}
