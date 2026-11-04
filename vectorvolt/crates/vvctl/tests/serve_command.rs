//! Tests for Layer 188: vvctl serve command args (VV-CTL-188).
// NOTE: ServeArgs fields match the actual struct in commands/serve.rs

use std::net::SocketAddr;
use std::path::PathBuf;
use vvctl::commands::serve::ServeArgs;

fn make_args(addr: &str) -> ServeArgs {
    ServeArgs {
        dir: PathBuf::from("."),
        columns: vec![],
        addr: addr.parse().unwrap(),
        max_connections: 256,
        secret: None,
        key_id: "default".to_string(),
        key_scope: None,
        key_file: None,
        cert: None,
        key: None,
        multi: false,
        config: None,
        master_config: None,
        flush_threshold: None,
        durability: None,
        max_partitions: None,
        catalog: false,
        restore_catalog: false,
        http_addr: None,
        http_token: None,
        transport: "tcp".to_string(),
        xdp_iface: "lo".to_string(),
        xdp_queue: 0,
        xdp_ring_size: 256,
        grpc_port: 50051,
        no_grpc: false,
        grpc_host: std::net::IpAddr::from([127, 0, 0, 1]),
        grpc_token: None,
        tpm_unseal: false,
        encrypt_at_rest: false,
        metrics_port: 0,
        metrics_host: std::net::IpAddr::from([127, 0, 0, 1]),
        pgwire_port: None,
        cluster_bind: None,
        cluster_seeds: vec![],
        cluster_node_id: None,
        cluster_status_port: 7900,
        listener: Default::default(),
    }
}

#[test]
fn serve_args_default_addr() {
    let a = make_args("127.0.0.1:7700");
    assert_eq!(a.addr.port(), 7700);
}
#[test]
fn serve_args_default_host() {
    let a = make_args("127.0.0.1:7700");
    assert_eq!(a.addr.ip().to_string(), "127.0.0.1");
}
#[test]
fn serve_args_port_9000() {
    let a = make_args("0.0.0.0:9000");
    assert_eq!(a.addr.port(), 9000);
}
#[test]
fn serve_args_columns_empty() {
    assert!(make_args("127.0.0.1:7700").columns.is_empty());
}
#[test]
fn serve_args_secret_none() {
    assert!(make_args("127.0.0.1:7700").secret.is_none());
}
#[test]
fn serve_args_cert_none() {
    assert!(make_args("127.0.0.1:7700").cert.is_none());
}
#[test]
fn serve_args_key_none() {
    assert!(make_args("127.0.0.1:7700").key.is_none());
}
#[test]
fn serve_args_config_none() {
    assert!(make_args("127.0.0.1:7700").config.is_none());
}
#[test]
fn serve_args_multi_false() {
    assert!(!make_args("127.0.0.1:7700").multi);
}
#[test]
fn serve_args_max_connections() {
    assert_eq!(make_args("127.0.0.1:7700").max_connections, 256);
}
#[test]
fn serve_args_key_id_default() {
    assert_eq!(make_args("127.0.0.1:7700").key_id, "default");
}
#[test]
fn serve_args_dir_is_dot() {
    assert_eq!(make_args("127.0.0.1:7700").dir, PathBuf::from("."));
}
#[test]
fn serve_args_addr_is_socket_addr() {
    let a: SocketAddr = "127.0.0.1:7700".parse().unwrap();
    assert_eq!(make_args("127.0.0.1:7700").addr, a);
}
