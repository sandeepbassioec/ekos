//! Tests for Layer 307.
use std::net::SocketAddr;
use vectorvolt_net::auth::AuthConfig;
use vectorvolt_net::server::ServerConfig;

fn addr() -> SocketAddr {
    "127.0.0.1:0".parse().unwrap()
}

#[test]
fn server_config_new() {
    let c = ServerConfig::new(addr());
    assert_eq!(c.addr, addr());
}
#[test]
fn default_max_connections() {
    let c = ServerConfig::new(addr());
    assert!(c.max_connections > 0);
}
#[test]
fn default_nodelay_true() {
    let c = ServerConfig::new(addr());
    assert!(c.nodelay);
}
#[test]
fn default_no_auth() {
    let c = ServerConfig::new(addr());
    assert!(!c.requires_auth());
}
#[test]
fn default_no_tls() {
    let c = ServerConfig::new(addr());
    assert!(!c.is_encrypted());
}
#[test]
fn with_auth_enables_auth() {
    let c = ServerConfig::new(addr()).with_auth(AuthConfig::disabled().with_key(
        "k",
        vectorvolt_net::auth::Secret::new(vec![0u8; 16]).unwrap(),
    ));
    assert!(c.requires_auth());
}
#[test]
fn server_config_debug() {
    let s = format!("{:?}", ServerConfig::new(addr()));
    assert!(!s.is_empty());
}
#[test]
fn server_config_clone() {
    let c = ServerConfig::new(addr());
    let c2 = c.clone();
    assert_eq!(c.addr, c2.addr);
}
#[test]
fn server_config_addr() {
    let a: SocketAddr = "127.0.0.1:7700".parse().unwrap();
    let c = ServerConfig::new(a);
    assert_eq!(c.addr.port(), 7700);
}
#[test]
fn server_config_nodelay_field() {
    let mut c = ServerConfig::new(addr());
    c.nodelay = false;
    assert!(!c.nodelay);
}
#[test]
fn server_config_max_connections_field() {
    let mut c = ServerConfig::new(addr());
    c.max_connections = 5;
    assert_eq!(c.max_connections, 5);
}
#[test]
fn server_config_auth_disabled() {
    let c = ServerConfig::new(addr()).with_auth(AuthConfig::disabled());
    assert!(!c.requires_auth());
}
#[test]
fn server_config_tls_none_by_default() {
    let c = ServerConfig::new(addr());
    assert!(c.tls.is_none());
}
