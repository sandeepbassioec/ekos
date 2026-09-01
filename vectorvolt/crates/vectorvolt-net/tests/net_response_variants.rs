//! Tests for Layer 305.
use vectorvolt_net::protocol::Response;

#[test]
fn resp_pong() {
    let r = Response::Pong;
    assert!(matches!(r, Response::Pong));
}
#[test]
fn resp_row() {
    let r = Response::Row { values: vec![1, 2] };
    if let Response::Row { values } = r {
        assert_eq!(values, vec![1, 2]);
    } else {
        panic!();
    }
}
#[test]
fn resp_not_found() {
    assert!(matches!(Response::NotFound, Response::NotFound));
}
#[test]
fn resp_ok() {
    let r = Response::Ok { affected: 5 };
    if let Response::Ok { affected } = r {
        assert_eq!(affected, 5);
    } else {
        panic!();
    }
}
#[test]
fn resp_rows() {
    let r = Response::Rows {
        rows: vec![vec![1], vec![2]],
    };
    if let Response::Rows { rows } = r {
        assert_eq!(rows.len(), 2);
    } else {
        panic!();
    }
}
#[test]
fn resp_stats() {
    let r = Response::Stats {
        rows: 10,
        partitions: 1,
        bytes: 100,
    };
    if let Response::Stats { rows, .. } = r {
        assert_eq!(rows, 10);
    } else {
        panic!();
    }
}
#[test]
fn resp_error() {
    let r = Response::Error {
        code: 1,
        message: "e".into(),
    };
    if let Response::Error { code, .. } = r {
        assert_eq!(code, 1);
    } else {
        panic!();
    }
}
#[test]
fn resp_challenge() {
    let r = Response::Challenge { nonce: vec![0; 32] };
    if let Response::Challenge { nonce } = r {
        assert_eq!(nonce.len(), 32);
    } else {
        panic!();
    }
}
#[test]
fn resp_exists() {
    let r = Response::Exists { found: true };
    if let Response::Exists { found } = r {
        assert!(found);
    } else {
        panic!();
    }
}
#[test]
fn resp_table_selected() {
    assert!(matches!(Response::TableSelected, Response::TableSelected));
}
#[test]
fn resp_table_list() {
    let r = Response::TableList {
        names: vec!["a".into()],
    };
    if let Response::TableList { names } = r {
        assert_eq!(names.len(), 1);
    } else {
        panic!();
    }
}
#[test]
fn resp_txn_started() {
    let r = Response::TxnStarted { txn_id: 99 };
    if let Response::TxnStarted { txn_id } = r {
        assert_eq!(txn_id, 99);
    } else {
        panic!();
    }
}
#[test]
fn resp_batch_result() {
    let r = Response::BatchResult {
        succeeded: 3,
        skipped: 1,
    };
    if let Response::BatchResult { succeeded, skipped } = r {
        assert_eq!(succeeded, 3);
        assert_eq!(skipped, 1);
    } else {
        panic!();
    }
}
