//! Tests for vvctl status (VV-CTL-194): queries a real server.

mod common;

use common::{TestServer, dead_port};
use vvctl::commands::status_cmd::{StatusArgs, query, run};
use vvctl::commands::wire::WireOpts;

fn make(port: u16) -> StatusArgs {
    StatusArgs {
        host: "127.0.0.1".into(),
        port,
        conn: WireOpts::default(),
    }
}

#[test]
fn status_reports_server_row_count_and_latency() {
    let s = TestServer::start(3);
    let st = query(&make(s.port)).unwrap();
    assert_eq!(st.rows, 3);
    assert!(st.rtt_ms > 0.0);
}

#[test]
fn status_tracks_server_state() {
    let empty = TestServer::start(0);
    let full = TestServer::start(7);
    assert_eq!(query(&make(empty.port)).unwrap().rows, 0);
    assert_eq!(query(&make(full.port)).unwrap().rows, 7);
}

#[test]
fn status_run_ok_against_live_server() {
    let s = TestServer::start(1);
    run(make(s.port)).unwrap();
}

#[test]
fn status_fails_when_server_down() {
    assert!(run(make(dead_port())).is_err());
}

#[test]
fn server_status_subcommand_uses_same_path_and_stop_is_unimplemented() {
    use vvctl::commands::server_cmd::{ServerCommand, ServerTarget, run};
    let s = TestServer::start(0);
    let target = |port| ServerTarget {
        host: "127.0.0.1".into(),
        port,
        conn: WireOpts::default(),
    };
    run(ServerCommand::Status(target(s.port))).unwrap();
    assert!(run(ServerCommand::Status(target(dead_port()))).is_err());
    let err = run(ServerCommand::Stop(target(s.port))).unwrap_err();
    assert!(err.to_string().contains("not implemented"));
}
