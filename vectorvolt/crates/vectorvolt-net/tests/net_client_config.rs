//! Tests for Layer 308: NetConfig fields (VV-NET-308).

use vectorvolt_net::NetConfig;

#[test]
fn net_config_default_addr() {
    let cfg = NetConfig::default();
    assert_eq!(cfg.addr, "127.0.0.1");
}

#[test]
fn net_config_default_port() {
    let cfg = NetConfig::default();
    assert_eq!(cfg.port, 7700);
}

#[test]
fn net_config_socket_addr() {
    let cfg = NetConfig::default();
    assert_eq!(cfg.socket_addr(), "127.0.0.1:7700");
}

#[test]
fn net_config_new_custom() {
    let cfg = NetConfig::new("0.0.0.0", 9000);
    assert_eq!(cfg.addr, "0.0.0.0");
    assert_eq!(cfg.port, 9000);
}

#[test]
fn net_config_socket_addr_custom() {
    let cfg = NetConfig::new("10.0.0.1", 8080);
    assert_eq!(cfg.socket_addr(), "10.0.0.1:8080");
}

#[test]
fn net_config_clone_eq() {
    let a = NetConfig::default();
    let b = a.clone();
    assert_eq!(a, b);
}

#[test]
fn net_config_ne_port() {
    assert_ne!(NetConfig::new("x", 1), NetConfig::new("x", 2));
}

#[test]
fn net_config_ne_addr() {
    assert_ne!(NetConfig::new("a", 1), NetConfig::new("b", 1));
}

#[test]
fn net_config_debug_nonempty() {
    let cfg = NetConfig::default();
    assert!(!format!("{:?}", cfg).is_empty());
}

#[test]
fn net_config_port_0() {
    let c = NetConfig::new("h", 0);
    assert_eq!(c.port, 0);
}

#[test]
fn net_config_port_65535() {
    let c = NetConfig::new("h", 65535);
    assert_eq!(c.port, 65535);
}

#[test]
fn net_config_socket_addr_port_80() {
    assert_eq!(
        NetConfig::new("localhost", 80).socket_addr(),
        "localhost:80"
    );
}
