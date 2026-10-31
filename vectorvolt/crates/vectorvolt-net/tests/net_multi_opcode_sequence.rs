//! Tests for Layer 315: Multiple Request variants constructed in sequence (VV-NET-315).

use vectorvolt_net::protocol::{OpCode, Request};

#[test]
fn ping_then_get_opcodes() {
    let ops = [Request::Ping, Request::Get { pk: 1 }];
    let codes: Vec<OpCode> = ops.iter().map(|r| r.opcode()).collect();
    assert_eq!(codes[0], OpCode::Ping);
    assert_eq!(codes[1], OpCode::Get);
}

#[test]
fn insert_opcode() {
    let r = Request::Insert {
        pk: 1,
        values: vec![42],
    };
    assert_eq!(r.opcode(), OpCode::Insert);
}

#[test]
fn delete_opcode() {
    let r = Request::Delete { pk: 99 };
    assert_eq!(r.opcode(), OpCode::Delete);
}

#[test]
fn flush_opcode() {
    let r = Request::Flush;
    assert_eq!(r.opcode(), OpCode::Flush);
}

#[test]
fn compact_opcode() {
    let r = Request::Compact;
    assert_eq!(r.opcode(), OpCode::Compact);
}

#[test]
fn stats_opcode() {
    let r = Request::Stats;
    assert_eq!(r.opcode(), OpCode::Stats);
}

#[test]
fn hello_opcode() {
    let r = Request::Hello;
    assert_eq!(r.opcode(), OpCode::Hello);
}

#[test]
fn list_tables_opcode() {
    let r = Request::ListTables;
    assert_eq!(r.opcode(), OpCode::ListTables);
}

#[test]
fn use_table_opcode() {
    let r = Request::UseTable { name: "t".into() };
    assert_eq!(r.opcode(), OpCode::UseTable);
}

#[test]
fn create_table_opcode() {
    let r = Request::CreateTable {
        name: "t".into(),
        schema: vec![],
    };
    assert_eq!(r.opcode(), OpCode::CreateTable);
}

#[test]
fn drop_table_opcode() {
    let r = Request::DropTable { name: "t".into() };
    assert_eq!(r.opcode(), OpCode::DropTable);
}

#[test]
fn batch_insert_opcode() {
    let r = Request::BatchInsert { rows: vec![] };
    assert_eq!(r.opcode(), OpCode::BatchInsert);
}

#[test]
fn batch_delete_opcode() {
    let r = Request::BatchDelete { pks: vec![1, 2, 3] };
    assert_eq!(r.opcode(), OpCode::BatchDelete);
}
