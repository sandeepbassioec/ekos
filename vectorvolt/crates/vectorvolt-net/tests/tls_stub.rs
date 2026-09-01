//! Tests for Layer 199: vectorvolt-net TLS (VV-NET-199).
//! Updated to use real ClientTlsConfig / ServerTlsConfig API.

use vectorvolt_net::tls::{ClientTlsConfig, ServerTlsConfig};
use vectorvolt_net::{NetConfig, bind};

#[test]
fn bind_and_local_addr() {
    let l = bind(&NetConfig::new("127.0.0.1", 0)).unwrap();
    assert_eq!(l.local_addr().unwrap().ip().to_string(), "127.0.0.1");
}

#[test]
fn server_tls_from_pem_bad_cert() {
    let result = ServerTlsConfig::from_pem(b"notacert", b"notakey");
    assert!(result.is_err());
}

#[test]
fn server_tls_from_pem_empty() {
    let result = ServerTlsConfig::from_pem(b"", b"");
    assert!(result.is_err());
}

#[test]
fn client_tls_pin_bad_cert() {
    let result = ClientTlsConfig::pin_certificate(b"not-a-cert", "localhost");
    assert!(result.is_err());
}

#[test]
fn netconfig_default_works() {
    let c = NetConfig::default();
    assert_eq!(c.port, 7700);
}

#[test]
fn bind_twice_gets_different_ports() {
    let a = bind(&NetConfig::new("127.0.0.1", 0)).unwrap();
    let b = bind(&NetConfig::new("127.0.0.1", 0)).unwrap();
    assert_ne!(
        a.local_addr().unwrap().port(),
        b.local_addr().unwrap().port()
    );
}

#[test]
fn netconfig_port_zero() {
    let c = NetConfig::new("127.0.0.1", 0);
    assert_eq!(c.port, 0);
}

#[test]
fn netconfig_custom() {
    let c = NetConfig::new("0.0.0.0", 9000);
    assert_eq!(c.port, 9000);
}

#[test]
fn netconfig_socket_addr_format() {
    let c = NetConfig::new("127.0.0.1", 8080);
    assert_eq!(c.socket_addr(), "127.0.0.1:8080");
}

#[test]
fn server_tls_bad_key() {
    let result = ServerTlsConfig::from_pem(b"cert", b"badkey");
    assert!(result.is_err());
}

#[test]
fn client_tls_empty_cert() {
    let result = ClientTlsConfig::pin_certificate(b"", "localhost");
    assert!(result.is_err());
}

#[test]
fn bind_succeeds() {
    let l = bind(&NetConfig::new("127.0.0.1", 0)).unwrap();
    assert!(l.local_addr().is_ok());
}
