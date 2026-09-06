use vectorvolt_tpm::{TpmError, is_tpm_present, seal, unseal};

#[test]
fn stub_seal_returns_not_supported() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("sealed.bin");
    let result = seal(b"secret key material", &path);
    // On non-Linux / no tpm feature: NotSupported. On Linux+tpm: may be Device error.
    assert!(result.is_err());
    match result {
        Err(TpmError::NotSupported) | Err(TpmError::Device(_)) => {}
        other => panic!("unexpected result: {other:?}"),
    }
}

#[test]
fn stub_unseal_returns_not_supported() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("no_such_blob.bin");
    let result = unseal(&path);
    assert!(result.is_err());
    match result {
        Err(TpmError::NotSupported) | Err(TpmError::Device(_)) | Err(TpmError::Io(_)) => {}
        other => panic!("unexpected result: {other:?}"),
    }
}

#[test]
fn is_tpm_present_returns_bool() {
    // Just check it doesn't panic; actual value depends on hardware.
    let _ = is_tpm_present();
}

#[test]
fn tpm_error_display_not_supported() {
    let e = TpmError::NotSupported;
    let msg = e.to_string();
    assert!(
        msg.contains("not available") || msg.contains("feature"),
        "{msg}"
    );
}

#[test]
fn tpm_error_display_seal() {
    let e = TpmError::Seal("test reason".into());
    assert!(e.to_string().contains("sealing failed"));
}

#[test]
fn tpm_error_display_unseal() {
    let e = TpmError::Unseal("pcr mismatch".into());
    assert!(e.to_string().contains("PCR") || e.to_string().contains("unsealing"));
}

#[test]
fn tpm_error_display_bad_blob() {
    let e = TpmError::BadBlob;
    assert!(e.to_string().contains("malformed") || e.to_string().contains("blob"));
}

#[test]
fn unseal_bad_blob_file_returns_io_or_bad_blob() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("garbage.bin");
    std::fs::write(&path, b"not a valid blob").unwrap();
    let result = unseal(&path);
    match result {
        Err(TpmError::NotSupported)
        | Err(TpmError::BadBlob)
        | Err(TpmError::Device(_))
        | Err(TpmError::Unseal(_)) => {}
        other => panic!("unexpected result: {other:?}"),
    }
}
