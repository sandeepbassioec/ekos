//! Tests for Layer 309.
use vectorvolt_net::{Frame, decode_frame, encode_frame};

#[test]
fn encode_empty_payload() {
    let b = encode_frame(&[]);
    assert_eq!(b.len(), 4);
}
#[test]
fn encode_length_prefix() {
    let b = encode_frame(&[1, 2, 3]);
    assert_eq!(u32::from_le_bytes([b[0], b[1], b[2], b[3]]), 3);
}
#[test]
fn encode_payload_follows() {
    let b = encode_frame(&[9, 8, 7]);
    assert_eq!(&b[4..], &[9, 8, 7]);
}
#[test]
fn decode_empty() {
    let b = encode_frame(&[]);
    let f = decode_frame(&b).unwrap();
    assert_eq!(f.len, 0);
}
#[test]
fn decode_payload() {
    let b = encode_frame(&[1, 2]);
    let f = decode_frame(&b).unwrap();
    assert_eq!(f.payload, vec![1, 2]);
}
#[test]
fn decode_truncated_returns_none() {
    assert!(decode_frame(&[0, 0, 0, 10]).is_none());
}
#[test]
fn decode_short_returns_none() {
    assert!(decode_frame(&[1]).is_none());
}
#[test]
fn frame_debug() {
    let f = Frame {
        len: 0,
        payload: vec![],
    };
    assert!(!format!("{f:?}").is_empty());
}
#[test]
fn frame_clone() {
    let f = Frame {
        len: 1,
        payload: vec![5],
    };
    let f2 = f.clone();
    assert_eq!(f, f2);
}
#[test]
fn frame_partial_eq() {
    let a = Frame {
        len: 0,
        payload: vec![],
    };
    let b = Frame {
        len: 0,
        payload: vec![],
    };
    assert_eq!(a, b);
}
#[test]
fn encode_large_payload() {
    let data = vec![0u8; 100];
    let b = encode_frame(&data);
    assert_eq!(b.len(), 104);
}
#[test]
fn round_trip() {
    let data = b"hello world";
    let b = encode_frame(data);
    let f = decode_frame(&b).unwrap();
    assert_eq!(f.payload, data);
}
#[test]
fn decode_with_trailing_bytes() {
    let mut b = encode_frame(&[1, 2, 3]);
    b.push(99);
    let f = decode_frame(&b).unwrap();
    assert_eq!(f.payload, vec![1, 2, 3]);
}
