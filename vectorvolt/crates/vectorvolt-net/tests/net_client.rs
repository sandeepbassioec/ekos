//! Tests for Layer 190: vectorvolt-net NetClient (VV-NET-190).

use vectorvolt_net::NetClient;

#[test]
fn connect_stores_addr() {
    let c = NetClient::connect("127.0.0.1:7700").unwrap();
    assert_eq!(c.addr, "127.0.0.1:7700");
}
#[test]
fn connect_returns_ok() {
    assert!(NetClient::connect("localhost:9999").is_ok());
}
#[test]
fn ping_returns_ok() {
    let c = NetClient::connect("127.0.0.1:7700").unwrap();
    assert!(c.ping().is_ok());
}
#[test]
fn connect_empty_addr_ok() {
    assert!(NetClient::connect("").is_ok());
}
#[test]
fn connect_custom_addr() {
    let c = NetClient::connect("10.0.0.1:1234").unwrap();
    assert_eq!(c.addr, "10.0.0.1:1234");
}
#[test]
fn clone_addr() {
    let c = NetClient::connect("a:1").unwrap();
    assert_eq!(c.clone().addr, "a:1");
}
#[test]
fn debug_contains_addr() {
    let c = NetClient::connect("host:42").unwrap();
    let s = format!("{:?}", c);
    assert!(s.contains("host:42"));
}
#[test]
fn ping_twice_ok() {
    let c = NetClient::connect("x:1").unwrap();
    assert!(c.ping().is_ok());
    assert!(c.ping().is_ok());
}
#[test]
fn multiple_clients_ok() {
    for i in 0..5 {
        let c = NetClient::connect(&format!("addr:{}", i)).unwrap();
        assert!(c.ping().is_ok());
    }
}
#[test]
fn addr_ipv6_style() {
    let c = NetClient::connect("[::1]:7700").unwrap();
    assert!(c.ping().is_ok());
}
#[test]
fn connect_preserves_port() {
    let c = NetClient::connect("127.0.0.1:8888").unwrap();
    assert!(c.addr.contains("8888"));
}
#[test]
fn ping_many_times() {
    let c = NetClient::connect("x:0").unwrap();
    for _ in 0..10 {
        assert!(c.ping().is_ok());
    }
}
#[test]
fn connect_addr_field_not_empty() {
    let c = NetClient::connect("127.0.0.1:7700").unwrap();
    assert!(!c.addr.is_empty());
}
