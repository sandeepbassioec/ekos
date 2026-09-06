use crate::error::{TpmError, TpmResult};

pub fn is_tpm_present() -> bool {
    false
}

pub fn seal(_data: &[u8], _output_path: &std::path::Path) -> TpmResult<()> {
    Err(TpmError::NotSupported)
}

pub fn unseal(_input_path: &std::path::Path) -> TpmResult<Vec<u8>> {
    Err(TpmError::NotSupported)
}
