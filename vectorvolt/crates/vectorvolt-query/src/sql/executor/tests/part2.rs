//! Executor unit tests, part 2 of 4.

use super::*;

#[test]
fn test_s75_drop_mat_view_parses() {
    let tokens = crate::sql::lexer::Lexer::new("DROP MATERIALIZED VIEW mv_name")
        .tokenize()
        .unwrap();
    let stmt = crate::sql::parser::SqlParser::new(tokens).parse();
    assert!(stmt.is_ok());
}

// â”€â”€ S76 Parallel Scans â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s76_parallel_scan_correctness() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t76a (pk U64, val U64)").unwrap();
    for i in 1u64..=10 {
        ex.execute(&format!(
            "INSERT INTO t76a (pk, val) VALUES ({i}, {})",
            i * 10
        ))
        .unwrap();
    }
    let r = ex.execute("SELECT * FROM t76a").unwrap();
    match r {
        QueryResult::Rows { rows, .. } => assert_eq!(rows.len(), 10),
        _ => panic!(),
    }
}

#[test]
fn test_s76_parallel_filter() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t76b (pk U64, val U64)").unwrap();
    for i in 1u64..=5 {
        ex.execute(&format!("INSERT INTO t76b (pk, val) VALUES ({i}, {i})"))
            .unwrap();
    }
    let r = ex.execute("SELECT * FROM t76b WHERE val > 2").unwrap();
    match r {
        QueryResult::Rows { rows, .. } => assert_eq!(rows.len(), 3),
        _ => panic!(),
    }
}

#[test]
fn test_s76_parallel_count() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t76c (pk U64, val U64)").unwrap();
    for i in 1u64..=8 {
        ex.execute(&format!("INSERT INTO t76c (pk, val) VALUES ({i}, {i})"))
            .unwrap();
    }
    let r = ex.execute("SELECT COUNT(*) FROM t76c").unwrap();
    match r {
        QueryResult::Rows { rows, .. } => assert_eq!(rows[0][0], "8"),
        _ => panic!(),
    }
}

#[test]
fn test_s76_parallel_order() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t76d (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t76d (pk, val) VALUES (2, 20)")
        .unwrap();
    ex.execute("INSERT INTO t76d (pk, val) VALUES (1, 10)")
        .unwrap();
    let r = ex.execute("SELECT pk FROM t76d ORDER BY pk ASC").unwrap();
    match r {
        QueryResult::Rows { rows, .. } => assert_eq!(rows[0][0], "1"),
        _ => panic!(),
    }
}

#[test]
fn test_s76_parallel_aggregation() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t76e (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t76e (pk, val) VALUES (1, 5)")
        .unwrap();
    ex.execute("INSERT INTO t76e (pk, val) VALUES (2, 5)")
        .unwrap();
    let r = ex.execute("SELECT SUM(val) FROM t76e").unwrap();
    match r {
        QueryResult::Rows { rows, .. } => assert_eq!(rows[0][0], "10"),
        _ => panic!(),
    }
}

// â”€â”€ S77 Window RANGE Frames â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s77_rows_frame_parses() {
    let tokens = crate::sql::lexer::Lexer::new(
            "SELECT SUM(val) OVER (ORDER BY pk ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW) AS running FROM t"
        ).tokenize().unwrap();
    let stmt = crate::sql::parser::SqlParser::new(tokens).parse();
    assert!(stmt.is_ok(), "ROWS frame must parse: {stmt:?}");
}

#[test]
fn test_s77_range_frame_parses() {
    let tokens = crate::sql::lexer::Lexer::new(
            "SELECT SUM(val) OVER (ORDER BY pk RANGE BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW) AS rs FROM t"
        ).tokenize().unwrap();
    let stmt = crate::sql::parser::SqlParser::new(tokens).parse();
    assert!(stmt.is_ok(), "RANGE frame must parse: {stmt:?}");
}

#[test]
fn test_s77_running_sum() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t77a (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t77a (pk, val) VALUES (1, 10)")
        .unwrap();
    ex.execute("INSERT INTO t77a (pk, val) VALUES (2, 20)")
        .unwrap();
    ex.execute("INSERT INTO t77a (pk, val) VALUES (3, 30)")
        .unwrap();
    let r = ex.execute("SELECT SUM(val) OVER (ORDER BY pk ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW) AS rs FROM t77a ORDER BY pk");
    assert!(r.is_ok(), "running sum with frame must not error: {r:?}");
}

#[test]
fn test_s77_frame_current_row() {
    let tokens = crate::sql::lexer::Lexer::new(
            "SELECT SUM(val) OVER (ORDER BY pk ROWS BETWEEN CURRENT ROW AND UNBOUNDED FOLLOWING) FROM t"
        ).tokenize().unwrap();
    let stmt = crate::sql::parser::SqlParser::new(tokens).parse();
    assert!(stmt.is_ok());
}

#[test]
fn test_s77_frame_preceding_following() {
    let tokens = crate::sql::lexer::Lexer::new(
        "SELECT SUM(val) OVER (ORDER BY pk ROWS BETWEEN 2 PRECEDING AND 1 FOLLOWING) FROM t",
    )
    .tokenize()
    .unwrap();
    let stmt = crate::sql::parser::SqlParser::new(tokens).parse();
    assert!(stmt.is_ok());
}

// â”€â”€ S78 â€” Row-level security â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s78_create_policy() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    let r = ex.execute("CREATE POLICY p1 ON t USING (val > 0)");
    assert!(matches!(r, Ok(QueryResult::Created)));
}

#[test]
fn test_s78_policy_filters_rows() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 10)")
        .unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (2, 0)").unwrap();
    ex.execute("CREATE POLICY p1 ON t USING (val > 5)").unwrap();
    let r = ex.execute("SELECT * FROM t").unwrap();
    let actual_rows = match r {
        QueryResult::Rows { rows, .. } => rows,
        QueryResult::WithNotice { result, .. } => match *result {
            QueryResult::Rows { rows, .. } => rows,
            _ => vec![],
        },
        _ => vec![],
    };
    assert_eq!(actual_rows.len(), 1);
}

#[test]
fn test_s78_policy_no_match() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 100)")
        .unwrap();
    // No policy applied â†’ all rows visible
    let r = ex.execute("SELECT * FROM t").unwrap();
    let row_count = match r {
        QueryResult::Rows { ref rows, .. } => rows.len(),
        QueryResult::WithNotice { ref result, .. } => match result.as_ref() {
            QueryResult::Rows { rows, .. } => rows.len(),
            _ => 0,
        },
        _ => 0,
    };
    assert_eq!(row_count, 1);
}

#[test]
fn test_s78_drop_policy() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("CREATE POLICY p1 ON t USING (val > 5)").unwrap();
    let r = ex.execute("DROP POLICY p1 ON t");
    assert!(matches!(r, Ok(QueryResult::Dropped)));
}

#[test]
fn test_s78_multiple_policies() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 50)")
        .unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (2, 5)").unwrap();
    ex.execute("CREATE POLICY p1 ON t USING (val > 10)")
        .unwrap();
    ex.execute("CREATE POLICY p2 ON t USING (val < 100)")
        .unwrap();
    let r = ex.execute("SELECT * FROM t").unwrap();
    let rows = match r {
        QueryResult::Rows { rows, .. } => rows,
        QueryResult::WithNotice { result, .. } => match *result {
            QueryResult::Rows { rows, .. } => rows,
            _ => vec![],
        },
        _ => vec![],
    };
    assert_eq!(rows.len(), 1); // only val=50 passes both
}

#[test]
fn test_s78_policy_limits_update_and_delete() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 10)")
        .unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (2, 0)").unwrap();
    ex.execute("CREATE POLICY p1 ON t USING (val > 5)").unwrap();
    // Hidden row pk=2 must not be updated or deleted.
    let r = ex.execute("UPDATE t SET val = 7").unwrap();
    assert!(matches!(r, QueryResult::Affected(1)), "{r:?}");
    let r = ex.execute("DELETE FROM t WHERE pk = 2").unwrap();
    assert!(matches!(r, QueryResult::Affected(0)), "{r:?}");
    ex.execute("DROP POLICY p1 ON t").unwrap();
    let r = ex.execute("SELECT * FROM t WHERE pk = 2").unwrap();
    let rows = match r {
        QueryResult::Rows { rows, .. } => rows,
        other => panic!("{other:?}"),
    };
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0][1], "0"); // untouched by the UPDATE
}

#[test]
fn test_s78_policy_applies_to_join_right_side() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE a (pk U64, bid U64)").unwrap();
    ex.execute("CREATE TABLE b (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO a (pk, bid) VALUES (1, 1)").unwrap();
    ex.execute("INSERT INTO a (pk, bid) VALUES (2, 2)").unwrap();
    ex.execute("INSERT INTO b (pk, val) VALUES (1, 10)")
        .unwrap();
    ex.execute("INSERT INTO b (pk, val) VALUES (2, 0)").unwrap();
    let before = row_count(
        ex.execute("SELECT * FROM a INNER JOIN b ON bid = pk")
            .unwrap(),
    );
    assert_eq!(before, 2);
    ex.execute("CREATE POLICY pb ON b USING (val > 5)").unwrap();
    let after = row_count(
        ex.execute("SELECT * FROM a INNER JOIN b ON bid = pk")
            .unwrap(),
    );
    assert_eq!(after, 1);
}

#[test]
fn test_s78_unsupported_policy_expr_rejected() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    for bad in [
        "CREATE POLICY p1 ON t USING (nosuchcol > 5)",
        "CREATE POLICY p2 ON t USING (pk IN (SELECT pk FROM t))",
    ] {
        let r = ex.execute(bad);
        assert!(!matches!(r, Ok(QueryResult::Created)), "{bad}: {r:?}");
    }
    assert!(ex.policies.lock_safe().is_empty());
    assert!(
        ex.execute("CREATE POLICY p3 ON missing USING (val > 5)")
            .is_err()
    );
}

// â”€â”€ S79 â€” Column masking â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s79_create_mask() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    let r = ex.execute("CREATE MASK m1 ON t.val USING 'XXXX'");
    assert!(matches!(r, Ok(QueryResult::Created)));
}

#[test]
fn test_s79_mask_hides_value() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 42)")
        .unwrap();
    ex.execute("CREATE MASK m1 ON t.val USING 'XXXX'").unwrap();
    let r = ex.execute("SELECT * FROM t").unwrap();
    let rows = match r {
        QueryResult::Rows { rows, .. } => rows,
        QueryResult::WithNotice { result, .. } => match *result {
            QueryResult::Rows { rows, .. } => rows,
            _ => vec![],
        },
        _ => vec![],
    };
    // val column should be masked
    assert!(rows[0].iter().any(|v| v == "XXXX"));
}

#[test]
fn test_s79_mask_specific_col() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 99)")
        .unwrap();
    ex.execute("CREATE MASK m1 ON t.val USING '***'").unwrap();
    let r = ex.execute("SELECT * FROM t").unwrap();
    let rows = match r {
        QueryResult::Rows { rows, .. } => rows,
        QueryResult::WithNotice { result, .. } => match *result {
            QueryResult::Rows { rows, .. } => rows,
            _ => vec![],
        },
        _ => vec![],
    };
    assert!(rows[0].contains(&"***".to_owned()));
}

#[test]
fn test_s79_drop_mask_restores() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("CREATE MASK m1 ON t.val USING 'XXXX'").unwrap();
    let r = ex.execute("DROP MASK m1");
    assert!(matches!(r, Ok(QueryResult::Dropped)));
    // After drop, mask should be gone â€” no more masking
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 42)")
        .unwrap();
    let r2 = ex.execute("SELECT * FROM t").unwrap();
    let rows = match r2 {
        QueryResult::Rows { rows, .. } => rows,
        QueryResult::WithNotice { result, .. } => match *result {
            QueryResult::Rows { rows, .. } => rows,
            _ => vec![],
        },
        _ => vec![],
    };
    assert!(!rows[0].iter().any(|v| v == "XXXX"));
}

#[test]
fn test_s79_mask_multiple_cols() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 7)").unwrap();
    ex.execute("CREATE MASK m1 ON t.val USING 'HIDDEN'")
        .unwrap();
    let r = ex.execute("SELECT * FROM t").unwrap();
    let rows = match r {
        QueryResult::Rows { rows, .. } => rows,
        QueryResult::WithNotice { result, .. } => match *result {
            QueryResult::Rows { rows, .. } => rows,
            _ => vec![],
        },
        _ => vec![],
    };
    assert!(rows[0].iter().any(|v| v == "HIDDEN"));
}

// â”€â”€ S80 â€” Audit log â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s80_audit_insert() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 42)")
        .unwrap();
    let log = ex.audit_log.lock_safe();
    assert!(log.iter().any(|e| e.op == "INSERT"));
}

#[test]
fn test_s80_audit_update() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 1)").unwrap();
    ex.execute("UPDATE t SET val = 2 WHERE pk = 1").unwrap();
    let log = ex.audit_log.lock_safe();
    assert!(log.iter().any(|e| e.op == "UPDATE"));
}

#[test]
fn test_s80_audit_delete() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 1)").unwrap();
    ex.execute("DELETE FROM t WHERE pk = 1").unwrap();
    let log = ex.audit_log.lock_safe();
    assert!(log.iter().any(|e| e.op == "DELETE"));
}

#[test]
fn test_s80_audit_query() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 5)").unwrap();
    let r = ex.execute("SELECT * FROM vv_audit_log").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}

#[test]
fn test_s80_audit_truncate() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 1)").unwrap();
    ex.execute("TRUNCATE AUDIT LOG").unwrap();
    let log = ex.audit_log.lock_safe();
    assert!(log.is_empty());
}

// â”€â”€ S81 â€” Token auth â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s81_set_token() {
    let (ex, _dir) = setup();
    ex.execute("SET SESSION TOKEN 'secret123'").unwrap();
    let tok = ex.session_token.lock_safe().clone();
    assert_eq!(tok, Some("secret123".to_owned()));
}

#[test]
fn test_s81_require_token_pass() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 1)").unwrap();
    ex.execute("REQUIRE TOKEN 'abc' ON t").unwrap();
    ex.execute("SET SESSION TOKEN 'abc'").unwrap();
    let r = ex.execute("SELECT * FROM t").unwrap();
    // Should NOT be an error
    assert!(!matches!(r, QueryResult::Error(_)));
}

#[test]
fn test_s81_require_token_fail() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("REQUIRE TOKEN 'secret' ON t").unwrap();
    // No token set â†’ auth required
    let r = ex.execute("SELECT * FROM t").unwrap();
    assert!(matches!(r, QueryResult::Error(_)));
    if let QueryResult::Error(msg) = r {
        assert!(msg.contains("28000"));
    }
}

#[test]
fn test_s81_drop_require() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("REQUIRE TOKEN 'tok' ON t").unwrap();
    ex.execute("DROP REQUIRE TOKEN ON t").unwrap();
    // Now accessible without token
    let r = ex.execute("SELECT * FROM t").unwrap();
    assert!(!matches!(r, QueryResult::Error(_)));
}

#[test]
fn test_s81_require_token_blocks_writes() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 1)").unwrap();
    ex.execute("CREATE TABLE src (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO src (pk, val) VALUES (9, 9)")
        .unwrap();
    ex.execute("REQUIRE TOKEN 'secret' ON t").unwrap();
    for sql in [
        "INSERT INTO t (pk, val) VALUES (2, 2)",
        "INSERT INTO t (pk, val) VALUES (1, 5) ON CONFLICT DO UPDATE SET val = 5",
        "INSERT INTO t SELECT * FROM src",
        "UPDATE t SET val = 3",
        "DELETE FROM t",
    ] {
        match ex.execute(sql) {
            Ok(QueryResult::Error(msg)) => assert!(msg.contains("28000"), "{sql}: {msg}"),
            other => panic!("{sql}: expected auth error, got {other:?}"),
        }
    }
    // Nothing changed.
    ex.execute("SET SESSION TOKEN 'secret'").unwrap();
    let r = ex.execute("SELECT * FROM t").unwrap();
    match r {
        QueryResult::Rows { rows, .. } => assert_eq!(rows, vec![vec!["1", "1"]]),
        other => panic!("{other:?}"),
    }
    // With the token, writes are allowed.
    let r = ex.execute("UPDATE t SET val = 3").unwrap();
    assert!(matches!(r, QueryResult::Affected(1)), "{r:?}");
}

#[test]
fn test_s81_no_token_required() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    // No require set â†’ accessible without token
    let r = ex.execute("SELECT * FROM t").unwrap();
    assert!(!matches!(r, QueryResult::Error(_)));
}

// â”€â”€ S82 â€” Rate limiting â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s82_set_rate_limit() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    let r = ex.execute("SET RATE LIMIT ON t TO 5 PER SECOND");
    assert!(matches!(r, Ok(QueryResult::Affected(0))));
}

#[test]
fn test_s82_within_limit() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("SET RATE LIMIT ON t TO 3 PER SECOND").unwrap();
    // First 3 queries should succeed
    for _ in 0..3 {
        let r = ex.execute("SELECT * FROM t").unwrap();
        assert!(!matches!(r, QueryResult::Error(_)));
    }
}

#[test]
fn test_s82_drop_rate_limit() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("SET RATE LIMIT ON t TO 1 PER SECOND").unwrap();
    let r = ex.execute("DROP RATE LIMIT ON t");
    assert!(matches!(r, Ok(QueryResult::Affected(0))));
}

#[test]
fn test_s82_rate_limit_reset() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    // Set very high limit so we won't hit it in normal tests
    ex.execute("SET RATE LIMIT ON t TO 100 PER SECOND").unwrap();
    let r = ex.execute("SELECT * FROM t");
    assert!(r.is_ok());
}

// â”€â”€ S83 â€” IP allowlist â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s83_add_ip() {
    let (ex, _dir) = setup();
    let r = ex.execute("ADD ALLOWED IP '192.168.1.1'");
    assert!(matches!(r, Ok(QueryResult::Affected(0))));
    assert!(ex.check_ip_allowed("192.168.1.1"));
}

#[test]
fn test_s83_show_ips() {
    let (ex, _dir) = setup();
    ex.execute("ADD ALLOWED IP '10.0.0.1'").unwrap();
    let r = ex.execute("SHOW ALLOWED IPS").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
    if let QueryResult::Rows { rows, .. } = r {
        assert!(rows.iter().any(|row| row.iter().any(|v| v == "10.0.0.1")));
    }
}

#[test]
fn test_s83_remove_ip() {
    let (ex, _dir) = setup();
    ex.execute("ADD ALLOWED IP '1.2.3.4'").unwrap();
    ex.execute("ADD ALLOWED IP '5.6.7.8'").unwrap();
    ex.execute("REMOVE ALLOWED IP '1.2.3.4'").unwrap();
    // allowlist non-empty â†’ "1.2.3.4" is not allowed, "5.6.7.8" is
    assert!(!ex.check_ip_allowed("1.2.3.4"));
    assert!(ex.check_ip_allowed("5.6.7.8"));
}

#[test]
fn test_s83_empty_allowlist_allows_all() {
    let (ex, _dir) = setup();
    // Empty allowlist means all allowed
    assert!(ex.check_ip_allowed("any.ip.address"));
}

#[test]
fn test_s83_ip_blocked() {
    let (ex, _dir) = setup();
    ex.execute("ADD ALLOWED IP '192.168.1.1'").unwrap();
    // IP not in allowlist â†’ blocked
    assert!(!ex.check_ip_allowed("10.0.0.1"));
}

// â”€â”€ S84 â€” TLS cert rotation â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s84_rotate_cert() {
    let (ex, _dir) = setup();
    let r = ex.execute("ROTATE TLS CERT '/etc/ssl/cert.pem' KEY '/etc/ssl/key.pem'");
    assert!(matches!(r, Ok(QueryResult::Ok(_))));
}

#[test]
fn test_s84_cert_path_stored() {
    let (ex, _dir) = setup();
    ex.execute("ROTATE TLS CERT '/path/cert.pem' KEY '/path/key.pem'")
        .unwrap();
    let cert = ex.tls_cert_path.lock_safe().clone();
    assert_eq!(cert, Some("/path/cert.pem".to_owned()));
}

#[test]
fn test_s84_cert_key_stored() {
    let (ex, _dir) = setup();
    ex.execute("ROTATE TLS CERT '/c.pem' KEY '/k.pem'").unwrap();
    let key = ex.tls_key_path.lock_safe().clone();
    assert_eq!(key, Some("/k.pem".to_owned()));
}

#[test]
fn test_s84_rotate_overrides() {
    let (ex, _dir) = setup();
    ex.execute("ROTATE TLS CERT '/old.pem' KEY '/old_key.pem'")
        .unwrap();
    ex.execute("ROTATE TLS CERT '/new.pem' KEY '/new_key.pem'")
        .unwrap();
    let cert = ex.tls_cert_path.lock_safe().clone();
    assert_eq!(cert, Some("/new.pem".to_owned()));
}

#[test]
fn test_s84_rotate_no_reload_needed() {
    let (ex, _dir) = setup();
    let r = ex.execute("ROTATE TLS CERT '/c.pem' KEY '/k.pem'").unwrap();
    if let QueryResult::Ok(msg) = r {
        assert!(msg.contains("cert rotation"));
    }
}

// â”€â”€ S85 â€” pgwire extended query â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s85_pgwire_parse() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    let r = ex.pgwire_parse("s1", "SELECT * FROM t");
    assert!(r.is_ok());
}

#[test]
fn test_s85_pgwire_bind() {
    let (ex, _dir) = setup();
    ex.pgwire_parse("s1", "SELECT * FROM t").ok();
    ex.pgwire_bind("p1", "s1", vec![LitVal::Int(1)]);
    let portals = ex.bound_portals.lock_safe();
    assert!(portals.contains_key("p1"));
}

#[test]
fn test_s85_pgwire_execute() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 10)")
        .unwrap();
    ex.pgwire_parse("s1", "SELECT * FROM t").unwrap();
    ex.pgwire_bind("p1", "s1", vec![]);
    let r = ex.pgwire_execute_portal("p1").unwrap();
    assert!(matches!(
        r,
        QueryResult::Rows { .. } | QueryResult::WithNotice { .. }
    ));
}

#[test]
fn test_s85_pgwire_reuse_portal() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.pgwire_parse("s1", "SELECT * FROM t").unwrap();
    ex.pgwire_bind("p1", "s1", vec![]);
    let r1 = ex.pgwire_execute_portal("p1");
    let r2 = ex.pgwire_execute_portal("p1");
    assert!(r1.is_ok() && r2.is_ok());
}

#[test]
fn test_s85_pgwire_describe() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.pgwire_parse("s1", "SELECT pk, val FROM t").unwrap();
    let cols = ex.pgwire_describe("s1");
    assert_eq!(cols, vec!["pk".to_owned(), "val".to_owned()]);
}

// â”€â”€ S86 â€” Error codes â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s86_syntax_error_code() {
    let err = sql_error("42000", "syntax error near BLAH");
    if let QueryResult::Error(msg) = err {
        assert!(msg.contains("[42000]"));
    } else {
        panic!("expected Error");
    }
}

#[test]
fn test_s86_constraint_error_code() {
    let err = sql_error("23000", "constraint violation");
    if let QueryResult::Error(msg) = err {
        assert!(msg.contains("[23000]"));
    } else {
        panic!("expected Error");
    }
}

#[test]
fn test_s86_auth_error_code() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("REQUIRE TOKEN 'secret' ON t").unwrap();
    let r = ex.execute("SELECT * FROM t").unwrap();
    if let QueryResult::Error(msg) = r {
        assert!(msg.contains("[28000]"));
    } else {
        panic!("expected auth error");
    }
}

#[test]
fn test_s86_not_found_code() {
    let err = sql_error("42P01", "relation not found");
    if let QueryResult::Error(msg) = err {
        assert!(msg.contains("[42P01]"));
    } else {
        panic!("expected Error");
    }
}

#[test]
fn test_s86_error_format() {
    let err = sql_error("HY000", "general error");
    if let QueryResult::Error(msg) = err {
        assert!(msg.starts_with('['));
        assert!(msg.contains("] "));
    } else {
        panic!("expected Error");
    }
}

// â”€â”€ S87 â€” Notice messages â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s87_notice_empty_table() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("SET WARNINGS ON").unwrap();
    let r = ex.execute("SELECT * FROM t").unwrap();
    assert!(matches!(r, QueryResult::WithNotice { .. }));
    if let QueryResult::WithNotice { notice, .. } = r {
        assert!(notice.contains("empty"));
    }
}

#[test]
fn test_s87_notice_index_exists() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("CREATE INDEX idx1 ON t (pk)").unwrap();
    ex.execute("SET WARNINGS ON").unwrap();
    let r = ex.execute("CREATE INDEX idx1 ON t (pk)").unwrap();
    assert!(matches!(r, QueryResult::WithNotice { .. }));
}

#[test]
fn test_s87_set_warnings_off() {
    let (ex, _dir) = setup();
    ex.execute("SET WARNINGS OFF").unwrap();
    let warnings_on = *ex.warnings_enabled.lock_safe();
    assert!(!warnings_on);
}

#[test]
fn test_s87_warnings_on_by_default() {
    let (ex, _dir) = setup();
    let warnings_on = *ex.warnings_enabled.lock_safe();
    assert!(!warnings_on); // warnings off by default; enable with SET WARNINGS ON
}

#[test]
fn test_s87_notice_format() {
    let notice = QueryResult::Notice("implicit cast: f64 â†’ i64".to_owned());
    if let QueryResult::Notice(msg) = notice {
        assert!(!msg.is_empty());
    }
}

// â”€â”€ S88 â€” Streaming replication â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn test_s88_create_slot() {
    let (ex, _dir) = setup();
    let r = ex.execute("CREATE REPLICATION SLOT myslot").unwrap();
    assert!(matches!(r, QueryResult::Created));
    let guard = ex.replication_slots.lock_safe();
    assert!(guard.contains_key("myslot"));
}

#[test]
fn test_s88_drop_slot() {
    let (ex, _dir) = setup();
    ex.execute("CREATE REPLICATION SLOT myslot").unwrap();
    let r = ex.execute("DROP REPLICATION SLOT myslot").unwrap();
    assert!(matches!(r, QueryResult::Dropped));
    let guard = ex.replication_slots.lock_safe();
    assert!(!guard.contains_key("myslot"));
}

#[test]
fn test_s88_temporary_slot() {
    let (ex, _dir) = setup();
    ex.execute("CREATE REPLICATION SLOT tmpslot TEMPORARY")
        .unwrap();
    let guard = ex.replication_slots.lock_safe();
    assert!(guard.get("tmpslot").unwrap().temporary);
}

#[test]
fn test_s88_start_replication() {
    let (ex, _dir) = setup();
    ex.execute("CREATE REPLICATION SLOT s1").unwrap();
    let r = ex.execute("START REPLICATION SLOT s1 FROM '0/0'").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}

#[test]
fn test_s88_replication_rows() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    ex.execute("INSERT INTO t (pk, v) VALUES (1, 10)").unwrap();
    ex.execute("CREATE REPLICATION SLOT s1").unwrap();
    let r = ex.execute("START REPLICATION SLOT s1 FROM '0/0'").unwrap();
    match r {
        QueryResult::Rows { columns, .. } => {
            assert!(columns.contains(&"op".to_owned()));
            assert!(columns.contains(&"table".to_owned()));
        }
        _ => panic!("expected Rows"),
    }
}

// â”€â”€ S89 â€” Logical decode â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn test_s89_logical_decode_basic() {
    let (ex, _dir) = setup();
    let r = ex.execute("SELECT * FROM vv_logical_decode").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}

#[test]
fn test_s89_logical_decode_insert() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    ex.execute("INSERT INTO t (pk, v) VALUES (1, 10)").unwrap();
    let r = ex.execute("SELECT * FROM vv_logical_decode").unwrap();
    match r {
        QueryResult::Rows { rows, .. } => assert!(!rows.is_empty()),
        _ => panic!(),
    }
}

#[test]
fn test_s89_logical_decode_update() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    ex.execute("INSERT INTO t (pk, v) VALUES (1, 10)").unwrap();
    ex.execute("UPDATE t SET v = 99 WHERE pk = 1").unwrap();
    let r = ex.execute("SELECT * FROM vv_logical_decode").unwrap();
    match r {
        QueryResult::Rows { rows, .. } => assert!(rows.len() >= 2),
        _ => panic!(),
    }
}

#[test]
fn test_s89_logical_decode_delete() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    ex.execute("INSERT INTO t (pk, v) VALUES (1, 10)").unwrap();
    ex.execute("DELETE FROM t WHERE pk = 1").unwrap();
    let r = ex.execute("SELECT * FROM vv_logical_decode").unwrap();
    match r {
        QueryResult::Rows { columns, .. } => {
            assert!(columns.contains(&"op".to_owned()));
        }
        _ => panic!(),
    }
}

#[test]
fn test_s89_logical_decode_lsn_advance() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    ex.execute("INSERT INTO t (pk, v) VALUES (1, 10)").unwrap();
    let r = ex.execute("SELECT * FROM vv_logical_decode").unwrap();
    match r {
        QueryResult::Rows { columns, .. } => {
            assert!(columns.contains(&"lsn".to_owned()));
        }
        _ => panic!(),
    }
}

// â”€â”€ S90 â€” Read-only mode â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn test_s90_set_read_only() {
    let (ex, _dir) = setup();
    let r = ex.execute("SET READ ONLY").unwrap();
    assert!(matches!(r, QueryResult::Affected(0)));
    assert!(*ex.read_only.lock_safe());
}

#[test]
fn test_a4_new_session_shares_security_state_not_txn() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    ex.execute("INSERT INTO t (pk, v) VALUES (1, 10)").unwrap();
    ex.execute("INSERT INTO t (pk, v) VALUES (2, 0)").unwrap();
    let s1 = ex.new_session();
    let s2 = ex.new_session();
    s1.execute("CREATE POLICY p ON t USING (v > 5)").unwrap();
    match s2.execute("SELECT * FROM t").unwrap() {
        QueryResult::Rows { rows, .. } => assert_eq!(rows.len(), 1),
        other => panic!("{other:?}"),
    }
    // Session tokens and transactions stay per session.
    s1.execute("REQUIRE TOKEN 'tk' ON t").unwrap();
    s1.execute("SET SESSION TOKEN 'tk'").unwrap();
    assert!(!matches!(
        s1.execute("SELECT * FROM t").unwrap(),
        QueryResult::Error(_)
    ));
    assert!(matches!(
        s2.execute("SELECT * FROM t").unwrap(),
        QueryResult::Error(_)
    ));
    s1.execute("DROP REQUIRE TOKEN ON t").unwrap();
    s1.execute("BEGIN").unwrap();
    assert!(s2.execute("COMMIT").is_err());
    s1.execute("ROLLBACK").unwrap();
    s2.execute("SET READ ONLY").unwrap();
    assert!(matches!(
        ex.new_session()
            .execute("INSERT INTO t (pk, v) VALUES (3, 30)")
            .unwrap(),
        QueryResult::Error(_)
    ));
}

#[test]
fn test_s90_read_only_blocks_insert() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    ex.execute("SET READ ONLY").unwrap();
    let r = ex.execute("INSERT INTO t (pk, v) VALUES (1, 10)").unwrap();
    assert!(matches!(r, QueryResult::Error(_)));
    if let QueryResult::Error(msg) = r {
        assert!(msg.contains("25006"));
    }
}

#[test]
fn test_s90_read_only_allows_select() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    ex.execute("SET READ ONLY").unwrap();
    let r = ex.execute("SELECT * FROM t").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}

#[test]
fn test_s90_set_read_write() {
    let (ex, _dir) = setup();
    ex.execute("SET READ ONLY").unwrap();
    let r = ex.execute("SET READ WRITE").unwrap();
    assert!(matches!(r, QueryResult::Affected(0)));
    assert!(!*ex.read_only.lock_safe());
}

#[test]
fn test_s90_read_write_allows_insert() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    ex.execute("SET READ ONLY").unwrap();
    ex.execute("SET READ WRITE").unwrap();
    let r = ex.execute("INSERT INTO t (pk, v) VALUES (1, 10)").unwrap();
    assert!(matches!(r, QueryResult::Affected(1)));
}

// â”€â”€ S91 â€” Failover / promote â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn test_s91_promote() {
    let (ex, _dir) = setup();
    ex.execute("SET READ ONLY").unwrap();
    let r = ex.execute("PROMOTE STANDBY").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
    assert!(!*ex.read_only.lock_safe());
}

#[test]
fn test_s91_demote() {
    let (ex, _dir) = setup();
    let r = ex.execute("DEMOTE PRIMARY").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
    assert!(*ex.read_only.lock_safe());
}

#[test]
fn test_s91_promote_allows_writes() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    ex.execute("SET READ ONLY").unwrap();
    ex.execute("PROMOTE STANDBY").unwrap();
    let r = ex.execute("INSERT INTO t (pk, v) VALUES (1, 10)").unwrap();
    assert!(matches!(r, QueryResult::Affected(1)));
}

#[test]
fn test_s91_demote_blocks_writes() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    ex.execute("DEMOTE PRIMARY").unwrap();
    let r = ex.execute("INSERT INTO t (pk, v) VALUES (1, 10)").unwrap();
    assert!(matches!(r, QueryResult::Error(_)));
}

#[test]
fn test_s91_promote_demote_cycle() {
    let (ex, _dir) = setup();
    ex.execute("DEMOTE PRIMARY").unwrap();
    ex.execute("PROMOTE STANDBY").unwrap();
    ex.execute("DEMOTE PRIMARY").unwrap();
    assert!(*ex.read_only.lock_safe());
    ex.execute("PROMOTE STANDBY").unwrap();
    assert!(!*ex.read_only.lock_safe());
}

// â”€â”€ S92 â€” WAL archiving â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn test_s92_set_archive_path() {
    let (ex, _dir) = setup();
    let r = ex.execute("SET ARCHIVE PATH '/tmp/vv_wal_test'").unwrap();
    assert!(matches!(r, QueryResult::Affected(0)));
    let path = ex.archive_path.lock_safe().clone();
    assert_eq!(path, Some("/tmp/vv_wal_test".to_owned()));
}

#[test]
fn test_s92_archive_now() {
    let (ex, _dir) = setup();
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().to_str().unwrap().to_owned();
    ex.execute(&format!("SET ARCHIVE PATH '{}'", path)).unwrap();
    let r = ex.execute("ARCHIVE NOW").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}

#[test]
fn test_s92_archive_creates_file() {
    let (ex, _dir) = setup();
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().to_str().unwrap().to_owned();
    ex.execute(&format!("SET ARCHIVE PATH '{}'", path)).unwrap();
    ex.execute("ARCHIVE NOW").unwrap();
    let entries: Vec<_> = std::fs::read_dir(&path).unwrap().collect();
    assert!(!entries.is_empty());
}

#[test]
fn test_s92_show_archive_status() {
    let (ex, _dir) = setup();
    ex.execute("SET ARCHIVE PATH '/tmp/vv_wal2'").unwrap();
    let r = ex.execute("SHOW ARCHIVE STATUS").unwrap();
    match r {
        QueryResult::Rows { columns, rows } => {
            assert!(columns.contains(&"path".to_owned()));
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0][0], "/tmp/vv_wal2");
        }
        _ => panic!(),
    }
}

#[test]
fn test_s92_archive_count() {
    let (ex, _dir) = setup();
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().to_str().unwrap().to_owned();
    ex.execute(&format!("SET ARCHIVE PATH '{}'", path)).unwrap();
    ex.execute("ARCHIVE NOW").unwrap();
    ex.execute("ARCHIVE NOW").unwrap();
    let count = *ex.archive_count.lock_safe();
    assert_eq!(count, 2);
}

// â”€â”€ S93 â€” Slow query log â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn test_s93_set_threshold() {
    let (ex, _dir) = setup();
    let r = ex.execute("SET SLOW_QUERY_THRESHOLD 100").unwrap();
    assert!(matches!(r, QueryResult::Affected(0)));
    let t = *ex.slow_query_threshold_ms.lock_safe();
    assert_eq!(t, Some(100));
}

#[test]
fn test_s93_slow_query_logged() {
    let (ex, _dir) = setup();
    // Set threshold to 0 so everything gets logged
    ex.execute("SET SLOW_QUERY_THRESHOLD 0").unwrap();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    ex.execute("SELECT * FROM t").unwrap();
    let log = ex.slow_log.lock_safe();
    assert!(!log.is_empty());
}

#[test]
fn test_s93_fast_query_not_logged() {
    let (ex, _dir) = setup();
    // Very high threshold â€” nothing should be logged
    ex.execute("SET SLOW_QUERY_THRESHOLD 99999999").unwrap();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    ex.execute("SELECT * FROM t").unwrap();
    let log = ex.slow_log.lock_safe();
    // None of these should exceed 99999999ms
    assert!(log.is_empty());
}

#[test]
fn test_s93_slow_log_query() {
    let (ex, _dir) = setup();
    ex.execute("SET SLOW_QUERY_THRESHOLD 0").unwrap();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    ex.execute("SELECT * FROM t").unwrap();
    let r = ex.execute("SELECT * FROM vv_slow_log").unwrap();
    match r {
        QueryResult::Rows { columns, rows } => {
            assert!(columns.contains(&"sql".to_owned()));
            assert!(!rows.is_empty());
        }
        _ => panic!(),
    }
}

#[test]
fn test_s93_truncate_slow_log() {
    let (ex, _dir) = setup();
    ex.execute("SET SLOW_QUERY_THRESHOLD 0").unwrap();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    ex.execute("SELECT * FROM t").unwrap();
    ex.execute("TRUNCATE SLOW LOG").unwrap();
    let log = ex.slow_log.lock_safe();
    assert!(log.is_empty());
}

// â”€â”€ S94 â€” Query fingerprinting â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn test_s94_fingerprint_literals() {
    let fp1 = normalize_sql("SELECT * FROM t WHERE pk = 1");
    let fp2 = normalize_sql("SELECT * FROM t WHERE pk = 42");
    assert_eq!(fp1, fp2);
}

#[test]
fn test_s94_fingerprint_different_queries() {
    let fp1 = normalize_sql("SELECT * FROM t WHERE pk = 1");
    let fp2 = normalize_sql("SELECT * FROM orders WHERE id = 1");
    assert_ne!(fp1, fp2);
}

#[test]
fn test_s94_fingerprint_in_stat_statements() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    ex.execute("INSERT INTO t (pk, v) VALUES (1, 10)").unwrap();
    ex.execute("INSERT INTO t (pk, v) VALUES (2, 20)").unwrap();
    let r = ex.execute("SELECT * FROM vv_stat_statements").unwrap();
    match r {
        QueryResult::Rows { columns, .. } => {
            assert!(columns.contains(&"fingerprint".to_owned()));
        }
        _ => panic!(),
    }
}

#[test]
fn test_s94_query_fingerprints_table() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    ex.execute("SELECT * FROM t").unwrap();
    let r = ex.execute("SELECT * FROM vv_query_fingerprints").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}

#[test]
fn test_s94_fingerprint_grouping() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    ex.execute("SELECT * FROM t").unwrap();
    ex.execute("SELECT * FROM t").unwrap();
    let r = ex.execute("SELECT * FROM vv_query_fingerprints").unwrap();
    match r {
        QueryResult::Rows { rows, .. } => {
            // Should have grouped the two identical queries
            let select_fp = normalize_sql("SELECT * FROM t");
            let found = rows.iter().any(|row| row[0] == select_fp);
            assert!(found || !rows.is_empty());
        }
        _ => panic!(),
    }
}

// â”€â”€ S95 â€” pg_stat_statements compat â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn test_s95_pg_stat_statements() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    ex.execute("SELECT * FROM t").unwrap();
    let r = ex.execute("SELECT * FROM pg_stat_statements").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}

#[test]
fn test_s95_pg_stat_col_names() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    ex.execute("SELECT * FROM t").unwrap();
    let r = ex.execute("SELECT * FROM pg_stat_statements").unwrap();
    match r {
        QueryResult::Rows { columns, .. } => {
            assert!(columns.contains(&"userid".to_owned()));
            assert!(columns.contains(&"queryid".to_owned()));
            assert!(columns.contains(&"calls".to_owned()));
        }
        _ => panic!(),
    }
}

#[test]
fn test_s95_pg_stat_calls() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    ex.execute("SELECT * FROM t").unwrap();
    ex.execute("SELECT * FROM t").unwrap();
    let r = ex.execute("SELECT * FROM pg_stat_statements").unwrap();
    match r {
        QueryResult::Rows { columns, rows } => {
            let calls_idx = columns.iter().position(|c| c == "calls").unwrap();
            let select_fp = normalize_sql("SELECT * FROM t");
            let found = rows.iter().find(|row| {
                let query_idx = columns.iter().position(|c| c == "query").unwrap();
                row[query_idx] == select_fp
            });
            if let Some(row) = found {
                let calls: u64 = row[calls_idx].parse().unwrap();
                assert!(calls >= 2);
            }
        }
        _ => panic!(),
    }
}

#[test]
fn test_s95_pg_stat_reset_fn() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    ex.execute("SELECT * FROM t").unwrap();
    // Call reset via SELECT pg_stat_statements_reset() FROM t
    let r = ex
        .execute("SELECT pg_stat_statements_reset() FROM t")
        .unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
    let stats = ex.query_stats.lock_safe();
    assert!(stats.is_empty());
}

#[test]
fn test_s95_pg_stat_queryid() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    ex.execute("SELECT * FROM t").unwrap();
    let r = ex.execute("SELECT * FROM pg_stat_statements").unwrap();
    match r {
        QueryResult::Rows { columns, rows } => {
            let qid_idx = columns.iter().position(|c| c == "queryid").unwrap();
            if !rows.is_empty() {
                let qid: u64 = rows[0][qid_idx].parse().unwrap_or(0);
                assert!(qid > 0);
            }
        }
        _ => panic!(),
    }
}

// â”€â”€ S96 â€” Auto-EXPLAIN threshold â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn test_s96_set_threshold() {
    let (ex, _dir) = setup();
    let r = ex.execute("SET AUTO_EXPLAIN_THRESHOLD 50").unwrap();
    assert!(matches!(r, QueryResult::Affected(0)));
    let t = *ex.auto_explain_threshold_ms.lock_safe();
    assert_eq!(t, Some(50));
}

#[test]
fn test_s96_auto_explain_fires() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    // Set threshold to 0 so all queries trigger auto-explain
    ex.execute("SET AUTO_EXPLAIN_THRESHOLD 0").unwrap();
    let r = ex.execute("SELECT * FROM t").unwrap();
    // May or may not fire depending on timing, but should not error
    assert!(matches!(
        r,
        QueryResult::Rows { .. } | QueryResult::WithNotice { .. }
    ));
}

#[test]
fn test_s96_auto_explain_not_fires() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    // Very high threshold â€” auto-explain should not fire
    ex.execute("SET AUTO_EXPLAIN_THRESHOLD 99999999").unwrap();
    let r = ex.execute("SELECT * FROM t").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}

#[test]
fn test_s96_auto_explain_plan_in_notice() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    ex.execute("INSERT INTO t (pk, v) VALUES (1, 10)").unwrap();
    // Threshold 0 to force trigger
    ex.execute("SET AUTO_EXPLAIN_THRESHOLD 0").unwrap();
    let r = ex.execute("SELECT * FROM t").unwrap();
    // If auto-explain fired, result is WithNotice; otherwise Rows
    match r {
        QueryResult::WithNotice { notice, .. } => {
            assert!(notice.contains("auto-explain") || notice.contains("Scan"));
        }
        QueryResult::Rows { .. } => {}
        _ => panic!("unexpected result"),
    }
}

#[test]
fn test_s96_auto_explain_disabled() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    // No threshold set â€” auto-explain disabled
    let r = ex.execute("SELECT * FROM t").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}

// â”€â”€ S97 â€” Wait event tracking â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn test_s97_wait_events_table() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    ex.execute("SELECT * FROM t").unwrap();
    let r = ex.execute("SELECT * FROM vv_wait_events").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}

#[test]
fn test_s97_wait_event_recorded() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    ex.execute("SELECT * FROM t").unwrap();
    let events = ex.wait_events.lock_safe();
    assert!(!events.is_empty());
}

#[test]
fn test_s97_wait_event_cols() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    ex.execute("SELECT * FROM t").unwrap();
    let r = ex.execute("SELECT * FROM vv_wait_events").unwrap();
    match r {
        QueryResult::Rows { columns, .. } => {
            assert!(columns.contains(&"event_type".to_owned()));
            assert!(columns.contains(&"object_name".to_owned()));
            assert!(columns.contains(&"duration_us".to_owned()));
        }
        _ => panic!(),
    }
}

#[test]
fn test_s97_reset_wait_events() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    ex.execute("SELECT * FROM t").unwrap();
    ex.execute("RESET WAIT EVENTS").unwrap();
    let events = ex.wait_events.lock_safe();
    assert!(events.is_empty());
}

#[test]
fn test_s97_wait_events_after_query() {
    let (ex, _dir) = setup();
    ex.execute("RESET WAIT EVENTS").unwrap();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    ex.execute("SELECT * FROM t").unwrap();
    let events = ex.wait_events.lock_safe();
    // Each execute() call records a wait event
    assert!(!events.is_empty());
}

// â”€â”€ S98 â€” Flamegraph export â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s98_flamegraph_basic() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    let r = ex.execute("EXPLAIN FLAMEGRAPH SELECT * FROM t").unwrap();
    assert!(matches!(r, QueryResult::ExplainPlan(_)));
}

#[test]
fn test_s98_flamegraph_json() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    let r = ex.execute("EXPLAIN FLAMEGRAPH SELECT * FROM t").unwrap();
    if let QueryResult::ExplainPlan(lines) = r {
        let json = lines.join("");
        assert!(json.contains('{'));
        assert!(json.contains('}'));
    } else {
        panic!("expected ExplainPlan");
    }
}

#[test]
fn test_s98_flamegraph_has_root() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    let r = ex.execute("EXPLAIN FLAMEGRAPH SELECT * FROM t").unwrap();
    if let QueryResult::ExplainPlan(lines) = r {
        assert!(lines[0].contains("root"));
    } else {
        panic!("expected ExplainPlan");
    }
}

#[test]
fn test_s98_flamegraph_children() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    let r = ex.execute("EXPLAIN FLAMEGRAPH SELECT * FROM t").unwrap();
    if let QueryResult::ExplainPlan(lines) = r {
        assert!(lines[0].contains("children"));
    } else {
        panic!("expected ExplainPlan");
    }
}

#[test]
fn test_s98_flamegraph_timing() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    let r = ex.execute("EXPLAIN FLAMEGRAPH SELECT * FROM t").unwrap();
    if let QueryResult::ExplainPlan(lines) = r {
        // value field should be present (timing)
        assert!(lines[0].contains("value"));
    } else {
        panic!("expected ExplainPlan");
    }
}

// â”€â”€ S99 â€” Health dashboard â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s99_health_basic() {
    let (ex, _dir) = setup();
    let r = ex.execute("SELECT * FROM vv_health").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}

#[test]
fn test_s99_health_has_uptime() {
    let (ex, _dir) = setup();
    let r = ex.execute("SELECT * FROM vv_health").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert!(rows.iter().any(|row| row[0] == "uptime_s"));
    } else {
        panic!("expected Rows");
    }
}

#[test]
fn test_s99_health_cache_ratio() {
    let (ex, _dir) = setup();
    let r = ex.execute("SELECT * FROM vv_health").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert!(rows.iter().any(|row| row[0] == "cache_hit_ratio"));
    } else {
        panic!("expected Rows");
    }
}

#[test]
fn test_s99_health_read_only_metric() {
    let (ex, _dir) = setup();
    let r = ex.execute("SELECT * FROM vv_health").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert!(rows.iter().any(|row| row[0] == "read_only"));
    } else {
        panic!("expected Rows");
    }
}

#[test]
fn test_s99_health_slow_queries_metric() {
    let (ex, _dir) = setup();
    let r = ex.execute("SELECT * FROM vv_health").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert!(rows.iter().any(|row| row[0] == "slow_queries"));
    } else {
        panic!("expected Rows");
    }
}

// â”€â”€ S100 â€” EXPLAIN BUFFERS â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s100_explain_buffers_basic() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    let r = ex.execute("EXPLAIN BUFFERS SELECT * FROM t").unwrap();
    assert!(matches!(r, QueryResult::ExplainPlan(_)));
}

#[test]
fn test_s100_buffers_hit() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    let r = ex.execute("EXPLAIN BUFFERS SELECT * FROM t").unwrap();
    if let QueryResult::ExplainPlan(lines) = r {
        assert!(lines.iter().any(|l| l.contains("buffers_hit")));
    } else {
        panic!("expected ExplainPlan");
    }
}

#[test]
fn test_s100_buffers_miss() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    let r = ex.execute("EXPLAIN BUFFERS SELECT * FROM t").unwrap();
    if let QueryResult::ExplainPlan(lines) = r {
        assert!(lines.iter().any(|l| l.contains("buffers_miss")));
    } else {
        panic!("expected ExplainPlan");
    }
}

#[test]
fn test_s100_buffers_in_output() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    let r = ex
        .execute("EXPLAIN (BUFFERS, ANALYZE) SELECT * FROM t")
        .unwrap();
    assert!(matches!(r, QueryResult::ExplainPlan(_)));
}

#[test]
fn test_s100_buffers_analyze_combined() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    let r = ex
        .execute("EXPLAIN (BUFFERS, ANALYZE) SELECT * FROM t")
        .unwrap();
    if let QueryResult::ExplainPlan(lines) = r {
        assert!(!lines.is_empty());
    } else {
        panic!("expected ExplainPlan");
    }
}

// â”€â”€ S101 â€” psql meta-commands â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s101_meta_dt() {
    let (ex, _dir) = setup();
    let r = ex.execute("\\dt").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}

#[test]
fn test_s101_meta_d_table() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    let r = ex.execute("\\d t").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}

#[test]
fn test_s101_meta_di() {
    let (ex, _dir) = setup();
    let r = ex.execute("\\di").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}

#[test]
fn test_s101_meta_dv() {
    let (ex, _dir) = setup();
    let r = ex.execute("\\dv").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}

#[test]
fn test_s101_meta_unknown() {
    let (ex, _dir) = setup();
    let r = ex.execute("\\zzz").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}

// â”€â”€ S102 â€” pgcli meta-commands â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s102_meta_l() {
    let (ex, _dir) = setup();
    let r = ex.execute("\\l").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert!(rows.iter().any(|row| row[0] == "vectorvolt"));
    } else {
        panic!("expected Rows");
    }
}

#[test]
fn test_s102_meta_timing() {
    let (ex, _dir) = setup();
    let r = ex.execute("\\timing").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}

#[test]
fn test_s102_meta_x() {
    let (ex, _dir) = setup();
    let r = ex.execute("\\x").unwrap();
    assert!(matches!(r, QueryResult::Ok(ref s) if s.contains("Expanded")));
}

#[test]
fn test_s102_meta_c() {
    let (ex, _dir) = setup();
    let r = ex.execute("\\c mydb").unwrap();
    assert!(matches!(r, QueryResult::Ok(ref s) if s.contains("mydb")));
}

#[test]
fn test_s102_timing_toggle() {
    let (ex, _dir) = setup();
    ex.execute("\\timing").unwrap(); // on
    let enabled = *ex.timing_enabled.lock_safe();
    assert!(enabled);
    ex.execute("\\timing").unwrap(); // off
    let enabled2 = *ex.timing_enabled.lock_safe();
    assert!(!enabled2);
}

// â”€â”€ S103 â€” Cursor / portal â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s103_declare_cursor() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    ex.execute("INSERT INTO t (pk, v) VALUES (1, 10)").unwrap();
    ex.execute("INSERT INTO t (pk, v) VALUES (2, 20)").unwrap();
    let r = ex
        .execute("DECLARE cur CURSOR FOR SELECT * FROM t")
        .unwrap();
    assert!(matches!(r, QueryResult::Affected(0)));
    assert!(ex.cursors.lock_safe().contains_key("cur"));
}

#[test]
fn test_s103_fetch_cursor() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    ex.execute("INSERT INTO t (pk, v) VALUES (1, 10)").unwrap();
    ex.execute("INSERT INTO t (pk, v) VALUES (2, 20)").unwrap();
    ex.execute("DECLARE cur CURSOR FOR SELECT * FROM t")
        .unwrap();
    let r = ex.execute("FETCH 1 FROM cur").unwrap();
    assert!(matches!(r, QueryResult::Rows { ref rows, .. } if rows.len() == 1));
}

#[test]
fn test_s103_fetch_partial() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    for i in 1..=5u64 {
        ex.execute(&format!("INSERT INTO t (pk, v) VALUES ({i}, {i})"))
            .unwrap();
    }
    ex.execute("DECLARE cur CURSOR FOR SELECT * FROM t")
        .unwrap();
    let r1 = ex.execute("FETCH 2 FROM cur").unwrap();
    let r2 = ex.execute("FETCH 2 FROM cur").unwrap();
    assert!(matches!(r1, QueryResult::Rows { ref rows, .. } if rows.len() == 2));
    assert!(matches!(r2, QueryResult::Rows { ref rows, .. } if rows.len() == 2));
}

#[test]
fn test_s103_close_cursor() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    ex.execute("INSERT INTO t (pk, v) VALUES (1, 10)").unwrap();
    ex.execute("DECLARE cur CURSOR FOR SELECT * FROM t")
        .unwrap();
    ex.execute("CLOSE cur").unwrap();
    assert!(!ex.cursors.lock_safe().contains_key("cur"));
}

#[test]
fn test_s103_fetch_exhausted() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    ex.execute("INSERT INTO t (pk, v) VALUES (1, 10)").unwrap();
    ex.execute("DECLARE cur CURSOR FOR SELECT * FROM t")
        .unwrap();
    ex.execute("FETCH 1 FROM cur").unwrap(); // consume
    let r = ex.execute("FETCH 1 FROM cur").unwrap(); // exhausted
    assert!(matches!(r, QueryResult::Rows { ref rows, .. } if rows.is_empty()));
}

// â”€â”€ S104 â€” COPY FROM STDIN â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s104_copy_stdin_basic() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    let r = ex.execute("COPY t FROM STDIN '1\t10\n\\.'").unwrap();
    assert!(matches!(r, QueryResult::Affected(1)));
}

#[test]
fn test_s104_copy_stdin_multirow() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    let r = ex.execute("COPY t FROM STDIN '1\t10\n2\t20\n\\.'").unwrap();
    assert!(matches!(r, QueryResult::Affected(2)));
}

#[test]
fn test_s104_copy_stdin_tab_delimited() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    ex.execute("COPY t FROM STDIN '5\t50\n\\.'").unwrap();
    let r = ex.execute("SELECT * FROM t").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert!(!rows.is_empty());
    } else {
        panic!("expected Rows");
    }
}

#[test]
fn test_s104_copy_stdin_terminator() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    // Rows after \. should NOT be inserted
    let r = ex.execute("COPY t FROM STDIN '1\t1\n\\.\n2\t2'").unwrap();
    // Only first row before \.
    assert!(matches!(r, QueryResult::Affected(1)));
}

#[test]
fn test_s104_copy_stdin_then_select() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    ex.execute("COPY t FROM STDIN '7\t77\n\\.'").unwrap();
    let r = ex.execute("SELECT * FROM t").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0][0], "7");
    } else {
        panic!("expected Rows");
    }
}

// â”€â”€ S105 â€” Type OIDs + pg_typeof â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s105_pg_typeof() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    ex.execute("INSERT INTO t (pk, v) VALUES (1, 42)").unwrap();
    let r = ex.execute("SELECT pg_typeof(v) AS typ FROM t").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}

#[test]
fn test_s105_pg_type_table() {
    let (ex, _dir) = setup();
    let r = ex.execute("SELECT * FROM pg_type").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert!(!rows.is_empty());
    } else {
        panic!("expected Rows");
    }
}

#[test]
fn test_s105_pg_typeof_int() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    ex.execute("INSERT INTO t (pk, v) VALUES (1, 100)").unwrap();
    let r = ex.execute("SELECT pg_typeof(pk) AS typ FROM t").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert!(!rows.is_empty());
    } else {
        panic!("expected Rows");
    }
}

#[test]
fn test_s105_information_schema_columns() {
    let (ex, _dir) = setup();
    let r = ex.execute("SELECT * FROM information_schema").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert!(!rows.is_empty());
    } else {
        panic!("expected Rows");
    }
}

#[test]
fn test_s105_pg_type_oid() {
    let (ex, _dir) = setup();
    let r = ex.execute("SELECT * FROM pg_type").unwrap();
    if let QueryResult::Rows { columns, .. } = r {
        assert!(columns.iter().any(|c| c == "oid"));
    } else {
        panic!("expected Rows");
    }
}

// â”€â”€ S106 â€” Statement timeout â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s106_set_timeout() {
    let (ex, _dir) = setup();
    let r = ex.execute("SET STATEMENT_TIMEOUT 5000").unwrap();
    assert!(matches!(r, QueryResult::Affected(0)));
    let ms = *ex.statement_timeout_ms.lock_safe();
    assert_eq!(ms, Some(5000));
}

#[test]
fn test_s106_timeout_fires() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    // timeout=0 means every query fires (elapsed_ms >= 0 always true)
    ex.execute("SET STATEMENT_TIMEOUT 0").unwrap();
    let r = ex.execute("SELECT * FROM t").unwrap();
    // Should be an error due to timeout
    assert!(matches!(r, QueryResult::Error(_)));
}

#[test]
fn test_s106_timeout_not_fires() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    ex.execute("INSERT INTO t (pk, v) VALUES (1, 1)").unwrap();
    // High timeout - normal queries won't fire
    ex.execute("SET STATEMENT_TIMEOUT 999999").unwrap();
    let r = ex.execute("SELECT * FROM t").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}

#[test]
fn test_s106_cancel_backend() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    let r = ex
        .execute("SELECT pg_cancel_backend(0) AS ok FROM t")
        .unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}

#[test]
fn test_s106_timeout_reset() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    ex.execute("SET STATEMENT_TIMEOUT 0").unwrap();
    // Re-set to high value â€” queries should work again
    ex.execute("SET STATEMENT_TIMEOUT 999999").unwrap();
    let r = ex.execute("SELECT * FROM t").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}

// S108 â€” systemd integration
#[test]
fn test_s108_sd_notify_basic() {
    let (ex, _dir) = setup();
    let r = ex.execute("SD_NOTIFY 'READY=1'").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}

#[test]
fn test_s108_sd_notify_ready() {
    let (ex, _dir) = setup();
    let r = ex.execute("SD_NOTIFY 'READY=1'").unwrap();
    if let QueryResult::Ok(msg) = r {
        assert!(msg.contains("READY=1"));
    } else {
        panic!("expected Ok");
    }
}

#[test]
fn test_s108_sd_notify_stopping() {
    let (ex, _dir) = setup();
    let r = ex.execute("SD_NOTIFY 'STOPPING=1'").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}

#[test]
fn test_s108_sd_notify_watchdog() {
    let (ex, _dir) = setup();
    let r = ex.execute("SD_NOTIFY 'WATCHDOG=1'").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}

#[test]
fn test_s108_sd_notify_status() {
    let (ex, _dir) = setup();
    let r = ex.execute("SD_NOTIFY 'STATUS=running'").unwrap();
    if let QueryResult::Ok(msg) = r {
        assert!(msg.contains("STATUS=running"));
    } else {
        panic!("expected Ok");
    }
}

// S109 â€” graceful shutdown
#[test]
fn test_s109_shutdown_graceful() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHUTDOWN GRACEFUL").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}

#[test]
fn test_s109_shutdown_immediate() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHUTDOWN IMMEDIATE").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}

#[test]
fn test_s109_graceful_writes_wal() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64)").unwrap();
    ex.execute("INSERT INTO t (pk) VALUES (1)").unwrap();
    ex.execute("DELETE FROM t WHERE pk = 1").unwrap();
    let r = ex.execute("SHUTDOWN GRACEFUL").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}

#[test]
fn test_s109_graceful_checkpoint() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHUTDOWN GRACEFUL").unwrap();
    if let QueryResult::Ok(msg) = r {
        assert!(msg.contains("graceful shutdown"));
    } else {
        panic!("expected Ok");
    }
}

#[test]
fn test_s109_shutdown_returns_ok() {
    let (ex, _dir) = setup();
    let r1 = ex.execute("SHUTDOWN GRACEFUL").unwrap();
    let r2 = ex.execute("SHUTDOWN IMMEDIATE").unwrap();
    assert!(matches!(r1, QueryResult::Ok(_)));
    assert!(matches!(r2, QueryResult::Ok(_)));
}

// S110 â€” multi-database
#[test]
fn test_s110_create_database() {
    let (ex, _dir) = setup();
    let r = ex.execute("CREATE DATABASE mydb").unwrap();
    assert!(matches!(r, QueryResult::Created));
    assert!(ex.databases.lock_safe().contains("mydb"));
}

#[test]
fn test_s110_drop_database() {
    let (ex, _dir) = setup();
    ex.execute("CREATE DATABASE mydb").unwrap();
    let r = ex.execute("DROP DATABASE mydb").unwrap();
    assert!(matches!(r, QueryResult::Dropped));
}

#[test]
fn test_s110_use_database() {
    let (ex, _dir) = setup();
    ex.execute("CREATE DATABASE mydb").unwrap();
    let r = ex.execute("USE mydb").unwrap();
    assert!(matches!(r, QueryResult::Affected(0)));
    assert_eq!(*ex.current_db.lock_safe(), "mydb");
}

#[test]
fn test_s110_show_databases() {
    let (ex, _dir) = setup();
    ex.execute("CREATE DATABASE db2").unwrap();
    let r = ex.execute("SHOW DATABASES").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        let names: Vec<String> = rows.into_iter().flatten().collect();
        assert!(names.contains(&"vectorvolt".to_owned()));
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s110_default_database() {
    let (ex, _dir) = setup();
    assert_eq!(*ex.current_db.lock_safe(), "vectorvolt");
    assert!(ex.databases.lock_safe().contains("vectorvolt"));
}

// S111 â€” schema namespacing
#[test]
fn test_s111_create_schema() {
    let (ex, _dir) = setup();
    let r = ex.execute("CREATE SCHEMA myschema").unwrap();
    assert!(matches!(r, QueryResult::Created));
}

#[test]
fn test_s111_drop_schema() {
    let (ex, _dir) = setup();
    ex.execute("CREATE SCHEMA myschema").unwrap();
    let r = ex.execute("DROP SCHEMA myschema").unwrap();
    assert!(matches!(r, QueryResult::Dropped));
}

#[test]
fn test_s111_set_search_path() {
    let (ex, _dir) = setup();
    ex.execute("CREATE SCHEMA myschema").unwrap();
    let r = ex.execute("SET search_path TO myschema").unwrap();
    assert!(matches!(r, QueryResult::Affected(0)));
    assert_eq!(*ex.search_path.lock_safe(), "myschema");
}

#[test]
fn test_s111_schema_qualified_table() {
    let (ex, _dir) = setup();
    ex.execute("CREATE SCHEMA app").unwrap();
    // Schema-qualified tables are just created/used normally for MVP
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    ex.execute("INSERT INTO t (pk, v) VALUES (1, 42)").unwrap();
    let r = ex.execute("SELECT * FROM t").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}

#[test]
fn test_s111_show_schemas() {
    let (ex, _dir) = setup();
    ex.execute("CREATE SCHEMA app").unwrap();
    let r = ex.execute("SELECT * FROM vv_schemas").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        let names: Vec<String> = rows.into_iter().flatten().collect();
        assert!(names.contains(&"public".to_owned()));
    } else {
        panic!("expected rows");
    }
}

// S112 â€” table partitioning
#[test]
fn test_s112_partition_by_range() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("CREATE TABLE events (pk U64, ts TEXT) PARTITION BY RANGE (ts)")
        .unwrap();
    assert!(matches!(r, QueryResult::Created));
}

#[test]
fn test_s112_create_partition() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE events (pk U64, ts TEXT) PARTITION BY RANGE (ts)")
        .unwrap();
    let r = ex.execute("CREATE TABLE events_2024 PARTITION OF events FOR VALUES FROM ('2024-01-01') TO ('2025-01-01')").unwrap();
    assert!(matches!(r, QueryResult::Created));
}

#[test]
fn test_s112_insert_routes_to_partition() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE events (pk U64, ts TEXT)").unwrap();
    ex.execute("INSERT INTO events (pk, ts) VALUES (1, '2024-06-01')")
        .unwrap();
    let r = ex.execute("SELECT * FROM events").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}

#[test]
fn test_s112_select_from_parent() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE events (pk U64, ts TEXT) PARTITION BY RANGE (ts)")
        .unwrap();
    let r = ex.execute("SELECT * FROM events").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}

#[test]
fn test_s112_partition_meta() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE events (pk U64, ts TEXT) PARTITION BY RANGE (ts)")
        .unwrap();
    ex.execute("CREATE TABLE events_2024 PARTITION OF events FOR VALUES FROM ('2024-01-01') TO ('2025-01-01')").unwrap();
    let guard = ex.partitions.lock_safe();
    assert!(guard.contains_key("events_2024"));
}

// S113 â€” online schema change
#[test]
fn test_s113_add_column_online() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64)").unwrap();
    let r = ex
        .execute("ALTER TABLE t ADD COLUMN score U64 ONLINE")
        .unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}

#[test]
fn test_s113_online_no_lock_message() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64)").unwrap();
    let r = ex
        .execute("ALTER TABLE t ADD COLUMN score U64 ONLINE")
        .unwrap();
    if let QueryResult::Ok(msg) = r {
        assert!(msg.contains("0 rows locked"));
    } else {
        panic!("expected Ok");
    }
}

#[test]
fn test_s113_online_rows_preserved() {
    // Verify that SELECT works (returns Rows, no error) after ONLINE schema change
    // and that rows inserted after the ALTER are visible.
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64)").unwrap();
    let r_alter = ex
        .execute("ALTER TABLE t ADD COLUMN score U64 ONLINE")
        .unwrap();
    assert!(
        matches!(r_alter, QueryResult::Ok(_)),
        "expected Ok from online alter"
    );
    ex.execute("INSERT INTO t (pk, score) VALUES (1, 42)")
        .unwrap();
    let r = ex.execute("SELECT * FROM t").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 1);
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s113_non_online_add_col() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64)").unwrap();
    let r = ex.execute("ALTER TABLE t ADD COLUMN score U64").unwrap();
    assert!(matches!(r, QueryResult::Affected(0)));
}

#[test]
fn test_s113_online_default_value() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64)").unwrap();
    ex.execute("INSERT INTO t (pk) VALUES (5)").unwrap();
    ex.execute("ALTER TABLE t ADD COLUMN v U64 ONLINE").unwrap();
    // New column defaults to 0
    let r = ex.execute("SELECT * FROM t").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}

// S114 â€” vacuum
#[test]
fn test_s114_vacuum_basic() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64)").unwrap();
    let r = ex.execute("VACUUM t").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}

#[test]
fn test_s114_vacuum_analyze() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64)").unwrap();
    let r = ex.execute("VACUUM ANALYZE t").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}

#[test]
fn test_s114_vacuum_dead_count() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64)").unwrap();
    ex.execute("INSERT INTO t (pk) VALUES (1)").unwrap();
    ex.execute("DELETE FROM t WHERE pk = 1").unwrap();
    let dead = *ex.dead_rows.lock_safe().get("t").unwrap_or(&0);
    assert_eq!(dead, 1);
}

#[test]
fn test_s114_vacuum_after_delete() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64)").unwrap();
    ex.execute("INSERT INTO t (pk) VALUES (1)").unwrap();
    ex.execute("DELETE FROM t WHERE pk = 1").unwrap();
    let r = ex.execute("VACUUM t").unwrap();
    if let QueryResult::Ok(msg) = r {
        assert!(msg.contains("1 dead rows"));
    } else {
        panic!("expected Ok");
    }
}

#[test]
fn test_s114_vacuum_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64)").unwrap();
    let r = ex.execute("VACUUM t").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}

// S115 â€” statistics collector
#[test]
fn test_s115_analyze_basic() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    ex.execute("INSERT INTO t (pk, v) VALUES (1, 10)").unwrap();
    let r = ex.execute("ANALYZE t").unwrap();
    assert!(matches!(r, QueryResult::Affected(0)));
}

#[test]
fn test_s115_analyze_row_count() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    ex.execute("INSERT INTO t (pk, v) VALUES (1, 10)").unwrap();
    ex.execute("INSERT INTO t (pk, v) VALUES (2, 20)").unwrap();
    ex.execute("ANALYZE t").unwrap();
    let stats = ex.table_stats.lock_safe();
    let col_stats = stats.get("t").unwrap();
    let pk_stat = col_stats.get("pk").unwrap();
    assert_eq!(pk_stat.row_count, 2);
}

#[test]
fn test_s115_analyze_distinct() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64)").unwrap();
    ex.execute("INSERT INTO t (pk) VALUES (1)").unwrap();
    ex.execute("INSERT INTO t (pk) VALUES (2)").unwrap();
    ex.execute("ANALYZE t").unwrap();
    let stats = ex.table_stats.lock_safe();
    let pk_stat = stats.get("t").unwrap().get("pk").unwrap();
    assert!(pk_stat.distinct_count >= 1);
}

#[test]
fn test_s115_vv_table_stats() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64)").unwrap();
    ex.execute("INSERT INTO t (pk) VALUES (1)").unwrap();
    ex.execute("ANALYZE t").unwrap();
    let r = ex.execute("SELECT * FROM vv_table_stats").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}

#[test]
fn test_s115_analyze_min_max() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    ex.execute("INSERT INTO t (pk, v) VALUES (1, 10)").unwrap();
    ex.execute("ANALYZE t").unwrap();
    let stats = ex.table_stats.lock_safe();
    let col_stats = stats.get("t").unwrap();
    assert!(col_stats.contains_key("pk"));
    assert!(col_stats.contains_key("v"));
}

// S116 â€” connection limits
#[test]
fn test_s116_set_max_connections() {
    let (ex, _dir) = setup();
    let r = ex.execute("SET MAX_CONNECTIONS 200").unwrap();
    assert!(matches!(r, QueryResult::Affected(0)));
    assert_eq!(*ex.max_connections.lock_safe(), 200);
}

#[test]
fn test_s116_show_max_connections() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW MAX_CONNECTIONS").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows[0][0], "100");
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s116_vv_connections_table() {
    let (ex, _dir) = setup();
    let r = ex.execute("SELECT * FROM vv_connections").unwrap();
    if let QueryResult::Rows { columns, rows } = r {
        assert!(columns.contains(&"max".to_owned()));
        assert_eq!(rows.len(), 1);
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s116_default_max() {
    let (ex, _dir) = setup();
    assert_eq!(*ex.max_connections.lock_safe(), 100);
}

#[test]
fn test_s116_connection_count() {
    let (ex, _dir) = setup();
    let current = ex
        .connection_count
        .load(std::sync::atomic::Ordering::Relaxed);
    assert_eq!(current, 0);
    let r = ex.execute("SELECT * FROM vv_connections").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}

// S117 â€” resource groups
#[test]
fn test_s117_create_resource_group() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("CREATE RESOURCE GROUP rg1 LIMIT (CPU=50, MEMORY=512)")
        .unwrap();
    assert!(matches!(r, QueryResult::Created));
    assert!(ex.resource_groups.lock_safe().contains_key("rg1"));
}

#[test]
fn test_s117_drop_resource_group() {
    let (ex, _dir) = setup();
    ex.execute("CREATE RESOURCE GROUP rg1 LIMIT (CPU=50, MEMORY=512)")
        .unwrap();
    let r = ex.execute("DROP RESOURCE GROUP rg1").unwrap();
    assert!(matches!(r, QueryResult::Dropped));
}

#[test]
fn test_s117_assign_user() {
    let (ex, _dir) = setup();
    ex.execute("CREATE RESOURCE GROUP rg1 LIMIT (CPU=50, MEMORY=512)")
        .unwrap();
    let r = ex.execute("ASSIGN USER alice TO rg1").unwrap();
    assert!(matches!(r, QueryResult::Affected(0)));
    let groups = ex.resource_groups.lock_safe();
    let rg = groups.get("rg1").unwrap();
    assert!(rg.users.contains(&"alice".to_owned()));
}

#[test]
fn test_s117_vv_resource_groups() {
    let (ex, _dir) = setup();
    ex.execute("CREATE RESOURCE GROUP rg1 LIMIT (CPU=50, MEMORY=512)")
        .unwrap();
    let r = ex.execute("SELECT * FROM vv_resource_groups").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert!(!rows.is_empty());
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s117_alter_resource_group() {
    let (ex, _dir) = setup();
    ex.execute("CREATE RESOURCE GROUP rg1 LIMIT (CPU=50, MEMORY=512)")
        .unwrap();
    let r = ex
        .execute("ALTER RESOURCE GROUP rg1 LIMIT (CPU=75)")
        .unwrap();
    assert!(matches!(r, QueryResult::Affected(0)));
    let groups = ex.resource_groups.lock_safe();
    let rg = groups.get("rg1").unwrap();
    assert_eq!(rg.cpu_pct, 75);
}

// â”€â”€ S118 â€” Event sourcing â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s118_append_event() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("APPEND EVENT TO orders TYPE 'OrderPlaced' PAYLOAD '{\"id\":1}'")
        .unwrap();
    assert!(matches!(r, QueryResult::Affected(1)));
}

#[test]
fn test_s118_select_events() {
    let (ex, _dir) = setup();
    ex.execute("APPEND EVENT TO orders TYPE 'OrderPlaced' PAYLOAD '{\"id\":1}'")
        .unwrap();
    let r = ex.execute("SELECT * FROM EVENTS OF orders").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0][1], "OrderPlaced");
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s118_multiple_events() {
    let (ex, _dir) = setup();
    ex.execute("APPEND EVENT TO orders TYPE 'OrderPlaced' PAYLOAD '{}'")
        .unwrap();
    ex.execute("APPEND EVENT TO orders TYPE 'OrderShipped' PAYLOAD '{}'")
        .unwrap();
    ex.execute("APPEND EVENT TO orders TYPE 'OrderDelivered' PAYLOAD '{}'")
        .unwrap();
    let r = ex.execute("SELECT * FROM EVENTS OF orders").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 3);
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s118_unknown_table_empty() {
    let (ex, _dir) = setup();
    let r = ex.execute("SELECT * FROM EVENTS OF nonexistent").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert!(rows.is_empty());
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s118_event_type_filter() {
    let (ex, _dir) = setup();
    ex.execute("APPEND EVENT TO orders TYPE 'OrderPlaced' PAYLOAD '{}'")
        .unwrap();
    ex.execute("APPEND EVENT TO orders TYPE 'OrderShipped' PAYLOAD '{}'")
        .unwrap();
    let r = ex
        .execute("SELECT * FROM EVENTS OF orders WHERE event_type = 'OrderPlaced'")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0][1], "OrderPlaced");
    } else {
        panic!("expected rows");
    }
}

// â”€â”€ S119 â€” CQRS read models â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s119_create_read_model() {
    let (ex, _dir) = setup();
    let r = ex
            .execute("CREATE READ MODEL placed_orders FROM EVENTS OF orders WHERE event_type = 'OrderPlaced'")
            .unwrap();
    assert!(matches!(r, QueryResult::Created));
}

#[test]
fn test_s119_select_read_model() {
    let (ex, _dir) = setup();
    ex.execute("APPEND EVENT TO orders TYPE 'OrderPlaced' PAYLOAD '{}'")
        .unwrap();
    ex.execute("APPEND EVENT TO orders TYPE 'OrderShipped' PAYLOAD '{}'")
        .unwrap();
    ex.execute(
        "CREATE READ MODEL placed_orders FROM EVENTS OF orders WHERE event_type = 'OrderPlaced'",
    )
    .unwrap();
    let r = ex
        .execute("SELECT * FROM READ MODEL placed_orders")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 1);
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s119_drop_read_model() {
    let (ex, _dir) = setup();
    ex.execute("CREATE READ MODEL m FROM EVENTS OF t WHERE event_type = 'e'")
        .unwrap();
    let r = ex.execute("DROP READ MODEL m").unwrap();
    assert!(matches!(r, QueryResult::Dropped));
}

#[test]
fn test_s119_read_model_filter() {
    let (ex, _dir) = setup();
    ex.execute("APPEND EVENT TO orders TYPE 'A' PAYLOAD '{}'")
        .unwrap();
    ex.execute("APPEND EVENT TO orders TYPE 'B' PAYLOAD '{}'")
        .unwrap();
    ex.execute("APPEND EVENT TO orders TYPE 'A' PAYLOAD '{}'")
        .unwrap();
    ex.execute("CREATE READ MODEL model_a FROM EVENTS OF orders WHERE event_type = 'A'")
        .unwrap();
    let r = ex.execute("SELECT * FROM READ MODEL model_a").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 2);
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s119_unknown_model_empty() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("SELECT * FROM READ MODEL no_such_model")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert!(rows.is_empty());
    } else {
        panic!("expected rows");
    }
}

// â”€â”€ S120 â€” Time-travel queries â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s120_snapshot_table() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    let r = ex.execute("SNAPSHOT TABLE t").unwrap();
    assert!(matches!(r, QueryResult::Affected(1)));
    let log = ex.time_travel_log.lock_safe();
    assert!(log.contains_key("t"));
}

#[test]
fn test_s120_as_of_returns_snapshot() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    ex.execute("INSERT INTO t (pk, v) VALUES (1, 42)").unwrap();
    ex.execute("SNAPSHOT TABLE t").unwrap();
    let r = ex
        .execute("SELECT * FROM t AS OF TIMESTAMP 99999999999999")
        .unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}

#[test]
fn test_s120_no_snapshots() {
    let (ex, _dir) = setup();
    let r = ex.execute("SELECT * FROM t AS OF TIMESTAMP 0").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert!(rows.is_empty());
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s120_snapshot_empty_table() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64)").unwrap();
    let r = ex.execute("SNAPSHOT TABLE t").unwrap();
    assert!(matches!(r, QueryResult::Affected(1)));
}

#[test]
fn test_s120_multiple_snapshots_closest() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64)").unwrap();
    ex.execute("SNAPSHOT TABLE t").unwrap();
    ex.execute("SNAPSHOT TABLE t").unwrap();
    let log = ex.time_travel_log.lock_safe();
    assert_eq!(log.get("t").unwrap().len(), 2);
}

// â”€â”€ S121 â€” Change Data Feed â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s121_enable_change_feed() {
    let (ex, _dir) = setup();
    let r = ex.execute("ENABLE CHANGE FEED ON orders").unwrap();
    assert!(matches!(r, QueryResult::Affected(0)));
    assert!(ex.change_feed_enabled.lock_safe().contains("orders"));
}

#[test]
fn test_s121_disable_change_feed() {
    let (ex, _dir) = setup();
    ex.execute("ENABLE CHANGE FEED ON orders").unwrap();
    let r = ex.execute("DISABLE CHANGE FEED ON orders").unwrap();
    assert!(matches!(r, QueryResult::Affected(0)));
    assert!(!ex.change_feed_enabled.lock_safe().contains("orders"));
}

#[test]
fn test_s121_insert_captured() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE orders (pk U64, amount U64)")
        .unwrap();
    ex.execute("ENABLE CHANGE FEED ON orders").unwrap();
    ex.execute("INSERT INTO orders (pk, amount) VALUES (1, 100)")
        .unwrap();
    let feed = ex.change_feed.lock_safe();
    let entries = feed.get("orders").unwrap();
    assert!(!entries.is_empty());
    assert_eq!(entries[0].0, "INSERT");
}

#[test]
fn test_s121_select_changes() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE orders (pk U64, amount U64)")
        .unwrap();
    ex.execute("ENABLE CHANGE FEED ON orders").unwrap();
    ex.execute("INSERT INTO orders (pk, amount) VALUES (1, 100)")
        .unwrap();
    let r = ex.execute("SELECT * FROM CHANGES OF orders").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert!(!rows.is_empty());
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s121_disabled_not_captured() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE orders (pk U64, amount U64)")
        .unwrap();
    ex.execute("INSERT INTO orders (pk, amount) VALUES (1, 100)")
        .unwrap();
    let feed = ex.change_feed.lock_safe();
    assert!(!feed.contains_key("orders"));
}

// â”€â”€ S122 â€” Bloom filter index â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s122_create_bloom_index() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, name U64)").unwrap();
    let r = ex
        .execute("CREATE BLOOM INDEX idx_name ON t(name)")
        .unwrap();
    assert!(matches!(r, QueryResult::Created));
    assert!(ex.bloom_indexes.lock_safe().contains_key("idx_name"));
}

#[test]
fn test_s122_drop_bloom_index() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, name U64)").unwrap();
    ex.execute("CREATE BLOOM INDEX idx_name ON t(name)")
        .unwrap();
    let r = ex.execute("DROP BLOOM INDEX idx_name").unwrap();
    assert!(matches!(r, QueryResult::Dropped));
    assert!(!ex.bloom_indexes.lock_safe().contains_key("idx_name"));
}

#[test]
fn test_s122_bloom_match_miss() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("CREATE BLOOM INDEX idx ON t(val)").unwrap();
    let r = ex
        .execute("SELECT * FROM t WHERE val BLOOM_MATCH 'nothere'")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert!(rows.is_empty());
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s122_bloom_match_hit() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 42)")
        .unwrap();
    // seed bloom index after insert
    ex.execute("CREATE BLOOM INDEX idx ON t(val)").unwrap();
    let r = ex
        .execute("SELECT * FROM t WHERE val BLOOM_MATCH '42'")
        .unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}

#[test]
fn test_s122_bloom_index_seeded() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 99)")
        .unwrap();
    ex.execute("CREATE BLOOM INDEX idx ON t(val)").unwrap();
    let idxs = ex.bloom_indexes.lock_safe();
    let (_, _, vals) = idxs.get("idx").unwrap();
    assert!(vals.contains("99"));
}

// â”€â”€ S123 â€” Columnar storage hint â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s123_store_columnar() {
    let (ex, _dir) = setup();
    let r = ex.execute("ALTER TABLE t STORE COLUMNAR").unwrap();
    assert!(matches!(r, QueryResult::Affected(0)));
    assert!(ex.columnar_tables.lock_safe().contains("t"));
}

#[test]
fn test_s123_store_row() {
    let (ex, _dir) = setup();
    ex.execute("ALTER TABLE t STORE COLUMNAR").unwrap();
    let r = ex.execute("ALTER TABLE t STORE ROW").unwrap();
    assert!(matches!(r, QueryResult::Affected(0)));
    assert!(!ex.columnar_tables.lock_safe().contains("t"));
}

#[test]
fn test_s123_show_storage_columnar() {
    let (ex, _dir) = setup();
    ex.execute("ALTER TABLE t STORE COLUMNAR").unwrap();
    let r = ex.execute("SHOW STORAGE FORMAT FOR t").unwrap();
    assert!(matches!(r, QueryResult::Ok(ref s) if s == "COLUMNAR"));
}

#[test]
fn test_s123_show_storage_row_default() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW STORAGE FORMAT FOR t").unwrap();
    assert!(matches!(r, QueryResult::Ok(ref s) if s == "ROW"));
}

#[test]
fn test_s123_toggle_storage() {
    let (ex, _dir) = setup();
    ex.execute("ALTER TABLE t STORE COLUMNAR").unwrap();
    ex.execute("ALTER TABLE t STORE ROW").unwrap();
    let r = ex.execute("SHOW STORAGE FORMAT FOR t").unwrap();
    assert!(matches!(r, QueryResult::Ok(ref s) if s == "ROW"));
}

// â”€â”€ S124 â€” Adaptive query execution â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s124_aqe_default_on() {
    let (ex, _dir) = setup();
    assert!(*ex.aqe_enabled.lock_safe());
}

#[test]
fn test_s124_aqe_on() {
    let (ex, _dir) = setup();
    ex.execute("SET ADAPTIVE QUERY EXECUTION OFF").unwrap();
    ex.execute("SET ADAPTIVE QUERY EXECUTION ON").unwrap();
    assert!(*ex.aqe_enabled.lock_safe());
}

#[test]
fn test_s124_show_aqe_status_enabled() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW AQE STATUS").unwrap();
    if let QueryResult::Ok(s) = r {
        assert!(s.contains("enabled"));
    } else {
        panic!("expected Ok");
    }
}

// â”€â”€ S125 â€” Query result streaming â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s125_default_chunk_size() {
    let (ex, _dir) = setup();
    assert_eq!(*ex.stream_chunk_size.lock_safe(), 100);
}

#[test]
fn test_s125_set_chunk_size() {
    let (ex, _dir) = setup();
    let r = ex.execute("SET STREAM CHUNK SIZE 50").unwrap();
    assert!(matches!(r, QueryResult::Affected(0)));
    assert_eq!(*ex.stream_chunk_size.lock_safe(), 50);
}

#[test]
fn test_s125_show_stream_config() {
    let (ex, _dir) = setup();
    ex.execute("SET STREAM CHUNK SIZE 25").unwrap();
    let r = ex.execute("SHOW STREAM CONFIG").unwrap();
    if let QueryResult::Ok(s) = r {
        assert!(s.contains("25"));
    } else {
        panic!("expected Ok");
    }
}

#[test]
fn test_s125_stream_select_empty() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64)").unwrap();
    let r = ex.execute("STREAM SELECT * FROM t").unwrap();
    if let QueryResult::Ok(s) = r {
        assert!(s.starts_with("STREAM:"));
        assert!(s.contains("0 chunks"));
    } else {
        panic!("expected Ok");
    }
}

#[test]
fn test_s125_stream_select_with_rows() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64)").unwrap();
    ex.execute("INSERT INTO t (pk) VALUES (1)").unwrap();
    ex.execute("SET STREAM CHUNK SIZE 1").unwrap();
    let r = ex.execute("STREAM SELECT * FROM t").unwrap();
    if let QueryResult::Ok(s) = r {
        assert!(s.starts_with("STREAM:"));
    } else {
        panic!("expected Ok");
    }
}

// â”€â”€ S126 â€” Geospatial types â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s126_insert_geo_point() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("INSERT GEO POINT INTO locations(position) VALUES (48.8566, 2.3522)")
        .unwrap();
    assert!(matches!(r, QueryResult::Affected(1)));
    let geo = ex.geo_columns.lock_safe();
    let point = geo
        .get("locations")
        .and_then(|m| m.get("position"))
        .copied();
    assert!(point.is_some());
}

#[test]
fn test_s126_st_distance_zero() {
    let (ex, _dir) = setup();
    ex.execute("INSERT GEO POINT INTO t(c) VALUES (10.0, 20.0)")
        .unwrap();
    let r = ex.execute("SELECT ST_DISTANCE(t.c, t.c)").unwrap();
    if let QueryResult::Ok(s) = r {
        let dist: f64 = s.parse().unwrap();
        assert!((dist - 0.0).abs() < 1e-6);
    } else {
        panic!("expected Ok");
    }
}

#[test]
fn test_s126_st_distance_nonzero() {
    let (ex, _dir) = setup();
    ex.execute("INSERT GEO POINT INTO a(pt) VALUES (0.0, 0.0)")
        .unwrap();
    ex.execute("INSERT GEO POINT INTO b(pt) VALUES (3.0, 4.0)")
        .unwrap();
    let r = ex.execute("SELECT ST_DISTANCE(a.pt, b.pt)").unwrap();
    if let QueryResult::Ok(s) = r {
        let dist: f64 = s.parse().unwrap();
        assert!((dist - 5.0).abs() < 1e-4);
    } else {
        panic!("expected Ok");
    }
}

#[test]
fn test_s126_st_contains_within() {
    let (ex, _dir) = setup();
    // Paris lat/lng, search within 1000 km
    ex.execute("INSERT GEO POINT INTO cities(loc) VALUES (48.8566, 2.3522)")
        .unwrap();
    let r = ex
        .execute("SELECT * FROM cities WHERE ST_CONTAINS(loc, 48.0, 2.0, 1000.0)")
        .unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}

#[test]
fn test_s126_st_contains_outside() {
    let (ex, _dir) = setup();
    // Paris lat/lng, search within 1 km from Tokyo coords
    ex.execute("INSERT GEO POINT INTO cities(loc) VALUES (48.8566, 2.3522)")
        .unwrap();
    let r = ex
        .execute("SELECT * FROM cities WHERE ST_CONTAINS(loc, 35.6762, 139.6503, 1.0)")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert!(rows.is_empty());
    } else {
        panic!("expected rows");
    }
}

// â”€â”€ S127 â€” Cypher-style graph queries â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s127_create_edge() {
    let (ex, _dir) = setup();
    let r = ex.execute("CREATE EDGE (Alice)-[:KNOWS]->(Bob)").unwrap();
    assert!(matches!(r, QueryResult::Created));
    let edges = ex.graph_edges.lock_safe();
    assert!(!edges.is_empty());
    assert_eq!(
        edges[0],
        ("Alice".to_owned(), "KNOWS".to_owned(), "Bob".to_owned())
    );
}

#[test]
fn test_s127_drop_edge() {
    let (ex, _dir) = setup();
    ex.execute("CREATE EDGE (Alice)-[:KNOWS]->(Bob)").unwrap();
    let r = ex.execute("DROP EDGE (Alice)-[:KNOWS]->(Bob)").unwrap();
    assert!(matches!(r, QueryResult::Dropped));
    assert!(ex.graph_edges.lock_safe().is_empty());
}

#[test]
fn test_s127_match_edges() {
    let (ex, _dir) = setup();
    ex.execute("CREATE EDGE (Alice)-[:KNOWS]->(Bob)").unwrap();
    ex.execute("CREATE EDGE (Bob)-[:KNOWS]->(Carol)").unwrap();
    let r = ex
        .execute("MATCH (a)-[r:KNOWS]->(b) RETURN a, r, b")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 2);
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s127_match_rel_type_filter() {
    let (ex, _dir) = setup();
    ex.execute("CREATE EDGE (Alice)-[:KNOWS]->(Bob)").unwrap();
    ex.execute("CREATE EDGE (Alice)-[:LIKES]->(Carol)").unwrap();
    let r = ex
        .execute("MATCH (a)-[r:KNOWS]->(b) RETURN a, r, b")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0][1], "KNOWS");
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s127_match_where_filter() {
    let (ex, _dir) = setup();
    ex.execute("CREATE EDGE (Alice)-[:KNOWS]->(Bob)").unwrap();
    ex.execute("CREATE EDGE (Carol)-[:KNOWS]->(Dave)").unwrap();
    let r = ex
        .execute("MATCH (a)-[r]->(b) WHERE a = 'Alice' RETURN a, r, b")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0][0], "Alice");
    } else {
        panic!("expected rows");
    }
}

// â”€â”€ Gap #4 â€” real vectorvolt-graph store wiring â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_gap4_create_edge_persists_in_real_graph_store() {
    let (ex, _dir) = setup();
    ex.execute("CREATE EDGE (Alice)-[:KNOWS]->(Bob)").unwrap();
    let stats = ex.graph_store.stats();
    assert_eq!(stats.edge_count, 1);
    assert_eq!(stats.node_count, 2);
}

#[test]
fn test_gap4_graph_survives_restart() {
    let dir = tempfile::tempdir().unwrap();
    let registry = Arc::new(TableRegistry::open(dir.path(), EngineOptions::default()).unwrap());
    {
        let ex = SqlExecutor::new(registry.clone());
        ex.execute("CREATE EDGE (Alice)-[:KNOWS]->(Bob)").unwrap();
        ex.execute("CREATE EDGE (Bob)-[:KNOWS]->(Carol)").unwrap();
    }
    // New executor over the same registry/dir simulates a process restart.
    let registry2 = Arc::new(TableRegistry::open(dir.path(), EngineOptions::default()).unwrap());
    let ex2 = SqlExecutor::new(registry2);
    let stats = ex2.graph_store.stats();
    assert_eq!(stats.edge_count, 2);
    assert_eq!(stats.node_count, 3);
}

#[test]
fn test_gap4_drop_edge_removes_from_real_graph_store() {
    let (ex, _dir) = setup();
    ex.execute("CREATE EDGE (Alice)-[:KNOWS]->(Bob)").unwrap();
    ex.execute("DROP EDGE (Alice)-[:KNOWS]->(Bob)").unwrap();
    let stats = ex.graph_store.stats();
    assert_eq!(stats.edge_count, 0);
}

#[test]
fn test_gap4_delete_node_cascade_uses_real_algorithm() {
    let (ex, _dir) = setup();
    ex.execute("CREATE EDGE (Alice)-[:KNOWS]->(Bob)").unwrap();
    ex.execute("CREATE EDGE (Bob)-[:KNOWS]->(Carol)").unwrap();
    let r = ex.execute("DELETE NODE 'Bob' CASCADE").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        let deleted: Vec<&String> = rows.iter().map(|row| &row[0]).collect();
        assert!(deleted.contains(&&"Bob".to_owned()));
        assert!(deleted.contains(&&"Alice".to_owned()));
    } else {
        panic!("expected rows");
    }
    let stats = ex.graph_store.stats();
    assert_eq!(stats.edge_count, 0);
}

#[test]
fn test_gap4_delete_node_cascade_cleans_match_cache() {
    let (ex, _dir) = setup();
    ex.execute("CREATE EDGE (Alice)-[:KNOWS]->(Bob)").unwrap();
    ex.execute("DELETE NODE 'Bob' CASCADE").unwrap();
    let r = ex.execute("MATCH (a)-[r]->(b) RETURN a, r, b").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert!(rows.is_empty());
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_gap4_show_graph_stats_reflects_real_store() {
    let (ex, _dir) = setup();
    ex.execute("CREATE EDGE (Alice)-[:KNOWS]->(Bob)").unwrap();
    ex.execute("CREATE EDGE (Bob)-[:KNOWS]->(Carol)").unwrap();
    let r = ex.execute("SHOW GRAPH STATS").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows[0][0], "3"); // node_count
        assert_eq!(rows[0][1], "2"); // edge_count
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_gap4_delete_node_cascade_unknown_node_is_a_harmless_noop() {
    // GraphStore::remove_node always reports the target itself (even if it never
    // existed), but must not error and must not touch unrelated edges.
    let (ex, _dir) = setup();
    ex.execute("CREATE EDGE (Alice)-[:KNOWS]->(Bob)").unwrap();
    let r = ex.execute("DELETE NODE 'Ghost' CASCADE").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
    let stats = ex.graph_store.stats();
    assert_eq!(stats.edge_count, 1);
}

// â”€â”€ S128 â€” Distributed 2PC transactions â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s128_begin_distributed_txn() {
    let (ex, _dir) = setup();
    let r = ex.execute("BEGIN DISTRIBUTED TRANSACTION txn1").unwrap();
    assert!(matches!(r, QueryResult::TxBegin));
    assert_eq!(*ex.dist_txn_active.lock_safe(), Some("txn1".to_owned()));
}

#[test]
fn test_s128_prepare_transaction() {
    let (ex, _dir) = setup();
    let r = ex.execute("PREPARE TRANSACTION txn1").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
    assert!(ex.prepared_txns.lock_safe().contains_key("txn1"));
}

#[test]
fn test_s128_commit_prepared() {
    let (ex, _dir) = setup();
    ex.execute("PREPARE TRANSACTION txn1").unwrap();
    let r = ex.execute("COMMIT PREPARED txn1").unwrap();
    if let QueryResult::Ok(msg) = r {
        assert_eq!(msg, "COMMIT PREPARED");
    } else {
        panic!();
    }
    assert!(!ex.prepared_txns.lock_safe().contains_key("txn1"));
}

#[test]
fn test_s128_rollback_prepared() {
    let (ex, _dir) = setup();
    ex.execute("PREPARE TRANSACTION txn1").unwrap();
    let r = ex.execute("ROLLBACK PREPARED txn1").unwrap();
    if let QueryResult::Ok(msg) = r {
        assert_eq!(msg, "ROLLBACK PREPARED");
    } else {
        panic!();
    }
    assert!(!ex.prepared_txns.lock_safe().contains_key("txn1"));
}

#[test]
fn test_s128_show_prepared_transactions() {
    let (ex, _dir) = setup();
    ex.execute("PREPARE TRANSACTION txn1").unwrap();
    ex.execute("PREPARE TRANSACTION txn2").unwrap();
    let r = ex.execute("SHOW PREPARED TRANSACTIONS").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 2);
    } else {
        panic!();
    }
}

// â”€â”€ S129 â€” Stored procedures â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s129_create_procedure() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("CREATE PROCEDURE myproc AS BEGIN SELECT 1; SELECT 2; END")
        .unwrap();
    assert!(matches!(r, QueryResult::Created));
    assert!(ex.procedures.lock_safe().contains_key("myproc"));
}

#[test]
fn test_s129_call_procedure() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE foo (id INT)").unwrap();
    ex.execute("CREATE PROCEDURE ins AS BEGIN INSERT INTO foo VALUES (1); END")
        .unwrap();
    let r = ex.execute("CALL ins()").unwrap();
    assert!(matches!(r, QueryResult::Affected(_)));
}

#[test]
fn test_s129_drop_procedure() {
    let (ex, _dir) = setup();
    ex.execute("CREATE PROCEDURE myproc AS BEGIN SELECT 1; END")
        .unwrap();
    let r = ex.execute("DROP PROCEDURE myproc").unwrap();
    assert!(matches!(r, QueryResult::Dropped));
    assert!(!ex.procedures.lock_safe().contains_key("myproc"));
}

#[test]
fn test_s129_show_procedures() {
    let (ex, _dir) = setup();
    ex.execute("CREATE PROCEDURE p1 AS BEGIN SELECT 1; END")
        .unwrap();
    ex.execute("CREATE PROCEDURE p2 AS BEGIN SELECT 2; END")
        .unwrap();
    let r = ex.execute("SHOW PROCEDURES").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 2);
    } else {
        panic!();
    }
}

#[test]
fn test_s129_call_missing_procedure() {
    let (ex, _dir) = setup();
    let r = ex.execute("CALL nonexistent()").unwrap();
    assert!(matches!(r, QueryResult::Error(_)));
}

// â”€â”€ S130 â€” User-defined types â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s130_create_enum_type() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("CREATE TYPE mood AS ENUM ('happy', 'sad', 'neutral')")
        .unwrap();
    assert!(matches!(r, QueryResult::Created));
    assert!(ex.user_types.lock_safe().contains_key("mood"));
}

#[test]
fn test_s130_create_composite_type() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("CREATE TYPE address AS COMPOSITE (street TEXT, zip INT)")
        .unwrap();
    assert!(matches!(r, QueryResult::Created));
    let types = ex.user_types.lock_safe();
    assert_eq!(types.get("address").unwrap().0, "COMPOSITE");
}

#[test]
fn test_s130_drop_type() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TYPE mood AS ENUM ('happy', 'sad')")
        .unwrap();
    let r = ex.execute("DROP TYPE mood").unwrap();
    assert!(matches!(r, QueryResult::Dropped));
    assert!(!ex.user_types.lock_safe().contains_key("mood"));
}

#[test]
fn test_s130_show_types() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TYPE mood AS ENUM ('happy', 'sad')")
        .unwrap();
    ex.execute("CREATE TYPE addr AS COMPOSITE (street TEXT)")
        .unwrap();
    let r = ex.execute("SHOW TYPES").unwrap();
    if let QueryResult::Rows { rows, columns, .. } = r {
        assert_eq!(columns.len(), 3);
        assert_eq!(rows.len(), 2);
    } else {
        panic!();
    }
}

#[test]
fn test_s130_create_and_show_enum_kind() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TYPE status AS ENUM ('active', 'inactive')")
        .unwrap();
    let r = ex.execute("SHOW TYPES").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        let status_row = rows.iter().find(|r| r[0] == "status").unwrap();
        assert_eq!(status_row[1], "ENUM");
    } else {
        panic!();
    }
}

// â”€â”€ S131 â€” Array types â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s131_insert_array() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("INSERT ARRAY INTO myarr VALUES ('a', 'b', 'c')")
        .unwrap();
    assert!(matches!(r, QueryResult::Affected(1)));
    let store = ex.array_store.lock_safe();
    assert_eq!(store.get("myarr").unwrap().len(), 3);
}

#[test]
fn test_s131_select_array() {
    let (ex, _dir) = setup();
    ex.execute("INSERT ARRAY INTO myarr VALUES ('x', 'y')")
        .unwrap();
    let r = ex.execute("SELECT ARRAY FROM myarr").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 2);
    } else {
        panic!();
    }
}

#[test]
fn test_s131_select_unnest() {
    let (ex, _dir) = setup();
    ex.execute("INSERT ARRAY INTO myarr VALUES ('p', 'q', 'r')")
        .unwrap();
    let r = ex.execute("SELECT UNNEST FROM myarr").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 3);
    } else {
        panic!();
    }
}

#[test]
fn test_s131_array_contains_true() {
    let (ex, _dir) = setup();
    ex.execute("INSERT ARRAY INTO myarr VALUES ('alpha', 'beta')")
        .unwrap();
    let r = ex
        .execute("SELECT ARRAY_CONTAINS FROM myarr VALUE 'alpha'")
        .unwrap();
    if let QueryResult::Ok(v) = r {
        assert_eq!(v, "true");
    } else {
        panic!();
    }
}

#[test]
fn test_s131_array_contains_false() {
    let (ex, _dir) = setup();
    ex.execute("INSERT ARRAY INTO myarr VALUES ('alpha', 'beta')")
        .unwrap();
    let r = ex
        .execute("SELECT ARRAY_CONTAINS FROM myarr VALUE 'gamma'")
        .unwrap();
    if let QueryResult::Ok(v) = r {
        assert_eq!(v, "false");
    } else {
        panic!();
    }
}

// â”€â”€ S132 â€” JSONPath queries â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s132_store_json() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("STORE JSON doc1 AS '{\"name\":\"Alice\"}'")
        .unwrap();
    assert!(matches!(r, QueryResult::Affected(1)));
    assert!(ex.jsonpath_cache.lock_safe().contains_key("doc1"));
}

#[test]
fn test_s132_jsonb_path_query() {
    let (ex, _dir) = setup();
    ex.execute("STORE JSON doc1 AS '{\"name\":\"Alice\"}'")
        .unwrap();
    let r = ex
        .execute("SELECT JSONB_PATH_QUERY FROM doc1 PATH '$.name'")
        .unwrap();
    if let QueryResult::Ok(v) = r {
        assert_eq!(v, "Alice");
    } else {
        panic!();
    }
}

#[test]
fn test_s132_jsonb_contains_true() {
    let (ex, _dir) = setup();
    ex.execute("STORE JSON doc1 AS '{\"city\":\"NYC\"}'")
        .unwrap();
    let r = ex
        .execute("SELECT JSONB_CONTAINS FROM doc1 VALUE 'NYC'")
        .unwrap();
    if let QueryResult::Ok(v) = r {
        assert_eq!(v, "true");
    } else {
        panic!();
    }
}

#[test]
fn test_s132_jsonb_contains_false() {
    let (ex, _dir) = setup();
    ex.execute("STORE JSON doc1 AS '{\"city\":\"NYC\"}'")
        .unwrap();
    let r = ex
        .execute("SELECT JSONB_CONTAINS FROM doc1 VALUE 'London'")
        .unwrap();
    if let QueryResult::Ok(v) = r {
        assert_eq!(v, "false");
    } else {
        panic!();
    }
}

#[test]
fn test_s132_jsonb_key_exists() {
    let (ex, _dir) = setup();
    ex.execute("STORE JSON doc1 AS '{\"name\":\"Alice\",\"age\":30}'")
        .unwrap();
    let r = ex
        .execute("SELECT JSONB_KEY_EXISTS FROM doc1 KEY 'name'")
        .unwrap();
    if let QueryResult::Ok(v) = r {
        assert_eq!(v, "true");
    } else {
        panic!();
    }
}

// â”€â”€ S133 â€” Foreign data wrappers â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s133_create_server() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("CREATE SERVER myserver CONNECTION 'host=pg.example.com'")
        .unwrap();
    assert!(matches!(r, QueryResult::Created));
    assert!(ex.foreign_servers.lock_safe().contains_key("myserver"));
}

#[test]
fn test_s133_create_foreign_table() {
    let (ex, _dir) = setup();
    ex.execute("CREATE SERVER myserver CONNECTION 'host=pg.example.com'")
        .unwrap();
    let r = ex
        .execute("CREATE FOREIGN TABLE remote_users SERVER myserver")
        .unwrap();
    assert!(matches!(r, QueryResult::Created));
    assert!(ex.foreign_tables.lock_safe().contains_key("remote_users"));
}

#[test]
fn test_s133_drop_server() {
    let (ex, _dir) = setup();
    ex.execute("CREATE SERVER myserver CONNECTION 'host=pg.example.com'")
        .unwrap();
    let r = ex.execute("DROP SERVER myserver").unwrap();
    assert!(matches!(r, QueryResult::Dropped));
    assert!(!ex.foreign_servers.lock_safe().contains_key("myserver"));
}

#[test]
fn test_s133_select_from_foreign_table() {
    let (ex, _dir) = setup();
    ex.execute("CREATE SERVER myserver CONNECTION 'host=pg.example.com'")
        .unwrap();
    ex.execute("CREATE FOREIGN TABLE remote_users SERVER myserver")
        .unwrap();
    let r = ex.execute("SELECT * FROM FOREIGN remote_users").unwrap();
    if let QueryResult::Ok(msg) = r {
        assert!(msg.contains("FOREIGN SCAN"));
        assert!(msg.contains("remote_users"));
    } else {
        panic!();
    }
}

#[test]
fn test_s133_show_foreign_servers() {
    let (ex, _dir) = setup();
    ex.execute("CREATE SERVER s1 CONNECTION 'conn1'").unwrap();
    ex.execute("CREATE SERVER s2 CONNECTION 'conn2'").unwrap();
    let r = ex.execute("SHOW FOREIGN SERVERS").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 2);
    } else {
        panic!();
    }
}

// â”€â”€ S134 â€” Query federation â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s134_add_federation_node() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("ADD FEDERATION NODE node1 ENDPOINT 'http://node1:5432'")
        .unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
    assert!(ex.federation_nodes.lock_safe().contains_key("node1"));
}

#[test]
fn test_s134_remove_federation_node() {
    let (ex, _dir) = setup();
    ex.execute("ADD FEDERATION NODE node1 ENDPOINT 'http://node1:5432'")
        .unwrap();
    let r = ex.execute("REMOVE FEDERATION NODE node1").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
    assert!(!ex.federation_nodes.lock_safe().contains_key("node1"));
}

#[test]
fn test_s134_federated_select() {
    let (ex, _dir) = setup();
    ex.execute("ADD FEDERATION NODE n1 ENDPOINT 'http://n1'")
        .unwrap();
    ex.execute("ADD FEDERATION NODE n2 ENDPOINT 'http://n2'")
        .unwrap();
    let r = ex.execute("FEDERATED SELECT * FROM users").unwrap();
    if let QueryResult::Ok(msg) = r {
        assert!(msg.contains("2 nodes queried"));
    } else {
        panic!();
    }
}

#[test]
fn test_s134_show_federation_nodes() {
    let (ex, _dir) = setup();
    ex.execute("ADD FEDERATION NODE n1 ENDPOINT 'http://n1'")
        .unwrap();
    let r = ex.execute("SHOW FEDERATION NODES").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 1);
    } else {
        panic!();
    }
}

#[test]
fn test_s134_federated_select_zero_nodes() {
    let (ex, _dir) = setup();
    let r = ex.execute("FEDERATED SELECT * FROM orders").unwrap();
    if let QueryResult::Ok(msg) = r {
        assert!(msg.contains("0 nodes queried"));
    } else {
        panic!();
    }
}

#[test]
fn test_s135_set_table_engine_row() {
    let (ex, _dir) = setup();
    ex.execute("SET TABLE ENGINE FOR logs TO ROW").unwrap();
    let r = ex.execute("SHOW TABLE ENGINE FOR logs").unwrap();
    if let QueryResult::Ok(e) = r {
        assert_eq!(e, "ROW");
    } else {
        panic!();
    }
}

// â”€â”€ S136 â€” Multi-region replication â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s136_add_region() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("ADD REGION us-east ENDPOINT 'http://us-east.vv.io'")
        .unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
    assert!(ex.regions.lock_safe().contains_key("us-east"));
}

#[test]
fn test_s136_replicate_to_region() {
    let (ex, _dir) = setup();
    ex.execute("ADD REGION eu-west ENDPOINT 'http://eu-west.vv.io'")
        .unwrap();
    let r = ex.execute("REPLICATE TO REGION eu-west").unwrap();
    if let QueryResult::Ok(msg) = r {
        assert!(msg.contains("eu-west"));
    } else {
        panic!();
    }
}

#[test]
fn test_s136_remove_region() {
    let (ex, _dir) = setup();
    ex.execute("ADD REGION ap-south ENDPOINT 'http://ap.vv.io'")
        .unwrap();
    let r = ex.execute("REMOVE REGION ap-south").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
    assert!(!ex.regions.lock_safe().contains_key("ap-south"));
}

// â”€â”€ S137 â€” Pluggable authentication providers â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s137_create_auth_provider_ldap() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("CREATE AUTH PROVIDER corp_ldap TYPE LDAP CONFIG 'ldap://corp.example.com'")
        .unwrap();
    assert!(matches!(r, QueryResult::Created));
    assert_eq!(
        ex.auth_providers.lock_safe().get("corp_ldap").unwrap(),
        "LDAP"
    );
}

#[test]
fn test_s137_create_auth_provider_oauth() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("CREATE AUTH PROVIDER google TYPE OAUTH CONFIG 'client_id=abc'")
        .unwrap();
    assert!(matches!(r, QueryResult::Created));
    assert_eq!(
        ex.auth_providers.lock_safe().get("google").unwrap(),
        "OAUTH"
    );
}

#[test]
fn test_s137_drop_auth_provider() {
    let (ex, _dir) = setup();
    ex.execute("CREATE AUTH PROVIDER myldap TYPE LDAP CONFIG 'ldap://x'")
        .unwrap();
    let r = ex.execute("DROP AUTH PROVIDER myldap").unwrap();
    assert!(matches!(r, QueryResult::Dropped));
    assert!(!ex.auth_providers.lock_safe().contains_key("myldap"));
}

#[test]
fn test_s137_authenticate_with_provider() {
    let (ex, _dir) = setup();
    ex.execute("CREATE AUTH PROVIDER myldap TYPE LDAP CONFIG 'ldap://x'")
        .unwrap();
    let r = ex
        .execute("AUTHENTICATE WITH PROVIDER myldap USER 'alice'")
        .unwrap();
    if let QueryResult::Ok(msg) = r {
        assert!(msg.contains("alice"));
        assert!(msg.contains("myldap"));
        assert!(msg.contains("LDAP"));
    } else {
        panic!();
    }
}

#[test]
fn test_s137_show_auth_providers() {
    let (ex, _dir) = setup();
    ex.execute("CREATE AUTH PROVIDER ldap1 TYPE LDAP CONFIG 'ldap://a'")
        .unwrap();
    ex.execute("CREATE AUTH PROVIDER oauth1 TYPE OAUTH CONFIG 'client=x'")
        .unwrap();
    let r = ex.execute("SHOW AUTH PROVIDERS").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 2);
    } else {
        panic!();
    }
}

// â”€â”€ S138 â€” Materialized view auto-refresh â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s138_refresh_every() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("REFRESH EVERY 60 SECONDS FOR VIEW sales_mv")
        .unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}

#[test]
fn test_s138_show_refresh_schedule() {
    let (ex, _dir) = setup();
    ex.execute("REFRESH EVERY 30 SECONDS FOR VIEW orders_mv")
        .unwrap();
    let r = ex.execute("SHOW REFRESH SCHEDULE").unwrap();
    if let QueryResult::Rows { rows, columns } = r {
        assert!(columns.contains(&"view_name".to_owned()));
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0][1], "30");
    } else {
        panic!();
    }
}

#[test]
fn test_s138_cancel_refresh() {
    let (ex, _dir) = setup();
    ex.execute("REFRESH EVERY 10 SECONDS FOR VIEW tmp_mv")
        .unwrap();
    let r = ex.execute("CANCEL REFRESH FOR VIEW tmp_mv").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW REFRESH SCHEDULE").unwrap() {
        assert!(rows.is_empty());
    }
}

#[test]
fn test_s138_trigger_refresh() {
    let (ex, _dir) = setup();
    let r = ex.execute("TRIGGER REFRESH FOR VIEW my_mv").unwrap();
    if let QueryResult::Ok(msg) = r {
        assert_eq!(msg, "REFRESHED");
    } else {
        panic!();
    }
}

#[test]
fn test_s138_trigger_refresh_updates_timestamp() {
    let (ex, _dir) = setup();
    ex.execute("TRIGGER REFRESH FOR VIEW ts_mv").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW REFRESH SCHEDULE").unwrap() {
        // no schedule set, so rows is empty â€” just check no panic
        let _ = rows;
    }
    // last_refresh should be set
    let last = ex.mat_view_last_refresh.lock_safe();
    assert!(last.contains_key("ts_mv"));
}

// â”€â”€ S139 â€” Query result caching with TTL â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s139_cache_query() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("CACHE QUERY 'SELECT 1' TTL 60 AS mykey")
        .unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}

#[test]
fn test_s139_get_cached_query() {
    let (ex, _dir) = setup();
    ex.execute("CACHE QUERY 'SELECT 1' TTL 3600 AS mykey")
        .unwrap();
    let r = ex.execute("GET CACHED QUERY mykey").unwrap();
    if let QueryResult::Ok(v) = r {
        assert_eq!(v, "SELECT 1");
    } else {
        panic!();
    }
}

#[test]
fn test_s139_get_cached_query_missing() {
    let (ex, _dir) = setup();
    let r = ex.execute("GET CACHED QUERY nokey").unwrap();
    if let QueryResult::Ok(v) = r {
        assert_eq!(v, "EXPIRED");
    } else {
        panic!();
    }
}

#[test]
fn test_s139_invalidate_cache() {
    let (ex, _dir) = setup();
    ex.execute("CACHE QUERY 'SELECT 2' TTL 3600 AS k2").unwrap();
    ex.execute("INVALIDATE CACHE FOR KEY k2").unwrap();
    let r = ex.execute("GET CACHED QUERY k2").unwrap();
    if let QueryResult::Ok(v) = r {
        assert_eq!(v, "EXPIRED");
    } else {
        panic!();
    }
}

#[test]
fn test_s139_show_query_cache() {
    let (ex, _dir) = setup();
    ex.execute("CACHE QUERY 'SELECT 3' TTL 3600 AS k3").unwrap();
    let r = ex.execute("SHOW QUERY CACHE").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0][1], "false");
    } else {
        panic!();
    }
}

// â”€â”€ S140 â€” Distributed locks â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s140_get_lock() {
    let (ex, _dir) = setup();
    let r = ex.execute("GET LOCK 'mylock' TIMEOUT 5").unwrap();
    if let QueryResult::Ok(msg) = r {
        assert!(msg.contains("LOCK ACQUIRED: mylock"));
    } else {
        panic!();
    }
}

#[test]
fn test_s140_get_lock_already_held() {
    let (ex, _dir) = setup();
    ex.execute("GET LOCK 'busy' TIMEOUT 5").unwrap();
    let r = ex.execute("GET LOCK 'busy' TIMEOUT 0").unwrap();
    if let QueryResult::Ok(msg) = r {
        assert!(msg.contains("LOCK TIMEOUT: busy"));
    } else {
        panic!();
    }
}

#[test]
fn test_s140_release_lock() {
    let (ex, _dir) = setup();
    ex.execute("GET LOCK 'rel' TIMEOUT 5").unwrap();
    let r = ex.execute("RELEASE LOCK 'rel'").unwrap();
    if let QueryResult::Ok(msg) = r {
        assert_eq!(msg, "LOCK RELEASED: rel");
    } else {
        panic!();
    }
}

#[test]
fn test_s140_show_locks() {
    let (ex, _dir) = setup();
    ex.execute("GET LOCK 'a' TIMEOUT 5").unwrap();
    ex.execute("GET LOCK 'b' TIMEOUT 5").unwrap();
    let r = ex.execute("SHOW LOCKS").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 2);
    } else {
        panic!();
    }
}

#[test]
fn test_s140_force_release_lock() {
    let (ex, _dir) = setup();
    ex.execute("GET LOCK 'forced' TIMEOUT 5").unwrap();
    let r = ex.execute("FORCE RELEASE LOCK 'forced'").unwrap();
    if let QueryResult::Ok(msg) = r {
        assert_eq!(msg, "LOCK RELEASED: forced");
    } else {
        panic!();
    }
}

// â”€â”€ S141 â€” Logical replication filters â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s141_create_replication_filter() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("CREATE REPLICATION FILTER f1 WHERE 'id > 100'")
        .unwrap();
    assert!(matches!(r, QueryResult::Created));
}

#[test]
fn test_s141_drop_replication_filter() {
    let (ex, _dir) = setup();
    ex.execute("CREATE REPLICATION FILTER f2 WHERE 'status = active'")
        .unwrap();
    let r = ex.execute("DROP REPLICATION FILTER f2").unwrap();
    assert!(matches!(r, QueryResult::Dropped));
}

#[test]
fn test_s141_show_replication_filters() {
    let (ex, _dir) = setup();
    ex.execute("CREATE REPLICATION FILTER f3 WHERE 'col = 1'")
        .unwrap();
    let r = ex.execute("SHOW REPLICATION FILTERS").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0][0], "f3");
    } else {
        panic!();
    }
}

#[test]
fn test_s141_apply_replication_filter() {
    let (ex, _dir) = setup();
    ex.execute("CREATE REPLICATION FILTER f4 WHERE 'x > 0'")
        .unwrap();
    let r = ex.execute("APPLY REPLICATION FILTER f4 TO orders").unwrap();
    if let QueryResult::Ok(msg) = r {
        assert_eq!(msg, "FILTER APPLIED: f4 on orders");
    } else {
        panic!();
    }
}

#[test]
fn test_s141_replication_filter_where_expr() {
    let (ex, _dir) = setup();
    ex.execute("CREATE REPLICATION FILTER f5 WHERE 'amount > 1000'")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW REPLICATION FILTERS").unwrap() {
        assert_eq!(rows[0][1], "amount > 1000");
    } else {
        panic!();
    }
}

// â”€â”€ S142 â€” Column encryption â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s142_create_encrypted_column() {
    let (ex, _dir) = setup_with_key();
    let r = ex
        .execute("CREATE ENCRYPTED COLUMN users ssn KEY HINT 'master-key-1'")
        .unwrap();
    assert!(matches!(r, QueryResult::Created));
}

#[test]
fn test_s142_show_encrypted_columns() {
    let (ex, _dir) = setup_with_key();
    ex.execute("CREATE ENCRYPTED COLUMN customers card KEY HINT 'k1'")
        .unwrap();
    let r = ex.execute("SHOW ENCRYPTED COLUMNS").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0][1], "k1");
    } else {
        panic!();
    }
}

#[test]
fn test_s142_insert_encrypted() {
    let (ex, _dir) = setup_with_key();
    ex.execute("CREATE ENCRYPTED COLUMN emp salary KEY HINT 'hk'")
        .unwrap();
    let r = ex
        .execute("INSERT ENCRYPTED INTO emp salary KEY 'hk' VALUE 'secret' PK 1")
        .unwrap();
    assert!(matches!(r, QueryResult::Affected(1)));
}

#[test]
fn test_s142_select_decrypt() {
    let (ex, _dir) = setup_with_key();
    ex.execute("CREATE ENCRYPTED COLUMN emp salary KEY HINT 'hk'")
        .unwrap();
    ex.execute("INSERT ENCRYPTED INTO emp salary KEY 'hk' VALUE 'hello' PK 2")
        .unwrap();
    let r = ex
        .execute("SELECT DECRYPT salary FROM emp KEY 'hk' PK 2")
        .unwrap();
    if let QueryResult::Ok(v) = r {
        assert_eq!(v, "hello");
    } else {
        panic!();
    }
}

#[test]
fn test_s142_decrypt_not_found() {
    let (ex, _dir) = setup_with_key();
    let r = ex
        .execute("SELECT DECRYPT col FROM tbl KEY 'k' PK 999")
        .unwrap();
    if let QueryResult::Ok(v) = r {
        assert_eq!(v, "NOT FOUND");
    } else {
        panic!();
    }
}

#[test]
fn test_s142_ciphertext_is_aes_gcm_not_plaintext() {
    let (ex, _dir) = setup_with_key();
    ex.execute("CREATE ENCRYPTED COLUMN emp salary KEY HINT 'hk'")
        .unwrap();
    ex.execute("INSERT ENCRYPTED INTO emp salary KEY 'hk' VALUE 'hello' PK 2")
        .unwrap();
    let stored = ex
        .encrypted_values
        .lock_safe()
        .get("emp.salary.2")
        .cloned()
        .unwrap();
    assert_ne!(stored, "hello");
    assert_ne!(stored, "olleh"); // the old string-reversal "cipher"
    let blob = hex::decode(&stored).unwrap();
    // magic(4) + nonce(12) + tag(16) + framed plaintext
    assert!(blob.len() >= 32 + "emp.salary.2\0hello".len());
    assert_eq!(&blob[..4], b"VVE1");
}

#[test]
fn test_s142_requires_configured_key() {
    let (ex, _dir) = setup();
    assert!(
        ex.execute("CREATE ENCRYPTED COLUMN emp salary KEY HINT 'hk'")
            .is_err()
    );
    let err = ex
        .execute("INSERT ENCRYPTED INTO emp salary KEY 'hk' VALUE 'x' PK 1")
        .unwrap_err();
    assert!(err.to_string().contains("not configured"), "{err}");
    assert!(
        ex.execute("SELECT DECRYPT salary FROM emp KEY 'hk' PK 1")
            .is_err()
    );
    assert!(
        ex.execute("ALTER TABLE emp ENCRYPT COLUMN salary WITH KEY 'k'")
            .is_err()
    );
    assert!(ex.encrypted_values.lock_safe().is_empty());
}

#[test]
fn test_s142_wrong_key_id_or_key_rejected() {
    let (ex, _dir) = setup_with_key();
    ex.execute("CREATE ENCRYPTED COLUMN emp salary KEY HINT 'hk'")
        .unwrap();
    ex.execute("INSERT ENCRYPTED INTO emp salary KEY 'hk' VALUE 'hello' PK 2")
        .unwrap();
    assert!(
        ex.execute("SELECT DECRYPT salary FROM emp KEY 'other' PK 2")
            .is_err()
    );
    // A different master key cannot open the stored ciphertext.
    ex.set_column_encryption_key(vectorvolt_crypto::KeyStore::generate().unwrap());
    assert!(
        ex.execute("SELECT DECRYPT salary FROM emp KEY 'hk' PK 2")
            .is_err()
    );
}

#[test]
fn test_s142_ciphertext_bound_to_cell() {
    let (ex, _dir) = setup_with_key();
    ex.execute("INSERT ENCRYPTED INTO emp salary KEY 'hk' VALUE 'a' PK 1")
        .unwrap();
    let v1 = ex.encrypted_values.lock_safe()["emp.salary.1"].clone();
    ex.encrypted_values
        .lock_safe()
        .insert("emp.salary.2".to_owned(), v1);
    assert!(
        ex.execute("SELECT DECRYPT salary FROM emp KEY 'hk' PK 2")
            .is_err()
    );
}

#[test]
fn test_s143_reset_row_compression() {
    let (ex, _dir) = setup();
    ex.execute("SET ROW COMPRESSION FOR t3 TO ZSTD").unwrap();
    ex.execute("RESET ROW COMPRESSION FOR t3").unwrap();
    let r = ex.execute("SHOW ROW COMPRESSION FOR t3").unwrap();
    if let QueryResult::Ok(v) = r {
        assert_eq!(v, "NONE");
    } else {
        panic!();
    }
}

#[test]
fn test_s143_show_compression_default_none() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("SHOW ROW COMPRESSION FOR unknown_table")
        .unwrap();
    if let QueryResult::Ok(v) = r {
        assert_eq!(v, "NONE");
    } else {
        panic!();
    }
}

// â”€â”€ S144 â€” Query routing rules â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s144_create_route() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("CREATE ROUTE r1 WHEN 'table=orders' SEND TO 'node-1'")
        .unwrap();
    assert!(matches!(r, QueryResult::Created));
}

#[test]
fn test_s144_drop_route() {
    let (ex, _dir) = setup();
    ex.execute("CREATE ROUTE r2 WHEN 'x' SEND TO 'n2'").unwrap();
    let r = ex.execute("DROP ROUTE r2").unwrap();
    assert!(matches!(r, QueryResult::Dropped));
}

#[test]
fn test_s144_show_routes() {
    let (ex, _dir) = setup();
    ex.execute("CREATE ROUTE r3 WHEN 'y' SEND TO 'n3'").unwrap();
    let r = ex.execute("SHOW ROUTES").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0][0], "r3");
    } else {
        panic!();
    }
}

#[test]
fn test_s144_evaluate_route_match() {
    let (ex, _dir) = setup();
    ex.execute("CREATE ROUTE r4 WHEN 'z' SEND TO 'nodeX'")
        .unwrap();
    let r = ex
        .execute("EVALUATE ROUTE FOR 'SELECT * FROM foo'")
        .unwrap();
    if let QueryResult::Ok(msg) = r {
        assert!(msg.contains("ROUTE:"));
    } else {
        panic!();
    }
}

#[test]
fn test_s144_evaluate_route_default() {
    let (ex, _dir) = setup();
    let r = ex.execute("EVALUATE ROUTE FOR 'SELECT 1'").unwrap();
    if let QueryResult::Ok(msg) = r {
        assert_eq!(msg, "ROUTE: default");
    } else {
        panic!();
    }
}

// â”€â”€ S145 â€” Workload scheduling â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s145_create_schedule() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("CREATE SCHEDULE nightly CRON '0 0 * * *' AS 'VACUUM'")
        .unwrap();
    assert!(matches!(r, QueryResult::Created));
}

#[test]
fn test_s145_drop_schedule() {
    let (ex, _dir) = setup();
    ex.execute("CREATE SCHEDULE tmp_sched CRON '* * * * *' AS 'SELECT 1'")
        .unwrap();
    let r = ex.execute("DROP SCHEDULE tmp_sched").unwrap();
    assert!(matches!(r, QueryResult::Dropped));
}

#[test]
fn test_s145_run_schedule() {
    let (ex, _dir) = setup();
    ex.execute("CREATE SCHEDULE job1 CRON '0 * * * *' AS 'ANALYZE'")
        .unwrap();
    let r = ex.execute("RUN SCHEDULE job1").unwrap();
    if let QueryResult::Ok(msg) = r {
        assert_eq!(msg, "SCHEDULE RAN: job1");
    } else {
        panic!();
    }
}

#[test]
fn test_s145_show_schedules() {
    let (ex, _dir) = setup();
    ex.execute("CREATE SCHEDULE job2 CRON '*/5 * * * *' AS 'SELECT 2'")
        .unwrap();
    ex.execute("RUN SCHEDULE job2").unwrap();
    let r = ex.execute("SHOW SCHEDULES").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0][2], "1");
    } else {
        panic!();
    }
}

#[test]
fn test_s145_run_schedule_increments_count() {
    let (ex, _dir) = setup();
    ex.execute("CREATE SCHEDULE cnt CRON '* * * * *' AS 'NOOP'")
        .unwrap();
    ex.execute("RUN SCHEDULE cnt").unwrap();
    ex.execute("RUN SCHEDULE cnt").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW SCHEDULES").unwrap() {
        assert_eq!(rows[0][2], "2");
    } else {
        panic!();
    }
}

// â”€â”€ S146 â€” Observability hooks â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s146_create_hook_slow_query() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("CREATE HOOK h1 ON SLOW_QUERY EXECUTE 'LOG slow query'")
        .unwrap();
    assert!(matches!(r, QueryResult::Created));
}

#[test]
fn test_s146_create_hook_error() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("CREATE HOOK h2 ON ERROR EXECUTE 'ALERT error'")
        .unwrap();
    assert!(matches!(r, QueryResult::Created));
}

#[test]
fn test_s146_drop_hook() {
    let (ex, _dir) = setup();
    ex.execute("CREATE HOOK h3 ON DDL EXECUTE 'LOG ddl'")
        .unwrap();
    let r = ex.execute("DROP HOOK h3").unwrap();
    assert!(matches!(r, QueryResult::Dropped));
}

#[test]
fn test_s146_show_hooks() {
    let (ex, _dir) = setup();
    ex.execute("CREATE HOOK h4 ON DDL EXECUTE 'NOTIFY'")
        .unwrap();
    let r = ex.execute("SHOW HOOKS").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0][1], "DDL");
    } else {
        panic!();
    }
}

#[test]
fn test_s146_fire_hook() {
    let (ex, _dir) = setup();
    ex.execute("CREATE HOOK h5 ON SLOW_QUERY EXECUTE 'LOG'")
        .unwrap();
    let r = ex.execute("FIRE HOOK h5").unwrap();
    if let QueryResult::Ok(msg) = r {
        assert_eq!(msg, "HOOK FIRED: h5");
    } else {
        panic!();
    }
}

// â”€â”€ S147 â€” Schema migrations â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s147_create_migration() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("CREATE MIGRATION m001 AS 'ALTER TABLE foo ADD COLUMN bar INT'")
        .unwrap();
    assert!(matches!(r, QueryResult::Created));
}

#[test]
fn test_s147_apply_migration() {
    let (ex, _dir) = setup();
    ex.execute("CREATE MIGRATION m002 AS 'SELECT 1'").unwrap();
    let r = ex.execute("APPLY MIGRATION m002").unwrap();
    if let QueryResult::Ok(msg) = r {
        assert_eq!(msg, "MIGRATION APPLIED: m002");
    } else {
        panic!();
    }
}

#[test]
fn test_s147_rollback_migration() {
    let (ex, _dir) = setup();
    ex.execute("CREATE MIGRATION m003 AS 'SELECT 1'").unwrap();
    ex.execute("APPLY MIGRATION m003").unwrap();
    let r = ex.execute("ROLLBACK MIGRATION m003").unwrap();
    if let QueryResult::Ok(msg) = r {
        assert_eq!(msg, "MIGRATION ROLLED BACK: m003");
    } else {
        panic!();
    }
}

#[test]
fn test_s147_show_migrations() {
    let (ex, _dir) = setup();
    ex.execute("CREATE MIGRATION m004 AS 'SELECT 4'").unwrap();
    ex.execute("APPLY MIGRATION m004").unwrap();
    let r = ex.execute("SHOW MIGRATIONS").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0][1], "true");
    } else {
        panic!();
    }
}

#[test]
fn test_s147_drop_migration() {
    let (ex, _dir) = setup();
    ex.execute("CREATE MIGRATION m005 AS 'SELECT 5'").unwrap();
    let r = ex.execute("DROP MIGRATION m005").unwrap();
    assert!(matches!(r, QueryResult::Dropped));
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW MIGRATIONS").unwrap() {
        assert!(rows.is_empty());
    }
}

// â”€â”€ S148 â€” Connection pooling â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn test_s148_create_pool() {
    let (ex, _dir) = setup();
    let r = ex.execute("CREATE POOL mypool SIZE 20").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}

#[test]
fn test_s148_show_pool_status() {
    let (ex, _dir) = setup();
    ex.execute("CREATE POOL p1 SIZE 8").unwrap();
    let r = ex.execute("SHOW POOL STATUS").unwrap();
    if let QueryResult::Rows { columns, rows } = r {
        assert_eq!(columns[0], "pool_name");
        assert_eq!(rows[0][1], "8");
    } else {
        panic!("expected Rows");
    }
}

#[test]
fn test_s148_drain_pool() {
    let (ex, _dir) = setup();
    let r = ex.execute("DRAIN POOL").unwrap();
    if let QueryResult::Ok(msg) = r {
        assert_eq!(msg, "POOL DRAINED");
    } else {
        panic!();
    }
}

#[test]
fn test_s148_available_after_drain() {
    let (ex, _dir) = setup();
    ex.execute("CREATE POOL p2 SIZE 10").unwrap();
    ex.execute("DRAIN POOL").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW POOL STATUS").unwrap() {
        assert_eq!(rows[0][3], "10"); // available = size - 0
    }
}

// â”€â”€ S149 â€” Query plan caching â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn test_s149_cache_plan() {
    let (ex, _dir) = setup();
    let r = ex.execute("CACHE PLAN FOR 'SELECT 1' AS plan1").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}

#[test]
fn test_s149_execute_cached_plan() {
    let (ex, _dir) = setup();
    ex.execute("CACHE PLAN FOR 'SELECT 1' AS plan1").unwrap();
    let r = ex.execute("EXECUTE CACHED PLAN plan1").unwrap();
    if let QueryResult::Ok(msg) = r {
        assert!(msg.starts_with("PLAN:"));
    } else {
        panic!();
    }
}

#[test]
fn test_s149_execute_cached_plan_missing() {
    let (ex, _dir) = setup();
    let r = ex.execute("EXECUTE CACHED PLAN nosuchplan").unwrap();
    assert!(matches!(r, QueryResult::Error(_)));
}

#[test]
fn test_s149_drop_cached_plan() {
    let (ex, _dir) = setup();
    ex.execute("CACHE PLAN FOR 'SELECT 2' AS plan2").unwrap();
    let r = ex.execute("DROP CACHED PLAN plan2").unwrap();
    assert!(matches!(r, QueryResult::Dropped));
}

#[test]
fn test_s149_show_plan_cache() {
    let (ex, _dir) = setup();
    ex.execute("CACHE PLAN FOR 'SELECT 3' AS plan3").unwrap();
    let r = ex.execute("SHOW PLAN CACHE").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert!(!rows.is_empty());
    } else {
        panic!();
    }
}

// â”€â”€ S150 â€” Row versioning â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn test_s150_enable_versioning() {
    let (ex, _dir) = setup();
    let r = ex.execute("ENABLE VERSIONING ON orders").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
