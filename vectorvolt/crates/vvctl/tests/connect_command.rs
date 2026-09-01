//! Tests for Layer 196: vvctl connect command args (VV-CTL-196).

use std::net::SocketAddr;
use std::path::PathBuf;
use vvctl::commands::connect::{ConnectArgs, ConnectCommand};

fn make(addr: &str) -> ConnectArgs {
    ConnectArgs {
        addr: addr.parse().unwrap(),
        secret: None,
        key_id: "default".to_string(),
        ca: None,
        tls_name: None,
        timeout_ms: 5000,
        command: ConnectCommand::Ping,
    }
}

#[test]
fn connect_args_addr() {
    assert_eq!(make("127.0.0.1:7700").addr.port(), 7700);
}
#[test]
fn connect_args_host() {
    assert_eq!(make("127.0.0.1:7700").addr.ip().to_string(), "127.0.0.1");
}
#[test]
fn connect_args_secret_none() {
    assert!(make("127.0.0.1:7700").secret.is_none());
}
#[test]
fn connect_args_key_id_default() {
    assert_eq!(make("127.0.0.1:7700").key_id, "default");
}
#[test]
fn connect_args_ca_none() {
    assert!(make("127.0.0.1:7700").ca.is_none());
}
#[test]
fn connect_args_port_9000() {
    assert_eq!(make("0.0.0.0:9000").addr.port(), 9000);
}
#[test]
fn connect_args_addr_eq() {
    let a: SocketAddr = "127.0.0.1:7700".parse().unwrap();
    assert_eq!(make("127.0.0.1:7700").addr, a);
}
#[test]
fn connect_args_timeout_default() {
    assert_eq!(make("127.0.0.1:7700").timeout_ms, 5000);
}
#[test]
fn connect_args_diff_port() {
    assert_eq!(make("127.0.0.1:1234").addr.port(), 1234);
}
#[test]
fn connect_args_ipv4() {
    assert!(make("127.0.0.1:7700").addr.is_ipv4());
}
#[test]
fn connect_args_key_id_custom() {
    let mut a = make("127.0.0.1:7700");
    a.key_id = "mykey".into();
    assert_eq!(a.key_id, "mykey");
}
#[test]
fn connect_args_ca_path() {
    let mut a = make("127.0.0.1:7700");
    a.ca = Some(PathBuf::from("ca.pem"));
    assert!(a.ca.is_some());
}
#[test]
fn connect_args_secret_set() {
    let mut a = make("127.0.0.1:7700");
    a.secret = Some("abc".into());
    assert_eq!(a.secret.unwrap(), "abc");
}
