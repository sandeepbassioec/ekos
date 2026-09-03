//! Tests for Layer 191: vvctl ping command (VV-CTL-191).

use vvctl::commands::ping::{PingArgs, run};

fn make(host: &str, port: u16, count: u32) -> PingArgs {
    PingArgs {
        host: host.to_string(),
        port,
        count,
    }
}

#[test]
fn ping_run_ok() {
    assert!(run(make("127.0.0.1", 7700, 1)).is_ok());
}
#[test]
fn ping_host_stored() {
    assert_eq!(make("localhost", 9000, 1).host, "localhost");
}
#[test]
fn ping_port_stored() {
    assert_eq!(make("127.0.0.1", 9000, 1).port, 9000);
}
#[test]
fn ping_count_stored() {
    assert_eq!(make("127.0.0.1", 7700, 5).count, 5);
}
#[test]
fn ping_count_zero_ok() {
    assert!(run(make("127.0.0.1", 7700, 0)).is_ok());
}
#[test]
fn ping_count_one_ok() {
    assert!(run(make("127.0.0.1", 7700, 1)).is_ok());
}
#[test]
fn ping_count_ten_ok() {
    assert!(run(make("127.0.0.1", 7700, 10)).is_ok());
}
#[test]
fn ping_port_zero_ok() {
    assert!(run(make("127.0.0.1", 0, 1)).is_ok());
}
#[test]
fn ping_port_max_ok() {
    assert!(run(make("127.0.0.1", 65535, 1)).is_ok());
}
#[test]
fn ping_different_host() {
    assert!(run(make("0.0.0.0", 7700, 1)).is_ok());
}
#[test]
fn ping_multiple_times() {
    for _ in 0..5 {
        assert!(run(make("127.0.0.1", 7700, 1)).is_ok());
    }
}
#[test]
fn ping_count_three() {
    assert_eq!(make("x", 1, 3).count, 3);
}
#[test]
fn ping_host_empty_ok() {
    assert!(run(make("", 7700, 1)).is_ok());
}
