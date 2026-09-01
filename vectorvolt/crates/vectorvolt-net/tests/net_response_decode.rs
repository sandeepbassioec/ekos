//! Tests for Layer 316: Response encode/decode round-trip (VV-CTL-316).

use vectorvolt_net::protocol::{HEADER_SIZE, Header, Response, encode_response};

fn round_trip(resp: &Response) -> Response {
    let bytes = encode_response(1, resp).unwrap();
    let header = Header::decode(&bytes[..HEADER_SIZE]).unwrap();
    let payload = &bytes[HEADER_SIZE..];
    Response::decode(header.opcode, payload).unwrap()
}

#[test]
fn round_trip_pong() {
    let r = round_trip(&Response::Pong);
    assert_eq!(r, Response::Pong);
}

#[test]
fn round_trip_ok() {
    let r = round_trip(&Response::Ok { affected: 5 });
    assert_eq!(r, Response::Ok { affected: 5 });
}

#[test]
fn round_trip_not_found() {
    let r = round_trip(&Response::NotFound);
    assert_eq!(r, Response::NotFound);
}

#[test]
fn round_trip_row() {
    let r = round_trip(&Response::Row {
        values: vec![1, 2, 3],
    });
    assert_eq!(
        r,
        Response::Row {
            values: vec![1, 2, 3]
        }
    );
}

#[test]
fn round_trip_rows_empty() {
    let r = round_trip(&Response::Rows { rows: vec![] });
    assert_eq!(r, Response::Rows { rows: vec![] });
}

#[test]
fn round_trip_rows_nonempty() {
    let r = round_trip(&Response::Rows {
        rows: vec![vec![1], vec![2]],
    });
    if let Response::Rows { rows } = r {
        assert_eq!(rows.len(), 2);
    } else {
        panic!();
    }
}

#[test]
fn round_trip_error() {
    let r = round_trip(&Response::Error {
        code: 1,
        message: "fail".into(),
    });
    if let Response::Error { code, .. } = r {
        assert_eq!(code, 1);
    } else {
        panic!();
    }
}

#[test]
fn round_trip_exists_true() {
    let r = round_trip(&Response::Exists { found: true });
    assert_eq!(r, Response::Exists { found: true });
}

#[test]
fn round_trip_exists_false() {
    let r = round_trip(&Response::Exists { found: false });
    assert_eq!(r, Response::Exists { found: false });
}

#[test]
fn round_trip_table_selected() {
    let r = round_trip(&Response::TableSelected);
    assert_eq!(r, Response::TableSelected);
}

#[test]
fn round_trip_schema() {
    let r = round_trip(&Response::Schema {
        columns: vec!["pk".into(), "val".into()],
    });
    if let Response::Schema { columns } = r {
        assert_eq!(columns.len(), 2);
    } else {
        panic!();
    }
}

#[test]
fn round_trip_keys_empty() {
    let r = round_trip(&Response::Keys { pks: vec![] });
    assert_eq!(r, Response::Keys { pks: vec![] });
}

#[test]
fn round_trip_keys_nonempty() {
    let r = round_trip(&Response::Keys { pks: vec![1, 2, 3] });
    assert_eq!(r, Response::Keys { pks: vec![1, 2, 3] });
}
