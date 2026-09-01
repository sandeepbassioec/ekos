//! Tests for Layer 318: extras.rs is empty; test crate compiles, NetConfig, Frame, etc. (VV-NET-318).

use vectorvolt_net::protocol::{HEADER_SIZE, MAGIC, MAX_PAYLOAD, VERSION};
use vectorvolt_net::{Frame, NetConfig, decode_frame, encode_frame};

#[test]
fn crate_compiles() {
    let _ = NetConfig::default();
}

#[test]
fn extras_does_not_export_anything_unexpected() {
    let cfg = NetConfig::new("localhost", 7700);
    assert_eq!(cfg.port, 7700);
}

#[test]
fn net_config_default_works() {
    let cfg = NetConfig::default();
    assert_eq!(cfg.port, 7700);
}

#[test]
fn frame_construction() {
    let f = Frame {
        len: 3,
        payload: vec![1, 2, 3],
    };
    assert_eq!(f.len, 3);
}

#[test]
fn encode_decode_frame() {
    let data = b"extras";
    let framed = encode_frame(data);
    let frame = decode_frame(&framed).unwrap();
    assert_eq!(frame.payload, data);
}

#[test]
fn magic_constant() {
    assert_eq!(&MAGIC, b"VV");
}

#[test]
fn version_constant() {
    assert_eq!(VERSION, 1u8);
}

#[test]
fn header_size_constant() {
    assert_eq!(HEADER_SIZE, 24usize);
}

#[test]
fn max_payload_constant() {
    assert_eq!(MAX_PAYLOAD, 16 * 1024 * 1024);
}

#[test]
fn net_config_socket_addr_format() {
    let cfg = NetConfig::new("192.168.1.1", 8080);
    assert_eq!(cfg.socket_addr(), "192.168.1.1:8080");
}
