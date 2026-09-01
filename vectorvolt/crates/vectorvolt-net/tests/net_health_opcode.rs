//! Tests for Layer 317: Stats request (opcode=8), StatsReply response (VV-NET-317).

use vectorvolt_net::protocol::{OpCode, Request, Response};

#[test]
fn stats_opcode_is_eight() {
    assert_eq!(OpCode::Stats as u8, 8);
}

#[test]
fn stats_reply_opcode_is_134() {
    assert_eq!(OpCode::StatsReply as u8, 134);
}

#[test]
fn request_stats_opcode() {
    assert_eq!(Request::Stats.opcode(), OpCode::Stats);
}

#[test]
fn stats_is_request() {
    assert!(OpCode::Stats.is_request());
}

#[test]
fn stats_reply_not_request() {
    assert!(!OpCode::StatsReply.is_request());
}

#[test]
fn response_stats_rows_field() {
    let r = Response::Stats {
        rows: 100,
        partitions: 2,
        bytes: 4096,
    };
    if let Response::Stats { rows, .. } = r {
        assert_eq!(rows, 100);
    } else {
        panic!("wrong variant");
    }
}

#[test]
fn response_stats_partitions_field() {
    let r = Response::Stats {
        rows: 0,
        partitions: 3,
        bytes: 0,
    };
    if let Response::Stats { partitions, .. } = r {
        assert_eq!(partitions, 3);
    } else {
        panic!("wrong variant");
    }
}

#[test]
fn response_stats_bytes_field() {
    let r = Response::Stats {
        rows: 0,
        partitions: 0,
        bytes: 8192,
    };
    if let Response::Stats { bytes, .. } = r {
        assert_eq!(bytes, 8192);
    } else {
        panic!("wrong variant");
    }
}

#[test]
fn response_stats_clone() {
    let a = Response::Stats {
        rows: 5,
        partitions: 1,
        bytes: 100,
    };
    let b = a.clone();
    assert_eq!(a, b);
}

#[test]
fn response_stats_debug_nonempty() {
    let r = Response::Stats {
        rows: 0,
        partitions: 0,
        bytes: 0,
    };
    assert!(!format!("{:?}", r).is_empty());
}

#[test]
fn stats_encode_payload_empty() {
    let encoded = Request::Stats.encode_payload().unwrap();
    assert!(encoded.is_empty());
}

#[test]
fn opcode_from_u8_stats() {
    let op = OpCode::from_u8(8).unwrap();
    assert_eq!(op, OpCode::Stats);
}
