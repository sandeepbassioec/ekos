//! Tests for Layer 302.
use vectorvolt_net::protocol::OpCode;

#[test]
fn ping_value() {
    assert_eq!(OpCode::Ping as u8, 1);
}
#[test]
fn get_value() {
    assert_eq!(OpCode::Get as u8, 2);
}
#[test]
fn insert_value() {
    assert_eq!(OpCode::Insert as u8, 3);
}
#[test]
fn delete_value() {
    assert_eq!(OpCode::Delete as u8, 4);
}
#[test]
fn pong_value() {
    assert_eq!(OpCode::Pong as u8, 129);
}
#[test]
fn row_value() {
    assert_eq!(OpCode::Row as u8, 130);
}
#[test]
fn not_found_value() {
    assert_eq!(OpCode::NotFound as u8, 131);
}
#[test]
fn ok_value() {
    assert_eq!(OpCode::Ok as u8, 132);
}
#[test]
fn error_value() {
    assert_eq!(OpCode::Error as u8, 135);
}
#[test]
fn opcode_clone() {
    let a = OpCode::Ping;
    let b = a;
    assert_eq!(a, b);
}
#[test]
fn opcode_debug() {
    assert!(!format!("{:?}", OpCode::Insert).is_empty());
}
#[test]
fn opcode_partial_eq() {
    assert_eq!(OpCode::Flush, OpCode::Flush);
    assert_ne!(OpCode::Flush, OpCode::Compact);
}
#[test]
fn opcode_is_request() {
    assert!(OpCode::Ping.is_request());
    assert!(!OpCode::Pong.is_request());
}
