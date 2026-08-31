//! Tests for Layer 187: vectorvolt-server start (VV-SRV-187).

use vectorvolt_net::NetConfig;
use vectorvolt_server::{start, ServerConfig};

#[test]
fn start_default_config_ok() {
    assert!(start(ServerConfig::default()).is_ok());
}
#[test]
fn start_custom_port_ok() {
    let cfg = ServerConfig {
        net: NetConfig::new("127.0.0.1", 0),
    };
    assert!(start(cfg).is_ok());
}
#[test]
fn start_twice_ok() {
    assert!(start(ServerConfig::default()).is_ok());
    assert!(start(ServerConfig::default()).is_ok());
}
#[test]
fn server_config_default_addr() {
    assert_eq!(ServerConfig::default().net.addr, "127.0.0.1");
}
#[test]
fn server_config_default_port() {
    assert_eq!(ServerConfig::default().net.port, 7700);
}
#[test]
fn server_config_custom_addr() {
    let cfg = ServerConfig {
        net: NetConfig::new("0.0.0.0", 9000),
    };
    assert_eq!(cfg.net.addr, "0.0.0.0");
}
#[test]
fn server_config_custom_port() {
    let cfg = ServerConfig {
        net: NetConfig::new("127.0.0.1", 1234),
    };
    assert_eq!(cfg.net.port, 1234);
}
#[test]
fn start_many_times() {
    for _ in 0..5 {
        assert!(start(ServerConfig::default()).is_ok());
    }
}
#[test]
fn start_with_port_zero() {
    let cfg = ServerConfig {
        net: NetConfig::new("127.0.0.1", 0),
    };
    assert!(start(cfg).is_ok());
}
#[test]
fn start_returns_unit_on_ok() {
    let r: anyhow::Result<()> = start(ServerConfig::default());
    assert!(r.is_ok());
}
#[test]
fn server_config_net_field_accessible() {
    let cfg = ServerConfig::default();
    let _ = cfg.net;
}
#[test]
fn start_with_ipv4_addr() {
    let cfg = ServerConfig {
        net: NetConfig::new("10.0.0.1", 8080),
    };
    assert!(start(cfg).is_ok());
}
#[test]
fn server_config_is_constructable() {
    let _cfg = ServerConfig {
        net: NetConfig::default(),
    };
}
