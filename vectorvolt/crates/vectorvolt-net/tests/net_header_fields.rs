//! Tests for Layer 303.
use vectorvolt_net::protocol::{HEADER_SIZE, Header, MAGIC, OpCode, VERSION};

fn make_header() -> Header {
    Header {
        opcode: OpCode::Ping,
        flags: 0,
        request_id: 42,
        payload_len: 0,
        payload_crc: 0,
    }
}

#[test]
fn header_size_is_24() {
    assert_eq!(HEADER_SIZE, 24);
}
#[test]
fn magic_is_vv() {
    assert_eq!(&MAGIC, b"VV");
}
#[test]
fn version_is_1() {
    assert_eq!(VERSION, 1);
}
#[test]
fn header_opcode_field() {
    assert_eq!(make_header().opcode, OpCode::Ping);
}
#[test]
fn header_flags_field() {
    assert_eq!(make_header().flags, 0);
}
#[test]
fn header_request_id_field() {
    assert_eq!(make_header().request_id, 42);
}
#[test]
fn header_payload_len_field() {
    assert_eq!(make_header().payload_len, 0);
}
#[test]
fn header_payload_crc_field() {
    assert_eq!(make_header().payload_crc, 0);
}
#[test]
fn header_encode_starts_with_magic() {
    let b = make_header().encode();
    assert_eq!(&b[0..2], b"VV");
}
#[test]
fn header_encode_version_byte() {
    let b = make_header().encode();
    assert_eq!(b[2], 1);
}
#[test]
fn header_encode_opcode_byte() {
    let b = make_header().encode();
    assert_eq!(b[3], OpCode::Ping as u8);
}
#[test]
fn header_round_trip() {
    let h = make_header();
    let b = h.encode();
    let h2 = Header::decode(&b).unwrap();
    assert_eq!(h, h2);
}
#[test]
fn header_bad_magic_errors() {
    let mut b = make_header().encode();
    b[0] = 0;
    assert!(Header::decode(&b).is_err());
}
