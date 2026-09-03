//! Tests for Layer 200: vvctl disconnect command (VV-CTL-200).

use vvctl::commands::disconnect::{DisconnectArgs, run};

fn args(host: &str, port: u16) -> DisconnectArgs {
    DisconnectArgs {
        host: host.into(),
        port,
    }
}

#[test]
fn disconnect_ok() {
    assert!(run(args("127.0.0.1", 7700)).is_ok());
}
#[test]
fn disconnect_localhost_ok() {
    assert!(run(args("localhost", 7700)).is_ok());
}
#[test]
fn disconnect_custom_port_ok() {
    assert!(run(args("127.0.0.1", 9000)).is_ok());
}
#[test]
fn disconnect_host_stored() {
    let a = args("10.0.0.1", 8080);
    assert_eq!(a.host, "10.0.0.1");
}
#[test]
fn disconnect_port_stored() {
    let a = args("127.0.0.1", 1234);
    assert_eq!(a.port, 1234);
}
#[test]
fn disconnect_multiple_ok() {
    for _ in 0..3 {
        assert!(run(args("127.0.0.1", 7700)).is_ok());
    }
}
#[test]
fn disconnect_high_port_ok() {
    assert!(run(args("127.0.0.1", 65000)).is_ok());
}
#[test]
fn disconnect_ipv4_ok() {
    assert!(run(args("192.168.1.1", 7700)).is_ok());
}
#[test]
fn disconnect_returns_result() {
    let r = run(args("127.0.0.1", 7700));
    assert!(r.is_ok());
}
#[test]
fn disconnect_zero_port_ok() {
    assert!(run(args("127.0.0.1", 0)).is_ok());
}
#[test]
fn disconnect_does_not_panic() {
    let r = std::panic::catch_unwind(|| run(args("127.0.0.1", 7700)));
    assert!(r.is_ok());
}
#[test]
fn disconnect_long_host_ok() {
    assert!(run(args("very-long-hostname.example.com", 7700)).is_ok());
}
#[test]
fn disconnect_port_max_ok() {
    assert!(run(args("127.0.0.1", u16::MAX)).is_ok());
}
