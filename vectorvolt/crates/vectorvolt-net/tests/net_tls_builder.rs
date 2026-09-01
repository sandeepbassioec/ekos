//! Tests for Layer 314: TlsConfig stub, ClientTlsConfig, ServerTlsConfig (VV-NET-314).

use rcgen::generate_simple_self_signed;
use vectorvolt_net::tls::{ClientTlsConfig, ServerTlsConfig};

fn self_signed() -> (String, String) {
    let cert = generate_simple_self_signed(vec!["localhost".to_owned()]).unwrap();
    (cert.cert.pem(), cert.key_pair.serialize_pem())
}

#[test]
fn client_tls_config_pin_cert_ok() {
    let (cert_pem, _) = self_signed();
    let result = ClientTlsConfig::pin_certificate(cert_pem.as_bytes(), "localhost");
    assert!(result.is_ok());
}

#[test]
fn client_tls_config_server_name_stored() {
    let (cert_pem, _) = self_signed();
    let cfg = ClientTlsConfig::pin_certificate(cert_pem.as_bytes(), "my-server.local").unwrap();
    assert_eq!(cfg.server_name(), "my-server.local");
}

#[test]
fn server_tls_config_from_pem_ok() {
    let (c, k) = self_signed();
    let result = ServerTlsConfig::from_pem(c.as_bytes(), k.as_bytes());
    assert!(result.is_ok());
}

#[test]
fn server_tls_config_from_pem_bad_cert() {
    let result = ServerTlsConfig::from_pem(b"notacert", b"notakey");
    assert!(result.is_err());
}

#[test]
fn client_tls_debug_nonempty() {
    let (cert_pem, _) = self_signed();
    let cfg = ClientTlsConfig::pin_certificate(cert_pem.as_bytes(), "host").unwrap();
    assert!(!format!("{:?}", cfg).is_empty());
}

#[test]
fn server_tls_config_from_pem_empty_err() {
    let result = ServerTlsConfig::from_pem(b"", b"");
    assert!(result.is_err());
}

#[test]
fn client_tls_pin_cert_bad_pem_err() {
    let result = ClientTlsConfig::pin_certificate(b"not-a-cert", "localhost");
    assert!(result.is_err());
}
