//! Layer 185 — vectorvolt-net wire frame encode/decode tests

use vectorvolt_net::protocol::{HEADER_SIZE, Header, OpCode, encode_frame};

// 1. HEADER_SIZE is 24 bytes
#[test]
fn header_size_is_24() {
    assert_eq!(HEADER_SIZE, 24);
}

// 2. encode_frame for Ping produces non-empty bytes
#[test]
fn encode_frame_ping_nonempty() {
    let bytes = encode_frame(1, OpCode::Ping, &[]).unwrap();
    assert!(!bytes.is_empty());
}

// 3. encode_frame length >= HEADER_SIZE
#[test]
fn encode_frame_at_least_header_size() {
    let bytes = encode_frame(1, OpCode::Get, &[]).unwrap();
    assert!(bytes.len() >= HEADER_SIZE);
}

// 4. decode header from encoded frame
#[test]
fn decode_header_from_frame() {
    let bytes = encode_frame(42, OpCode::Get, &[]).unwrap();
    let hdr = Header::decode(&bytes[..HEADER_SIZE]).unwrap();
    assert_eq!(hdr.opcode, OpCode::Get);
}

// 5. header request_id roundtrips
#[test]
fn header_request_id_roundtrips() {
    let bytes = encode_frame(999, OpCode::Ping, &[]).unwrap();
    let hdr = Header::decode(&bytes[..HEADER_SIZE]).unwrap();
    assert_eq!(hdr.request_id, 999);
}

// 6. opcode Ping is 1
#[test]
fn opcode_ping_is_1() {
    assert_eq!(OpCode::Ping as u8, 1);
}

// 7. opcode Get is 2
#[test]
fn opcode_get_is_2() {
    assert_eq!(OpCode::Get as u8, 2);
}

// 8. opcode Insert is 3
#[test]
fn opcode_insert_is_3() {
    assert_eq!(OpCode::Insert as u8, 3);
}

// 9. opcode Delete is 4
#[test]
fn opcode_delete_is_4() {
    assert_eq!(OpCode::Delete as u8, 4);
}

// 10. opcode RangeScan is 5
#[test]
fn opcode_rangescan_is_5() {
    assert_eq!(OpCode::RangeScan as u8, 5);
}

// 11. frame with payload encodes correctly
#[test]
fn frame_with_payload_ok() {
    let payload = [1u8, 2, 3, 4, 5, 6, 7, 8];
    let bytes = encode_frame(1, OpCode::Insert, &payload).unwrap();
    assert_eq!(bytes.len(), HEADER_SIZE + payload.len());
}

// 12. different request_ids produce different frames
#[test]
fn different_request_ids_differ() {
    let b1 = encode_frame(1, OpCode::Ping, &[]).unwrap();
    let b2 = encode_frame(2, OpCode::Ping, &[]).unwrap();
    assert_ne!(b1, b2);
}

// 13. header payload_len matches frame payload
#[test]
fn header_payload_len_matches() {
    let payload = vec![0u8; 16];
    let bytes = encode_frame(1, OpCode::Get, &payload).unwrap();
    let hdr = Header::decode(&bytes[..HEADER_SIZE]).unwrap();
    assert_eq!(hdr.payload_len as usize, payload.len());
}
