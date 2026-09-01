//! Tests for Layer 314: Wire constants (VV-CTL-314).

use vectorvolt_net::protocol::{HEADER_SIZE, MAGIC, MAX_PAYLOAD, OpCode, VERSION};

#[test]
fn header_size_is_24() {
    assert_eq!(HEADER_SIZE, 24);
}

#[test]
fn max_payload_is_16_mib() {
    assert_eq!(MAX_PAYLOAD, 16 * 1024 * 1024);
}

#[test]
fn magic_bytes_are_vv() {
    assert_eq!(MAGIC, *b"VV");
}

#[test]
fn version_is_1() {
    assert_eq!(VERSION, 1);
}

#[test]
fn opcode_ping_is_1() {
    assert_eq!(OpCode::Ping as u8, 1);
}

#[test]
fn opcode_get_is_2() {
    assert_eq!(OpCode::Get as u8, 2);
}

#[test]
fn opcode_insert_is_3() {
    assert_eq!(OpCode::Insert as u8, 3);
}

#[test]
fn opcode_pong_is_129() {
    assert_eq!(OpCode::Pong as u8, 129);
}

#[test]
fn opcode_row_is_130() {
    assert_eq!(OpCode::Row as u8, 130);
}

#[test]
fn opcode_ok_is_132() {
    assert_eq!(OpCode::Ok as u8, 132);
}

#[test]
fn opcode_error_is_135() {
    assert_eq!(OpCode::Error as u8, 135);
}

#[test]
fn error_code_engine_is_1() {
    assert_eq!(vectorvolt_net::error_code::ENGINE, 1);
}

#[test]
fn error_code_not_found_is_4() {
    assert_eq!(vectorvolt_net::error_code::NOT_FOUND, 4);
}
