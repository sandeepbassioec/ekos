//! Tests for Layer 316: Ping request, opcode value=1, Pong response (VV-NET-316).

use vectorvolt_net::protocol::{OpCode, Request, Response};

#[test]
fn ping_opcode_is_one() {
    assert_eq!(OpCode::Ping as u8, 1);
}

#[test]
fn pong_opcode_is_129() {
    assert_eq!(OpCode::Pong as u8, 129);
}

#[test]
fn request_ping_opcode() {
    assert_eq!(Request::Ping.opcode(), OpCode::Ping);
}

#[test]
fn request_ping_is_request() {
    assert!(Request::Ping.opcode().is_request());
}

#[test]
fn pong_is_not_request() {
    assert!(!OpCode::Pong.is_request());
}

#[test]
fn response_pong_variant() {
    let r = Response::Pong;
    assert!(matches!(r, Response::Pong));
}

#[test]
fn response_pong_debug_nonempty() {
    assert!(!format!("{:?}", Response::Pong).is_empty());
}

#[test]
fn response_pong_clone() {
    let a = Response::Pong;
    let b = a.clone();
    assert_eq!(a, b);
}

#[test]
fn ping_encode_payload_ok() {
    let encoded = Request::Ping.encode_payload().unwrap();
    assert!(encoded.is_empty());
}

#[test]
fn ping_request_clone() {
    let a = Request::Ping;
    let b = a.clone();
    assert_eq!(a, b);
}

#[test]
fn ping_request_debug() {
    assert!(!format!("{:?}", Request::Ping).is_empty());
}

#[test]
fn opcode_from_u8_ping() {
    let op = OpCode::from_u8(1).unwrap();
    assert_eq!(op, OpCode::Ping);
}

#[test]
fn opcode_from_u8_unknown_err() {
    let result = OpCode::from_u8(200);
    assert!(result.is_err());
}
