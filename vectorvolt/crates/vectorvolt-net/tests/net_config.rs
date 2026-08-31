//! Tests for Layer 185: vectorvolt-net NetConfig (VV-NET-185).

use vectorvolt_net::NetConfig;

#[test]
fn default_addr() {
    assert_eq!(NetConfig::default().addr, "127.0.0.1");
}
#[test]
fn default_port() {
    assert_eq!(NetConfig::default().port, 7700);
}
#[test]
fn custom_addr() {
    let c = NetConfig::new("0.0.0.0", 8080);
    assert_eq!(c.addr, "0.0.0.0");
}
#[test]
fn custom_port() {
    let c = NetConfig::new("0.0.0.0", 8080);
    assert_eq!(c.port, 8080);
}
#[test]
fn socket_addr_default() {
    assert_eq!(NetConfig::default().socket_addr(), "127.0.0.1:7700");
}
#[test]
fn socket_addr_custom() {
    let c = NetConfig::new("10.0.0.1", 9000);
    assert_eq!(c.socket_addr(), "10.0.0.1:9000");
}
#[test]
fn clone_equal() {
    let a = NetConfig::default();
    assert_eq!(a.clone(), a);
}
#[test]
fn debug_contains_addr() {
    let s = format!("{:?}", NetConfig::default());
    assert!(s.contains("127.0.0.1"));
}
#[test]
fn debug_contains_port() {
    let s = format!("{:?}", NetConfig::default());
    assert!(s.contains("7700"));
}
#[test]
fn new_with_string() {
    let c = NetConfig::new(String::from("192.168.1.1"), 1234);
    assert_eq!(c.addr, "192.168.1.1");
}
#[test]
fn port_zero_allowed() {
    let c = NetConfig::new("127.0.0.1", 0);
    assert_eq!(c.port, 0);
}
#[test]
fn port_max() {
    let c = NetConfig::new("127.0.0.1", 65535);
    assert_eq!(c.port, 65535);
}
#[test]
fn eq_same_values() {
    assert_eq!(NetConfig::new("a", 1), NetConfig::new("a", 1));
}
