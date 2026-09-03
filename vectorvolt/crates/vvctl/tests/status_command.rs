//! Tests for Layer 194: vvctl status command (VV-CTL-194).

use vvctl::commands::status_cmd::{StatusArgs, run};

fn make(host: &str, port: u16) -> StatusArgs {
    StatusArgs {
        host: host.to_string(),
        port,
    }
}

#[test]
fn status_run_ok() {
    assert!(run(make("127.0.0.1", 7700)).is_ok());
}
#[test]
fn status_host_stored() {
    assert_eq!(make("localhost", 9000).host, "localhost");
}
#[test]
fn status_port_stored() {
    assert_eq!(make("127.0.0.1", 9000).port, 9000);
}
#[test]
fn status_port_zero() {
    assert!(run(make("127.0.0.1", 0)).is_ok());
}
#[test]
fn status_port_max() {
    assert!(run(make("127.0.0.1", 65535)).is_ok());
}
#[test]
fn status_different_host() {
    assert!(run(make("0.0.0.0", 7700)).is_ok());
}
#[test]
fn status_empty_host_ok() {
    assert!(run(make("", 7700)).is_ok());
}
#[test]
fn status_multiple_times() {
    for _ in 0..5 {
        assert!(run(make("127.0.0.1", 7700)).is_ok());
    }
}
#[test]
fn status_host_ipv6_style() {
    assert!(run(make("[::1]", 7700)).is_ok());
}
#[test]
fn status_result_is_unit() {
    let r: anyhow::Result<()> = run(make("x", 1));
    assert!(r.is_ok());
}
#[test]
fn status_host_not_empty() {
    assert!(!make("127.0.0.1", 7700).host.is_empty());
}
#[test]
fn status_port_1234() {
    assert_eq!(make("a", 1234).port, 1234);
}
#[test]
fn status_host_custom() {
    assert_eq!(make("my-server", 7700).host, "my-server");
}
