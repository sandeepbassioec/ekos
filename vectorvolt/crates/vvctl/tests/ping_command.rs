//! Tests for vvctl ping (VV-CTL-191): real wire-protocol round trips.

mod common;

use common::{TestServer, dead_port};
use vvctl::commands::ping::{PingArgs, ping_rtts, run};
use vvctl::commands::wire::WireOpts;

fn make(port: u16, count: u32) -> PingArgs {
    PingArgs {
        host: "127.0.0.1".into(),
        port,
        count,
        conn: WireOpts::default(),
    }
}

#[test]
fn ping_measures_one_rtt_per_request() {
    let s = TestServer::start(0);
    let rtts = ping_rtts(&make(s.port, 5)).unwrap();
    assert_eq!(rtts.len(), 5);
    assert!(rtts.iter().all(|d| !d.is_zero()));
}

#[test]
fn ping_run_succeeds_against_live_server() {
    let s = TestServer::start(0);
    run(make(s.port, 3)).unwrap();
}

#[test]
fn ping_fails_when_nothing_listens() {
    let err = run(make(dead_port(), 1)).unwrap_err();
    assert!(err.to_string().contains("connect"), "{err:#}");
}

#[test]
fn ping_zero_count_is_rejected() {
    let s = TestServer::start(0);
    assert!(run(make(s.port, 0)).is_err());
}

#[test]
fn ping_empty_host_is_rejected() {
    let mut a = make(1, 1);
    a.host = String::new();
    assert!(run(a).is_err());
}
