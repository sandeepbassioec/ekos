//! Tests for Layer 189: vectorvolt-net frame protocol (VV-NET-189).

use vectorvolt_net::{Frame, decode_frame, encode_frame};

#[test]
fn encode_empty() {
    let b = encode_frame(&[]);
    assert_eq!(b.len(), 4);
}
#[test]
fn encode_len_field() {
    let b = encode_frame(b"hello");
    assert_eq!(u32::from_le_bytes([b[0], b[1], b[2], b[3]]), 5);
}
#[test]
fn roundtrip_basic() {
    let enc = encode_frame(b"hello");
    let f = decode_frame(&enc).unwrap();
    assert_eq!(f.payload, b"hello");
}
#[test]
fn roundtrip_empty() {
    let enc = encode_frame(&[]);
    let f = decode_frame(&enc).unwrap();
    assert!(f.payload.is_empty());
}
#[test]
fn decode_too_short_none() {
    assert!(decode_frame(&[0, 0, 0]).is_none());
}
#[test]
fn decode_exact_header_empty_payload() {
    let enc = encode_frame(&[]);
    assert!(decode_frame(&enc).is_some());
}
#[test]
fn decode_truncated_payload_none() {
    let mut enc = encode_frame(b"hello");
    enc.truncate(6);
    assert!(decode_frame(&enc).is_none());
}
#[test]
fn frame_len_field() {
    let enc = encode_frame(b"abc");
    let f = decode_frame(&enc).unwrap();
    assert_eq!(f.len, 3);
}
#[test]
fn encode_large_payload() {
    let payload = vec![42u8; 1000];
    let enc = encode_frame(&payload);
    assert_eq!(decode_frame(&enc).unwrap().payload, payload);
}
#[test]
fn decode_zero_bytes_none() {
    assert!(decode_frame(&[]).is_none());
}
#[test]
fn frame_clone() {
    let f = Frame {
        len: 3,
        payload: b"abc".to_vec(),
    };
    assert_eq!(f.clone().payload, b"abc");
}
#[test]
fn frame_eq() {
    let a = Frame {
        len: 2,
        payload: b"hi".to_vec(),
    };
    let b = Frame {
        len: 2,
        payload: b"hi".to_vec(),
    };
    assert_eq!(a, b);
}
#[test]
fn encode_binary_payload() {
    let payload: Vec<u8> = (0..=255).collect();
    let enc = encode_frame(&payload);
    assert_eq!(decode_frame(&enc).unwrap().payload, payload);
}
