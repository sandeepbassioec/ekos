//! Tests for Layer 318 (updated to current API).
use vectorvolt_net::{Frame, NetConfig, decode_frame, encode_frame};

#[test]
fn netconfig_default() {
    let c = NetConfig::default();
    assert_eq!(c.port, 7700);
}

#[test]
fn netconfig_socket_addr() {
    assert_eq!(NetConfig::new("1.2.3.4", 9).socket_addr(), "1.2.3.4:9");
}

#[test]
fn frame_encode_decode() {
    let b = encode_frame(&[1, 2, 3]);
    let f = decode_frame(&b).unwrap();
    assert_eq!(f.payload, vec![1, 2, 3]);
}

#[test]
fn frame_len_field() {
    let f = decode_frame(&encode_frame(&[9, 8])).unwrap();
    assert_eq!(f.len, 2);
}

#[test]
fn netconfig_clone() {
    let c = NetConfig::default();
    assert_eq!(c, c.clone());
}

#[test]
fn frame_clone() {
    let f = Frame {
        len: 0,
        payload: vec![],
    };
    assert_eq!(f, f.clone());
}

#[test]
fn netconfig_ne_different_port() {
    assert_ne!(NetConfig::new("x", 1), NetConfig::new("x", 2));
}
