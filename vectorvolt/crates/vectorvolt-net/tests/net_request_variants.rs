//! Tests for Layer 304.
use vectorvolt_net::protocol::{OpCode, Request};

#[test]
fn req_ping() {
    assert_eq!(Request::Ping.opcode(), OpCode::Ping);
}
#[test]
fn req_get() {
    assert_eq!(Request::Get { pk: 1 }.opcode(), OpCode::Get);
}
#[test]
fn req_insert() {
    assert_eq!(
        Request::Insert {
            pk: 1,
            values: vec![2]
        }
        .opcode(),
        OpCode::Insert
    );
}
#[test]
fn req_delete() {
    assert_eq!(Request::Delete { pk: 1 }.opcode(), OpCode::Delete);
}
#[test]
fn req_flush() {
    assert_eq!(Request::Flush.opcode(), OpCode::Flush);
}
#[test]
fn req_compact() {
    assert_eq!(Request::Compact.opcode(), OpCode::Compact);
}
#[test]
fn req_stats() {
    assert_eq!(Request::Stats.opcode(), OpCode::Stats);
}
#[test]
fn req_hello() {
    assert_eq!(Request::Hello.opcode(), OpCode::Hello);
}
#[test]
fn req_truncate() {
    assert_eq!(Request::Truncate.opcode(), OpCode::Truncate);
}
#[test]
fn req_list_tables() {
    assert_eq!(Request::ListTables.opcode(), OpCode::ListTables);
}
#[test]
fn req_begin_txn() {
    assert_eq!(Request::BeginTxn.opcode(), OpCode::BeginTxn);
}
#[test]
fn req_describe_table() {
    assert_eq!(Request::DescribeTable.opcode(), OpCode::DescribeTable);
}
#[test]
fn req_range_scan() {
    assert_eq!(
        Request::RangeScan {
            col: 0,
            lo: 0,
            hi: 100,
            limit: 10
        }
        .opcode(),
        OpCode::RangeScan
    );
}
