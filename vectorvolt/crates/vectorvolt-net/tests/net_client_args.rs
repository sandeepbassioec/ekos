//! Tests for Layer 308.
use vectorvolt_net::{ConnectionPool, NetClient, NetConfig};

#[test]
fn net_config_default_addr() {
    assert_eq!(NetConfig::default().addr, "127.0.0.1");
}
#[test]
fn net_config_default_port() {
    assert_eq!(NetConfig::default().port, 7700);
}
#[test]
fn net_config_new() {
    let c = NetConfig::new("0.0.0.0", 8080);
    assert_eq!(c.port, 8080);
}
#[test]
fn net_config_socket_addr() {
    let c = NetConfig::new("127.0.0.1", 9000);
    assert_eq!(c.socket_addr(), "127.0.0.1:9000");
}
#[test]
fn net_config_debug() {
    let s = format!("{:?}", NetConfig::default());
    assert!(!s.is_empty());
}
#[test]
fn net_config_clone() {
    let c = NetConfig::default();
    let c2 = c.clone();
    assert_eq!(c, c2);
}
#[test]
fn net_config_partial_eq() {
    assert_eq!(NetConfig::default(), NetConfig::default());
}
#[test]
fn net_config_custom_addr() {
    let c = NetConfig::new("10.0.0.1", 1234);
    assert_eq!(c.addr, "10.0.0.1");
}
#[test]
fn net_client_connect_stub() {
    let c = NetClient::connect("127.0.0.1:9999").unwrap();
    assert_eq!(c.addr, "127.0.0.1:9999");
}
#[test]
fn net_client_ping_stub() {
    let c = NetClient::connect("localhost:1").unwrap();
    assert!(c.ping().is_ok());
}
#[test]
fn net_client_debug() {
    let c = NetClient::connect("x").unwrap();
    let s = format!("{c:?}");
    assert!(!s.is_empty());
}
#[test]
fn net_client_clone() {
    let c = NetClient::connect("x").unwrap();
    let c2 = c.clone();
    assert_eq!(c.addr, c2.addr);
}
#[test]
fn connection_pool_max() {
    let p = ConnectionPool::new(10);
    assert_eq!(p.size(), 10);
}
