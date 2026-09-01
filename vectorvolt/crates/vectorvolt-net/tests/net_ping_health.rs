//! Tests for Layer 317.
use vectorvolt_net::protocol::{OpCode, Request, Response, encode_request, encode_response, Header, HEADER_SIZE};

#[test] fn ping_opcode_is_1() { assert_eq!(OpCode::Ping as u8, 1); }
#[test] fn pong_opcode_is_129() { assert_eq!(OpCode::Pong as u8, 129); }
#[test] fn ping_is_request() { assert!(OpCode::Ping.is_request()); }
#[test] fn pong_is_not_request() { assert!(!OpCode::Pong.is_request()); }
#[test] fn request_ping_encodes() { let b = encode_request(1, &Request::Ping).unwrap(); assert!(b.len() >= HEADER_SIZE); }
#[test] fn response_pong_encodes() { let b = encode_response(1, &Response::Pong).unwrap(); assert!(b.len() >= HEADER_SIZE); }
#[test] fn ping_round_trip() {
    let b = encode_request(1, &Request::Ping).unwrap();
    let h = Header::decode(&b[..HEADER_SIZE]).unwrap();
    let r = Request::decode(h.opcode, &b[HEADER_SIZE..]).unwrap();
    assert_eq!(r, Request::Ping);
}
#[test] fn pong_round_trip() {
    let b = encode_response(1, &Response::Pong).unwrap();
    let h = Header::decode(&b[..HEADER_SIZE]).unwrap();
    let r = Response::decode(h.opcode, &b[HEADER_SIZE..]).unwrap();
    assert_eq!(r, Response::Pong);
}
#[test] fn stats_opcode_is_8() { assert_eq!(OpCode::Stats as u8, 8); }
#[test] fn stats_is_request() { assert!(OpCode::Stats.is_request()); }
#[test] fn stats_reply_is_response() { assert!(!OpCode::StatsReply.is_request()); }
#[test] fn hello_opcode_is_9() { assert_eq!(OpCode::Hello as u8, 9); }
#[test] fn flush_is_request() { assert!(OpCode::Flush.is_request()); }
