//! Tests for Layer 319: Response::Rows with various row counts (VV-NET-319).

use vectorvolt_net::protocol::Response;

#[test]
fn rows_empty() {
    let r = Response::Rows { rows: vec![] };
    if let Response::Rows { rows } = r {
        assert!(rows.is_empty());
    }
}

#[test]
fn rows_single_row() {
    let r = Response::Rows {
        rows: vec![vec![1, 2, 3]],
    };
    if let Response::Rows { rows } = r {
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0], vec![1, 2, 3]);
    }
}

#[test]
fn rows_multiple_rows() {
    let rows_data = vec![vec![1u64], vec![2], vec![3]];
    let r = Response::Rows {
        rows: rows_data.clone(),
    };
    if let Response::Rows { rows } = r {
        assert_eq!(rows.len(), 3);
        assert_eq!(rows, rows_data);
    }
}

#[test]
fn rows_multi_column() {
    let r = Response::Rows {
        rows: vec![vec![10, 20], vec![30, 40]],
    };
    if let Response::Rows { rows } = r {
        assert_eq!(rows[0][1], 20);
        assert_eq!(rows[1][0], 30);
    }
}

#[test]
fn rows_clone_equal() {
    let a = Response::Rows {
        rows: vec![vec![1, 2]],
    };
    let b = a.clone();
    assert_eq!(a, b);
}

#[test]
fn rows_debug_nonempty() {
    let r = Response::Rows { rows: vec![] };
    assert!(!format!("{:?}", r).is_empty());
}

#[test]
fn row_single_variant() {
    let r = Response::Row {
        values: vec![42, 99],
    };
    if let Response::Row { values } = r {
        assert_eq!(values, vec![42, 99]);
    }
}

#[test]
fn row_empty_values() {
    let r = Response::Row { values: vec![] };
    if let Response::Row { values } = r {
        assert!(values.is_empty());
    }
}

#[test]
fn not_found_variant() {
    let r = Response::NotFound;
    assert!(matches!(r, Response::NotFound));
}

#[test]
fn ok_affected_zero() {
    let r = Response::Ok { affected: 0 };
    if let Response::Ok { affected } = r {
        assert_eq!(affected, 0);
    }
}

#[test]
fn ok_affected_nonzero() {
    let r = Response::Ok { affected: 5 };
    if let Response::Ok { affected } = r {
        assert_eq!(affected, 5);
    }
}

#[test]
fn response_rows_100_entries() {
    let big: Vec<Vec<u64>> = (0..100).map(|i| vec![i]).collect();
    let r = Response::Rows { rows: big };
    if let Response::Rows { rows } = r {
        assert_eq!(rows.len(), 100);
    }
}

#[test]
fn response_error_variant() {
    let r = Response::Error {
        code: 1,
        message: "oops".into(),
    };
    if let Response::Error { code, message } = r {
        assert_eq!(code, 1);
        assert_eq!(message, "oops");
    }
}
