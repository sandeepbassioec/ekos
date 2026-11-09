//! Pins the wire bytes the Python/Go/TypeScript SDK golden tests expect
//! (copied from sdk/go/vectorvolt/client_test.go) to what the Rust encoder
//! actually produces, so the SDK fixtures cannot drift from the server.

use vectorvolt_net::protocol::{encode_request, encode_response};
use vectorvolt_net::{Request, Response};

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[test]
fn ping_request_matches_sdk_fixture() {
    let got = encode_request(1, &Request::Ping).unwrap();
    assert_eq!(
        hex(&got),
        "565601010000000001000000000000000000000000000000"
    );
}

#[test]
fn insert_request_matches_sdk_fixture() {
    let req = Request::Insert {
        pk: 7,
        values: vec![10, 20],
    };
    let got = encode_request(2, &req).unwrap();
    assert_eq!(
        hex(&got),
        "565601030000000002000000000000001c000000cb77f5420700000000000000\
         020000000a000000000000001400000000000000"
    );
}

#[test]
fn get_request_matches_sdk_fixture() {
    let got = encode_request(3, &Request::Get { pk: 7 }).unwrap();
    assert_eq!(
        hex(&got),
        "565601020000000003000000000000000800000070d6e76f0700000000000000"
    );
}

#[test]
fn use_table_request_matches_sdk_fixture() {
    let req = Request::UseTable {
        name: "users".into(),
    };
    let got = encode_request(5, &req).unwrap();
    assert_eq!(
        hex(&got),
        "5656010d0000000005000000000000000900000015033f7e050000007573657273"
    );
}

#[test]
fn row_response_matches_sdk_fixture() {
    let resp = Response::Row {
        values: vec![10, 20],
    };
    let got = encode_response(3, &resp).unwrap();
    assert_eq!(
        hex(&got),
        "5656018200000000030000000000000014000000fc6859c9020000000a000000\
         000000001400000000000000"
    );
}

#[test]
fn error_response_matches_sdk_fixture() {
    let resp = Response::Error {
        code: 4,
        message: "not found".into(),
    };
    let got = encode_response(4, &resp).unwrap();
    assert_eq!(
        hex(&got),
        "565601870000000004000000000000000f000000285977ce0400090000006e6f7420666f756e64"
    );
}
