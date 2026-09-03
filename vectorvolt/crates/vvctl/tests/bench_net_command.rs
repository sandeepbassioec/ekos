//! Tests for Layer 198: vvctl bench-net command (VV-CTL-198).

use vvctl::commands::bench_net::{BenchNetArgs, run};

fn args(host: &str, port: u16, requests: u32) -> BenchNetArgs {
    BenchNetArgs {
        host: host.into(),
        port,
        requests,
    }
}

#[test]
fn bench_net_ok() {
    assert!(run(args("127.0.0.1", 7700, 10)).is_ok());
}
#[test]
fn bench_net_zero_requests_ok() {
    assert!(run(args("127.0.0.1", 7700, 0)).is_ok());
}
#[test]
fn bench_net_many_requests_ok() {
    assert!(run(args("127.0.0.1", 7700, 1000)).is_ok());
}
#[test]
fn bench_net_localhost_ok() {
    assert!(run(args("localhost", 7700, 1)).is_ok());
}
#[test]
fn bench_net_custom_port_ok() {
    assert!(run(args("127.0.0.1", 9000, 5)).is_ok());
}
#[test]
fn bench_net_host_stored() {
    let a = args("10.0.0.1", 8080, 1);
    assert_eq!(a.host, "10.0.0.1");
}
#[test]
fn bench_net_port_stored() {
    let a = args("127.0.0.1", 1234, 1);
    assert_eq!(a.port, 1234);
}
#[test]
fn bench_net_requests_stored() {
    let a = args("127.0.0.1", 7700, 42);
    assert_eq!(a.requests, 42);
}
#[test]
fn bench_net_returns_ok() {
    let r = run(args("127.0.0.1", 7700, 5));
    assert!(r.is_ok());
}
#[test]
fn bench_net_high_port_ok() {
    assert!(run(args("127.0.0.1", 65000, 1)).is_ok());
}
#[test]
fn bench_net_ipv4_ok() {
    assert!(run(args("192.168.1.1", 7700, 1)).is_ok());
}
#[test]
fn bench_net_does_not_panic() {
    let r = std::panic::catch_unwind(|| run(args("127.0.0.1", 7700, 3)));
    assert!(r.is_ok());
}
#[test]
fn bench_net_large_count_ok() {
    assert!(run(args("127.0.0.1", 7700, 10000)).is_ok());
}
