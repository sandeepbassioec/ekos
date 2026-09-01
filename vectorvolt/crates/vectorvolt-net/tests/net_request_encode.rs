//! Tests for Layer 315: Request encode (VV-CTL-315).

use vectorvolt_net::protocol::{HEADER_SIZE, Request, encode_request};

#[test]
fn encode_ping_nonempty() {
    let bytes = encode_request(1, &Request::Ping).unwrap();
    assert!(!bytes.is_empty());
}

#[test]
fn encode_ping_at_least_header_size() {
    let bytes = encode_request(1, &Request::Ping).unwrap();
    assert!(bytes.len() >= HEADER_SIZE);
}

#[test]
fn encode_get_nonempty() {
    let bytes = encode_request(2, &Request::Get { pk: 42 }).unwrap();
    assert!(!bytes.is_empty());
}

#[test]
fn encode_insert_nonempty() {
    let bytes = encode_request(
        3,
        &Request::Insert {
            pk: 1,
            values: vec![10],
        },
    )
    .unwrap();
    assert!(!bytes.is_empty());
}

#[test]
fn encode_delete_nonempty() {
    let bytes = encode_request(4, &Request::Delete { pk: 5 }).unwrap();
    assert!(!bytes.is_empty());
}

#[test]
fn encode_range_scan_nonempty() {
    let bytes = encode_request(
        5,
        &Request::RangeScan {
            col: 0,
            lo: 0,
            hi: 100,
            limit: 10,
        },
    )
    .unwrap();
    assert!(!bytes.is_empty());
}

#[test]
fn encode_flush_nonempty() {
    let bytes = encode_request(6, &Request::Flush).unwrap();
    assert!(!bytes.is_empty());
}

#[test]
fn encode_compact_nonempty() {
    let bytes = encode_request(7, &Request::Compact).unwrap();
    assert!(!bytes.is_empty());
}

#[test]
fn encode_stats_nonempty() {
    let bytes = encode_request(8, &Request::Stats).unwrap();
    assert!(!bytes.is_empty());
}

#[test]
fn encode_truncate_nonempty() {
    let bytes = encode_request(9, &Request::Truncate).unwrap();
    assert!(!bytes.is_empty());
}

#[test]
fn encode_upsert_nonempty() {
    let bytes = encode_request(
        10,
        &Request::Upsert {
            pk: 1,
            values: vec![5],
        },
    )
    .unwrap();
    assert!(!bytes.is_empty());
}

#[test]
fn encode_list_tables_nonempty() {
    let bytes = encode_request(11, &Request::ListTables).unwrap();
    assert!(!bytes.is_empty());
}

#[test]
fn encode_batch_delete_nonempty() {
    let bytes = encode_request(12, &Request::BatchDelete { pks: vec![1, 2, 3] }).unwrap();
    assert!(!bytes.is_empty());
}
