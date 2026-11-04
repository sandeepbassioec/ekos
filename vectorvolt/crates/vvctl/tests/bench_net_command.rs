//! Tests for vvctl bench-net (VV-CTL-198): real round trips.

mod common;

use common::{TestServer, dead_port};
use vvctl::commands::bench_net::{BenchNetArgs, bench, run};
use vvctl::commands::wire::WireOpts;

fn args(port: u16, requests: u32) -> BenchNetArgs {
    BenchNetArgs {
        host: "127.0.0.1".into(),
        port,
        requests,
        conn: WireOpts::default(),
    }
}

#[test]
fn bench_net_performs_requested_round_trips() {
    let s = TestServer::start(0);
    let r = bench(&args(s.port, 50)).unwrap();
    assert_eq!(r.latencies.len(), 50);
    assert!(r.total >= r.percentile(100.0));
    assert!(r.percentile(50.0) <= r.percentile(99.0));
}

#[test]
fn bench_net_run_ok() {
    let s = TestServer::start(0);
    run(args(s.port, 10)).unwrap();
}

#[test]
fn bench_net_fails_when_peer_is_down() {
    assert!(bench(&args(dead_port(), 5)).is_err());
}

#[test]
fn bench_net_zero_requests_rejected() {
    let s = TestServer::start(0);
    assert!(run(args(s.port, 0)).is_err());
}
