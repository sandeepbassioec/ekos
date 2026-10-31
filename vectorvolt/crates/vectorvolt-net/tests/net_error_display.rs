//! Tests for Layer 306.
use vectorvolt_net::NetError;

#[test]
fn bad_magic_display() {
    let s = format!("{}", NetError::BadMagic);
    assert!(!s.is_empty());
}
#[test]
fn bad_magic_debug() {
    let s = format!("{:?}", NetError::BadMagic);
    assert!(!s.is_empty());
}
#[test]
fn unsupported_version_display() {
    let s = format!("{}", NetError::UnsupportedVersion(2));
    assert!(s.contains('2'));
}
#[test]
fn unknown_opcode_display() {
    let s = format!("{}", NetError::UnknownOpcode(200));
    assert!(s.contains("200"));
}
#[test]
fn not_a_request_display() {
    let s = format!("{}", NetError::NotARequest(130));
    assert!(!s.is_empty());
}
#[test]
fn not_a_response_display() {
    let s = format!("{}", NetError::NotAResponse(1));
    assert!(!s.is_empty());
}
#[test]
fn payload_too_large_display() {
    let s = format!("{}", NetError::PayloadTooLarge { len: 100, max: 50 });
    assert!(!s.is_empty());
}
#[test]
fn truncated_display() {
    let s = format!("{}", NetError::Truncated { need: 10, got: 5 });
    assert!(!s.is_empty());
}
#[test]
fn crc_mismatch_display() {
    let s = format!(
        "{}",
        NetError::CrcMismatch {
            expected: 0xAB,
            actual: 0xCD
        }
    );
    assert!(!s.is_empty());
}
#[test]
fn malformed_display() {
    let s = format!("{}", NetError::Malformed("bad"));
    assert!(s.contains("bad"));
}
#[test]
fn engine_display() {
    let s = format!("{}", NetError::Engine("fail".into()));
    assert!(s.contains("fail"));
}
#[test]
fn tls_display() {
    let s = format!("{}", NetError::Tls("cert".into()));
    assert!(s.contains("cert"));
}
#[test]
fn io_from() {
    let io = std::io::Error::other("io");
    let e = NetError::from(io);
    let s = format!("{e}");
    assert!(!s.is_empty());
}
