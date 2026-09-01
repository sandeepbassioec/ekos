//! Tests for Layer 308.
use vectorvolt_net::NetConfig;

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
fn net_config_ne_different_addr() {
    assert_ne!(NetConfig::new("a", 1), NetConfig::new("b", 1));
}
#[test]
fn net_config_ne_different_port() {
    assert_ne!(NetConfig::new("a", 1), NetConfig::new("a", 2));
}
#[test]
fn net_config_port_max() {
    let c = NetConfig::new("h", 65535);
    assert_eq!(c.port, 65535);
}
#[test]
fn net_config_socket_addr_zero() {
    let c = NetConfig::new("127.0.0.1", 0);
    assert_eq!(c.socket_addr(), "127.0.0.1:0");
}
