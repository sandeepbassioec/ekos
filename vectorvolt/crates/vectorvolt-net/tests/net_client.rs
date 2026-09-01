//! Tests for Layer 190: vectorvolt-net Client (VV-NET-190).
//! Updated to use real Connection API.

use vectorvolt_net::NetConfig;

#[test]
fn netconfig_stores_addr() {
    let c = NetConfig::new("127.0.0.1", 7700);
    assert_eq!(c.addr, "127.0.0.1");
}

#[test]
fn netconfig_stores_port() {
    let c = NetConfig::new("127.0.0.1", 9999);
    assert_eq!(c.port, 9999);
}

#[test]
fn netconfig_socket_addr() {
    let c = NetConfig::new("127.0.0.1", 7700);
    assert_eq!(c.socket_addr(), "127.0.0.1:7700");
}

#[test]
fn netconfig_default_addr() {
    assert_eq!(NetConfig::default().addr, "127.0.0.1");
}

#[test]
fn netconfig_default_port() {
    assert_eq!(NetConfig::default().port, 7700);
}

#[test]
fn netconfig_clone() {
    let c = NetConfig::default();
    assert_eq!(c, c.clone());
}

#[test]
fn netconfig_custom_port() {
    let c = NetConfig::new("0.0.0.0", 8080);
    assert_eq!(c.port, 8080);
}

#[test]
fn netconfig_custom_addr() {
    let c = NetConfig::new("192.168.1.1", 1234);
    assert_eq!(c.addr, "192.168.1.1");
}

#[test]
fn netconfig_ne() {
    assert_ne!(NetConfig::new("a", 1), NetConfig::new("a", 2));
}

#[test]
fn netconfig_eq() {
    assert_eq!(NetConfig::new("a", 1), NetConfig::new("a", 1));
}

#[test]
fn netconfig_debug() {
    assert!(!format!("{:?}", NetConfig::default()).is_empty());
}

#[test]
fn netconfig_socket_addr_custom() {
    assert_eq!(
        NetConfig::new("10.0.0.1", 9000).socket_addr(),
        "10.0.0.1:9000"
    );
}

#[test]
fn netconfig_port_zero() {
    let c = NetConfig::new("localhost", 0);
    assert_eq!(c.port, 0);
}
