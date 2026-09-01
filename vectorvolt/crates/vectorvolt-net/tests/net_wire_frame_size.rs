//! Tests for Layer 309: HEADER_SIZE=24, encode_frame, decode_frame, Frame (VV-NET-309).

use vectorvolt_net::protocol::HEADER_SIZE;
use vectorvolt_net::{Frame, decode_frame, encode_frame};

#[test]
fn header_size_is_24() {
    assert_eq!(HEADER_SIZE, 24);
}

#[test]
fn encode_frame_prefix_length() {
    let payload = b"hello";
    let framed = encode_frame(payload);
    assert_eq!(framed.len(), 4 + payload.len());
}

#[test]
fn encode_frame_length_field() {
    let payload = b"world";
    let framed = encode_frame(payload);
    let len = u32::from_le_bytes([framed[0], framed[1], framed[2], framed[3]]);
    assert_eq!(len as usize, payload.len());
}

#[test]
fn encode_frame_payload_content() {
    let payload = b"test";
    let framed = encode_frame(payload);
    assert_eq!(&framed[4..], payload);
}

#[test]
fn decode_frame_empty_input_none() {
    assert!(decode_frame(&[]).is_none());
}

#[test]
fn decode_frame_truncated_none() {
    let framed = encode_frame(b"hello");
    assert!(decode_frame(&framed[..3]).is_none());
}

#[test]
fn decode_frame_roundtrip() {
    let payload = b"vectorvolt";
    let framed = encode_frame(payload);
    let frame = decode_frame(&framed).unwrap();
    assert_eq!(frame.payload, payload);
}

#[test]
fn decode_frame_len_field() {
    let payload = b"data";
    let framed = encode_frame(payload);
    let frame = decode_frame(&framed).unwrap();
    assert_eq!(frame.len as usize, payload.len());
}

#[test]
fn frame_struct_len_field() {
    let f = Frame {
        len: 5,
        payload: vec![1, 2, 3, 4, 5],
    };
    assert_eq!(f.len, 5);
}

#[test]
fn frame_struct_payload_field() {
    let f = Frame {
        len: 2,
        payload: vec![0xAA, 0xBB],
    };
    assert_eq!(f.payload, vec![0xAA, 0xBB]);
}

#[test]
fn frame_clone_eq() {
    let a = Frame {
        len: 1,
        payload: vec![42],
    };
    let b = a.clone();
    assert_eq!(a, b);
}

#[test]
fn encode_frame_empty_payload() {
    let framed = encode_frame(b"");
    assert_eq!(framed.len(), 4);
    let frame = decode_frame(&framed).unwrap();
    assert!(frame.payload.is_empty());
}

#[test]
fn encode_decode_large_payload() {
    let payload: Vec<u8> = (0..100).collect();
    let framed = encode_frame(&payload);
    let frame = decode_frame(&framed).unwrap();
    assert_eq!(frame.payload, payload);
}
