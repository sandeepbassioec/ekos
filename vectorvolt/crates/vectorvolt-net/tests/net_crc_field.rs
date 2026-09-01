//! Tests for Layer 310: Header payload_crc field, CrcMismatch error, encode/decode (VV-NET-310).

use vectorvolt_net::NetError;
use vectorvolt_net::protocol::{HEADER_SIZE, Header, OpCode};

fn make_ping_header(payload_crc: u32) -> Header {
    Header {
        opcode: OpCode::Ping,
        flags: 0,
        request_id: 1,
        payload_len: 0,
        payload_crc,
    }
}

#[test]
fn header_crc_field_stored() {
    let h = make_ping_header(0xDEAD_CAFE);
    assert_eq!(h.payload_crc, 0xDEAD_CAFE);
}

#[test]
fn header_crc_zero_roundtrip() {
    let h = make_ping_header(0);
    let bytes = h.encode();
    let d = Header::decode(&bytes).unwrap();
    assert_eq!(d.payload_crc, 0);
}

#[test]
fn header_crc_nonzero_roundtrip() {
    let h = make_ping_header(0x1234_5678);
    let bytes = h.encode();
    let d = Header::decode(&bytes).unwrap();
    assert_eq!(d.payload_crc, 0x1234_5678);
}

#[test]
fn header_crc_max_value_roundtrip() {
    let h = make_ping_header(u32::MAX);
    let bytes = h.encode();
    let d = Header::decode(&bytes).unwrap();
    assert_eq!(d.payload_crc, u32::MAX);
}

#[test]
fn header_encode_length_with_crc() {
    let h = make_ping_header(0xABCD_EF01);
    assert_eq!(h.encode().len(), HEADER_SIZE);
}

#[test]
fn crc_mismatch_error_fields() {
    let err = NetError::CrcMismatch {
        expected: 1,
        actual: 2,
    };
    let msg = err.to_string();
    assert!(msg.contains("CRC") || msg.contains("crc") || msg.contains("mismatch"));
}

#[test]
fn crc_mismatch_expected_field() {
    match (NetError::CrcMismatch {
        expected: 10,
        actual: 20,
    }) {
        NetError::CrcMismatch { expected, .. } => assert_eq!(expected, 10),
        _ => panic!("wrong variant"),
    }
}

#[test]
fn crc_mismatch_actual_field() {
    match (NetError::CrcMismatch {
        expected: 5,
        actual: 99,
    }) {
        NetError::CrcMismatch { actual, .. } => assert_eq!(actual, 99),
        _ => panic!("wrong variant"),
    }
}

#[test]
fn header_crc_bytes_position() {
    let h = make_ping_header(0x0100_0000);
    let bytes = h.encode();
    let crc = u32::from_le_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
    assert_eq!(crc, 0x0100_0000);
}

#[test]
fn header_payload_len_field() {
    let h = Header {
        opcode: OpCode::Insert,
        flags: 0,
        request_id: 0,
        payload_len: 128,
        payload_crc: 0,
    };
    assert_eq!(h.payload_len, 128);
}

#[test]
fn header_payload_len_roundtrip() {
    let h = Header {
        opcode: OpCode::Get,
        flags: 0,
        request_id: 0,
        payload_len: 255,
        payload_crc: 0,
    };
    let bytes = h.encode();
    let d = Header::decode(&bytes).unwrap();
    assert_eq!(d.payload_len, 255);
}

#[test]
fn net_error_truncated_display() {
    let err = NetError::Truncated { need: 24, got: 10 };
    let msg = err.to_string();
    assert!(!msg.is_empty());
}

#[test]
fn header_different_crcs_differ() {
    let h1 = make_ping_header(1).encode();
    let h2 = make_ping_header(2).encode();
    assert_ne!(h1, h2);
}
