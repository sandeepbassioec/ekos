//! Tests for Layer 320: NetError from io::Error, Display non-empty, error_code constants (VV-NET-320).

use vectorvolt_net::{NetError, error_code};

#[test]
fn error_code_engine_is_one() {
    assert_eq!(error_code::ENGINE, 1);
}

#[test]
fn error_code_malformed_is_two() {
    assert_eq!(error_code::MALFORMED, 2);
}

#[test]
fn error_code_duplicate_key_is_three() {
    assert_eq!(error_code::DUPLICATE_KEY, 3);
}

#[test]
fn error_code_not_found_is_four() {
    assert_eq!(error_code::NOT_FOUND, 4);
}

#[test]
fn error_code_schema_is_five() {
    assert_eq!(error_code::SCHEMA, 5);
}

#[test]
fn error_code_unauthenticated_is_six() {
    assert_eq!(error_code::UNAUTHENTICATED, 6);
}

#[test]
fn net_error_bad_magic_display() {
    let err = NetError::BadMagic;
    assert!(!err.to_string().is_empty());
}

#[test]
fn net_error_unknown_opcode_display() {
    let err = NetError::UnknownOpcode(99);
    let msg = err.to_string();
    assert!(msg.contains("99") || !msg.is_empty());
}

#[test]
fn net_error_from_io() {
    let io_err = std::io::Error::new(std::io::ErrorKind::BrokenPipe, "broken");
    let net_err: NetError = io_err.into();
    assert!(matches!(net_err, NetError::Io(_)));
}

#[test]
fn net_error_io_display() {
    let io_err = std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "refused");
    let net_err: NetError = io_err.into();
    assert!(!net_err.to_string().is_empty());
}

#[test]
fn net_error_engine_display() {
    let err = NetError::Engine("fail".into());
    assert!(err.to_string().contains("fail") || !err.to_string().is_empty());
}

#[test]
fn net_error_tls_display() {
    let err = NetError::Tls("cert error".into());
    assert!(!err.to_string().is_empty());
}

#[test]
fn error_code_forbidden_is_eight() {
    assert_eq!(error_code::FORBIDDEN, 8);
}
