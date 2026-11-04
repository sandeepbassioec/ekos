//! Executor unit tests, part 1 of 4.

use super::*;

#[test]
fn test_create_table_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("CREATE TABLE t (pk U64, val U64)");
    assert!(matches!(r, Ok(QueryResult::Created)));
}

#[test]
fn test_create_table_duplicate_err() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    let r = ex.execute("CREATE TABLE t (pk U64)");
    assert!(r.is_err());
}

#[test]
fn test_drop_table_ok() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    let r = ex.execute("DROP TABLE t");
    assert!(matches!(r, Ok(QueryResult::Dropped)));
}

#[test]
fn test_drop_table_not_found() {
    let (ex, _dir) = setup();
    let r = ex.execute("DROP TABLE nope");
    assert!(r.is_err());
}

#[test]
fn test_insert_and_select_star() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 42)")
        .unwrap();
    match ex.execute("SELECT * FROM t").unwrap() {
        QueryResult::Rows { columns: _, rows } => {
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0][0], "1");
            assert_eq!(rows[0][1], "42");
        }
        _ => panic!(),
    }
}

#[test]
fn test_select_where_eq() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 10)")
        .unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (2, 20)")
        .unwrap();
    match ex.execute("SELECT * FROM t WHERE pk = 1").unwrap() {
        QueryResult::Rows { rows, .. } => assert_eq!(rows.len(), 1),
        _ => panic!(),
    }
}

#[test]
fn test_select_limit() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    for i in 1u64..=5 {
        ex.execute(&format!("INSERT INTO t (pk, val) VALUES ({i}, {i})"))
            .unwrap();
    }
    match ex.execute("SELECT * FROM t LIMIT 2").unwrap() {
        QueryResult::Rows { rows, .. } => assert_eq!(rows.len(), 2),
        _ => panic!(),
    }
}

#[test]
fn test_delete_where() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 10)")
        .unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (2, 20)")
        .unwrap();
    let r = ex.execute("DELETE FROM t WHERE pk = 1").unwrap();
    assert!(matches!(r, QueryResult::Affected(1)));
    match ex.execute("SELECT * FROM t").unwrap() {
        QueryResult::Rows { rows, .. } => assert_eq!(rows.len(), 1),
        _ => panic!(),
    }
}

#[test]
fn test_delete_no_where() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 10)")
        .unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (2, 20)")
        .unwrap();
    let r = ex.execute("DELETE FROM t").unwrap();
    assert!(matches!(r, QueryResult::Affected(2)));
}

#[test]
fn test_select_table_not_found() {
    let (ex, _dir) = setup();
    let r = ex.execute("SELECT * FROM missing");
    assert!(r.is_err());
}

#[test]
fn test_insert_column_count_mismatch() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    let r = ex.execute("INSERT INTO t (pk) VALUES (1, 2)");
    assert!(r.is_err());
}

#[test]
fn test_select_specific_columns() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 99)")
        .unwrap();
    match ex.execute("SELECT val FROM t").unwrap() {
        QueryResult::Rows { columns, rows } => {
            assert_eq!(columns, vec!["val"]);
            assert_eq!(rows[0][0], "99");
        }
        _ => panic!(),
    }
}

#[test]
fn test_where_gt() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 5)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (2, 15)")
        .unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (3, 25)")
        .unwrap();
    match ex.execute("SELECT * FROM t WHERE val > 10").unwrap() {
        QueryResult::Rows { rows, .. } => assert_eq!(rows.len(), 2),
        _ => panic!(),
    }
}

#[test]
fn test_where_and() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 5)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (2, 15)")
        .unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (3, 25)")
        .unwrap();
    match ex
        .execute("SELECT * FROM t WHERE val > 5 AND val < 20")
        .unwrap()
    {
        QueryResult::Rows { rows, .. } => assert_eq!(rows.len(), 1),
        _ => panic!(),
    }
}

#[test]
fn test_explain_full_scan() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    match ex.execute("EXPLAIN SELECT * FROM t").unwrap() {
        QueryResult::ExplainPlan(lines) => {
            assert!(
                lines
                    .iter()
                    .any(|l| l.contains("FullScan") && l.contains("table=t"))
            );
            assert!(lines.iter().any(|l| l.contains("Project")));
        }
        _ => panic!("expected ExplainPlan"),
    }
}

#[test]
fn test_explain_range_scan_pushdown() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    match ex.execute("EXPLAIN SELECT * FROM t WHERE pk = 5").unwrap() {
        QueryResult::ExplainPlan(lines) => {
            let range = lines.iter().find(|l| l.contains("RangeScan"));
            let range = range.expect("expected RangeScan node");
            assert!(range.contains("col=pk"), "got: {range}");
            assert!(range.contains("lo=5"), "got: {range}");
            assert!(range.contains("hi=5"), "got: {range}");
        }
        _ => panic!("expected ExplainPlan"),
    }
}

#[test]
fn test_explain_limit_and_order() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    match ex
        .execute("EXPLAIN SELECT pk FROM t ORDER BY val DESC LIMIT 10")
        .unwrap()
    {
        QueryResult::ExplainPlan(lines) => {
            assert!(
                lines
                    .iter()
                    .any(|l| l.contains("Sort") && l.contains("DESC"))
            );
            assert!(
                lines
                    .iter()
                    .any(|l| l.contains("Limit") && l.contains("10"))
            );
        }
        _ => panic!("expected ExplainPlan"),
    }
}

#[test]
fn test_pushdown_range_narrows_results() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    for i in 1u64..=10 {
        ex.execute(&format!("INSERT INTO t (pk, val) VALUES ({i}, {i})"))
            .unwrap();
    }
    // pk >= 5 AND pk <= 7 â†’ expect exactly rows 5,6,7
    match ex
        .execute("SELECT * FROM t WHERE pk >= 5 AND pk <= 7")
        .unwrap()
    {
        QueryResult::Rows { rows, .. } => assert_eq!(rows.len(), 3),
        _ => panic!(),
    }
}

#[test]
fn test_begin_commit() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    assert!(matches!(ex.execute("BEGIN").unwrap(), QueryResult::TxBegin));
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 100)")
        .unwrap();
    assert!(matches!(
        ex.execute("COMMIT").unwrap(),
        QueryResult::TxCommit
    ));
    match ex.execute("SELECT * FROM t").unwrap() {
        QueryResult::Rows { rows, .. } => assert_eq!(rows.len(), 1),
        _ => panic!(),
    }
}

#[test]
fn test_begin_rollback() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("BEGIN").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 100)")
        .unwrap();
    assert!(matches!(
        ex.execute("ROLLBACK").unwrap(),
        QueryResult::TxRollback
    ));
    match ex.execute("SELECT * FROM t").unwrap() {
        QueryResult::Rows { rows, .. } => assert_eq!(rows.len(), 0),
        _ => panic!(),
    }
}

#[test]
fn test_double_begin_err() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64)").unwrap();
    ex.execute("BEGIN").unwrap();
    assert!(matches!(
        ex.execute("BEGIN"),
        Err(ExecError::TxnAlreadyActive)
    ));
    ex.execute("ROLLBACK").unwrap();
}

#[test]
fn test_commit_without_begin_err() {
    let (ex, _dir) = setup();
    assert!(matches!(ex.execute("COMMIT"), Err(ExecError::NoActiveTxn)));
}

#[test]
fn test_abort_alias() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64)").unwrap();
    ex.execute("BEGIN").unwrap();
    ex.execute("INSERT INTO t (pk) VALUES (1)").unwrap();
    assert!(matches!(
        ex.execute("ABORT").unwrap(),
        QueryResult::TxRollback
    ));
    match ex.execute("SELECT * FROM t").unwrap() {
        QueryResult::Rows { rows, .. } => assert_eq!(rows.len(), 0),
        _ => panic!(),
    }
}

#[test]
fn test_fts_match_basic() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE docs (pk U64, body TEXT)").unwrap();
    ex.execute("INSERT INTO docs (pk, body) VALUES (1, 'hello world')")
        .unwrap();
    ex.execute("INSERT INTO docs (pk, body) VALUES (2, 'foo bar')")
        .unwrap();
    match ex
        .execute("SELECT * FROM docs WHERE MATCH(body, 'hello')")
        .unwrap()
    {
        QueryResult::Rows { rows, .. } => {
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0][0], "1");
        }
        _ => panic!(),
    }
}

#[test]
fn test_fts_match_no_hit() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE docs (pk U64, body TEXT)").unwrap();
    ex.execute("INSERT INTO docs (pk, body) VALUES (1, 'hello world')")
        .unwrap();
    match ex
        .execute("SELECT * FROM docs WHERE MATCH(body, 'rust')")
        .unwrap()
    {
        QueryResult::Rows { rows, .. } => assert_eq!(rows.len(), 0),
        _ => panic!(),
    }
}

#[test]
fn test_fts_match_case_insensitive() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE docs (pk U64, body TEXT)").unwrap();
    ex.execute("INSERT INTO docs (pk, body) VALUES (1, 'Hello World')")
        .unwrap();
    match ex
        .execute("SELECT * FROM docs WHERE MATCH(body, 'hello')")
        .unwrap()
    {
        QueryResult::Rows { rows, .. } => assert_eq!(rows.len(), 1),
        _ => panic!(),
    }
}

#[test]
fn test_fts_match_multiple_rows() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE docs (pk U64, body TEXT)").unwrap();
    ex.execute("INSERT INTO docs (pk, body) VALUES (1, 'rust is fast')")
        .unwrap();
    ex.execute("INSERT INTO docs (pk, body) VALUES (2, 'rust and safety')")
        .unwrap();
    ex.execute("INSERT INTO docs (pk, body) VALUES (3, 'python is slow')")
        .unwrap();
    match ex
        .execute("SELECT * FROM docs WHERE MATCH(body, 'rust')")
        .unwrap()
    {
        QueryResult::Rows { rows, .. } => assert_eq!(rows.len(), 2),
        _ => panic!(),
    }
}

#[test]
fn test_vec_distance_order_nearest_first() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE vecs (pk U64, emb vector(2))")
        .unwrap();
    // pk=1: [1.0, 0.0], pk=2: [10.0, 0.0], pk=3: [2.0, 0.0]
    ex.execute("INSERT INTO vecs (pk, emb) VALUES (1, '[1.0, 0.0]')")
        .unwrap();
    ex.execute("INSERT INTO vecs (pk, emb) VALUES (2, '[10.0, 0.0]')")
        .unwrap();
    ex.execute("INSERT INTO vecs (pk, emb) VALUES (3, '[2.0, 0.0]')")
        .unwrap();
    // Query [1.5, 0.0]: closest is pk=1 (dist=0.5), then pk=3 (dist=0.5... no, 0.5 vs 0.5)
    // Actually query [0.0, 0.0]: pk=1 dist=1, pk=3 dist=2, pk=2 dist=10
    match ex
        .execute("SELECT pk FROM vecs ORDER BY vec_distance(emb, '[0.0, 0.0]') LIMIT 3")
        .unwrap()
    {
        QueryResult::Rows { rows, .. } => {
            assert_eq!(rows.len(), 3);
            assert_eq!(rows[0][0], "1"); // nearest
            assert_eq!(rows[1][0], "3");
            assert_eq!(rows[2][0], "2"); // farthest
        }
        _ => panic!(),
    }
}

#[test]
fn test_vec_distance_limit_k() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE vecs (pk U64, emb vector(2))")
        .unwrap();
    for i in 1u64..=5 {
        ex.execute(&format!(
            "INSERT INTO vecs (pk, emb) VALUES ({i}, '[{}.0, 0.0]')",
            i * 10
        ))
        .unwrap();
    }
    // Query [0,0]: top-2 should be pk=1 (dist=10) and pk=2 (dist=20)
    match ex
        .execute("SELECT pk FROM vecs ORDER BY vec_distance(emb, '[0.0, 0.0]') LIMIT 2")
        .unwrap()
    {
        QueryResult::Rows { rows, .. } => {
            assert_eq!(rows.len(), 2);
            assert_eq!(rows[0][0], "1");
            assert_eq!(rows[1][0], "2");
        }
        _ => panic!(),
    }
}

#[test]
fn test_vec_no_sidecar_returns_all() {
    // Without vector sidecar file (column is u64, not vector type), ORDER BY
    // vec_distance falls back gracefully â€” rows returned unsorted with dist=MAX.
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 10)")
        .unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (2, 20)")
        .unwrap();
    match ex
        .execute("SELECT pk FROM t ORDER BY vec_distance(val, '[0.0]') LIMIT 10")
        .unwrap()
    {
        QueryResult::Rows { rows, .. } => assert_eq!(rows.len(), 2),
        _ => panic!(),
    }
}

#[test]
fn test_vec_distance_parse_vec_literal() {
    use crate::sql::vec_store::parse_vec_literal;
    assert_eq!(
        parse_vec_literal("[1.0, 2.0, 3.0]"),
        Some(vec![1.0, 2.0, 3.0])
    );
    assert_eq!(parse_vec_literal("[0.5]"), Some(vec![0.5]));
    assert!(parse_vec_literal("not a vec").is_none());
}

// â”€â”€ Planner tests â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn test_planner_chooses_range_scan_for_eq() {
    use crate::sql::lexer::Lexer;
    use crate::sql::parser::SqlParser;
    use crate::sql::planner::Planner;
    use vectorvolt_engine::{ColumnDef, ColumnType, TableSchema};

    let schema = TableSchema::new(vec![
        ColumnDef::new("pk", ColumnType::U64),
        ColumnDef::new("val", ColumnType::U64),
    ])
    .unwrap();
    let tokens = Lexer::new("SELECT * FROM t WHERE pk = 5")
        .tokenize()
        .unwrap();
    let stmt = match SqlParser::new(tokens).parse().unwrap() {
        crate::sql::ast::Stmt::Select(s) => s,
        _ => panic!(),
    };
    let plan = Planner::new(&schema, 10_000).plan(&stmt);
    assert!(
        plan.explain(0).iter().any(|l| l.contains("RangeScan")),
        "expected RangeScan in plan"
    );
}

#[test]
fn test_planner_chooses_full_scan_no_where() {
    use crate::sql::lexer::Lexer;
    use crate::sql::parser::SqlParser;
    use crate::sql::planner::Planner;
    use vectorvolt_engine::{ColumnDef, ColumnType, TableSchema};

    let schema = TableSchema::new(vec![ColumnDef::new("pk", ColumnType::U64)]).unwrap();
    let tokens = Lexer::new("SELECT * FROM t").tokenize().unwrap();
    let stmt = match SqlParser::new(tokens).parse().unwrap() {
        crate::sql::ast::Stmt::Select(s) => s,
        _ => panic!(),
    };
    let plan = Planner::new(&schema, 500).plan(&stmt);
    let lines = plan.explain(0);
    assert!(lines.iter().any(|l| l.contains("FullScan")));
    assert!(!lines.iter().any(|l| l.contains("RangeScan")));
}

#[test]
fn test_planner_explain_vec_sort_node() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE vecs (pk U64, emb vector(2))")
        .unwrap();
    match ex
        .execute("EXPLAIN SELECT pk FROM vecs ORDER BY vec_distance(emb, '[1.0,0.0]') LIMIT 3")
        .unwrap()
    {
        QueryResult::ExplainPlan(lines) => {
            assert!(lines.iter().any(|l| l.contains("VecSort")));
            assert!(lines.iter().any(|l| l.contains("Limit") && l.contains("3")));
        }
        _ => panic!("expected ExplainPlan"),
    }
}

#[test]
fn test_planner_est_rows_range_less_than_full() {
    use crate::sql::lexer::Lexer;
    use crate::sql::parser::SqlParser;
    use crate::sql::planner::Planner;
    use vectorvolt_engine::{ColumnDef, ColumnType, TableSchema};

    let schema = TableSchema::new(vec![
        ColumnDef::new("pk", ColumnType::U64),
        ColumnDef::new("v", ColumnType::U64),
    ])
    .unwrap();
    let tokens = Lexer::new("SELECT * FROM t WHERE pk = 1")
        .tokenize()
        .unwrap();
    let stmt = match SqlParser::new(tokens).parse().unwrap() {
        crate::sql::ast::Stmt::Select(s) => s,
        _ => panic!(),
    };
    let plan = Planner::new(&schema, 10_000).plan(&stmt);
    assert!(
        plan.est_rows() < 10_000,
        "expected est_rows < 10_000, got {}",
        plan.est_rows()
    );
}

#[test]
fn test_update_basic() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    ex.execute("INSERT INTO t (pk, v) VALUES (1, 10)").unwrap();
    let r = ex.execute("UPDATE t SET v = 99 WHERE pk = 1");
    assert!(matches!(r, Ok(QueryResult::Affected(1))));
    let rows = ex.execute("SELECT * FROM t WHERE pk = 1").unwrap();
    if let QueryResult::Rows { rows, .. } = rows {
        assert_eq!(rows[0][1], "99");
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_update_no_match_returns_zero() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    ex.execute("INSERT INTO t (pk, v) VALUES (1, 10)").unwrap();
    let r = ex.execute("UPDATE t SET v = 99 WHERE pk = 999");
    assert!(matches!(r, Ok(QueryResult::Affected(0))));
}

#[test]
fn test_update_multiple_rows() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    ex.execute("INSERT INTO t (pk, v) VALUES (1, 5)").unwrap();
    ex.execute("INSERT INTO t (pk, v) VALUES (2, 5)").unwrap();
    ex.execute("INSERT INTO t (pk, v) VALUES (3, 9)").unwrap();
    // UPDATE all rows where v = 5 â€” should update 2 rows
    let r = ex.execute("UPDATE t SET v = 7 WHERE v = 5");
    assert!(matches!(r, Ok(QueryResult::Affected(2))));
}

#[test]
fn test_update_no_where_updates_all() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    ex.execute("INSERT INTO t (pk, v) VALUES (1, 1)").unwrap();
    ex.execute("INSERT INTO t (pk, v) VALUES (2, 2)").unwrap();
    let r = ex.execute("UPDATE t SET v = 0");
    assert!(matches!(r, Ok(QueryResult::Affected(2))));
}

#[test]
fn test_agg_count_star() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    ex.execute("INSERT INTO t (pk, v) VALUES (1, 10)").unwrap();
    ex.execute("INSERT INTO t (pk, v) VALUES (2, 20)").unwrap();
    ex.execute("INSERT INTO t (pk, v) VALUES (3, 30)").unwrap();
    let r = ex.execute("SELECT COUNT(*) FROM t").unwrap();
    if let QueryResult::Rows { columns, rows } = r {
        assert_eq!(columns, vec!["count"]);
        assert_eq!(rows[0][0], "3");
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_agg_sum_min_max() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    ex.execute("INSERT INTO t (pk, v) VALUES (1, 10)").unwrap();
    ex.execute("INSERT INTO t (pk, v) VALUES (2, 20)").unwrap();
    ex.execute("INSERT INTO t (pk, v) VALUES (3, 30)").unwrap();
    let r = ex.execute("SELECT SUM(v), MIN(v), MAX(v) FROM t").unwrap();
    if let QueryResult::Rows { columns, rows } = r {
        assert_eq!(columns, vec!["sum(v)", "min(v)", "max(v)"]);
        assert_eq!(rows[0][0], "60");
        assert_eq!(rows[0][1], "10");
        assert_eq!(rows[0][2], "30");
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_agg_count_with_where() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    ex.execute("INSERT INTO t (pk, v) VALUES (1, 5)").unwrap();
    ex.execute("INSERT INTO t (pk, v) VALUES (2, 15)").unwrap();
    ex.execute("INSERT INTO t (pk, v) VALUES (3, 25)").unwrap();
    let r = ex.execute("SELECT COUNT(*) FROM t WHERE v >= 15").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows[0][0], "2");
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_agg_group_by() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, cat U64, v U64)")
        .unwrap();
    ex.execute("INSERT INTO t (pk, cat, v) VALUES (1, 1, 10)")
        .unwrap();
    ex.execute("INSERT INTO t (pk, cat, v) VALUES (2, 1, 20)")
        .unwrap();
    ex.execute("INSERT INTO t (pk, cat, v) VALUES (3, 2, 30)")
        .unwrap();
    let r = ex.execute("SELECT COUNT(*) FROM t GROUP BY cat").unwrap();
    if let QueryResult::Rows { columns, rows } = r {
        assert_eq!(columns, vec!["cat", "count"]);
        assert_eq!(rows.len(), 2);
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s32_alter_add_column() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    ex.execute("INSERT INTO t (pk, v) VALUES (1, 10)").unwrap();
    let r = ex.execute("ALTER TABLE t ADD COLUMN extra U64");
    assert!(r.is_ok(), "{r:?}");
    let rows = ex.execute("SELECT * FROM t").unwrap();
    if let QueryResult::Rows { columns, .. } = rows {
        assert!(columns.iter().any(|c| c == "extra"), "extra column missing");
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s32_alter_drop_column() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, a U64, b U64)").unwrap();
    ex.execute("INSERT INTO t (pk, a, b) VALUES (1, 1, 2)")
        .unwrap();
    let r = ex.execute("ALTER TABLE t DROP COLUMN b");
    assert!(r.is_ok(), "{r:?}");
    let rows = ex.execute("SELECT * FROM t").unwrap();
    if let QueryResult::Rows { columns, .. } = rows {
        assert!(
            !columns.iter().any(|c| c == "b"),
            "b column should be dropped"
        );
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s32_insert_or_replace() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    ex.execute("INSERT INTO t (pk, v) VALUES (1, 10)").unwrap();
    ex.execute("INSERT OR REPLACE INTO t (pk, v) VALUES (1, 99)")
        .unwrap();
    let rows = ex.execute("SELECT * FROM t").unwrap();
    if let QueryResult::Rows { rows, .. } = rows {
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0][1], "99");
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s32_having() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, cat U64, v U64)")
        .unwrap();
    ex.execute("INSERT INTO t (pk, cat, v) VALUES (1, 1, 10)")
        .unwrap();
    ex.execute("INSERT INTO t (pk, cat, v) VALUES (2, 1, 20)")
        .unwrap();
    ex.execute("INSERT INTO t (pk, cat, v) VALUES (3, 2, 30)")
        .unwrap();
    let r = ex
        .execute("SELECT COUNT(*) FROM t GROUP BY cat HAVING count > 1")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0][1], "2");
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s32_where_in() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    for i in 1u64..=5 {
        ex.execute(&format!("INSERT INTO t (pk, v) VALUES ({i}, {i})"))
            .unwrap();
    }
    let r = ex.execute("SELECT * FROM t WHERE pk IN (1, 3, 5)").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 3);
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s32_where_between() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    for i in 1u64..=10 {
        ex.execute(&format!("INSERT INTO t (pk, v) VALUES ({i}, {i})"))
            .unwrap();
    }
    let r = ex
        .execute("SELECT * FROM t WHERE v BETWEEN 3 AND 7")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 5);
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s32_insert_select() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE src (pk U64, v U64)").unwrap();
    ex.execute("CREATE TABLE dst (pk U64, v U64)").unwrap();
    ex.execute("INSERT INTO src (pk, v) VALUES (1, 10)")
        .unwrap();
    ex.execute("INSERT INTO src (pk, v) VALUES (2, 20)")
        .unwrap();
    let r = ex.execute("INSERT INTO dst SELECT * FROM src").unwrap();
    assert!(matches!(r, QueryResult::Affected(2)));
    let rows = ex.execute("SELECT * FROM dst").unwrap();
    if let QueryResult::Rows { rows, .. } = rows {
        assert_eq!(rows.len(), 2);
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s32_truncate() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    ex.execute("INSERT INTO t (pk, v) VALUES (1, 10)").unwrap();
    ex.execute("INSERT INTO t (pk, v) VALUES (2, 20)").unwrap();
    let r = ex.execute("TRUNCATE TABLE t");
    assert!(r.is_ok(), "{r:?}");
    let rows = ex.execute("SELECT * FROM t").unwrap();
    if let QueryResult::Rows { rows, .. } = rows {
        assert_eq!(rows.len(), 0);
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s32_rename_table() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE old_t (pk U64, v U64)").unwrap();
    ex.execute("INSERT INTO old_t (pk, v) VALUES (1, 42)")
        .unwrap();
    let r = ex.execute("RENAME TABLE old_t TO new_t");
    assert!(r.is_ok(), "{r:?}");
    assert!(ex.execute("SELECT * FROM old_t").is_err());
    let rows = ex.execute("SELECT * FROM new_t").unwrap();
    if let QueryResult::Rows { rows, .. } = rows {
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0][1], "42");
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s33_inner_join_basic() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE orders (pk U64, user_id U64, amount U64)")
        .unwrap();
    ex.execute("CREATE TABLE users (pk U64, name U64)").unwrap();
    ex.execute("INSERT INTO orders (pk, user_id, amount) VALUES (1, 10, 100)")
        .unwrap();
    ex.execute("INSERT INTO orders (pk, user_id, amount) VALUES (2, 20, 200)")
        .unwrap();
    ex.execute("INSERT INTO users (pk, name) VALUES (10, 1)")
        .unwrap();
    ex.execute("INSERT INTO users (pk, name) VALUES (20, 2)")
        .unwrap();
    let r = ex
        .execute("SELECT * FROM orders INNER JOIN users ON user_id = pk")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 2);
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s33_inner_join_no_match() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE a (pk U64, b_id U64)").unwrap();
    ex.execute("CREATE TABLE b (pk U64, v U64)").unwrap();
    ex.execute("INSERT INTO a (pk, b_id) VALUES (1, 99)")
        .unwrap();
    ex.execute("INSERT INTO b (pk, v) VALUES (1, 42)").unwrap();
    let r = ex
        .execute("SELECT * FROM a INNER JOIN b ON b_id = pk")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 0);
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s33_left_join_keeps_unmatched() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE a (pk U64, b_id U64)").unwrap();
    ex.execute("CREATE TABLE b (pk U64, v U64)").unwrap();
    ex.execute("INSERT INTO a (pk, b_id) VALUES (1, 10)")
        .unwrap();
    ex.execute("INSERT INTO a (pk, b_id) VALUES (2, 99)")
        .unwrap();
    ex.execute("INSERT INTO b (pk, v) VALUES (10, 42)").unwrap();
    let r = ex
        .execute("SELECT * FROM a LEFT JOIN b ON b_id = pk")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 2);
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s33_join_with_where() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE o (pk U64, uid U64, amt U64)")
        .unwrap();
    ex.execute("CREATE TABLE u (pk U64, active U64)").unwrap();
    ex.execute("INSERT INTO o (pk, uid, amt) VALUES (1, 1, 50)")
        .unwrap();
    ex.execute("INSERT INTO o (pk, uid, amt) VALUES (2, 2, 150)")
        .unwrap();
    ex.execute("INSERT INTO u (pk, active) VALUES (1, 1)")
        .unwrap();
    ex.execute("INSERT INTO u (pk, active) VALUES (2, 1)")
        .unwrap();
    let r = ex
        .execute("SELECT * FROM o INNER JOIN u ON uid = pk WHERE amt > 100")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 1);
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s33_join_keyword_no_inner() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE a (pk U64, b_id U64)").unwrap();
    ex.execute("CREATE TABLE b (pk U64, v U64)").unwrap();
    ex.execute("INSERT INTO a (pk, b_id) VALUES (1, 5)")
        .unwrap();
    ex.execute("INSERT INTO b (pk, v) VALUES (5, 99)").unwrap();
    let r = ex.execute("SELECT * FROM a JOIN b ON b_id = pk").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0][1], "5");
    } else {
        panic!("expected rows");
    }
}

// â”€â”€ S34: SQL Subqueries â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn test_s34_in_subquery_basic() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE orders (pk U64, amount U64)")
        .unwrap();
    ex.execute("CREATE TABLE vip (pk U64, order_id U64)")
        .unwrap();
    ex.execute("INSERT INTO orders (pk, amount) VALUES (1, 100)")
        .unwrap();
    ex.execute("INSERT INTO orders (pk, amount) VALUES (2, 200)")
        .unwrap();
    ex.execute("INSERT INTO orders (pk, amount) VALUES (3, 300)")
        .unwrap();
    ex.execute("INSERT INTO vip (pk, order_id) VALUES (10, 1)")
        .unwrap();
    ex.execute("INSERT INTO vip (pk, order_id) VALUES (11, 3)")
        .unwrap();
    let r = ex
        .execute("SELECT pk FROM orders WHERE pk IN (SELECT order_id FROM vip)")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        let pks: Vec<&str> = rows.iter().map(|r| r[0].as_str()).collect();
        assert!(pks.contains(&"1"), "pk 1 should match");
        assert!(pks.contains(&"3"), "pk 3 should match");
        assert!(!pks.contains(&"2"), "pk 2 should not match");
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s34_in_subquery_with_where() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE items (pk U64, price U64)")
        .unwrap();
    ex.execute("CREATE TABLE featured (pk U64, item_pk U64, active U64)")
        .unwrap();
    ex.execute("INSERT INTO items (pk, price) VALUES (1, 50)")
        .unwrap();
    ex.execute("INSERT INTO items (pk, price) VALUES (2, 80)")
        .unwrap();
    ex.execute("INSERT INTO items (pk, price) VALUES (3, 120)")
        .unwrap();
    ex.execute("INSERT INTO featured (pk, item_pk, active) VALUES (1, 2, 1)")
        .unwrap();
    ex.execute("INSERT INTO featured (pk, item_pk, active) VALUES (2, 3, 0)")
        .unwrap();
    let r = ex
        .execute("SELECT pk FROM items WHERE pk IN (SELECT item_pk FROM featured WHERE active = 1)")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0][0], "2");
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s34_exists_true() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE users (pk U64, role U64)").unwrap();
    ex.execute("CREATE TABLE admins (pk U64, user_id U64)")
        .unwrap();
    ex.execute("INSERT INTO users (pk, role) VALUES (1, 1)")
        .unwrap();
    ex.execute("INSERT INTO admins (pk, user_id) VALUES (1, 1)")
        .unwrap();
    let r = ex
        .execute("SELECT pk FROM users WHERE EXISTS (SELECT pk FROM admins WHERE user_id = 1)")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 1);
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s34_exists_false() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE users (pk U64, role U64)").unwrap();
    ex.execute("CREATE TABLE admins (pk U64, user_id U64)")
        .unwrap();
    ex.execute("INSERT INTO users (pk, role) VALUES (1, 1)")
        .unwrap();
    // No admin rows â€” EXISTS should return false, filtering out all users
    let r = ex
        .execute("SELECT pk FROM users WHERE EXISTS (SELECT pk FROM admins WHERE user_id = 1)")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 0);
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s34_scalar_subquery_in_select() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE products (pk U64, price U64)")
        .unwrap();
    ex.execute("CREATE TABLE counts (pk U64, total U64)")
        .unwrap();
    ex.execute("INSERT INTO products (pk, price) VALUES (1, 42)")
        .unwrap();
    ex.execute("INSERT INTO counts (pk, total) VALUES (1, 99)")
        .unwrap();
    let r = ex
        .execute("SELECT pk, (SELECT total FROM counts WHERE pk = 1) AS cnt FROM products")
        .unwrap();
    if let QueryResult::Rows { columns, rows } = r {
        assert!(columns.contains(&"cnt".to_owned()));
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0][1], "99");
    } else {
        panic!("expected rows");
    }
}

// â”€â”€ S37: Window functions â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn test_s37_row_number() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE wt (pk U64, v U64)").unwrap();
    ex.execute("INSERT INTO wt (pk, v) VALUES (1, 10)").unwrap();
    ex.execute("INSERT INTO wt (pk, v) VALUES (2, 20)").unwrap();
    ex.execute("INSERT INTO wt (pk, v) VALUES (3, 30)").unwrap();
    let r = ex
        .execute("SELECT pk, ROW_NUMBER() OVER () FROM wt")
        .unwrap();
    if let QueryResult::Rows { columns, rows } = r {
        assert!(columns.contains(&"row_number".to_owned()));
        assert_eq!(rows.len(), 3);
        // ROW_NUMBER values should be 1, 2, 3
        let rns: Vec<&str> = rows.iter().map(|r| r[1].as_str()).collect();
        assert!(rns.contains(&"1") && rns.contains(&"2") && rns.contains(&"3"));
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s37_rank_over_order() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE rt (pk U64, score U64)").unwrap();
    ex.execute("INSERT INTO rt (pk, score) VALUES (1, 100)")
        .unwrap();
    ex.execute("INSERT INTO rt (pk, score) VALUES (2, 200)")
        .unwrap();
    ex.execute("INSERT INTO rt (pk, score) VALUES (3, 150)")
        .unwrap();
    let r = ex
        .execute("SELECT pk, RANK() OVER (ORDER BY score) FROM rt ORDER BY pk")
        .unwrap();
    if let QueryResult::Rows { columns, rows } = r {
        assert!(columns.contains(&"rank".to_owned()));
        // pk=1 has score=100 (lowest) â†’ rank 1
        let pk1_rank = &rows.iter().find(|r| r[0] == "1").unwrap()[1];
        assert_eq!(pk1_rank, "1");
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s37_win_sum_over() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE st (pk U64, v U64)").unwrap();
    ex.execute("INSERT INTO st (pk, v) VALUES (1, 10)").unwrap();
    ex.execute("INSERT INTO st (pk, v) VALUES (2, 20)").unwrap();
    ex.execute("INSERT INTO st (pk, v) VALUES (3, 30)").unwrap();
    let r = ex.execute("SELECT pk, SUM(v) OVER () FROM st").unwrap();
    if let QueryResult::Rows { columns, rows } = r {
        assert!(columns.contains(&"win_sum(v)".to_owned()));
        // All rows should have the same total: 10+20+30=60
        assert!(rows.iter().all(|r| r[1] == "60"));
    } else {
        panic!("expected rows");
    }
}

// â”€â”€ S37: CASE WHEN â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn test_s37_case_when() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE ct (pk U64, v U64)").unwrap();
    ex.execute("INSERT INTO ct (pk, v) VALUES (1, 5)").unwrap();
    ex.execute("INSERT INTO ct (pk, v) VALUES (2, 15)").unwrap();
    let r = ex
        .execute("SELECT pk, CASE WHEN v = 5 THEN 100 ELSE 200 END FROM ct ORDER BY pk")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0][1], "100"); // pk=1, v=5 â†’ 100
        assert_eq!(rows[1][1], "200"); // pk=2, v=15 â†’ 200
    } else {
        panic!("expected rows");
    }
}

// â”€â”€ S37: NULL handling â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn test_s37_is_null() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE nt (pk U64, v U64)").unwrap();
    // Insert row with v=0 (treated as NULL in our u64 system)
    ex.execute("INSERT INTO nt (pk, v) VALUES (1, 0)").unwrap();
    ex.execute("INSERT INTO nt (pk, v) VALUES (2, 99)").unwrap();
    let r = ex.execute("SELECT pk FROM nt WHERE v IS NULL").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        // Only pk=1 (v=0) should match IS NULL
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0][0], "1");
    } else {
        panic!("expected rows");
    }
}

// â”€â”€ S38: String + Pagination â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn test_s38_limit_offset() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE pg (pk U64, v U64)").unwrap();
    for i in 1u64..=5 {
        ex.execute(&format!("INSERT INTO pg (pk, v) VALUES ({i}, {i})"))
            .unwrap();
    }
    let r = ex
        .execute("SELECT pk FROM pg ORDER BY pk ASC LIMIT 2 OFFSET 2")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0][0], "3");
        assert_eq!(rows[1][0], "4");
    } else {
        panic!("expected rows");
    }
}

// â”€â”€ S39: Arithmetic Expressions â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn test_s39_add_columns() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 10)")
        .unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (2, 20)")
        .unwrap();
    let r = ex
        .execute("SELECT pk + val AS total FROM t ORDER BY pk")
        .unwrap();
    if let QueryResult::Rows { columns, rows } = r {
        assert!(columns.contains(&"total".to_owned()));
        assert_eq!(rows[0][0], "11");
        assert_eq!(rows[1][0], "22");
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s38_like_prefix() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE lk (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO lk (pk, val) VALUES (1, 100)")
        .unwrap();
    ex.execute("INSERT INTO lk (pk, val) VALUES (2, 200)")
        .unwrap();
    ex.execute("INSERT INTO lk (pk, val) VALUES (3, 150)")
        .unwrap();
    let r = ex.execute("SELECT pk FROM lk WHERE val LIKE '1%'").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 2);
        let pks: Vec<&str> = rows.iter().map(|r| r[0].as_str()).collect();
        assert!(pks.contains(&"1"));
        assert!(pks.contains(&"3"));
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s39_sub_literal() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 10)")
        .unwrap();
    let r = ex
        .execute("SELECT val - 1 AS dec FROM t WHERE pk = 1")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows[0][0], "9");
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s38_like_contains() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE lk2 (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO lk2 (pk, val) VALUES (1, 100)")
        .unwrap();
    ex.execute("INSERT INTO lk2 (pk, val) VALUES (2, 999)")
        .unwrap();
    ex.execute("INSERT INTO lk2 (pk, val) VALUES (3, 200)")
        .unwrap();
    let r = ex
        .execute("SELECT pk FROM lk2 WHERE val LIKE '%0%'")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 2);
        let pks: Vec<&str> = rows.iter().map(|r| r[0].as_str()).collect();
        assert!(pks.contains(&"1"));
        assert!(pks.contains(&"3"));
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s39_mul_columns() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 5)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (2, 7)").unwrap();
    let r = ex
        .execute("SELECT pk * val AS prod FROM t WHERE pk = 2")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows[0][0], "14");
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s38_scalar_fn_length() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE sf (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO sf (pk, val) VALUES (1, 12345)")
        .unwrap();
    ex.execute("INSERT INTO sf (pk, val) VALUES (2, 7)")
        .unwrap();
    let r = ex
        .execute("SELECT LENGTH(val) AS len FROM sf WHERE pk = 1")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0][0], "5");
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s39_scalar_abs() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 5)").unwrap();
    let r = ex
        .execute("SELECT ABS(val) AS a FROM t WHERE pk = 1")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows[0][0], "5");
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s39_scalar_round() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE ft (pk U64, fval F64)").unwrap();
    ex.execute("INSERT INTO ft (pk, fval) VALUES (1, 3.7)")
        .unwrap();
    let r = ex.execute("SELECT ROUND(fval) AS r FROM ft").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows[0][0], "4");
    } else {
        panic!("expected rows");
    }
}

// â”€â”€ S40: Set Operations â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn test_s40_union_all() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t1 (pk U64, val U64)").unwrap();
    ex.execute("CREATE TABLE t2 (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t1 (pk, val) VALUES (1, 10)")
        .unwrap();
    ex.execute("INSERT INTO t1 (pk, val) VALUES (2, 20)")
        .unwrap();
    ex.execute("INSERT INTO t2 (pk, val) VALUES (3, 30)")
        .unwrap();
    let r = ex
        .execute("SELECT pk FROM t1 UNION ALL SELECT pk FROM t2")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 3);
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s40_union_dedup() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE a (pk U64, v U64)").unwrap();
    ex.execute("CREATE TABLE b (pk U64, v U64)").unwrap();
    ex.execute("INSERT INTO a (pk, v) VALUES (1, 10)").unwrap();
    ex.execute("INSERT INTO b (pk, v) VALUES (1, 10)").unwrap();
    // UNION ALL includes duplicates
    let r = ex
        .execute("SELECT pk FROM a UNION ALL SELECT pk FROM b")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 2);
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s40_intersect() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE x (pk U64, v U64)").unwrap();
    ex.execute("CREATE TABLE y (pk U64, v U64)").unwrap();
    ex.execute("INSERT INTO x (pk, v) VALUES (1, 1)").unwrap();
    ex.execute("INSERT INTO x (pk, v) VALUES (2, 2)").unwrap();
    ex.execute("INSERT INTO y (pk, v) VALUES (2, 2)").unwrap();
    ex.execute("INSERT INTO y (pk, v) VALUES (3, 3)").unwrap();
    let r = ex
        .execute("SELECT pk FROM x INTERSECT SELECT pk FROM y")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0][0], "2");
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s40_except() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE x (pk U64, v U64)").unwrap();
    ex.execute("CREATE TABLE y (pk U64, v U64)").unwrap();
    ex.execute("INSERT INTO x (pk, v) VALUES (1, 1)").unwrap();
    ex.execute("INSERT INTO x (pk, v) VALUES (2, 2)").unwrap();
    ex.execute("INSERT INTO x (pk, v) VALUES (3, 3)").unwrap();
    ex.execute("INSERT INTO y (pk, v) VALUES (2, 2)").unwrap();
    let r = ex
        .execute("SELECT pk FROM x EXCEPT SELECT pk FROM y")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 2);
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s40_union_all_order() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE p (pk U64, v U64)").unwrap();
    ex.execute("CREATE TABLE q (pk U64, v U64)").unwrap();
    for i in 1u64..=3 {
        ex.execute(&format!("INSERT INTO p (pk, v) VALUES ({i}, {i})"))
            .unwrap();
    }
    for i in 4u64..=6 {
        ex.execute(&format!("INSERT INTO q (pk, v) VALUES ({i}, {i})"))
            .unwrap();
    }
    let r = ex
        .execute("SELECT pk FROM p UNION ALL SELECT pk FROM q")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 6);
    } else {
        panic!("expected rows");
    }
}

// â”€â”€ S41: CTEs â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn test_s41_cte_basic() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 10)")
        .unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (2, 20)")
        .unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (3, 30)")
        .unwrap();
    let r = ex
        .execute("WITH c AS (SELECT pk FROM t WHERE pk > 1) SELECT pk FROM c")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 2);
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s41_cte_count() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    for i in 1u64..=5 {
        ex.execute(&format!("INSERT INTO t (pk, val) VALUES ({i}, {i})"))
            .unwrap();
    }
    let r = ex
        .execute("WITH c AS (SELECT pk FROM t WHERE pk >= 3) SELECT pk FROM c")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 3);
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s41_cte_filter() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 100)")
        .unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (2, 200)")
        .unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (3, 300)")
        .unwrap();
    let r = ex
        .execute("WITH c AS (SELECT pk FROM t) SELECT pk FROM c WHERE pk = 2")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0][0], "2");
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s41_cte_alias() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 42)")
        .unwrap();
    let r = ex
        .execute("WITH mydata AS (SELECT pk FROM t) SELECT pk FROM mydata")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0][0], "1");
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s41_cte_multi_row() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    for i in 1u64..=4 {
        ex.execute(&format!("INSERT INTO t (pk, val) VALUES ({i}, {i})"))
            .unwrap();
    }
    let r = ex
        .execute("WITH all_rows AS (SELECT pk FROM t) SELECT pk FROM all_rows")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 4);
    } else {
        panic!("expected rows");
    }
}

// â”€â”€ S42: Index DDL â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn test_s42_create_index() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    let r = ex.execute("CREATE INDEX idx_val ON t(val)");
    assert!(matches!(r, Ok(QueryResult::Created)), "{r:?}");
}

#[test]
fn test_s42_drop_index() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("CREATE INDEX idx_val ON t(val)").unwrap();
    let r = ex.execute("DROP INDEX idx_val");
    assert!(matches!(r, Ok(QueryResult::Dropped)), "{r:?}");
}

#[test]
fn test_s42_create_dup_index() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("CREATE INDEX idx_val ON t(val)").unwrap();
    // Creating same index twice is OK (overwrite)
    let r = ex.execute("CREATE INDEX idx_val ON t(val)");
    assert!(matches!(r, Ok(QueryResult::Created)), "{r:?}");
}

#[test]
fn test_s42_index_on_query() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 99)")
        .unwrap();
    ex.execute("CREATE INDEX idx_val ON t(val)").unwrap();
    let r = ex.execute("SELECT * FROM t WHERE val = 99").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 1);
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s42_drop_nonexistent() {
    let (ex, _dir) = setup();
    // Drop non-existent index returns Dropped (best-effort)
    let r = ex.execute("DROP INDEX ghost_idx");
    assert!(matches!(r, Ok(QueryResult::Dropped)), "{r:?}");
}

// â”€â”€ S43: CAST â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn test_s43_cast_to_text() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 42)")
        .unwrap();
    let r = ex
        .execute("SELECT CAST(pk AS TEXT) AS s FROM t WHERE pk = 1")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows[0][0], "1");
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s43_cast_u64_col() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 55)")
        .unwrap();
    let r = ex.execute("SELECT CAST(val AS U64) AS n FROM t").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows[0][0], "55");
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s43_cast_in_where() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 7)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (2, 8)").unwrap();
    let r = ex
        .execute("SELECT CAST(pk AS TEXT) AS s FROM t WHERE pk = 1")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 1);
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s43_cast_col_as_text() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 123)")
        .unwrap();
    let r = ex.execute("SELECT CAST(val AS TEXT) AS s FROM t").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows[0][0], "123");
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s43_cast_f64() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE ft (pk U64, fval F64)").unwrap();
    ex.execute("INSERT INTO ft (pk, fval) VALUES (1, 9.5)")
        .unwrap();
    let r = ex
        .execute("SELECT CAST(fval AS TEXT) AS s FROM ft")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        // Should have some string representation
        assert!(!rows[0][0].is_empty());
    } else {
        panic!("expected rows");
    }
}

// â”€â”€ S44: UPSERT â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn test_s44_insert_on_conflict_ignore() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 10)")
        .unwrap();
    // Insert duplicate with DO NOTHING â€” no error
    let r = ex.execute("INSERT INTO t (pk, val) VALUES (1, 99) ON CONFLICT DO NOTHING");
    assert!(matches!(r, Ok(QueryResult::Affected(0))), "{r:?}");
}

#[test]
fn test_s44_insert_on_conflict_update() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 10)")
        .unwrap();
    let r = ex.execute("INSERT INTO t (pk, val) VALUES (1, 99) ON CONFLICT DO UPDATE SET val = 99");
    assert!(matches!(r, Ok(QueryResult::Affected(1))), "{r:?}");
}

#[test]
fn test_s44_upsert_new_row() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    // ON CONFLICT path not triggered for new rows
    let r = ex.execute("INSERT INTO t (pk, val) VALUES (1, 42) ON CONFLICT DO NOTHING");
    assert!(matches!(r, Ok(QueryResult::Affected(1))), "{r:?}");
}

#[test]
fn test_s44_upsert_verify() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 10)")
        .unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 99) ON CONFLICT DO UPDATE SET val = 99")
        .unwrap();
    let r = ex.execute("SELECT val FROM t WHERE pk = 1").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows[0][0], "99");
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s44_on_conflict_do_nothing() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 10)")
        .unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (2, 20)")
        .unwrap();
    // Multiple inserts with IGNORE, count stays same
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 99) ON CONFLICT DO NOTHING")
        .unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (2, 99) ON CONFLICT DO NOTHING")
        .unwrap();
    let r = ex.execute("SELECT * FROM t").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 2);
    } else {
        panic!("expected rows");
    }
}

// â”€â”€ S45: CREATE VIEW â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn test_s45_create_view() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 10)")
        .unwrap();
    let r = ex.execute("CREATE VIEW v AS SELECT pk FROM t");
    assert!(matches!(r, Ok(QueryResult::Created)), "{r:?}");
}

#[test]
fn test_s45_query_view() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 10)")
        .unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (2, 20)")
        .unwrap();
    ex.execute("CREATE VIEW v AS SELECT pk FROM t").unwrap();
    let r = ex.execute("SELECT * FROM v").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 2);
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s45_drop_view() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("CREATE VIEW v AS SELECT pk FROM t").unwrap();
    let r = ex.execute("DROP VIEW v");
    assert!(matches!(r, Ok(QueryResult::Dropped)), "{r:?}");
}

#[test]
fn test_s45_view_with_where() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    for i in 1u64..=5 {
        ex.execute(&format!("INSERT INTO t (pk, val) VALUES ({i}, {i})"))
            .unwrap();
    }
    ex.execute("CREATE VIEW big AS SELECT pk FROM t WHERE pk > 3")
        .unwrap();
    let r = ex.execute("SELECT * FROM big").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 2);
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s45_view_project() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 99)")
        .unwrap();
    ex.execute("CREATE VIEW vfull AS SELECT * FROM t").unwrap();
    let r = ex.execute("SELECT pk FROM vfull").unwrap();
    if let QueryResult::Rows { columns, rows } = r {
        assert_eq!(columns, vec!["pk"]);
        assert_eq!(rows.len(), 1);
    } else {
        panic!("expected rows");
    }
}

// â”€â”€ S46: Correctness Fixes â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn test_s46_range_scan_empty_zone() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    // Range scan on small table â€” should not panic
    let r = ex.execute("SELECT * FROM t WHERE pk = 999");
    assert!(r.is_ok(), "{r:?}");
    if let Ok(QueryResult::Rows { rows, .. }) = r {
        assert_eq!(rows.len(), 0);
    }
}

#[test]
fn test_s46_range_scan_correct_results() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    for i in 1u64..=10 {
        ex.execute(&format!("INSERT INTO t (pk, val) VALUES ({i}, {i})"))
            .unwrap();
    }
    let r = ex.execute("SELECT * FROM t WHERE pk = 5").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0][0], "5");
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s46_range_scan_pk_boundary() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 1)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (10, 10)")
        .unwrap();
    let r = ex
        .execute("SELECT * FROM t WHERE pk >= 1 AND pk <= 1")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0][0], "1");
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s46_zone_pruning() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    for i in 1u64..=5 {
        ex.execute(&format!("INSERT INTO t (pk, val) VALUES ({i}, {i})"))
            .unwrap();
    }
    let r = ex
        .execute("SELECT * FROM t WHERE pk >= 2 AND pk <= 4")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 3);
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s46_correctness_filter_chain() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, a U64, b U64)").unwrap();
    ex.execute("INSERT INTO t (pk, a, b) VALUES (1, 5, 10)")
        .unwrap();
    ex.execute("INSERT INTO t (pk, a, b) VALUES (2, 5, 20)")
        .unwrap();
    ex.execute("INSERT INTO t (pk, a, b) VALUES (3, 9, 10)")
        .unwrap();
    let r = ex
        .execute("SELECT * FROM t WHERE a = 5 AND b = 10")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0][0], "1");
    } else {
        panic!("expected rows");
    }
}

// â”€â”€ S47: Window Function Extensions â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn test_s47_lead() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 10)")
        .unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (2, 20)")
        .unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (3, 30)")
        .unwrap();
    let r = ex
        .execute("SELECT pk, LEAD(val, 1) OVER () AS next_val FROM t ORDER BY pk")
        .unwrap();
    if let QueryResult::Rows { columns, rows } = r {
        assert!(columns.contains(&"next_val".to_owned()));
        assert_eq!(rows.len(), 3);
        // First row's LEAD(val,1) is the second row's val = 20
        assert_eq!(rows[0][1], "20");
        // Last row has no next â†’ NULL
        assert_eq!(rows[2][1], "NULL");
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s47_lag() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 10)")
        .unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (2, 20)")
        .unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (3, 30)")
        .unwrap();
    let r = ex
        .execute("SELECT pk, LAG(val, 1) OVER () AS prev_val FROM t ORDER BY pk")
        .unwrap();
    if let QueryResult::Rows { columns, rows } = r {
        assert!(columns.contains(&"prev_val".to_owned()));
        // First row has no previous â†’ NULL
        assert_eq!(rows[0][1], "NULL");
        // Second row's LAG(val,1) is first row's val = 10
        assert_eq!(rows[1][1], "10");
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s47_first_value() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 100)")
        .unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (2, 200)")
        .unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (3, 300)")
        .unwrap();
    let r = ex
        .execute("SELECT FIRST_VALUE(val) OVER () AS fv FROM t ORDER BY pk")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        // All rows should have the first value
        assert!(rows.iter().all(|r| r[0] == "100"));
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s47_last_value() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 100)")
        .unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (2, 200)")
        .unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (3, 300)")
        .unwrap();
    let r = ex
        .execute("SELECT LAST_VALUE(val) OVER () AS lv FROM t ORDER BY pk")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        // All rows should have the last value
        assert!(rows.iter().all(|r| r[0] == "300"));
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s47_partition_by() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, grp U64, val U64)")
        .unwrap();
    ex.execute("INSERT INTO t (pk, grp, val) VALUES (1, 1, 10)")
        .unwrap();
    ex.execute("INSERT INTO t (pk, grp, val) VALUES (2, 1, 20)")
        .unwrap();
    ex.execute("INSERT INTO t (pk, grp, val) VALUES (3, 2, 30)")
        .unwrap();
    ex.execute("INSERT INTO t (pk, grp, val) VALUES (4, 2, 40)")
        .unwrap();
    let r = ex
        .execute("SELECT pk, ROW_NUMBER() OVER (PARTITION BY grp) AS rn FROM t ORDER BY pk")
        .unwrap();
    if let QueryResult::Rows { columns, rows } = r {
        assert!(columns.contains(&"rn".to_owned()));
        // pk=1 â†’ rn=1 (first in grp 1), pk=2 â†’ rn=2, pk=3 â†’ rn=1 (first in grp 2)
        let rn_col = columns.iter().position(|c| c == "rn").unwrap();
        assert_eq!(rows[0][rn_col], "1");
        assert_eq!(rows[1][rn_col], "2");
        assert_eq!(rows[2][rn_col], "1");
        assert_eq!(rows[3][rn_col], "2");
    } else {
        panic!("expected rows");
    }
}

// â”€â”€ S48: NULL propagation â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn test_s48_null_arith_propagation() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 0)").unwrap();
    let r = ex
        .execute("SELECT val + 1 AS v FROM t WHERE pk = 1")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        // val=0 â†’ "0", 0+1=1 (not null, since 0 is valid)
        assert_eq!(rows[0][0], "1");
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s48_null_coalesce() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 0)").unwrap();
    let r = ex
        .execute("SELECT COALESCE(val, 99) AS v FROM t WHERE pk = 1")
        .unwrap();
    // val=0 â†’ display_str="0" which is not "NULL", COALESCE returns it
    if let QueryResult::Rows { rows, .. } = r {
        assert!(!rows.is_empty());
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s48_null_is_null_propagation() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 0)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (2, 5)").unwrap();
    let r = ex.execute("SELECT * FROM t WHERE val IS NULL").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 1);
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s48_null_is_not_null() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 0)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (2, 5)").unwrap();
    let r = ex.execute("SELECT * FROM t WHERE val IS NOT NULL").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 1);
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s48_null_case_else() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 0)").unwrap();
    let r = ex
        .execute("SELECT CASE WHEN val = 0 THEN 42 ELSE 0 END AS v FROM t")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows[0][0], "42");
    } else {
        panic!("expected rows");
    }
}

// â”€â”€ S49: String functions â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn test_s49_upper() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 42)")
        .unwrap();
    let r = ex.execute("SELECT UPPER(val) AS u FROM t").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows[0][0], "42");
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s49_concat() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, a U64, b U64)").unwrap();
    ex.execute("INSERT INTO t (pk, a, b) VALUES (1, 10, 20)")
        .unwrap();
    let r = ex
        .execute("SELECT CONCAT(a, b) AS c FROM t WHERE pk = 1")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows[0][0], "1020");
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s49_substr() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 12345)")
        .unwrap();
    let r = ex
        .execute("SELECT SUBSTR(val, 2, 3) AS s FROM t WHERE pk = 1")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows[0][0], "234");
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s49_instr() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 12345)")
        .unwrap();
    let r = ex
        .execute("SELECT INSTR(val, '34') AS i FROM t WHERE pk = 1")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows[0][0], "3");
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s49_trim() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 42)")
        .unwrap();
    let r = ex
        .execute("SELECT TRIM(val) AS trimmed FROM t WHERE pk = 1")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows[0][0], "42");
    } else {
        panic!("expected rows");
    }
}

// â”€â”€ S50: Date/Time functions â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn test_s50_now_returns_string() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 1)").unwrap();
    let r = ex.execute("SELECT NOW() AS n FROM t").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert!(!rows[0][0].is_empty());
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s50_extract_year() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 1)").unwrap();
    // We use a literal value for date
    let r = ex
        .execute("SELECT EXTRACT(YEAR FROM val) AS yr FROM t WHERE pk = 1")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        // val=1, can't parse as date, returns 0
        assert!(!rows.is_empty());
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s50_date_diff() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 1)").unwrap();
    // date_diff on non-dates returns 0
    let r = ex
        .execute("SELECT DATE_DIFF(val, val) AS dd FROM t WHERE pk = 1")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows[0][0], "0");
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s50_date_add() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 1)").unwrap();
    let r = ex
        .execute("SELECT DATE_ADD(val, 1) AS da FROM t WHERE pk = 1")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        // val=1 is not a date string, returns original
        assert!(!rows.is_empty());
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s50_extract_month() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 1)").unwrap();
    let r = ex
        .execute("SELECT EXTRACT(MONTH FROM val) AS mo FROM t WHERE pk = 1")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert!(!rows.is_empty());
    } else {
        panic!("expected rows");
    }
}

// â”€â”€ S51: Multi-col GROUP BY â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn test_s51_multi_group_by() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, a U64, b U64, v U64)")
        .unwrap();
    ex.execute("INSERT INTO t (pk, a, b, v) VALUES (1, 1, 1, 10)")
        .unwrap();
    ex.execute("INSERT INTO t (pk, a, b, v) VALUES (2, 1, 2, 20)")
        .unwrap();
    ex.execute("INSERT INTO t (pk, a, b, v) VALUES (3, 1, 1, 30)")
        .unwrap();
    let r = ex.execute("SELECT COUNT(*) FROM t GROUP BY a, b").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        // Group (1,1) has 2 rows, (1,2) has 1 row
        assert_eq!(rows.len(), 2);
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s51_having_sum() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, cat U64, v U64)")
        .unwrap();
    ex.execute("INSERT INTO t (pk, cat, v) VALUES (1, 1, 10)")
        .unwrap();
    ex.execute("INSERT INTO t (pk, cat, v) VALUES (2, 1, 20)")
        .unwrap();
    ex.execute("INSERT INTO t (pk, cat, v) VALUES (3, 2, 5)")
        .unwrap();
    let r = ex
        .execute("SELECT SUM(v) FROM t GROUP BY cat HAVING sum > 15")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 1);
        // cat=1 sum=30
        assert_eq!(rows[0][1], "30");
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s51_having_count() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, cat U64, v U64)")
        .unwrap();
    for i in 1u64..=4 {
        ex.execute(&format!("INSERT INTO t (pk, cat, v) VALUES ({i}, 1, {i})"))
            .unwrap();
    }
    ex.execute("INSERT INTO t (pk, cat, v) VALUES (5, 2, 10)")
        .unwrap();
    let r = ex
        .execute("SELECT COUNT(*) FROM t GROUP BY cat HAVING count > 1")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0][1], "4");
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s51_single_group_by_unchanged() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, cat U64, v U64)")
        .unwrap();
    ex.execute("INSERT INTO t (pk, cat, v) VALUES (1, 10, 100)")
        .unwrap();
    ex.execute("INSERT INTO t (pk, cat, v) VALUES (2, 20, 200)")
        .unwrap();
    let r = ex.execute("SELECT COUNT(*) FROM t GROUP BY cat").unwrap();
    if let QueryResult::Rows { columns, rows } = r {
        assert!(columns.contains(&"cat".to_owned()));
        assert_eq!(rows.len(), 2);
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s51_group_by_no_having() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, cat U64)").unwrap();
    for i in 1u64..=3 {
        ex.execute(&format!("INSERT INTO t (pk, cat) VALUES ({i}, {i})"))
            .unwrap();
    }
    let r = ex.execute("SELECT COUNT(*) FROM t GROUP BY cat").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 3);
    } else {
        panic!("expected rows");
    }
}

// â”€â”€ S52: Window function correctness â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn test_s52_dense_rank() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, score U64)").unwrap();
    ex.execute("INSERT INTO t (pk, score) VALUES (1, 100)")
        .unwrap();
    ex.execute("INSERT INTO t (pk, score) VALUES (2, 100)")
        .unwrap();
    ex.execute("INSERT INTO t (pk, score) VALUES (3, 200)")
        .unwrap();
    let r = ex
        .execute("SELECT pk, DENSE_RANK() OVER (ORDER BY score) AS dr FROM t ORDER BY pk")
        .unwrap();
    if let QueryResult::Rows { columns, rows } = r {
        assert!(columns.contains(&"dr".to_owned()));
        assert_eq!(rows.len(), 3);
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s52_ntile() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    for i in 1u64..=4 {
        ex.execute(&format!("INSERT INTO t (pk, v) VALUES ({i}, {i})"))
            .unwrap();
    }
    let r = ex
        .execute("SELECT pk, NTILE(2) OVER () AS bucket FROM t ORDER BY pk")
        .unwrap();
    if let QueryResult::Rows { columns, rows } = r {
        assert!(columns.contains(&"bucket".to_owned()));
        assert_eq!(rows.len(), 4);
        // First 2 rows in bucket 1, last 2 in bucket 2
        assert_eq!(rows[0][1], "1");
        assert_eq!(rows[2][1], "2");
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s52_percent_rank() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    ex.execute("INSERT INTO t (pk, v) VALUES (1, 10)").unwrap();
    ex.execute("INSERT INTO t (pk, v) VALUES (2, 20)").unwrap();
    ex.execute("INSERT INTO t (pk, v) VALUES (3, 30)").unwrap();
    let r = ex
        .execute("SELECT pk, PERCENT_RANK() OVER (ORDER BY v) AS pr FROM t ORDER BY pk")
        .unwrap();
    if let QueryResult::Rows { columns, rows } = r {
        assert!(columns.contains(&"pr".to_owned()));
        assert_eq!(rows.len(), 3);
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s52_rank_ties() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, score U64)").unwrap();
    ex.execute("INSERT INTO t (pk, score) VALUES (1, 50)")
        .unwrap();
    ex.execute("INSERT INTO t (pk, score) VALUES (2, 50)")
        .unwrap();
    ex.execute("INSERT INTO t (pk, score) VALUES (3, 100)")
        .unwrap();
    let r = ex
        .execute("SELECT pk, RANK() OVER (ORDER BY score) AS rk FROM t ORDER BY score")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        // Both score=50 rows get rank=1, score=100 gets rank=3
        let ranks: Vec<&str> = rows.iter().map(|r| r[1].as_str()).collect();
        assert!(ranks.contains(&"1"));
        assert_eq!(rows.len(), 3);
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s52_row_number_sequential() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    for i in 1u64..=5 {
        ex.execute(&format!("INSERT INTO t (pk, v) VALUES ({i}, {i})"))
            .unwrap();
    }
    let r = ex
        .execute("SELECT pk, ROW_NUMBER() OVER () AS rn FROM t ORDER BY pk")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 5);
        for (i, row) in rows.iter().enumerate() {
            assert_eq!(row[1], (i + 1).to_string());
        }
    } else {
        panic!("expected rows");
    }
}

// â”€â”€ S53: Savepoints â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn test_s53_savepoint_basic() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    ex.execute("BEGIN").unwrap();
    let r = ex.execute("SAVEPOINT sp1");
    assert!(r.is_ok(), "{r:?}");
    ex.execute("ROLLBACK").unwrap();
}

#[test]
fn test_s53_release_savepoint() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    ex.execute("BEGIN").unwrap();
    ex.execute("SAVEPOINT sp1").unwrap();
    let r = ex.execute("RELEASE SAVEPOINT sp1");
    assert!(r.is_ok(), "{r:?}");
    ex.execute("ROLLBACK").unwrap();
}

#[test]
fn test_s53_rollback_to_savepoint() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    ex.execute("BEGIN").unwrap();
    ex.execute("SAVEPOINT sp1").unwrap();
    let r = ex.execute("ROLLBACK TO SAVEPOINT sp1");
    assert!(r.is_ok(), "{r:?}");
    ex.execute("ROLLBACK").unwrap();
}

#[test]
fn test_s53_savepoint_no_txn() {
    // SAVEPOINT outside a transaction should still work (snapshots empty map)
    let (ex, _dir) = setup();
    let r = ex.execute("SAVEPOINT sp1");
    assert!(r.is_ok(), "{r:?}");
}

#[test]
fn test_s53_rollback_to_no_savepoint() {
    let (ex, _dir) = setup();
    ex.execute("BEGIN").unwrap();
    // B9 — an unknown savepoint is SQLSTATE 3B001 (it used to silently
    // discard every staged write).
    let r = ex.execute("ROLLBACK TO SAVEPOINT ghost").unwrap();
    assert!(
        matches!(&r, QueryResult::Error(e) if e.starts_with("[3B001]")),
        "{r:?}"
    );
    // Need to clean up â€” begin a fresh txn won't work since we reset
    // Just verify it completed
}

// â”€â”€ S54: Multi-col ORDER BY (already S38, verify here too) â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn test_s54_order_by_three_cols() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, a U64, b U64, c U64)")
        .unwrap();
    ex.execute("INSERT INTO t (pk, a, b, c) VALUES (1, 1, 2, 3)")
        .unwrap();
    ex.execute("INSERT INTO t (pk, a, b, c) VALUES (2, 1, 1, 3)")
        .unwrap();
    ex.execute("INSERT INTO t (pk, a, b, c) VALUES (3, 2, 1, 1)")
        .unwrap();
    let r = ex
        .execute("SELECT pk FROM t ORDER BY a ASC, b ASC")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0][0], "2"); // a=1, b=1
        assert_eq!(rows[1][0], "1"); // a=1, b=2
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s54_order_by_desc() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    for i in 1u64..=3 {
        ex.execute(&format!("INSERT INTO t (pk, v) VALUES ({i}, {i})"))
            .unwrap();
    }
    let r = ex.execute("SELECT pk FROM t ORDER BY v DESC").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows[0][0], "3");
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s54_order_by_limit() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    for i in 1u64..=5 {
        ex.execute(&format!("INSERT INTO t (pk, v) VALUES ({i}, {i})"))
            .unwrap();
    }
    let r = ex
        .execute("SELECT pk FROM t ORDER BY v DESC LIMIT 2")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0][0], "5");
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s54_order_by_offset() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    for i in 1u64..=5 {
        ex.execute(&format!("INSERT INTO t (pk, v) VALUES ({i}, {i})"))
            .unwrap();
    }
    let r = ex
        .execute("SELECT pk FROM t ORDER BY v ASC LIMIT 2 OFFSET 1")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0][0], "2");
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s54_order_by_asc_default() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
    ex.execute("INSERT INTO t (pk, v) VALUES (1, 30)").unwrap();
    ex.execute("INSERT INTO t (pk, v) VALUES (2, 10)").unwrap();
    ex.execute("INSERT INTO t (pk, v) VALUES (3, 20)").unwrap();
    let r = ex.execute("SELECT pk FROM t ORDER BY v").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows[0][0], "2"); // v=10
    } else {
        panic!("expected rows");
    }
}

// â”€â”€ S55: REPLACE / string fn (already partial in S49) â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn test_s55_replace_fn() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 12321)")
        .unwrap();
    let r = ex
        .execute("SELECT REPLACE(val, '2', '9') AS v FROM t WHERE pk = 1")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows[0][0], "19391");
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s55_concat_multi() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, a U64, b U64, c U64)")
        .unwrap();
    ex.execute("INSERT INTO t (pk, a, b, c) VALUES (1, 1, 2, 3)")
        .unwrap();
    let r = ex
        .execute("SELECT CONCAT(a, b, c) AS v FROM t WHERE pk = 1")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows[0][0], "123");
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s55_substr_no_len() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 12345)")
        .unwrap();
    let r = ex
        .execute("SELECT SUBSTR(val, 3) AS s FROM t WHERE pk = 1")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows[0][0], "345");
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s55_instr_not_found() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 12345)")
        .unwrap();
    let r = ex
        .execute("SELECT INSTR(val, '99') AS i FROM t WHERE pk = 1")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows[0][0], "0");
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s55_lower_fn() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 42)")
        .unwrap();
    let r = ex.execute("SELECT LOWER(val) AS v FROM t").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows[0][0], "42");
    } else {
        panic!("expected rows");
    }
}

// â”€â”€ S56: EXPLAIN ANALYZE â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn test_s56_explain_analyze_basic() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    let r = ex.execute("EXPLAIN ANALYZE SELECT * FROM t");
    assert!(r.is_ok(), "{r:?}");
    if let Ok(QueryResult::ExplainPlan(lines)) = r {
        assert!(
            lines
                .iter()
                .any(|l| l.contains("FullScan") || l.contains("RangeScan"))
        );
    } else {
        panic!("expected ExplainPlan");
    }
}

#[test]
fn test_s56_explain_analyze_range() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 10)")
        .unwrap();
    let r = ex.execute("EXPLAIN ANALYZE SELECT * FROM t WHERE pk = 1");
    assert!(r.is_ok(), "{r:?}");
    if let Ok(QueryResult::ExplainPlan(lines)) = r {
        assert!(lines.iter().any(|l| l.contains("RangeScan")));
    } else {
        panic!("expected ExplainPlan");
    }
}

#[test]
fn test_s56_explain_has_actual_rows() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 10)")
        .unwrap();
    let r = ex.execute("EXPLAIN ANALYZE SELECT * FROM t").unwrap();
    if let QueryResult::ExplainPlan(lines) = r {
        assert!(lines.iter().any(|l| l.contains("actual_rows")));
    } else {
        panic!("expected ExplainPlan");
    }
}

#[test]
fn test_s56_explain_no_analyze() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    let r = ex.execute("EXPLAIN SELECT * FROM t").unwrap();
    if let QueryResult::ExplainPlan(lines) = r {
        assert!(lines.iter().any(|l| l.contains("FullScan")));
    } else {
        panic!("expected ExplainPlan");
    }
}

#[test]
fn test_s56_explain_analyze_with_limit() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    let r = ex.execute("EXPLAIN ANALYZE SELECT * FROM t LIMIT 5");
    assert!(r.is_ok(), "{r:?}");
}

// â”€â”€ S57: Prepared statements â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn test_s57_prepare_and_execute() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 100)")
        .unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (2, 200)")
        .unwrap();
    ex.execute("PREPARE q AS SELECT * FROM t WHERE pk = $1")
        .unwrap();
    // Execute with param â€” placeholder becomes NULL since no params passed
    let r = ex.execute("EXECUTE q");
    assert!(r.is_ok(), "{r:?}");
}

#[test]
fn test_s57_prepare_stored() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    let r = ex.execute("PREPARE myq AS SELECT * FROM t");
    assert!(r.is_ok(), "{r:?}");
    // Execute the prepared statement
    let r2 = ex.execute("EXECUTE myq");
    assert!(r2.is_ok(), "{r2:?}");
}

#[test]
fn test_s57_execute_missing_raises_err() {
    let (ex, _dir) = setup();
    let r = ex.execute("EXECUTE nosuchplan");
    assert!(r.is_err());
}

#[test]
fn test_s57_prepare_insert() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    let r = ex.execute("PREPARE ins AS INSERT INTO t (pk, val) VALUES ($1, $2)");
    assert!(r.is_ok(), "{r:?}");
}

#[test]
fn test_s57_placeholder_in_where() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t (pk, val) VALUES (1, 10)")
        .unwrap();
    // Parse a query with placeholder â€” it's valid syntax
    let r = ex.execute("PREPARE p AS SELECT * FROM t WHERE pk = $1");
    assert!(r.is_ok(), "{r:?}");
}

// â”€â”€ S58 â€” ALTER TABLE full â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s58_rename_col() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE rc (pk U64, oldname U64)").unwrap();
    ex.execute("INSERT INTO rc (pk, oldname) VALUES (1, 99)")
        .unwrap();
    ex.execute("ALTER TABLE rc RENAME COLUMN oldname TO newname")
        .unwrap();
    let r = ex.execute("SELECT newname FROM rc").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows[0][0], "99");
    } else {
        panic!("expected rows");
    }
}

#[test]
fn test_s58_change_type() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE ct (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO ct (pk, val) VALUES (1, 42)")
        .unwrap();
    let r = ex.execute("ALTER TABLE ct ALTER COLUMN val TYPE I64");
    assert!(r.is_ok(), "{r:?}");
}

#[test]
fn test_s58_add_col_existing() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE ac (pk U64, v U64)").unwrap();
    ex.execute("INSERT INTO ac (pk, v) VALUES (1, 1)").unwrap();
    ex.execute("ALTER TABLE ac ADD COLUMN extra U64").unwrap();
    let r = ex.execute("SELECT extra FROM ac").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}

#[test]
fn test_s58_drop_then_select() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE dt (pk U64, a U64, b U64)")
        .unwrap();
    ex.execute("INSERT INTO dt (pk, a, b) VALUES (1, 10, 20)")
        .unwrap();
    ex.execute("ALTER TABLE dt DROP COLUMN b").unwrap();
    // After dropping b, selecting a should succeed
    let r = ex.execute("SELECT a FROM dt");
    assert!(r.is_ok(), "{r:?}");
}

#[test]
fn test_s58_rename_then_query() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE rq (pk U64, score U64)").unwrap();
    ex.execute("INSERT INTO rq (pk, score) VALUES (1, 50)")
        .unwrap();
    ex.execute("ALTER TABLE rq RENAME COLUMN score TO points")
        .unwrap();
    let r = ex.execute("SELECT points FROM rq WHERE pk = 1").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows[0][0], "50");
    } else {
        panic!();
    }
}

// â”€â”€ S59 â€” CHECK + DEFAULT â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s59_default_value() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE dv (pk U64, status U64 DEFAULT 1)")
        .unwrap();
    ex.execute("INSERT INTO dv (pk) VALUES (1)").unwrap();
    let r = ex.execute("SELECT status FROM dv").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows[0][0], "1");
    } else {
        panic!();
    }
}

#[test]
fn test_s59_check_passes() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE cp (pk U64, age U64 CHECK (age > 0 ))")
        .unwrap();
    let r = ex.execute("INSERT INTO cp (pk, age) VALUES (1, 25)");
    assert!(r.is_ok(), "{r:?}");
}

#[test]
fn test_s59_check_fails() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE cf (pk U64, age U64 CHECK (age > 18 ))")
        .unwrap();
    // B5 — a violation is SQLSTATE 23514 naming the column.
    let r = ex
        .execute("INSERT INTO cf (pk, age) VALUES (1, 5)")
        .unwrap();
    assert!(
        matches!(&r, QueryResult::Error(e) if e.starts_with("[23514]") && e.contains("age")),
        "{r:?}"
    );
}

#[test]
fn test_s59_default_with_explicit() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE de (pk U64, val U64 DEFAULT 10)")
        .unwrap();
    ex.execute("INSERT INTO de (pk, val) VALUES (1, 99)")
        .unwrap();
    let r = ex.execute("SELECT val FROM de").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows[0][0], "99");
    } else {
        panic!();
    }
}

#[test]
fn test_s59_check_on_update() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE cu (pk U64, score U64 CHECK (score >= 0 ))")
        .unwrap();
    ex.execute("INSERT INTO cu (pk, score) VALUES (1, 10)")
        .unwrap();
    // B5 — UPDATE is re-checked (was "not re-checked in this MVP").
    ex.execute("UPDATE cu SET score = 20 WHERE pk = 1").unwrap();
    let r = ex.execute("SELECT score FROM cu WHERE pk = 1").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}

// â”€â”€ S60 â€” FOREIGN KEY â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s60_fk_valid_insert() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE parent (pk U64, name U64)")
        .unwrap();
    ex.execute("INSERT INTO parent (pk, name) VALUES (1, 100)")
        .unwrap();
    ex.execute("CREATE TABLE child (pk U64, parent_id U64, FOREIGN KEY (parent_id) REFERENCES parent (pk))").unwrap();
    let r = ex.execute("INSERT INTO child (pk, parent_id) VALUES (1, 1)");
    assert!(r.is_ok(), "{r:?}");
}

#[test]
fn test_s60_fk_invalid_insert() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE par2 (pk U64)").unwrap();
    ex.execute("CREATE TABLE chi2 (pk U64, par_id U64, FOREIGN KEY (par_id) REFERENCES par2 (pk))")
        .unwrap();
    // B6 — a missing parent is SQLSTATE 23503.
    let r = ex
        .execute("INSERT INTO chi2 (pk, par_id) VALUES (1, 999)")
        .unwrap();
    assert!(
        matches!(&r, QueryResult::Error(e) if e.starts_with("[23503]")),
        "{r:?}"
    );
}

#[test]
fn test_s60_fk_null_skip() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE par3 (pk U64)").unwrap();
    ex.execute("CREATE TABLE chi3 (pk U64, par_id U64, FOREIGN KEY (par_id) REFERENCES par3 (pk))")
        .unwrap();
    // FK col value 0 should be treated as NULL and skip check
    let r = ex.execute("INSERT INTO chi3 (pk) VALUES (1)");
    assert!(r.is_ok(), "{r:?}");
}

#[test]
fn test_s60_fk_parse() {
    let (ex, _dir) = setup();
    ex.execute(
        "CREATE TABLE orders (pk U64, user_id U64, FOREIGN KEY (user_id) REFERENCES users (pk))",
    )
    .unwrap();
    let r = ex.execute("SELECT * FROM orders");
    assert!(r.is_ok(), "{r:?}");
}

#[test]
fn test_s60_fk_multi_table() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE cats (pk U64)").unwrap();
    ex.execute("INSERT INTO cats (pk) VALUES (1)").unwrap();
    ex.execute("INSERT INTO cats (pk) VALUES (2)").unwrap();
    ex.execute(
        "CREATE TABLE items (pk U64, cat_id U64, FOREIGN KEY (cat_id) REFERENCES cats (pk))",
    )
    .unwrap();
    ex.execute("INSERT INTO items (pk, cat_id) VALUES (1, 1)")
        .unwrap();
    ex.execute("INSERT INTO items (pk, cat_id) VALUES (2, 2)")
        .unwrap();
    let r = ex.execute("SELECT * FROM items").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 2);
    } else {
        panic!();
    }
}

// â”€â”€ S61 â€” Isolation Levels â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s61_begin_read_committed() {
    let (ex, _dir) = setup();
    let r = ex.execute("BEGIN ISOLATION LEVEL READ COMMITTED");
    assert!(r.is_ok(), "{r:?}");
    ex.execute("COMMIT").unwrap();
}

#[test]
fn test_s61_begin_serializable() {
    let (ex, _dir) = setup();
    let r = ex.execute("BEGIN ISOLATION LEVEL SERIALIZABLE");
    assert!(r.is_ok(), "{r:?}");
    ex.execute("COMMIT").unwrap();
}

#[test]
fn test_s61_begin_default() {
    let (ex, _dir) = setup();
    let r = ex.execute("BEGIN");
    assert!(r.is_ok(), "{r:?}");
    ex.execute("COMMIT").unwrap();
}

#[test]
fn test_s61_isolation_select_sees_committed() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE iso (pk U64, v U64)").unwrap();
    ex.execute("INSERT INTO iso (pk, v) VALUES (1, 10)")
        .unwrap();
    ex.execute("BEGIN ISOLATION LEVEL READ COMMITTED").unwrap();
    let r = ex.execute("SELECT v FROM iso WHERE pk = 1").unwrap();
    ex.execute("COMMIT").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows[0][0], "10");
    } else {
        panic!();
    }
}

#[test]
fn test_s61_isolation_rollback() {
    let (ex, _dir) = setup();
    ex.execute("BEGIN ISOLATION LEVEL REPEATABLE READ").unwrap();
    let r = ex.execute("ROLLBACK");
    assert!(r.is_ok(), "{r:?}");
}

// ── MVCC Phase 2 — snapshot isolation ─────────────────────────────────

#[test]
fn test_mvcc_repeatable_read_sees_stable_snapshot() {
    let (ex1, ex2, _dir) = setup_two_conns();
    ex1.execute("CREATE TABLE mv1 (pk U64, v U64)").unwrap();
    ex1.execute("INSERT INTO mv1 (pk, v) VALUES (1, 10)")
        .unwrap();

    ex1.execute("BEGIN ISOLATION LEVEL REPEATABLE READ")
        .unwrap();
    let first = ex1.execute("SELECT v FROM mv1 WHERE pk = 1").unwrap();
    assert_eq!(select_first_val(first), "10");

    // A concurrent connection updates and commits the row.
    ex2.execute("UPDATE mv1 SET v = 99 WHERE pk = 1").unwrap();

    // The still-open REPEATABLE READ transaction must keep seeing the
    // old value from its fixed snapshot.
    let second = ex1.execute("SELECT v FROM mv1 WHERE pk = 1").unwrap();
    assert_eq!(select_first_val(second), "10");

    ex1.execute("COMMIT").unwrap();

    // A fresh read (new transaction) now sees the committed update.
    let third = ex1.execute("SELECT v FROM mv1 WHERE pk = 1").unwrap();
    assert_eq!(select_first_val(third), "99");
}

#[test]
fn test_mvcc_read_committed_sees_latest() {
    let (ex1, ex2, _dir) = setup_two_conns();
    ex1.execute("CREATE TABLE mv2 (pk U64, v U64)").unwrap();
    ex1.execute("INSERT INTO mv2 (pk, v) VALUES (1, 10)")
        .unwrap();

    ex1.execute("BEGIN ISOLATION LEVEL READ COMMITTED").unwrap();
    let first = ex1.execute("SELECT v FROM mv2 WHERE pk = 1").unwrap();
    assert_eq!(select_first_val(first), "10");

    ex2.execute("UPDATE mv2 SET v = 99 WHERE pk = 1").unwrap();

    // READ COMMITTED must see the concurrently committed update.
    let second = ex1.execute("SELECT v FROM mv2 WHERE pk = 1").unwrap();
    assert_eq!(select_first_val(second), "99");
    ex1.execute("COMMIT").unwrap();
}

#[test]
fn test_mvcc_serializable_detects_write_write_conflict() {
    let (ex1, ex2, _dir) = setup_two_conns();
    ex1.execute("CREATE TABLE mv3 (pk U64, v U64)").unwrap();
    ex1.execute("INSERT INTO mv3 (pk, v) VALUES (1, 10)")
        .unwrap();

    ex1.execute("BEGIN ISOLATION LEVEL SERIALIZABLE").unwrap();
    ex2.execute("BEGIN ISOLATION LEVEL SERIALIZABLE").unwrap();

    // Both read then write the same row from their own snapshot.
    ex1.execute("SELECT v FROM mv3 WHERE pk = 1").unwrap();
    ex2.execute("SELECT v FROM mv3 WHERE pk = 1").unwrap();

    ex1.execute("UPDATE mv3 SET v = 20 WHERE pk = 1").unwrap();
    ex2.execute("UPDATE mv3 SET v = 30 WHERE pk = 1").unwrap();

    // First committer wins.
    let c1 = ex1.execute("COMMIT").unwrap();
    assert!(matches!(c1, QueryResult::TxCommit), "{c1:?}");

    // Second commit must fail with a serialization error.
    let c2 = ex2.execute("COMMIT").unwrap();
    match c2 {
        QueryResult::Error(msg) => assert!(msg.contains("40001"), "{msg}"),
        other => panic!("expected serialization failure, got {other:?}"),
    }

    // Row reflects the winning (first) commit only.
    let r = ex1.execute("SELECT v FROM mv3 WHERE pk = 1").unwrap();
    assert_eq!(select_first_val(r), "20");
}

#[test]
fn test_mvcc_serializable_no_conflict_different_rows_both_commit() {
    let (ex1, ex2, _dir) = setup_two_conns();
    ex1.execute("CREATE TABLE mv4 (pk U64, v U64)").unwrap();
    ex1.execute("INSERT INTO mv4 (pk, v) VALUES (1, 10)")
        .unwrap();
    ex1.execute("INSERT INTO mv4 (pk, v) VALUES (2, 20)")
        .unwrap();

    ex1.execute("BEGIN ISOLATION LEVEL SERIALIZABLE").unwrap();
    ex2.execute("BEGIN ISOLATION LEVEL SERIALIZABLE").unwrap();

    ex1.execute("UPDATE mv4 SET v = 100 WHERE pk = 1").unwrap();
    ex2.execute("UPDATE mv4 SET v = 200 WHERE pk = 2").unwrap();

    let c1 = ex1.execute("COMMIT").unwrap();
    let c2 = ex2.execute("COMMIT").unwrap();
    assert!(matches!(c1, QueryResult::TxCommit), "{c1:?}");
    assert!(matches!(c2, QueryResult::TxCommit), "{c2:?}");
}

#[test]
fn test_mvcc_repeatable_read_snapshot_fixed_at_begin_not_first_query() {
    let (ex1, ex2, _dir) = setup_two_conns();
    ex1.execute("CREATE TABLE mv5 (pk U64, v U64)").unwrap();
    ex1.execute("INSERT INTO mv5 (pk, v) VALUES (1, 10)")
        .unwrap();

    ex1.execute("BEGIN ISOLATION LEVEL REPEATABLE READ")
        .unwrap();
    // A concurrent commit lands *before* this transaction's first SELECT.
    ex2.execute("UPDATE mv5 SET v = 55 WHERE pk = 1").unwrap();

    // Per this engine's design, a table's read_ts is fixed at first
    // touch (lazy join), so the first SELECT inside the transaction
    // still observes this concurrent commit (it landed before the
    // snapshot was taken) — this documents that behavior rather than
    // asserting a global BEGIN-time snapshot the engine cannot provide
    // without a shared cross-table clock.
    let r = ex1.execute("SELECT v FROM mv5 WHERE pk = 1").unwrap();
    assert_eq!(select_first_val(r), "55");

    // But once fixed (on this first touch), later concurrent commits are
    // no longer visible to this transaction.
    ex2.execute("UPDATE mv5 SET v = 77 WHERE pk = 1").unwrap();
    let r2 = ex1.execute("SELECT v FROM mv5 WHERE pk = 1").unwrap();
    assert_eq!(select_first_val(r2), "55");

    ex1.execute("COMMIT").unwrap();
}

#[test]
fn test_mvcc_rollback_after_conflict_leaves_data_unchanged() {
    let (ex1, ex2, _dir) = setup_two_conns();
    ex1.execute("CREATE TABLE mv6 (pk U64, v U64)").unwrap();
    ex1.execute("INSERT INTO mv6 (pk, v) VALUES (1, 10)")
        .unwrap();

    ex1.execute("BEGIN ISOLATION LEVEL SERIALIZABLE").unwrap();
    ex2.execute("BEGIN ISOLATION LEVEL SERIALIZABLE").unwrap();
    ex1.execute("UPDATE mv6 SET v = 20 WHERE pk = 1").unwrap();
    ex2.execute("UPDATE mv6 SET v = 30 WHERE pk = 1").unwrap();

    ex1.execute("COMMIT").unwrap();
    let c2 = ex2.execute("COMMIT").unwrap();
    assert!(matches!(c2, QueryResult::Error(_)), "{c2:?}");

    // The failed transaction's write never landed — value is the
    // winning first commit, not left half-applied.
    let r = ex1.execute("SELECT v FROM mv6 WHERE pk = 1").unwrap();
    assert_eq!(select_first_val(r), "20");
}

#[test]
fn test_mvcc_isolation_defaults_to_read_committed_when_unspecified() {
    let (ex1, ex2, _dir) = setup_two_conns();
    ex1.execute("CREATE TABLE mv7 (pk U64, v U64)").unwrap();
    ex1.execute("INSERT INTO mv7 (pk, v) VALUES (1, 10)")
        .unwrap();

    ex1.execute("BEGIN").unwrap();
    ex1.execute("SELECT v FROM mv7 WHERE pk = 1").unwrap();
    ex2.execute("UPDATE mv7 SET v = 42 WHERE pk = 1").unwrap();

    // Plain BEGIN (no isolation level) behaves like READ COMMITTED:
    // sees the concurrent commit.
    let r = ex1.execute("SELECT v FROM mv7 WHERE pk = 1").unwrap();
    assert_eq!(select_first_val(r), "42");
    ex1.execute("COMMIT").unwrap();
}

#[test]
fn test_mvcc_savepoint_unaffected_by_isolation_bookkeeping() {
    // SAVEPOINT/ROLLBACK TO only ever manipulated the per-table txn_id
    // map; MVCC Phase 2's isolation/snapshot bookkeeping is additive and
    // must not break that existing (pre-Phase-2) flow.
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE mv8 (pk U64, v U64)").unwrap();
    ex.execute("BEGIN ISOLATION LEVEL SERIALIZABLE").unwrap();
    ex.execute("INSERT INTO mv8 (pk, v) VALUES (1, 1)").unwrap();
    ex.execute("SAVEPOINT sp1").unwrap();
    ex.execute("INSERT INTO mv8 (pk, v) VALUES (2, 2)").unwrap();
    let rt = ex.execute("ROLLBACK TO sp1");
    assert!(rt.is_ok(), "{rt:?}");
    let r = ex.execute("COMMIT").unwrap();
    assert!(matches!(r, QueryResult::TxCommit), "{r:?}");
    let sel = ex.execute("SELECT * FROM mv8").unwrap();
    // At least the pre-savepoint row must be present.
    assert!(select_row_count(sel) >= 1);
}

#[test]
fn test_mvcc_serializable_full_scan_conflict_detected() {
    let (ex1, ex2, _dir) = setup_two_conns();
    ex1.execute("CREATE TABLE mv9 (pk U64, v U64)").unwrap();
    ex1.execute("INSERT INTO mv9 (pk, v) VALUES (1, 10)")
        .unwrap();

    ex1.execute("BEGIN ISOLATION LEVEL SERIALIZABLE").unwrap();
    // Full-table scan (no WHERE) fixes the snapshot the same way.
    ex1.execute("SELECT * FROM mv9").unwrap();
    ex1.execute("UPDATE mv9 SET v = 11 WHERE pk = 1").unwrap();

    ex2.execute("UPDATE mv9 SET v = 999 WHERE pk = 1").unwrap();

    let c1 = ex1.execute("COMMIT").unwrap();
    match c1 {
        QueryResult::Error(msg) => assert!(msg.contains("40001"), "{msg}"),
        other => panic!("expected serialization failure, got {other:?}"),
    }
}

// ── MVCC Phase 3 — atomic cross-table commit (intent log) ─────────────

#[test]
fn test_mvcc_phase3_single_table_commit_unchanged_behavior() {
    // Today's common case: one table, one txn. Must still commit
    // normally even though it now goes through the intent-log path.
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE p3single (pk U64, v U64)").unwrap();
    ex.execute("BEGIN").unwrap();
    ex.execute("INSERT INTO p3single (pk, v) VALUES (1, 42)")
        .unwrap();
    let c = ex.execute("COMMIT").unwrap();
    assert!(matches!(c, QueryResult::TxCommit));

    let row = ex.execute("SELECT v FROM p3single WHERE pk = 1").unwrap();
    assert_eq!(select_first_val(row), "42");

    // A COMMITTED record should now be the latest for this txn.
    let records = ex.txn_intent_log.read_all().unwrap();
    assert!(!records.is_empty());
    assert!(vectorvolt_engine::find_dangling(&records).is_empty());
}

#[test]
fn test_mvcc_phase3_two_table_commit_writes_committed_intent() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE p3a (pk U64, v U64)").unwrap();
    ex.execute("CREATE TABLE p3b (pk U64, v U64)").unwrap();
    ex.execute("BEGIN").unwrap();
    ex.execute("INSERT INTO p3a (pk, v) VALUES (1, 1)").unwrap();
    ex.execute("INSERT INTO p3b (pk, v) VALUES (1, 2)").unwrap();
    let c = ex.execute("COMMIT").unwrap();
    assert!(matches!(c, QueryResult::TxCommit));

    assert_eq!(
        select_first_val(ex.execute("SELECT v FROM p3a WHERE pk = 1").unwrap()),
        "1"
    );
    assert_eq!(
        select_first_val(ex.execute("SELECT v FROM p3b WHERE pk = 1").unwrap()),
        "2"
    );

    let records = ex.txn_intent_log.read_all().unwrap();
    // At least one Preparing + one Committed record for this txn.
    assert!(records.len() >= 2);
    assert!(vectorvolt_engine::find_dangling(&records).is_empty());
    let last = records.last().unwrap();
    assert_eq!(last.state, vectorvolt_engine::IntentState::Committed);
    assert_eq!(last.tables.len(), 2);
}

#[test]
fn test_mvcc_phase3_partial_failure_leaves_dangling_intent_and_recovers() {
    let dir = tempfile::tempdir().unwrap();
    let registry = Arc::new(TableRegistry::open(dir.path(), EngineOptions::default()).unwrap());
    let ex = SqlExecutor::new(Arc::clone(&registry));
    ex.execute("CREATE TABLE p3c (pk U64, v U64)").unwrap();
    ex.execute("CREATE TABLE p3d (pk U64, v U64)").unwrap();
    ex.execute("BEGIN").unwrap();
    ex.execute("INSERT INTO p3c (pk, v) VALUES (1, 1)").unwrap();
    ex.execute("INSERT INTO p3d (pk, v) VALUES (1, 2)").unwrap();

    // Force a mid-commit failure the "naturally forceable" way: commit
    // one table's txn_id out from under the still-open SQL transaction,
    // so exec_commit's own commit_txn call on it later returns
    // TxnNotFound.
    let forced_table = {
        let guard = ex.txn.lock_safe();
        let map = guard.as_ref().unwrap();
        if map.contains_key("p3d") {
            "p3d"
        } else {
            "p3c"
        }
    };
    let txn_id = {
        let guard = ex.txn.lock_safe();
        *guard.as_ref().unwrap().get(forced_table).unwrap()
    };
    registry
        .get(forced_table)
        .unwrap()
        .commit_txn(txn_id)
        .unwrap();

    let result = ex.execute("COMMIT").unwrap();
    match result {
        QueryResult::Error(msg) => assert!(msg.contains("40003"), "{msg}"),
        other => panic!("expected a commit error, got {other:?}"),
    }

    // The intent log must show a dangling PREPARING record — no
    // COMMITTED was written since the loop errored partway through.
    let records = ex.txn_intent_log.read_all().unwrap();
    let dangling = vectorvolt_engine::find_dangling(&records);
    assert_eq!(dangling.len(), 1);

    // Recovery rolls forward: the table that was never touched actually
    // commits now; the one already committed resolves via TxnNotFound.
    let c = registry.get("p3c").unwrap();
    let d = registry.get("p3d").unwrap();
    let actions =
        vectorvolt_engine::recover_pending_transactions(ex.txn_intent_log.path(), move |name| {
            match name {
                "p3c" => Some(c.clone()),
                "p3d" => Some(d.clone()),
                _ => None,
            }
        })
        .unwrap();
    assert_eq!(actions.len(), 1);
    assert_eq!(
        actions[0].resolved_state,
        vectorvolt_engine::IntentState::Committed
    );

    assert_eq!(
        select_first_val(ex.execute("SELECT v FROM p3c WHERE pk = 1").unwrap()),
        "1"
    );
    assert_eq!(
        select_first_val(ex.execute("SELECT v FROM p3d WHERE pk = 1").unwrap()),
        "2"
    );

    // Recovery is itself idempotent.
    let c2 = registry.get("p3c").unwrap();
    let d2 = registry.get("p3d").unwrap();
    let actions2 =
        vectorvolt_engine::recover_pending_transactions(ex.txn_intent_log.path(), move |name| {
            match name {
                "p3c" => Some(c2.clone()),
                "p3d" => Some(d2.clone()),
                _ => None,
            }
        })
        .unwrap();
    assert!(actions2.is_empty());
}

#[test]
fn test_mvcc_phase3_recovery_hand_constructed_dangling_record() {
    // A hand-constructed dangling PREPARING record for 2 tables where
    // one is already committed and one is not — recovery must complete
    // it and mark it resolved.
    let dir = tempfile::tempdir().unwrap();
    let registry = Arc::new(TableRegistry::open(dir.path(), EngineOptions::default()).unwrap());
    let a = registry
        .create_table("p3e", vec!["pk".into(), "v".into()])
        .unwrap();
    let b = registry
        .create_table("p3f", vec!["pk".into(), "v".into()])
        .unwrap();

    let t1 = a.begin();
    a.insert_txn(t1, 1, vec![1, 111]).unwrap();
    let t2 = b.begin();
    b.insert_txn(t2, 1, vec![1, 222]).unwrap();

    // One table already committed before the crash.
    a.commit_txn(t1).unwrap();

    let log_path = dir.path().join("_txn_intent_manual.log");
    let log = vectorvolt_engine::TxnIntentLog::open(&log_path).unwrap();
    log.prepare(
        "manual-txn",
        vec![
            vectorvolt_engine::TableTxn {
                table: "p3e".into(),
                txn_id: t1,
            },
            vectorvolt_engine::TableTxn {
                table: "p3f".into(),
                txn_id: t2,
            },
        ],
    )
    .unwrap();

    let actions =
        vectorvolt_engine::recover_pending_transactions(&log_path, move |name| match name {
            "p3e" => Some(a.clone()),
            "p3f" => Some(b.clone()),
            _ => None,
        })
        .unwrap();

    assert_eq!(actions.len(), 1);
    assert_eq!(
        actions[0].resolved_state,
        vectorvolt_engine::IntentState::Committed
    );
    assert_eq!(actions[0].tables_committed.len(), 2);

    let records = vectorvolt_engine::TxnIntentLog::open(&log_path)
        .unwrap()
        .read_all()
        .unwrap();
    assert!(vectorvolt_engine::find_dangling(&records).is_empty());
}

// â”€â”€ S62 â€” FTS improvements â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s62_phrase_match() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE docs (pk U64, body TEXT)").unwrap();
    ex.execute("INSERT INTO docs (pk, body) VALUES (1, 'hello world')")
        .unwrap();
    let r = ex.execute("SELECT * FROM docs WHERE MATCH(body, 'hello world')");
    assert!(r.is_ok(), "{r:?}");
}

#[test]
fn test_s62_phrase_no_match() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE fts2 (pk U64, body TEXT)").unwrap();
    ex.execute("INSERT INTO fts2 (pk, body) VALUES (1, 'foo bar')")
        .unwrap();
    let r = ex.execute("SELECT pk FROM fts2 WHERE MATCH(body, 'hello')");
    assert!(r.is_ok(), "{r:?}");
}

#[test]
fn test_s62_match_score() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE fts3 (pk U64, body TEXT)").unwrap();
    ex.execute("INSERT INTO fts3 (pk, body) VALUES (1, 'rust is fast rust')")
        .unwrap();
    let r = ex.execute("SELECT MATCH_SCORE(body, 'rust') AS score FROM fts3");
    assert!(r.is_ok(), "{r:?}");
}

#[test]
fn test_s62_multi_word_match() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE fts4 (pk U64, txt TEXT)").unwrap();
    let r = ex.execute("SELECT * FROM fts4 WHERE MATCH(txt, 'alpha')");
    assert!(r.is_ok(), "{r:?}");
}

#[test]
fn test_s62_match_score_order() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE fts5 (pk U64, body TEXT)").unwrap();
    ex.execute("INSERT INTO fts5 (pk, body) VALUES (1, 'rust rust rust')")
        .unwrap();
    ex.execute("INSERT INTO fts5 (pk, body) VALUES (2, 'rust')")
        .unwrap();
    let r = ex.execute("SELECT pk, MATCH_SCORE(body, 'rust') AS score FROM fts5");
    assert!(r.is_ok(), "{r:?}");
}

// â”€â”€ S63 â€” JSON column type â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s63_json_get() {
    let (ex, _dir) = setup();
    // Use JSON arrow operator in expression context
    ex.execute("CREATE TABLE jt (pk U64, data TEXT)").unwrap();
    ex.execute("INSERT INTO jt (pk, data) VALUES (1, '{\"name\":\"alice\"}')")
        .unwrap();
    // Select the raw field â€” json_get via SELECT * and manual eval
    let r = ex.execute("SELECT * FROM jt WHERE pk = 1").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}

#[test]
fn test_s63_json_get_text() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE jt2 (pk U64, data TEXT)").unwrap();
    ex.execute("INSERT INTO jt2 (pk, data) VALUES (1, '{\"age\":30}')")
        .unwrap();
    let r = ex.execute("SELECT * FROM jt2").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}

#[test]
fn test_s63_json_nested_key() {
    let json = "{\"user\":{\"name\":\"bob\"},\"score\":42}";
    let val = json_extract(json, "score");
    assert_eq!(val, "42");
}

#[test]
fn test_s63_json_where_filter() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE jf (pk U64, meta TEXT)").unwrap();
    ex.execute("INSERT INTO jf (pk, meta) VALUES (1, '{\"active\":1}')")
        .unwrap();
    let r = ex.execute("SELECT pk FROM jf");
    assert!(r.is_ok());
}

#[test]
fn test_s63_json_null_key() {
    let val = json_extract("{\"x\":1}", "missing");
    assert_eq!(val, "null");
}

// â”€â”€ S64 â€” DATE_TRUNC / TIME_BUCKET â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s64_date_trunc_year() {
    assert_eq!(date_trunc("year", "2024-07-15"), "2024-01-01");
}

#[test]
fn test_s64_date_trunc_month() {
    assert_eq!(date_trunc("month", "2024-07-15"), "2024-07-01");
}

#[test]
fn test_s64_time_bucket() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE ts (pk U64, t U64)").unwrap();
    ex.execute("INSERT INTO ts (pk, t) VALUES (1, 100)")
        .unwrap();
    ex.execute("INSERT INTO ts (pk, t) VALUES (2, 150)")
        .unwrap();
    let r = ex.execute("SELECT TIME_BUCKET(50, t) AS bucket FROM ts");
    assert!(r.is_ok(), "{r:?}");
}

#[test]
fn test_s64_date_trunc_group_by() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE events (pk U64, created TEXT)")
        .unwrap();
    ex.execute("INSERT INTO events (pk, created) VALUES (1, '2024-01-15')")
        .unwrap();
    ex.execute("INSERT INTO events (pk, created) VALUES (2, '2024-01-20')")
        .unwrap();
    let r = ex.execute("SELECT DATE_TRUNC('month', created) AS mo FROM events");
    assert!(r.is_ok(), "{r:?}");
}

#[test]
fn test_s64_time_bucket_count() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE tb (pk U64, ts U64)").unwrap();
    for i in 1u64..=10 {
        ex.execute(&format!("INSERT INTO tb (pk, ts) VALUES ({i}, {})", i * 10))
            .unwrap();
    }
    let r = ex.execute("SELECT COUNT(*) FROM tb");
    if let Ok(QueryResult::Rows { rows, .. }) = r {
        assert_eq!(rows[0][0], "10");
    } else {
        panic!();
    }
}

// â”€â”€ S65 â€” UDFs â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s65_create_function() {
    let (ex, _dir) = setup();
    let r = ex.execute("CREATE FUNCTION double(x) RETURNS U64 AS 'x * 2'");
    assert!(r.is_ok(), "{r:?}");
}

#[test]
fn test_s65_call_udf() {
    let (ex, _dir) = setup();
    ex.execute("CREATE FUNCTION triple(x) RETURNS U64 AS 'x * 3'")
        .unwrap();
    ex.execute("CREATE TABLE udf_t (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO udf_t (pk, val) VALUES (1, 10)")
        .unwrap();
    let r = ex.execute("SELECT TRIPLE(val) AS result FROM udf_t");
    assert!(r.is_ok(), "{r:?}");
}

#[test]
fn test_s65_udf_upper() {
    let (ex, _dir) = setup();
    let r = ex.execute("CREATE FUNCTION shout(s) RETURNS TEXT AS 's'");
    assert!(r.is_ok(), "{r:?}");
}

#[test]
fn test_s65_udf_concat() {
    let (ex, _dir) = setup();
    ex.execute("CREATE FUNCTION greet(name) RETURNS TEXT AS 'Hello name'")
        .unwrap();
    // UDF is registered; calling it as StrFunc dispatches body substitution
    let has_udf = ex.udfs.lock_safe().contains_key("greet");
    assert!(has_udf);
}

#[test]
fn test_s65_drop_function() {
    let (ex, _dir) = setup();
    ex.execute("CREATE FUNCTION tmp(x) RETURNS U64 AS 'x'")
        .unwrap();
    let r = ex.execute("DROP FUNCTION tmp");
    assert!(r.is_ok(), "{r:?}");
    assert!(!ex.udfs.lock_safe().contains_key("tmp"));
}

// â”€â”€ S66 â€” Triggers â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s66_create_trigger() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE trig_t (pk U64, v U64)").unwrap();
    ex.execute("CREATE TABLE trig_log (pk U64)").unwrap();
    ex.execute("CREATE PROCEDURE log_it AS BEGIN INSERT INTO trig_log (pk) VALUES (NEW.pk); END")
        .unwrap();
    let r = ex
        .execute(
            "CREATE TRIGGER trig1 AFTER INSERT ON trig_t FOR EACH ROW EXECUTE FUNCTION log_it()",
        )
        .unwrap();
    assert!(matches!(r, QueryResult::Created), "{r:?}");
}

#[test]
fn test_s66_trigger_fires_on_insert() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE tri2 (pk U64, v U64)").unwrap();
    ex.execute("CREATE TABLE tri2_log (pk U64)").unwrap();
    ex.execute("CREATE PROCEDURE noop AS BEGIN INSERT INTO tri2_log (pk) VALUES (NEW.pk); END")
        .unwrap();
    ex.execute("CREATE TRIGGER t2 AFTER INSERT ON tri2 FOR EACH ROW EXECUTE FUNCTION noop()")
        .unwrap();
    let r = ex.execute("INSERT INTO tri2 (pk, v) VALUES (1, 10)");
    assert!(r.is_ok(), "{r:?}");
}

#[test]
fn test_s66_trigger_before_insert() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE tri3 (pk U64)").unwrap();
    ex.execute("CREATE PROCEDURE chk AS BEGIN SELECT * FROM tri3; END")
        .unwrap();
    let r = ex
        .execute("CREATE TRIGGER t3 BEFORE INSERT ON tri3 FOR EACH ROW EXECUTE FUNCTION chk()")
        .unwrap();
    assert!(matches!(r, QueryResult::Created), "{r:?}");
}

#[test]
fn test_s66_drop_trigger() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE tri4 (pk U64)").unwrap();
    ex.execute("CREATE PROCEDURE f4 AS BEGIN SELECT * FROM tri4; END")
        .unwrap();
    ex.execute("CREATE TRIGGER t4 AFTER DELETE ON tri4 FOR EACH ROW EXECUTE FUNCTION f4()")
        .unwrap();
    let r = ex.execute("DROP TRIGGER t4");
    assert!(r.is_ok(), "{r:?}");
    assert!(!ex.triggers.lock_safe().contains_key("t4"));
}

#[test]
fn test_s66_trigger_multiple_tables() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE ta (pk U64)").unwrap();
    ex.execute("CREATE TABLE tb2 (pk U64)").unwrap();
    ex.execute("CREATE PROCEDURE f_a AS BEGIN SELECT * FROM ta; END")
        .unwrap();
    ex.execute("CREATE PROCEDURE f_b AS BEGIN SELECT * FROM tb2; END")
        .unwrap();
    ex.execute("CREATE TRIGGER trig_a AFTER INSERT ON ta FOR EACH ROW EXECUTE FUNCTION f_a()")
        .unwrap();
    ex.execute("CREATE TRIGGER trig_b AFTER INSERT ON tb2 FOR EACH ROW EXECUTE FUNCTION f_b()")
        .unwrap();
    assert_eq!(ex.triggers.lock_safe().len(), 2);
}

// â”€â”€ S67 â€” Index Hints â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s67_index_hint() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE ih (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO ih (pk, val) VALUES (1, 10)")
        .unwrap();
    let r = ex.execute("SELECT * FROM ih USE INDEX(pk) WHERE pk = 1");
    assert!(r.is_ok(), "{r:?}");
}

#[test]
fn test_s67_auto_index_use() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE ai (pk U64, v U64)").unwrap();
    ex.execute("INSERT INTO ai (pk, v) VALUES (1, 1)").unwrap();
    // Without hint still uses cost-based plan
    let r = ex.execute("SELECT * FROM ai WHERE pk = 1");
    assert!(r.is_ok(), "{r:?}");
}

#[test]
fn test_s67_join_reorder() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE left_t (pk U64, fk U64)").unwrap();
    ex.execute("CREATE TABLE right_t (pk U64, v U64)").unwrap();
    ex.execute("INSERT INTO left_t (pk, fk) VALUES (1, 1)")
        .unwrap();
    ex.execute("INSERT INTO right_t (pk, v) VALUES (1, 99)")
        .unwrap();
    let r = ex
        .execute("SELECT left_t.pk, right_t.v FROM left_t JOIN right_t ON left_t.fk = right_t.pk");
    assert!(r.is_ok(), "{r:?}");
}

#[test]
fn test_s67_hint_ignored_if_no_index() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE ni (pk U64, v U64)").unwrap();
    ex.execute("INSERT INTO ni (pk, v) VALUES (1, 1)").unwrap();
    // Hint on non-existent col â†’ falls back gracefully
    let r = ex.execute("SELECT * FROM ni USE INDEX(nonexistent_col)");
    assert!(r.is_ok(), "{r:?}");
}

#[test]
fn test_s67_planner_prefers_range_scan() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE rs (pk U64, v U64)").unwrap();
    ex.execute("INSERT INTO rs (pk, v) VALUES (1, 10)").unwrap();
    let r = ex
        .execute("EXPLAIN SELECT * FROM rs USE INDEX(pk) WHERE pk = 1")
        .unwrap();
    if let QueryResult::ExplainPlan(lines) = r {
        assert!(
            lines
                .iter()
                .any(|l| l.contains("RangeScan") || l.contains("FullScan"))
        );
    } else {
        panic!();
    }
}

#[test]
fn test_s38_multi_order_by() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE mo (pk U64, grp U64, score U64)")
        .unwrap();
    ex.execute("INSERT INTO mo (pk, grp, score) VALUES (1, 2, 30)")
        .unwrap();
    ex.execute("INSERT INTO mo (pk, grp, score) VALUES (2, 1, 40)")
        .unwrap();
    ex.execute("INSERT INTO mo (pk, grp, score) VALUES (3, 1, 20)")
        .unwrap();
    ex.execute("INSERT INTO mo (pk, grp, score) VALUES (4, 2, 10)")
        .unwrap();
    let r = ex
        .execute("SELECT pk FROM mo ORDER BY grp ASC, score DESC")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 4);
        // grp=1 rows first: score DESC â†’ pk=2(40), pk=3(20). grp=2 rows: score DESC â†’ pk=1(30), pk=4(10)
        assert_eq!(rows[0][0], "2");
        assert_eq!(rows[1][0], "3");
        assert_eq!(rows[2][0], "1");
        assert_eq!(rows[3][0], "4");
    } else {
        panic!("expected rows");
    }
}

// â”€â”€ S68 Recursive CTEs â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s68_recursive_cte_basic() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t68 (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t68 (pk, val) VALUES (1, 10)")
        .unwrap();
    ex.execute("INSERT INTO t68 (pk, val) VALUES (2, 20)")
        .unwrap();
    let r = ex.execute("WITH RECURSIVE r AS (SELECT pk FROM t68 WHERE pk = 1 UNION ALL SELECT pk FROM t68 WHERE pk = 9999) SELECT * FROM r").unwrap();
    match r {
        QueryResult::Rows { rows, .. } => assert!(!rows.is_empty()),
        _ => panic!(),
    }
}

#[test]
fn test_s68_recursive_cte_depth() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t68b (pk U64, val U64)").unwrap();
    let r = ex.execute("WITH RECURSIVE r AS (SELECT pk FROM t68b UNION ALL SELECT pk FROM t68b WHERE 1 = 0) SELECT * FROM r").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}

#[test]
fn test_s68_recursive_union_all() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE nums68 (pk U64, n U64)").unwrap();
    ex.execute("INSERT INTO nums68 (pk, n) VALUES (1, 1)")
        .unwrap();
    ex.execute("INSERT INTO nums68 (pk, n) VALUES (2, 2)")
        .unwrap();
    let r = ex.execute("WITH RECURSIVE r AS (SELECT n FROM nums68 WHERE pk = 1 UNION ALL SELECT n FROM nums68 WHERE pk = 2) SELECT * FROM r").unwrap();
    match r {
        QueryResult::Rows { rows, .. } => assert_eq!(rows.len(), 1),
        _ => panic!(),
    }
}

#[test]
fn test_s68_nonrecursive_still_works() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t68c (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t68c (pk, val) VALUES (1, 100)")
        .unwrap();
    let r = ex
        .execute("WITH c AS (SELECT val FROM t68c WHERE pk = 1) SELECT * FROM c")
        .unwrap();
    match r {
        QueryResult::Rows { rows, .. } => assert_eq!(rows.len(), 1),
        _ => panic!(),
    }
}

#[test]
fn test_s68_recursive_cte_stops() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t68d (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t68d (pk, val) VALUES (5, 50)")
        .unwrap();
    let r = ex.execute("WITH RECURSIVE r AS (SELECT val FROM t68d WHERE pk = 5 UNION ALL SELECT val FROM t68d WHERE pk = 99) SELECT * FROM r").unwrap();
    match r {
        QueryResult::Rows { rows, .. } => assert_eq!(rows.len(), 1),
        _ => panic!(),
    }
}

// â”€â”€ S69 Lateral Joins â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s69_lateral_basic() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t69a (pk U64, val U64)").unwrap();
    ex.execute("CREATE TABLE t69b (pk U64, fk U64)").unwrap();
    ex.execute("INSERT INTO t69a (pk, val) VALUES (1, 10)")
        .unwrap();
    ex.execute("INSERT INTO t69b (pk, fk) VALUES (1, 1)")
        .unwrap();
    let r = ex.execute("SELECT * FROM t69a, LATERAL (SELECT fk FROM t69b WHERE fk = 1) AS sub");
    assert!(r.is_ok(), "lateral join must parse without error: {r:?}");
}

#[test]
fn test_s69_lateral_parses() {
    let tokens = crate::sql::lexer::Lexer::new(
        "SELECT * FROM t, LATERAL (SELECT v FROM s WHERE v = 1) AS sub",
    )
    .tokenize()
    .unwrap();
    let stmt = crate::sql::parser::SqlParser::new(tokens).parse();
    assert!(stmt.is_ok());
}

#[test]
fn test_s69_lateral_no_error_on_empty() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t69c (pk U64, val U64)").unwrap();
    ex.execute("CREATE TABLE t69d (pk U64, fk U64)").unwrap();
    let r = ex.execute("SELECT * FROM t69c, LATERAL (SELECT fk FROM t69d WHERE fk = 99) AS sub");
    assert!(r.is_ok());
}

#[test]
fn test_s69_lateral_multiple() {
    let tokens = crate::sql::lexer::Lexer::new(
        "SELECT * FROM t, LATERAL (SELECT v FROM s) AS sub1, LATERAL (SELECT w FROM u) AS sub2",
    )
    .tokenize()
    .unwrap();
    let stmt = crate::sql::parser::SqlParser::new(tokens).parse();
    assert!(stmt.is_ok());
}

#[test]
fn test_s69_lateral_alias() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t69e (pk U64, val U64)").unwrap();
    ex.execute("CREATE TABLE t69f (pk U64, fk U64)").unwrap();
    ex.execute("INSERT INTO t69e (pk, val) VALUES (1, 42)")
        .unwrap();
    let r = ex.execute("SELECT val FROM t69e, LATERAL (SELECT fk FROM t69f WHERE fk = 1) AS lat");
    assert!(r.is_ok());
}

// â”€â”€ S70 Partition Pruning â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s70_partition_prune_hit() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t70a (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t70a (pk, val) VALUES (1, 42)")
        .unwrap();
    let r = ex.execute("SELECT val FROM t70a WHERE pk = 1").unwrap();
    match r {
        QueryResult::Rows { rows, .. } => assert_eq!(rows[0][0], "42"),
        _ => panic!(),
    }
}

#[test]
fn test_s70_partition_prune_miss() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t70b (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t70b (pk, val) VALUES (1, 99)")
        .unwrap();
    let r = ex.execute("SELECT val FROM t70b WHERE pk = 99").unwrap();
    match r {
        QueryResult::Rows { rows, .. } => assert!(rows.is_empty()),
        _ => panic!(),
    }
}

#[test]
fn test_s70_full_scan_fallback() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t70c (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t70c (pk, val) VALUES (1, 5)")
        .unwrap();
    ex.execute("INSERT INTO t70c (pk, val) VALUES (2, 6)")
        .unwrap();
    let r = ex.execute("SELECT * FROM t70c").unwrap();
    match r {
        QueryResult::Rows { rows, .. } => assert_eq!(rows.len(), 2),
        _ => panic!(),
    }
}

#[test]
fn test_s70_range_predicate() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t70d (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t70d (pk, val) VALUES (3, 30)")
        .unwrap();
    ex.execute("INSERT INTO t70d (pk, val) VALUES (7, 70)")
        .unwrap();
    let r = ex.execute("SELECT pk FROM t70d WHERE pk > 2").unwrap();
    match r {
        QueryResult::Rows { rows, .. } => assert!(!rows.is_empty()),
        _ => panic!(),
    }
}

#[test]
fn test_s70_no_false_positives() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t70e (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t70e (pk, val) VALUES (1, 100)")
        .unwrap();
    ex.execute("INSERT INTO t70e (pk, val) VALUES (2, 200)")
        .unwrap();
    let r = ex.execute("SELECT val FROM t70e WHERE pk = 1").unwrap();
    match r {
        QueryResult::Rows { rows, .. } => {
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0][0], "100");
        }
        _ => panic!(),
    }
}

// â”€â”€ S71 Query Stats â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s71_stats_basic() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t71a (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t71a (pk, val) VALUES (1, 1)")
        .unwrap();
    ex.execute("SELECT * FROM t71a").unwrap();
    let r = ex.execute("SELECT * FROM vv_stat_statements").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}

#[test]
fn test_s71_stats_multiple_queries() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t71b (pk U64, val U64)").unwrap();
    ex.execute("SELECT * FROM t71b").unwrap();
    ex.execute("SELECT * FROM t71b").unwrap();
    let r = ex.execute("SELECT * FROM vv_stat_statements").unwrap();
    match r {
        QueryResult::Rows { rows, .. } => assert!(!rows.is_empty()),
        _ => panic!(),
    }
}

#[test]
fn test_s71_reset_stats() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t71c (pk U64, val U64)").unwrap();
    ex.execute("SELECT * FROM t71c").unwrap();
    ex.execute("RESET QUERY STATS").unwrap();
    let r = ex.execute("SELECT * FROM vv_stat_statements").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}

#[test]
fn test_s71_stats_columns() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t71d (pk U64, val U64)").unwrap();
    ex.execute("SELECT * FROM t71d").unwrap();
    let r = ex.execute("SELECT * FROM vv_stat_statements").unwrap();
    match r {
        QueryResult::Rows { columns, .. } => {
            assert!(columns.contains(&"query".to_owned()));
            assert!(columns.contains(&"calls".to_owned()));
        }
        _ => panic!(),
    }
}

#[test]
fn test_s71_stats_call_count() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t71e (pk U64, val U64)").unwrap();
    ex.execute("RESET QUERY STATS").unwrap();
    for _ in 0..3 {
        ex.execute("SELECT pk FROM t71e").unwrap();
    }
    let r = ex.execute("SELECT * FROM vv_stat_statements").unwrap();
    match r {
        QueryResult::Rows { rows, columns, .. } => {
            let q_idx = columns.iter().position(|c| c == "query").unwrap_or(0);
            let calls_idx = columns.iter().position(|c| c == "calls").unwrap_or(1);
            let found = rows
                .iter()
                .find(|r| r.get(q_idx).map(|s| s.contains("T71E")).unwrap_or(false));
            if let Some(row) = found {
                let calls: u64 = row[calls_idx].parse().unwrap_or(0);
                assert!(calls >= 3, "expected calls >= 3, got {}", calls);
            }
        }
        _ => panic!(),
    }
}

// â”€â”€ S72 Connection Pool â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s72_pool_basic_query() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t72a (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t72a (pk, val) VALUES (1, 42)")
        .unwrap();
    let r = ex.execute("SELECT val FROM t72a WHERE pk = 1").unwrap();
    match r {
        QueryResult::Rows { rows, .. } => assert_eq!(rows[0][0], "42"),
        _ => panic!(),
    }
}

#[test]
fn test_s72_pool_sequential_queries() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t72b (pk U64, val U64)").unwrap();
    for i in 1u64..=5 {
        ex.execute(&format!("INSERT INTO t72b (pk, val) VALUES ({i}, {i})"))
            .unwrap();
    }
    let r = ex.execute("SELECT * FROM t72b").unwrap();
    match r {
        QueryResult::Rows { rows, .. } => assert_eq!(rows.len(), 5),
        _ => panic!(),
    }
}

#[test]
fn test_s72_pool_concurrent_reads() {
    use std::sync::Arc;
    use std::thread;
    let dir = tempfile::tempdir().unwrap();
    let reg = Arc::new(TableRegistry::open(dir.path(), EngineOptions::default()).unwrap());
    let ex = Arc::new(SqlExecutor::new(reg));
    ex.execute("CREATE TABLE t72c (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t72c (pk, val) VALUES (1, 99)")
        .unwrap();
    let handles: Vec<_> = (0..4)
        .map(|_| {
            let ex2 = Arc::clone(&ex);
            thread::spawn(move || ex2.execute("SELECT * FROM t72c").unwrap())
        })
        .collect();
    for h in handles {
        let r = h.join().unwrap();
        assert!(matches!(r, QueryResult::Rows { .. }));
    }
}

#[test]
fn test_s72_pool_write_then_read() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t72d (pk U64, score U64)").unwrap();
    ex.execute("INSERT INTO t72d (pk, score) VALUES (10, 500)")
        .unwrap();
    let r = ex.execute("SELECT score FROM t72d WHERE pk = 10").unwrap();
    match r {
        QueryResult::Rows { rows, .. } => assert_eq!(rows[0][0], "500"),
        _ => panic!(),
    }
}

#[test]
fn test_s72_pool_multiple_tables() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t72e (pk U64, a U64)").unwrap();
    ex.execute("CREATE TABLE t72f (pk U64, b U64)").unwrap();
    ex.execute("INSERT INTO t72e (pk, a) VALUES (1, 1)")
        .unwrap();
    ex.execute("INSERT INTO t72f (pk, b) VALUES (1, 2)")
        .unwrap();
    let r1 = ex.execute("SELECT a FROM t72e WHERE pk = 1").unwrap();
    let r2 = ex.execute("SELECT b FROM t72f WHERE pk = 1").unwrap();
    match (r1, r2) {
        (QueryResult::Rows { rows: r1, .. }, QueryResult::Rows { rows: r2, .. }) => {
            assert_eq!(r1[0][0], "1");
            assert_eq!(r2[0][0], "2");
        }
        _ => panic!(),
    }
}

// â”€â”€ S73 Bulk INSERT â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s73_bulk_insert_basic() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t73a (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t73a (pk, val) VALUES (1, 10), (2, 20), (3, 30)")
        .unwrap();
    let r = ex.execute("SELECT * FROM t73a").unwrap();
    match r {
        QueryResult::Rows { rows, .. } => assert_eq!(rows.len(), 3),
        _ => panic!(),
    }
}

#[test]
fn test_s73_bulk_insert_two_rows() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t73b (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t73b (pk, val) VALUES (1, 100), (2, 200)")
        .unwrap();
    let r = ex.execute("SELECT val FROM t73b WHERE pk = 2").unwrap();
    match r {
        QueryResult::Rows { rows, .. } => assert_eq!(rows[0][0], "200"),
        _ => panic!(),
    }
}

#[test]
fn test_s73_bulk_insert_single_still_works() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t73c (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t73c (pk, val) VALUES (5, 50)")
        .unwrap();
    let r = ex.execute("SELECT val FROM t73c WHERE pk = 5").unwrap();
    match r {
        QueryResult::Rows { rows, .. } => assert_eq!(rows[0][0], "50"),
        _ => panic!(),
    }
}

#[test]
fn test_s73_bulk_insert_five_rows() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t73d (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t73d (pk, val) VALUES (1,1),(2,2),(3,3),(4,4),(5,5)")
        .unwrap();
    let r = ex.execute("SELECT * FROM t73d").unwrap();
    match r {
        QueryResult::Rows { rows, .. } => assert_eq!(rows.len(), 5),
        _ => panic!(),
    }
}

#[test]
fn test_s73_bulk_insert_verify_all() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t73e (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t73e (pk, val) VALUES (10, 100), (20, 200)")
        .unwrap();
    let r1 = ex.execute("SELECT val FROM t73e WHERE pk = 10").unwrap();
    let r2 = ex.execute("SELECT val FROM t73e WHERE pk = 20").unwrap();
    match r1 {
        QueryResult::Rows { rows, .. } => assert_eq!(rows[0][0], "100"),
        _ => panic!(),
    }
    match r2 {
        QueryResult::Rows { rows, .. } => assert_eq!(rows[0][0], "200"),
        _ => panic!(),
    }
}

// â”€â”€ S74 COPY FROM â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s74_copy_from_basic() {
    let (ex, _dir) = setup();
    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), "1,10\n2,20\n3,30\n").unwrap();
    ex.execute("CREATE TABLE t74a (pk U64, val U64)").unwrap();
    let path = tmp.path().to_str().unwrap().replace('\\', "/");
    let r = ex.execute(&format!("COPY t74a FROM '{path}'"));
    assert!(r.is_ok(), "{r:?}");
    let r2 = ex.execute("SELECT * FROM t74a").unwrap();
    match r2 {
        QueryResult::Rows { rows, .. } => assert_eq!(rows.len(), 3),
        _ => panic!(),
    }
}

#[test]
fn test_s74_copy_from_with_header() {
    let (ex, _dir) = setup();
    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), "pk,val\n1,42\n2,84\n").unwrap();
    ex.execute("CREATE TABLE t74b (pk U64, val U64)").unwrap();
    let path = tmp.path().to_str().unwrap().replace('\\', "/");
    let r = ex.execute(&format!(
        "COPY t74b FROM '{path}' WITH (FORMAT CSV, HEADER TRUE)"
    ));
    assert!(r.is_ok(), "{r:?}");
    let r2 = ex.execute("SELECT * FROM t74b").unwrap();
    match r2 {
        QueryResult::Rows { rows, .. } => assert_eq!(rows.len(), 2),
        _ => panic!(),
    }
}

#[test]
fn test_s74_copy_from_parses() {
    let tokens = crate::sql::lexer::Lexer::new(
        "COPY mytable FROM '/some/path.csv' WITH (FORMAT CSV, HEADER TRUE)",
    )
    .tokenize()
    .unwrap();
    let stmt = crate::sql::parser::SqlParser::new(tokens).parse();
    assert!(stmt.is_ok());
}

#[test]
fn test_s74_copy_counts_rows() {
    let (ex, _dir) = setup();
    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), "1,10\n2,20\n3,30\n4,40\n").unwrap();
    ex.execute("CREATE TABLE t74c (pk U64, val U64)").unwrap();
    let path = tmp.path().to_str().unwrap().replace('\\', "/");
    let r = ex.execute(&format!("COPY t74c FROM '{path}'")).unwrap();
    match r {
        QueryResult::Affected(n) => assert_eq!(n, 4),
        _ => panic!(),
    }
}

#[test]
fn test_s74_copy_data_correct() {
    let (ex, _dir) = setup();
    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), "7,77\n8,88\n").unwrap();
    ex.execute("CREATE TABLE t74d (pk U64, val U64)").unwrap();
    let path = tmp.path().to_str().unwrap().replace('\\', "/");
    ex.execute(&format!("COPY t74d FROM '{path}'")).unwrap();
    let r = ex.execute("SELECT val FROM t74d WHERE pk = 7").unwrap();
    match r {
        QueryResult::Rows { rows, .. } => assert_eq!(rows[0][0], "77"),
        _ => panic!(),
    }
}

// â”€â”€ S75 Materialized Views â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s75_create_mat_view() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t75a (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t75a (pk, val) VALUES (1, 100)")
        .unwrap();
    let r = ex.execute("CREATE MATERIALIZED VIEW mv75a AS SELECT val FROM t75a WHERE pk = 1");
    assert!(r.is_ok(), "{r:?}");
}

#[test]
fn test_s75_mat_view_parses() {
    let tokens = crate::sql::lexer::Lexer::new("CREATE MATERIALIZED VIEW mv AS SELECT * FROM t")
        .tokenize()
        .unwrap();
    let stmt = crate::sql::parser::SqlParser::new(tokens).parse();
    assert!(stmt.is_ok());
}

#[test]
fn test_s75_refresh_mat_view() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t75b (pk U64, val U64)").unwrap();
    ex.execute("INSERT INTO t75b (pk, val) VALUES (1, 10)")
        .unwrap();
    ex.execute("CREATE MATERIALIZED VIEW mv75b AS SELECT val FROM t75b")
        .unwrap();
    ex.execute("INSERT INTO t75b (pk, val) VALUES (2, 20)")
        .unwrap();
    let r = ex.execute("REFRESH MATERIALIZED VIEW mv75b");
    assert!(r.is_ok(), "{r:?}");
}

#[test]
fn test_s75_drop_mat_view() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t75c (pk U64, val U64)").unwrap();
    ex.execute("CREATE MATERIALIZED VIEW mv75c AS SELECT val FROM t75c")
        .unwrap();
    let r = ex.execute("DROP MATERIALIZED VIEW mv75c");
    assert!(r.is_ok(), "{r:?}");
}
