//! Tests for Layer 319.
use vectorvolt_net::protocol::{Response, encode_response, Header, HEADER_SIZE};

fn rt(resp: &Response) -> Response {
    let b = encode_response(1, resp).unwrap();
    let h = Header::decode(&b[..HEADER_SIZE]).unwrap();
    Response::decode(h.opcode, &b[HEADER_SIZE..]).unwrap()
}

#[test] fn rows_empty() { let r = rt(&Response::Rows { rows: vec![] }); assert_eq!(r, Response::Rows { rows: vec![] }); }
#[test] fn rows_one() { let r = rt(&Response::Rows { rows: vec![vec![1,2]] }); if let Response::Rows { rows } = r { assert_eq!(rows.len(), 1); } else { panic!(); } }
#[test] fn rows_multi() { let r = rt(&Response::Rows { rows: vec![vec![1], vec![2], vec![3]] }); if let Response::Rows { rows } = r { assert_eq!(rows.len(), 3); } else { panic!(); } }
#[test] fn row_set_empty() { let r = rt(&Response::RowSet { entries: vec![] }); if let Response::RowSet { entries } = r { assert!(entries.is_empty()); } else { panic!(); } }
#[test] fn row_set_one() { let r = rt(&Response::RowSet { entries: vec![(1, vec![10])] }); if let Response::RowSet { entries } = r { assert_eq!(entries.len(), 1); } else { panic!(); } }
#[test] fn keys_empty() { let r = rt(&Response::Keys { pks: vec![] }); assert_eq!(r, Response::Keys { pks: vec![] }); }
#[test] fn keys_nonempty() { let r = rt(&Response::Keys { pks: vec![1,2,3] }); if let Response::Keys { pks } = r { assert_eq!(pks, vec![1,2,3]); } else { panic!(); } }
#[test] fn min_max_result() { let r = rt(&Response::MinMaxResult { min: 1, max: 10, count: 5 }); if let Response::MinMaxResult { min, max, count } = r { assert_eq!(min, 1); assert_eq!(max, 10); assert_eq!(count, 5); } else { panic!(); } }
#[test] fn schema_columns() { let r = rt(&Response::Schema { columns: vec!["pk".into(), "val".into()] }); if let Response::Schema { columns } = r { assert_eq!(columns.len(), 2); } else { panic!(); } }
#[test] fn table_list_empty() { let r = rt(&Response::TableList { names: vec![] }); if let Response::TableList { names } = r { assert!(names.is_empty()); } else { panic!(); } }
#[test] fn batch_result_fields() { let r = rt(&Response::BatchResult { succeeded: 5, skipped: 2 }); if let Response::BatchResult { succeeded, skipped } = r { assert_eq!(succeeded, 5); assert_eq!(skipped, 2); } else { panic!(); } }
#[test] fn txn_started_id() { let r = rt(&Response::TxnStarted { txn_id: 42 }); if let Response::TxnStarted { txn_id } = r { assert_eq!(txn_id, 42); } else { panic!(); } }
#[test] fn stats_fields() { let r = rt(&Response::Stats { rows: 100, partitions: 2, bytes: 500 }); if let Response::Stats { rows, partitions, bytes } = r { assert_eq!(rows, 100); assert_eq!(partitions, 2); assert_eq!(bytes, 500); } else { panic!(); } }
