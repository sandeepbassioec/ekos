//! Executor unit tests, part 4 of 4.

use super::*;

#[test]
fn test_s260_show_async_status_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("SUBMIT ASYNC QUERY 'SELECT 1'").unwrap();
    let r = ex.execute("SHOW ASYNC QUERY STATUS 1").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s260_fetch_async_result_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("SUBMIT ASYNC QUERY 'SELECT 1'").unwrap();
    let r = ex.execute("FETCH ASYNC RESULT 1").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s260_status_is_done() {
    let (ex, _dir) = setup();
    ex.execute("SUBMIT ASYNC QUERY 'SELECT 1'").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW ASYNC QUERY STATUS 1").unwrap() {
        assert_eq!(rows[0][1], "DONE");
    } else {
        panic!();
    }
}
#[test]
fn test_s260_ids_increment() {
    let (ex, _dir) = setup();
    ex.execute("SUBMIT ASYNC QUERY 'SELECT 1'").unwrap();
    ex.execute("SUBMIT ASYNC QUERY 'SELECT 2'").unwrap();
    assert_eq!(ex.s260_async_queries.lock_safe().len(), 2);
}

// â”€â”€ S261 â€" Column expression indexes â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s261_create_expr_index_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("CREATE INDEX idx_expr ON orders(total + total)")
        .unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s261_show_expression_indexes_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("CREATE INDEX idx_expr ON orders(total + total)")
        .unwrap();
    let r = ex.execute("SHOW EXPRESSION INDEXES").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s261_drop_expression_index_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("CREATE INDEX idx_expr ON orders(total + total)")
        .unwrap();
    let r = ex.execute("DROP EXPRESSION INDEX idx_expr").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s261_stores_expr() {
    let (ex, _dir) = setup();
    ex.execute("CREATE INDEX idx_expr ON orders(total + total)")
        .unwrap();
    let idxs = ex.s261_expr_indexes.lock_safe();
    assert_eq!(idxs.get("idx_expr").unwrap().1, "total + total");
}
#[test]
fn test_s261_drop_removes_entry() {
    let (ex, _dir) = setup();
    ex.execute("CREATE INDEX idx_expr ON orders(total + total)")
        .unwrap();
    ex.execute("DROP EXPRESSION INDEX idx_expr").unwrap();
    assert!(ex.s261_expr_indexes.lock_safe().is_empty());
}

#[test]
fn test_s262_show_shard_keys_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("SET SHARD KEY FOR orders TO id").unwrap();
    let r = ex.execute("SHOW SHARD KEYS").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s262_drop_shard_key_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("SET SHARD KEY FOR orders TO id").unwrap();
    let r = ex.execute("DROP SHARD KEY FOR orders").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s262_drop_removes_entry() {
    let (ex, _dir) = setup();
    ex.execute("SET SHARD KEY FOR orders TO id").unwrap();
    ex.execute("DROP SHARD KEY FOR orders").unwrap();
    assert!(ex.s262_shard_keys.lock_safe().is_empty());
}

// â”€â”€ S263 â€" Query result diff â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s263_diff_query_result_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("DIFF QUERY RESULT 'SELECT * FROM orders' AGAINST SNAPSHOT snap1")
        .unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s263_show_result_diffs_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("DIFF QUERY RESULT 'SELECT * FROM orders' AGAINST SNAPSHOT snap1")
        .unwrap();
    let r = ex.execute("SHOW RESULT DIFFS").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s263_clear_result_diffs_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("DIFF QUERY RESULT 'SELECT * FROM orders' AGAINST SNAPSHOT snap1")
        .unwrap();
    let r = ex.execute("CLEAR RESULT DIFFS").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s263_diff_count() {
    let (ex, _dir) = setup();
    ex.execute("DIFF QUERY RESULT 'SELECT 1' AGAINST SNAPSHOT s1")
        .unwrap();
    ex.execute("DIFF QUERY RESULT 'SELECT 2' AGAINST SNAPSHOT s2")
        .unwrap();
    assert_eq!(ex.s263_result_diffs.lock_safe().len(), 2);
}
#[test]
fn test_s263_clear_empties_diffs() {
    let (ex, _dir) = setup();
    ex.execute("DIFF QUERY RESULT 'SELECT 1' AGAINST SNAPSHOT s1")
        .unwrap();
    ex.execute("CLEAR RESULT DIFFS").unwrap();
    assert!(ex.s263_result_diffs.lock_safe().is_empty());
}

#[test]
fn test_s264_show_multiplexing_status_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("SET CONNECTION POOL MULTIPLEXING ON").unwrap();
    let r = ex.execute("SHOW MULTIPLEXING STATUS").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s264_reset_multiplexing_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("SET CONNECTION POOL MULTIPLEXING ON").unwrap();
    let r = ex.execute("RESET MULTIPLEXING").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s264_reset_sets_false() {
    let (ex, _dir) = setup();
    ex.execute("SET CONNECTION POOL MULTIPLEXING ON").unwrap();
    ex.execute("RESET MULTIPLEXING").unwrap();
    assert!(!*ex.s264_multiplexing.lock_safe());
}

#[test]
fn test_s265_show_compaction_policy_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("SET TABLE orders COMPACTION THRESHOLD 1000")
        .unwrap();
    let r = ex.execute("SHOW TABLE COMPACTION POLICY").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s265_reset_compaction_policy_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("SET TABLE orders COMPACTION THRESHOLD 1000")
        .unwrap();
    let r = ex.execute("RESET TABLE COMPACTION POLICY").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s265_reset_clears_map() {
    let (ex, _dir) = setup();
    ex.execute("SET TABLE orders COMPACTION THRESHOLD 1000")
        .unwrap();
    ex.execute("RESET TABLE COMPACTION POLICY").unwrap();
    assert!(ex.s265_compaction_threshold.lock_safe().is_empty());
}

// â”€â”€ S266 â€" Query plan versioning â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s266_save_plan_version_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("SAVE QUERY PLAN VERSION v1 FOR 'SELECT 1'")
        .unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s266_show_plan_versions_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("SAVE QUERY PLAN VERSION v1 FOR 'SELECT 1'")
        .unwrap();
    let r = ex.execute("SHOW QUERY PLAN VERSIONS").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s266_drop_plan_version_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("SAVE QUERY PLAN VERSION v1 FOR 'SELECT 1'")
        .unwrap();
    let r = ex.execute("DROP QUERY PLAN VERSION v1").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s266_stores_sql() {
    let (ex, _dir) = setup();
    ex.execute("SAVE QUERY PLAN VERSION v1 FOR 'SELECT 1'")
        .unwrap();
    let pv = ex.s266_plan_versions.lock_safe();
    assert_eq!(pv.get("v1").unwrap().0, "SELECT 1");
}
#[test]
fn test_s266_drop_removes_entry() {
    let (ex, _dir) = setup();
    ex.execute("SAVE QUERY PLAN VERSION v1 FOR 'SELECT 1'")
        .unwrap();
    ex.execute("DROP QUERY PLAN VERSION v1").unwrap();
    assert!(ex.s266_plan_versions.lock_safe().is_empty());
}

// â”€â”€ S267 â€" Schema diff apply â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s267_apply_schema_diff_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("APPLY SCHEMA DIFF FROM 'diff.sql' TO public")
        .unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s267_show_diff_history_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("APPLY SCHEMA DIFF FROM 'diff.sql' TO public")
        .unwrap();
    let r = ex.execute("SHOW SCHEMA DIFF HISTORY").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s267_undo_last_schema_diff_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("APPLY SCHEMA DIFF FROM 'diff.sql' TO public")
        .unwrap();
    let r = ex.execute("UNDO LAST SCHEMA DIFF").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s267_history_count() {
    let (ex, _dir) = setup();
    ex.execute("APPLY SCHEMA DIFF FROM 'a.sql' TO public")
        .unwrap();
    ex.execute("APPLY SCHEMA DIFF FROM 'b.sql' TO public")
        .unwrap();
    assert_eq!(ex.s267_diff_history.lock_safe().len(), 2);
}
#[test]
fn test_s267_undo_removes_last() {
    let (ex, _dir) = setup();
    ex.execute("APPLY SCHEMA DIFF FROM 'a.sql' TO public")
        .unwrap();
    ex.execute("UNDO LAST SCHEMA DIFF").unwrap();
    assert!(ex.s267_diff_history.lock_safe().is_empty());
}

#[test]
fn test_s268_show_watermarks_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("SET WATERMARK FOR orders ON created_at DELAY 5s")
        .unwrap();
    let r = ex.execute("SHOW WATERMARKS").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s268_drop_watermark_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("SET WATERMARK FOR orders ON created_at DELAY 5s")
        .unwrap();
    let r = ex.execute("DROP WATERMARK FOR orders").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s268_drop_removes_entry() {
    let (ex, _dir) = setup();
    ex.execute("SET WATERMARK FOR orders ON created_at DELAY 5s")
        .unwrap();
    ex.execute("DROP WATERMARK FOR orders").unwrap();
    assert!(ex.s268_watermarks.lock_safe().is_empty());
}

#[test]
fn test_s269_show_cost_budget_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("SET QUERY COST BUDGET 1000").unwrap();
    let r = ex.execute("SHOW QUERY COST BUDGET").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s269_reset_cost_budget_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("SET QUERY COST BUDGET 1000").unwrap();
    let r = ex.execute("RESET QUERY COST BUDGET").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s269_reset_clears_budget() {
    let (ex, _dir) = setup();
    ex.execute("SET QUERY COST BUDGET 500").unwrap();
    ex.execute("RESET QUERY COST BUDGET").unwrap();
    assert!(ex.s269_cost_budget.lock_safe().is_none());
}

// â”€â”€ S270 â€" Table alias mapping â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s270_create_table_alias_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("CREATE TABLE ALIAS o FOR orders").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s270_drop_table_alias_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE ALIAS o FOR orders").unwrap();
    let r = ex.execute("DROP TABLE ALIAS o").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s270_show_table_aliases_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE ALIAS o FOR orders").unwrap();
    let r = ex.execute("SHOW TABLE ALIASES").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s270_stored_mapping() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE ALIAS o FOR orders").unwrap();
    let al = ex.s270_table_aliases.lock_safe();
    assert_eq!(al.get("o").unwrap(), "orders");
}
#[test]
fn test_s270_drop_removes_entry() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE ALIAS o FOR orders").unwrap();
    ex.execute("DROP TABLE ALIAS o").unwrap();
    assert!(ex.s270_table_aliases.lock_safe().is_empty());
}

// â”€â”€ S271 â€" Index build progress â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s271_show_index_build_progress_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW INDEX BUILD PROGRESS").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s271_cancel_index_build_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("CANCEL INDEX BUILD idx1").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s271_show_index_build_history_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("CANCEL INDEX BUILD idx1").unwrap();
    let r = ex.execute("SHOW INDEX BUILD HISTORY").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s271_history_records_cancel() {
    let (ex, _dir) = setup();
    ex.execute("CANCEL INDEX BUILD idx1").unwrap();
    let h = ex.s271_build_history.lock_safe();
    assert!(h[0].contains("idx1"));
}
#[test]
fn test_s271_history_count() {
    let (ex, _dir) = setup();
    ex.execute("CANCEL INDEX BUILD idx1").unwrap();
    ex.execute("CANCEL INDEX BUILD idx2").unwrap();
    assert_eq!(ex.s271_build_history.lock_safe().len(), 2);
}

// â”€â”€ S272 â€" Query semaphores â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s272_create_semaphore_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("CREATE QUERY SEMAPHORE sem1 LIMIT 5").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s272_drop_semaphore_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("CREATE QUERY SEMAPHORE sem1 LIMIT 5").unwrap();
    let r = ex.execute("DROP QUERY SEMAPHORE sem1").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s272_show_semaphores_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("CREATE QUERY SEMAPHORE sem1 LIMIT 5").unwrap();
    let r = ex.execute("SHOW QUERY SEMAPHORES").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s272_stored_limit() {
    let (ex, _dir) = setup();
    ex.execute("CREATE QUERY SEMAPHORE sem1 LIMIT 7").unwrap();
    let sems = ex.s272_semaphores.lock_safe();
    assert_eq!(*sems.get("sem1").unwrap(), 7);
}
#[test]
fn test_s272_drop_removes_entry() {
    let (ex, _dir) = setup();
    ex.execute("CREATE QUERY SEMAPHORE sem1 LIMIT 5").unwrap();
    ex.execute("DROP QUERY SEMAPHORE sem1").unwrap();
    assert!(ex.s272_semaphores.lock_safe().is_empty());
}

// â”€â”€ S273 â€" Table freeze â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s273_freeze_table_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("FREEZE TABLE orders").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s273_unfreeze_table_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("FREEZE TABLE orders").unwrap();
    let r = ex.execute("UNFREEZE TABLE orders").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s273_show_frozen_tables_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("FREEZE TABLE orders").unwrap();
    let r = ex.execute("SHOW FROZEN TABLES").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s273_frozen_contains_table() {
    let (ex, _dir) = setup();
    ex.execute("FREEZE TABLE orders").unwrap();
    assert!(ex.s273_frozen.lock_safe().contains("orders"));
}
#[test]
fn test_s273_unfreeze_removes_entry() {
    let (ex, _dir) = setup();
    ex.execute("FREEZE TABLE orders").unwrap();
    ex.execute("UNFREEZE TABLE orders").unwrap();
    assert!(!ex.s273_frozen.lock_safe().contains("orders"));
}

// â”€â”€ S274 â€" Column value distribution â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s274_show_value_distribution_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("SHOW VALUE DISTRIBUTION FOR orders.status")
        .unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s274_refresh_value_distribution_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("REFRESH VALUE DISTRIBUTION FOR orders.status")
        .unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s274_clear_value_distribution_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("REFRESH VALUE DISTRIBUTION FOR orders.status")
        .unwrap();
    let r = ex.execute("CLEAR VALUE DISTRIBUTION").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s274_refresh_populates_map() {
    let (ex, _dir) = setup();
    ex.execute("REFRESH VALUE DISTRIBUTION FOR orders.status")
        .unwrap();
    assert!(ex.s274_value_dist.lock_safe().contains_key("orders.status"));
}
#[test]
fn test_s274_clear_empties_map() {
    let (ex, _dir) = setup();
    ex.execute("REFRESH VALUE DISTRIBUTION FOR orders.status")
        .unwrap();
    ex.execute("CLEAR VALUE DISTRIBUTION").unwrap();
    assert!(ex.s274_value_dist.lock_safe().is_empty());
}

#[test]
fn test_s275_show_circuit_breaker_status_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("SET CIRCUIT BREAKER ON orders THRESHOLD 10")
        .unwrap();
    let r = ex.execute("SHOW CIRCUIT BREAKER STATUS").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s275_reset_circuit_breaker_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("SET CIRCUIT BREAKER ON orders THRESHOLD 10")
        .unwrap();
    let r = ex.execute("RESET CIRCUIT BREAKER FOR orders").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s275_reset_removes_entry() {
    let (ex, _dir) = setup();
    ex.execute("SET CIRCUIT BREAKER ON orders THRESHOLD 10")
        .unwrap();
    ex.execute("RESET CIRCUIT BREAKER FOR orders").unwrap();
    assert!(ex.s275_circuit_breakers.lock_safe().is_empty());
}

// â”€â”€ S276 â€" Table checksum â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s276_compute_checksum_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("COMPUTE CHECKSUM FOR orders").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s276_show_table_checksums_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("COMPUTE CHECKSUM FOR orders").unwrap();
    let r = ex.execute("SHOW TABLE CHECKSUMS").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s276_verify_checksum_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("COMPUTE CHECKSUM FOR orders").unwrap();
    let r = ex.execute("VERIFY CHECKSUM FOR orders").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s276_verify_valid_after_compute() {
    let (ex, _dir) = setup();
    ex.execute("COMPUTE CHECKSUM FOR orders").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("VERIFY CHECKSUM FOR orders").unwrap() {
        assert_eq!(rows[0][1], "true");
    } else {
        panic!();
    }
}
#[test]
fn test_s276_verify_without_compute_invalid() {
    let (ex, _dir) = setup();
    if let QueryResult::Rows { rows, .. } = ex.execute("VERIFY CHECKSUM FOR orders").unwrap() {
        assert_eq!(rows[0][1], "false");
    } else {
        panic!();
    }
}

#[test]
fn test_s277_show_batch_size_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("SET QUERY BATCH SIZE 100").unwrap();
    let r = ex.execute("SHOW QUERY BATCH SIZE").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s277_reset_batch_size_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("SET QUERY BATCH SIZE 100").unwrap();
    let r = ex.execute("RESET QUERY BATCH SIZE").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s277_reset_restores_default() {
    let (ex, _dir) = setup();
    ex.execute("SET QUERY BATCH SIZE 250").unwrap();
    ex.execute("RESET QUERY BATCH SIZE").unwrap();
    assert_eq!(*ex.s277_batch_size.lock_safe(), 1);
}

// â”€â”€ S278 â€" Schema lock â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s278_lock_schema_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("LOCK SCHEMA public").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s278_unlock_schema_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("LOCK SCHEMA public").unwrap();
    let r = ex.execute("UNLOCK SCHEMA public").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s278_show_schema_locks_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("LOCK SCHEMA public").unwrap();
    let r = ex.execute("SHOW SCHEMA LOCKS").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s278_locks_contain_schema() {
    let (ex, _dir) = setup();
    ex.execute("LOCK SCHEMA public").unwrap();
    assert!(ex.s278_schema_locks.lock_safe().contains("public"));
}
#[test]
fn test_s278_unlock_removes_entry() {
    let (ex, _dir) = setup();
    ex.execute("LOCK SCHEMA public").unwrap();
    ex.execute("UNLOCK SCHEMA public").unwrap();
    assert!(!ex.s278_schema_locks.lock_safe().contains("public"));
}

// â”€â”€ S279 â€" Table growth prediction â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s279_predict_table_growth_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("PREDICT TABLE GROWTH FOR orders").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s279_show_growth_predictions_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("PREDICT TABLE GROWTH FOR orders").unwrap();
    let r = ex.execute("SHOW GROWTH PREDICTIONS").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s279_clear_growth_predictions_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("PREDICT TABLE GROWTH FOR orders").unwrap();
    let r = ex.execute("CLEAR GROWTH PREDICTIONS").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s279_prediction_stored() {
    let (ex, _dir) = setup();
    ex.execute("PREDICT TABLE GROWTH FOR orders").unwrap();
    assert!(
        ex.s279_growth_predictions
            .lock_safe()
            .contains_key("orders")
    );
}
#[test]
fn test_s279_clear_empties_map() {
    let (ex, _dir) = setup();
    ex.execute("PREDICT TABLE GROWTH FOR orders").unwrap();
    ex.execute("CLEAR GROWTH PREDICTIONS").unwrap();
    assert!(ex.s279_growth_predictions.lock_safe().is_empty());
}

// â”€â”€ S280 â€" Index defrag scheduling â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s280_schedule_defrag_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("SCHEDULE INDEX DEFRAG FOR idx1 AT '2026-01-01'")
        .unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s280_show_defrag_schedule_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("SCHEDULE INDEX DEFRAG FOR idx1 AT '2026-01-01'")
        .unwrap();
    let r = ex.execute("SHOW INDEX DEFRAG SCHEDULE").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s280_cancel_defrag_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("SCHEDULE INDEX DEFRAG FOR idx1 AT '2026-01-01'")
        .unwrap();
    let r = ex.execute("CANCEL INDEX DEFRAG idx1").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s280_stored_schedule() {
    let (ex, _dir) = setup();
    ex.execute("SCHEDULE INDEX DEFRAG FOR idx1 AT '2026-01-01'")
        .unwrap();
    let ds = ex.s280_defrag_schedule.lock_safe();
    assert_eq!(ds.get("idx1").unwrap(), "2026-01-01");
}
#[test]
fn test_s280_cancel_removes_entry() {
    let (ex, _dir) = setup();
    ex.execute("SCHEDULE INDEX DEFRAG FOR idx1 AT '2026-01-01'")
        .unwrap();
    ex.execute("CANCEL INDEX DEFRAG idx1").unwrap();
    assert!(ex.s280_defrag_schedule.lock_safe().is_empty());
}

// â”€â”€ S281 â€" Query result masking â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s281_set_result_mask_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("SET RESULT MASK ON orders.email PATTERN 'XXX'")
        .unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s281_show_result_masks_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("SET RESULT MASK ON orders.email PATTERN 'XXX'")
        .unwrap();
    let r = ex.execute("SHOW RESULT MASKS").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s281_drop_result_mask_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("SET RESULT MASK ON orders.email PATTERN 'XXX'")
        .unwrap();
    let r = ex.execute("DROP RESULT MASK ON orders.email").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s281_stored_pattern() {
    let (ex, _dir) = setup();
    ex.execute("SET RESULT MASK ON orders.email PATTERN 'XXX'")
        .unwrap();
    let masks = ex.s281_result_masks.lock_safe();
    assert_eq!(
        masks
            .get(&("orders".to_owned(), "email".to_owned()))
            .unwrap(),
        "XXX"
    );
}
#[test]
fn test_s281_drop_removes_entry() {
    let (ex, _dir) = setup();
    ex.execute("SET RESULT MASK ON orders.email PATTERN 'XXX'")
        .unwrap();
    ex.execute("DROP RESULT MASK ON orders.email").unwrap();
    assert!(ex.s281_result_masks.lock_safe().is_empty());
}

// â”€â”€ S282 â€" Table replication lag â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s282_show_replication_lag_for_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW REPLICATION LAG FOR orders").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s282_reset_lag_threshold_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("SET REPLICATION LAG THRESHOLD 5s").unwrap();
    let r = ex.execute("RESET REPLICATION LAG THRESHOLD").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s282_reset_clears_threshold() {
    let (ex, _dir) = setup();
    ex.execute("SET REPLICATION LAG THRESHOLD 5s").unwrap();
    ex.execute("RESET REPLICATION LAG THRESHOLD").unwrap();
    assert!(ex.s282_repl_lag_threshold.lock_safe().is_none());
}

// â”€â”€ S283 â€" Connection idle timeout â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s283_set_idle_timeout_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("SET CONNECTION IDLE TIMEOUT 30s").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s283_show_idle_timeout_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("SET CONNECTION IDLE TIMEOUT 30s").unwrap();
    let r = ex.execute("SHOW CONNECTION IDLE TIMEOUT").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s283_reset_idle_timeout_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("SET CONNECTION IDLE TIMEOUT 30s").unwrap();
    let r = ex.execute("RESET CONNECTION IDLE TIMEOUT").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s283_timeout_value_ms() {
    let (ex, _dir) = setup();
    ex.execute("SET CONNECTION IDLE TIMEOUT 30s").unwrap();
    assert_eq!(ex.s283_idle_timeout_ms.lock_safe().unwrap(), 30000);
}
#[test]
fn test_s283_reset_clears_timeout() {
    let (ex, _dir) = setup();
    ex.execute("SET CONNECTION IDLE TIMEOUT 30s").unwrap();
    ex.execute("RESET CONNECTION IDLE TIMEOUT").unwrap();
    assert!(ex.s283_idle_timeout_ms.lock_safe().is_none());
}
#[test]
fn test_s284_show_tiers_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("SET ARCHIVAL TIER FOR t TO GLACIER").unwrap();
    let r = ex.execute("SHOW ARCHIVAL TIERS").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s284_reset_tier_ok() {
    let (ex, _dir) = setup();
    ex.execute("SET ARCHIVAL TIER FOR t TO GLACIER").unwrap();
    let r = ex.execute("RESET ARCHIVAL TIER FOR t").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s284_reset_removes_entry() {
    let (ex, _dir) = setup();
    ex.execute("SET ARCHIVAL TIER FOR t TO GLACIER").unwrap();
    ex.execute("RESET ARCHIVAL TIER FOR t").unwrap();
    let tiers = ex.s284_archival_tiers.lock_safe();
    assert!(tiers.get("t").is_none());
}
// ── S285 — Query plan diffing ────────────────────────────────────────────────
#[test]
fn test_s285_diff_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("DIFF QUERY PLAN v1 AGAINST v2").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s285_show_diffs_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("DIFF QUERY PLAN v1 AGAINST v2").unwrap();
    let r = ex.execute("SHOW PLAN DIFFS").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s285_clear_diffs_ok() {
    let (ex, _dir) = setup();
    ex.execute("DIFF QUERY PLAN v1 AGAINST v2").unwrap();
    let r = ex.execute("CLEAR PLAN DIFFS").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s285_diff_stored() {
    let (ex, _dir) = setup();
    ex.execute("DIFF QUERY PLAN v1 AGAINST v2").unwrap();
    let diffs = ex.s285_plan_diffs.lock_safe();
    assert_eq!(diffs.len(), 1);
}
#[test]
fn test_s285_clear_empties_diffs() {
    let (ex, _dir) = setup();
    ex.execute("DIFF QUERY PLAN v1 AGAINST v2").unwrap();
    ex.execute("CLEAR PLAN DIFFS").unwrap();
    let diffs = ex.s285_plan_diffs.lock_safe();
    assert!(diffs.is_empty());
}
// ── S286 — Table access audit v2 ────────────────────────────────────────────────
#[test]
fn test_s286_enable_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("ENABLE ACCESS AUDIT ON t").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s286_disable_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("ENABLE ACCESS AUDIT ON t").unwrap();
    let r = ex.execute("DISABLE ACCESS AUDIT ON t").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s286_show_log_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("ENABLE ACCESS AUDIT ON t").unwrap();
    let r = ex.execute("SHOW ACCESS AUDIT LOG FOR t").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s286_enable_sets_flag() {
    let (ex, _dir) = setup();
    ex.execute("ENABLE ACCESS AUDIT ON t").unwrap();
    assert!(ex.s286_access_audit.lock_safe().contains("t"));
}
#[test]
fn test_s286_disable_clears_flag() {
    let (ex, _dir) = setup();
    ex.execute("ENABLE ACCESS AUDIT ON t").unwrap();
    ex.execute("DISABLE ACCESS AUDIT ON t").unwrap();
    assert!(!ex.s286_access_audit.lock_safe().contains("t"));
}
// ── S287 — Index bloat detection ────────────────────────────────────────────────
#[test]
fn test_s287_show_for_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW INDEX BLOAT FOR idx").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s287_show_all_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW ALL INDEX BLOAT").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s287_clear_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("CLEAR INDEX BLOAT STATS").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s287_default_bloat_zero() {
    let (ex, _dir) = setup();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW INDEX BLOAT FOR idx").unwrap() {
        assert_eq!(rows[0][1], "0");
    } else {
        panic!();
    }
}
#[test]
fn test_s287_clear_empties_map() {
    let (ex, _dir) = setup();
    ex.execute("CLEAR INDEX BLOAT STATS").unwrap();
    assert!(ex.s287_index_bloat.lock_safe().is_empty());
}
// ── S288 — Query priority queue ────────────────────────────────────────────────
#[test]
fn test_s288_show_queue_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW QUERY PRIORITY QUEUE").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s288_promote_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("PROMOTE QUERY 1 IN QUEUE").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s288_demote_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("DEMOTE QUERY 1 IN QUEUE").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s288_promote_increments_priority() {
    let (ex, _dir) = setup();
    ex.execute("PROMOTE QUERY 1 IN QUEUE").unwrap();
    let pq = ex.s288_priority_queue.lock_safe();
    assert_eq!(*pq.get(&1).unwrap(), 1);
}
#[test]
fn test_s288_demote_decrements_priority() {
    let (ex, _dir) = setup();
    ex.execute("DEMOTE QUERY 1 IN QUEUE").unwrap();
    let pq = ex.s288_priority_queue.lock_safe();
    assert_eq!(*pq.get(&1).unwrap(), -1);
}
// ── S289 — Table statistics export v2 ────────────────────────────────────────────────
#[test]
fn test_s289_export_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("EXPORT TABLE STATS FOR t FORMAT JSON").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s289_show_formats_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("EXPORT TABLE STATS FOR t FORMAT JSON").unwrap();
    let r = ex.execute("SHOW STATS EXPORT FORMATS V2").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s289_reset_format_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("RESET STATS EXPORT FORMAT V2").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s289_format_stored() {
    let (ex, _dir) = setup();
    ex.execute("EXPORT TABLE STATS FOR t FORMAT JSON").unwrap();
    assert_eq!(*ex.s289_export_format.lock_safe(), "JSON");
}
#[test]
fn test_s289_reset_clears_log() {
    let (ex, _dir) = setup();
    ex.execute("EXPORT TABLE STATS FOR t FORMAT JSON").unwrap();
    ex.execute("RESET STATS EXPORT FORMAT V2").unwrap();
    assert!(ex.s289_export_log.lock_safe().is_empty());
}
// ── S290 — Schema compatibility check ────────────────────────────────────────────────
#[test]
fn test_s290_check_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("CHECK SCHEMA COMPATIBILITY s1 WITH s2").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s290_show_report_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("CHECK SCHEMA COMPATIBILITY s1 WITH s2").unwrap();
    let r = ex.execute("SHOW COMPATIBILITY REPORT").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s290_clear_report_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("CHECK SCHEMA COMPATIBILITY s1 WITH s2").unwrap();
    let r = ex.execute("CLEAR COMPATIBILITY REPORT").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s290_report_populated() {
    let (ex, _dir) = setup();
    ex.execute("CHECK SCHEMA COMPATIBILITY s1 WITH s2").unwrap();
    assert!(ex.s290_compat_report.lock_safe().is_some());
}
#[test]
fn test_s290_clear_empties_report() {
    let (ex, _dir) = setup();
    ex.execute("CHECK SCHEMA COMPATIBILITY s1 WITH s2").unwrap();
    ex.execute("CLEAR COMPATIBILITY REPORT").unwrap();
    assert!(ex.s290_compat_report.lock_safe().is_none());
}
// ── S291 — Table row estimation ────────────────────────────────────────────────
#[test]
fn test_s291_estimate_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    let r = ex.execute("ESTIMATE ROW COUNT FOR t").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s291_show_estimates_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("ESTIMATE ROW COUNT FOR t").unwrap();
    let r = ex.execute("SHOW ROW COUNT ESTIMATES").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s291_refresh_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("ESTIMATE ROW COUNT FOR t").unwrap();
    let r = ex.execute("REFRESH ROW COUNT ESTIMATE FOR t").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s291_estimate_zero_for_empty_table() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("ESTIMATE ROW COUNT FOR t").unwrap();
    let est = ex.s291_row_estimates.lock_safe();
    assert_eq!(*est.get("t").unwrap(), 0);
}
#[test]
fn test_s291_refresh_updates_estimate() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
    ex.execute("REFRESH ROW COUNT ESTIMATE FOR t").unwrap();
    assert!(ex.s291_row_estimates.lock_safe().contains_key("t"));
}
#[test]
fn test_s292_show_policy_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("SET QUERY TIMEOUT ESCALATION '10s,30s,60s'")
        .unwrap();
    let r = ex.execute("SHOW TIMEOUT ESCALATION POLICY").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s292_reset_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("RESET TIMEOUT ESCALATION POLICY").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s292_reset_clears_policy() {
    let (ex, _dir) = setup();
    ex.execute("SET QUERY TIMEOUT ESCALATION '10s,30s,60s'")
        .unwrap();
    ex.execute("RESET TIMEOUT ESCALATION POLICY").unwrap();
    assert!(ex.s292_timeout_escalation.lock_safe().is_empty());
}
// ── S293 — Index usage forecasting ────────────────────────────────────────────────
#[test]
fn test_s293_forecast_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("FORECAST INDEX USAGE FOR idx").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s293_show_forecast_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("FORECAST INDEX USAGE FOR idx").unwrap();
    let r = ex.execute("SHOW INDEX USAGE FORECAST").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s293_clear_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("FORECAST INDEX USAGE FOR idx").unwrap();
    let r = ex.execute("CLEAR INDEX USAGE FORECAST").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s293_forecast_stored() {
    let (ex, _dir) = setup();
    ex.execute("FORECAST INDEX USAGE FOR idx").unwrap();
    assert!(ex.s293_usage_forecast.lock_safe().contains_key("idx"));
}
#[test]
fn test_s293_clear_empties_map() {
    let (ex, _dir) = setup();
    ex.execute("FORECAST INDEX USAGE FOR idx").unwrap();
    ex.execute("CLEAR INDEX USAGE FORECAST").unwrap();
    assert!(ex.s293_usage_forecast.lock_safe().is_empty());
}
// ── S294 — Table snapshot diffing ────────────────────────────────────────────────
#[test]
fn test_s294_diff_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("DIFF SNAPSHOT s1 AGAINST s2 FOR t").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s294_show_diffs_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("DIFF SNAPSHOT s1 AGAINST s2 FOR t").unwrap();
    let r = ex.execute("SHOW SNAPSHOT DIFFS").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s294_clear_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("DIFF SNAPSHOT s1 AGAINST s2 FOR t").unwrap();
    let r = ex.execute("CLEAR SNAPSHOT DIFFS").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s294_diff_stored() {
    let (ex, _dir) = setup();
    ex.execute("DIFF SNAPSHOT s1 AGAINST s2 FOR t").unwrap();
    assert_eq!(ex.s294_snapshot_diffs.lock_safe().len(), 1);
}
#[test]
fn test_s294_clear_empties_diffs() {
    let (ex, _dir) = setup();
    ex.execute("DIFF SNAPSHOT s1 AGAINST s2 FOR t").unwrap();
    ex.execute("CLEAR SNAPSHOT DIFFS").unwrap();
    assert!(ex.s294_snapshot_diffs.lock_safe().is_empty());
}
// ── S295 — Query resource limits ────────────────────────────────────────────────
#[test]
fn test_s295_set_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("SET QUERY MEMORY LIMIT 512MB").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s295_show_limits_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("SET QUERY MEMORY LIMIT 512MB").unwrap();
    let r = ex.execute("SHOW QUERY RESOURCE LIMITS").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s295_reset_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("RESET QUERY RESOURCE LIMITS").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s295_limit_stored() {
    let (ex, _dir) = setup();
    ex.execute("SET QUERY MEMORY LIMIT 512MB").unwrap();
    assert_eq!(ex.s295_memory_limit.lock_safe().clone().unwrap(), "512MB");
}
#[test]
fn test_s295_reset_clears_limit() {
    let (ex, _dir) = setup();
    ex.execute("SET QUERY MEMORY LIMIT 512MB").unwrap();
    ex.execute("RESET QUERY RESOURCE LIMITS").unwrap();
    assert!(ex.s295_memory_limit.lock_safe().is_none());
}
// ── S296 — Table partition pruning stats ────────────────────────────────────────────────
#[test]
fn test_s296_show_for_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW PARTITION PRUNING STATS FOR t").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s296_reset_for_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("RESET PARTITION PRUNING STATS FOR t").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s296_show_all_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW ALL PARTITION PRUNING STATS").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s296_default_zero() {
    let (ex, _dir) = setup();
    if let QueryResult::Rows { rows, .. } =
        ex.execute("SHOW PARTITION PRUNING STATS FOR t").unwrap()
    {
        assert_eq!(rows[0][1], "0");
    } else {
        panic!();
    }
}
#[test]
fn test_s296_reset_removes_entry() {
    let (ex, _dir) = setup();
    ex.s296_pruning_stats.lock_safe().insert("t".to_owned(), 5);
    ex.execute("RESET PARTITION PRUNING STATS FOR t").unwrap();
    assert!(ex.s296_pruning_stats.lock_safe().get("t").is_none());
}
// ── S297 — Schema validation rules ────────────────────────────────────────────────
#[test]
fn test_s297_create_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("CREATE SCHEMA VALIDATION RULE r ON s CHECK 'expr'")
        .unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s297_drop_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("CREATE SCHEMA VALIDATION RULE r ON s CHECK 'expr'")
        .unwrap();
    let r = ex.execute("DROP SCHEMA VALIDATION RULE r").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s297_show_rules_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("CREATE SCHEMA VALIDATION RULE r ON s CHECK 'expr'")
        .unwrap();
    let r = ex.execute("SHOW SCHEMA VALIDATION RULES").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s297_rule_stored() {
    let (ex, _dir) = setup();
    ex.execute("CREATE SCHEMA VALIDATION RULE r ON s CHECK 'expr'")
        .unwrap();
    let rules = ex.s297_validation_rules.lock_safe();
    assert_eq!(rules.get("r").unwrap().0, "s");
}
#[test]
fn test_s297_drop_removes_rule() {
    let (ex, _dir) = setup();
    ex.execute("CREATE SCHEMA VALIDATION RULE r ON s CHECK 'expr'")
        .unwrap();
    ex.execute("DROP SCHEMA VALIDATION RULE r").unwrap();
    assert!(ex.s297_validation_rules.lock_safe().get("r").is_none());
}
// ── S298 — Query explain history ────────────────────────────────────────────────
#[test]
fn test_s298_show_for_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW EXPLAIN HISTORY FOR 'SELECT 1'").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s298_clear_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("CLEAR EXPLAIN HISTORY").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s298_show_summary_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("SHOW EXPLAIN HISTORY FOR 'SELECT 1'").unwrap();
    let r = ex.execute("SHOW EXPLAIN HISTORY SUMMARY").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s298_history_grows() {
    let (ex, _dir) = setup();
    ex.execute("SHOW EXPLAIN HISTORY FOR 'SELECT 1'").unwrap();
    ex.execute("SHOW EXPLAIN HISTORY FOR 'SELECT 2'").unwrap();
    assert_eq!(ex.s298_explain_history.lock_safe().len(), 2);
}
#[test]
fn test_s298_clear_empties_history() {
    let (ex, _dir) = setup();
    ex.execute("SHOW EXPLAIN HISTORY FOR 'SELECT 1'").unwrap();
    ex.execute("CLEAR EXPLAIN HISTORY").unwrap();
    assert!(ex.s298_explain_history.lock_safe().is_empty());
}
// ── S299 — Table compression ratio ────────────────────────────────────────────────
#[test]
fn test_s299_show_ratio_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW COMPRESSION RATIO FOR t").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s299_show_all_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("SET COMPRESSION ALGORITHM FOR t TO ZSTD")
        .unwrap();
    let r = ex.execute("SHOW ALL COMPRESSION RATIOS").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s299_ratio_default_one() {
    let (ex, _dir) = setup();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW COMPRESSION RATIO FOR t").unwrap() {
        assert_eq!(rows[0][1], "1");
    } else {
        panic!();
    }
}
// ── S300 — Milestone 300 sprints ────────────────────────────────────────────────
#[test]
fn test_s300_show_engine_version_info_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW ENGINE VERSION INFO").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s300_show_sprint_milestone_300_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW SPRINT MILESTONE 300").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s300_select_from_vv_milestones_300_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex.execute("SELECT * FROM vv_sprint_300_status").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s300_milestone_value_is_300() {
    let (ex, _dir) = setup();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW SPRINT MILESTONE 300").unwrap() {
        assert_eq!(rows[0][0], "300");
    } else {
        panic!();
    }
}
#[test]
fn test_s300_vv_milestones_status_complete() {
    let (ex, _dir) = setup();
    if let QueryResult::Rows { rows, .. } =
        ex.execute("SELECT * FROM vv_sprint_300_status").unwrap()
    {
        assert_eq!(rows[0][1], "COMPLETE");
    } else {
        panic!();
    }
}

// ── S301 — Query fingerprint caching ─────────────────────────────────────
#[test]
fn test_s301_compute_fingerprint_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex.execute("COMPUTE QUERY FINGERPRINT 'SELECT 1'").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s301_show_fingerprints_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("COMPUTE QUERY FINGERPRINT 'SELECT 1'").unwrap();
    let r = ex.execute("SHOW QUERY FINGERPRINTS").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s301_clear_fingerprints_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("COMPUTE QUERY FINGERPRINT 'SELECT 1'").unwrap();
    let r = ex.execute("CLEAR QUERY FINGERPRINTS").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
    assert!(ex.s301_fingerprints.lock_safe().is_empty());
}
#[test]
fn test_s301_fingerprint_stored_in_map() {
    let (ex, _dir) = setup();
    ex.execute("COMPUTE QUERY FINGERPRINT 'SELECT 2'").unwrap();
    assert_eq!(ex.s301_fingerprints.lock_safe().len(), 1);
}
#[test]
fn test_s301_multiple_fingerprints_accumulate() {
    let (ex, _dir) = setup();
    ex.execute("COMPUTE QUERY FINGERPRINT 'SELECT 1'").unwrap();
    ex.execute("COMPUTE QUERY FINGERPRINT 'SELECT 2'").unwrap();
    assert_eq!(ex.s301_fingerprints.lock_safe().len(), 2);
}

// ── S302 — Table column reordering ───────────────────────────────────────
#[test]
fn test_s302_reorder_columns_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("ALTER TABLE orders REORDER COLUMNS (id,name,total)")
        .unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s302_show_column_order_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("ALTER TABLE orders REORDER COLUMNS (id,name,total)")
        .unwrap();
    let r = ex.execute("SHOW COLUMN ORDER FOR orders").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 3);
    } else {
        panic!();
    }
}
#[test]
fn test_s302_reset_column_order_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("ALTER TABLE orders REORDER COLUMNS (id,name)")
        .unwrap();
    let r = ex.execute("RESET COLUMN ORDER FOR orders").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s302_column_order_values_correct() {
    let (ex, _dir) = setup();
    ex.execute("ALTER TABLE orders REORDER COLUMNS (a,b,c)")
        .unwrap();
    let m = ex.s302_column_order.lock_safe();
    assert_eq!(
        m.get("orders").unwrap(),
        &vec!["a".to_owned(), "b".to_owned(), "c".to_owned()]
    );
}
#[test]
fn test_s302_show_column_order_empty_for_unknown() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW COLUMN ORDER FOR unknown_table").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}

#[test]
fn test_s303_reset_parallelism_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("SET INDEX BUILD PARALLELISM 8").unwrap();
    ex.execute("RESET INDEX BUILD PARALLELISM").unwrap();
    assert_eq!(*ex.s303_index_build_parallelism.lock_safe(), 1);
}
#[test]
fn test_s303_default_parallelism_is_one() {
    let (ex, _dir) = setup();
    assert_eq!(*ex.s303_index_build_parallelism.lock_safe(), 1);
}

#[test]
fn test_s304_reset_sample_rate_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("SET QUERY RESULT SAMPLE RATE 0.2").unwrap();
    ex.execute("RESET QUERY RESULT SAMPLE RATE").unwrap();
    assert_eq!(*ex.s304_sample_rate.lock_safe(), 1.0);
}
#[test]
fn test_s304_default_sample_rate_is_one() {
    let (ex, _dir) = setup();
    assert_eq!(*ex.s304_sample_rate.lock_safe(), 1.0);
}

#[test]
fn test_s305_show_storage_quotas_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("SET STORAGE QUOTA FOR orders TO 10GB").unwrap();
    let r = ex.execute("SHOW STORAGE QUOTAS").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s305_drop_storage_quota_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("SET STORAGE QUOTA FOR orders TO 10GB").unwrap();
    let r = ex.execute("DROP STORAGE QUOTA FOR orders").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
    assert!(ex.s305_storage_quotas.lock_safe().is_empty());
}

// ── S306 — Schema migration rollback points ──────────────────────────────
#[test]
fn test_s306_create_rollback_point_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("CREATE MIGRATION ROLLBACK POINT rp1 FOR public")
        .unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s306_rollback_to_point_succeeds() {
    let (ex, _dir) = setup();
    ex.execute("CREATE MIGRATION ROLLBACK POINT rp1 FOR public")
        .unwrap();
    let r = ex.execute("ROLLBACK TO MIGRATION POINT rp1").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s306_rollback_to_unknown_point_errors() {
    let (ex, _dir) = setup();
    let r = ex.execute("ROLLBACK TO MIGRATION POINT unknown").unwrap();
    assert!(matches!(r, QueryResult::Error(_)));
}
#[test]
fn test_s306_show_rollback_points_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("CREATE MIGRATION ROLLBACK POINT rp1 FOR public")
        .unwrap();
    let r = ex.execute("SHOW MIGRATION ROLLBACK POINTS").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s306_rollback_point_stores_schema() {
    let (ex, _dir) = setup();
    ex.execute("CREATE MIGRATION ROLLBACK POINT rp2 FOR sales")
        .unwrap();
    assert_eq!(
        ex.s306_rollback_points.lock_safe().get("rp2").unwrap(),
        "sales"
    );
}

// ── S307 — Query execution graph export ──────────────────────────────────
#[test]
fn test_s307_export_execution_graph_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("EXPORT EXECUTION GRAPH FOR 'SELECT 1' TO '/tmp/graph.json'")
        .unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s307_show_execution_graphs_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("EXPORT EXECUTION GRAPH FOR 'SELECT 1' TO '/tmp/g.json'")
        .unwrap();
    let r = ex.execute("SHOW EXECUTION GRAPHS").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s307_clear_execution_graphs_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("EXPORT EXECUTION GRAPH FOR 'SELECT 1' TO '/tmp/g.json'")
        .unwrap();
    let r = ex.execute("CLEAR EXECUTION GRAPHS").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
    assert!(ex.s307_execution_graphs.lock_safe().is_empty());
}
#[test]
fn test_s307_graph_path_stored() {
    let (ex, _dir) = setup();
    ex.execute("EXPORT EXECUTION GRAPH FOR 'SELECT 2' TO '/tmp/h.json'")
        .unwrap();
    assert_eq!(
        ex.s307_execution_graphs
            .lock_safe()
            .get("SELECT 2")
            .unwrap(),
        "/tmp/h.json"
    );
}
#[test]
fn test_s307_multiple_graphs_accumulate() {
    let (ex, _dir) = setup();
    ex.execute("EXPORT EXECUTION GRAPH FOR 'SELECT 1' TO '/tmp/a.json'")
        .unwrap();
    ex.execute("EXPORT EXECUTION GRAPH FOR 'SELECT 2' TO '/tmp/b.json'")
        .unwrap();
    assert_eq!(ex.s307_execution_graphs.lock_safe().len(), 2);
}

#[test]
fn test_s308_show_access_rate_limits_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("SET TABLE orders ACCESS RATE LIMIT 100 PER SECOND")
        .unwrap();
    let r = ex.execute("SHOW ACCESS RATE LIMITS").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s308_drop_access_rate_limit_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("SET TABLE orders ACCESS RATE LIMIT 100 PER SECOND")
        .unwrap();
    let r = ex.execute("DROP ACCESS RATE LIMIT FOR orders").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
    assert!(ex.s308_access_rate_limits.lock_safe().is_empty());
}

// ── S309 — Column null ratio stats ───────────────────────────────────────
#[test]
fn test_s309_compute_null_ratio_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex.execute("COMPUTE NULL RATIO FOR orders.total").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s309_show_null_ratios_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("COMPUTE NULL RATIO FOR orders.total").unwrap();
    let r = ex.execute("SHOW NULL RATIOS").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s309_clear_null_ratios_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("COMPUTE NULL RATIO FOR orders.total").unwrap();
    let r = ex.execute("CLEAR NULL RATIOS").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
    assert!(ex.s309_null_ratios.lock_safe().is_empty());
}
#[test]
fn test_s309_ratio_key_stored() {
    let (ex, _dir) = setup();
    ex.execute("COMPUTE NULL RATIO FOR orders.total").unwrap();
    assert!(
        ex.s309_null_ratios
            .lock_safe()
            .contains_key(&("orders".to_owned(), "total".to_owned()))
    );
}
#[test]
fn test_s309_multiple_ratios_accumulate() {
    let (ex, _dir) = setup();
    ex.execute("COMPUTE NULL RATIO FOR orders.total").unwrap();
    ex.execute("COMPUTE NULL RATIO FOR orders.name").unwrap();
    assert_eq!(ex.s309_null_ratios.lock_safe().len(), 2);
}

// ── S310 — Query plan cost comparison ────────────────────────────────────
#[test]
fn test_s310_compare_plan_cost_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex.execute("COMPARE QUERY PLAN COST v1 WITH v2").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s310_show_comparisons_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("COMPARE QUERY PLAN COST v1 WITH v2").unwrap();
    let r = ex.execute("SHOW PLAN COST COMPARISONS").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s310_clear_comparisons_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("COMPARE QUERY PLAN COST v1 WITH v2").unwrap();
    let r = ex.execute("CLEAR PLAN COST COMPARISONS").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
    assert!(ex.s310_plan_cost_comparisons.lock_safe().is_empty());
}
#[test]
fn test_s310_comparison_values_stored() {
    let (ex, _dir) = setup();
    ex.execute("COMPARE QUERY PLAN COST va WITH vb").unwrap();
    let comps = ex.s310_plan_cost_comparisons.lock_safe();
    assert_eq!(comps[0], ("va".to_owned(), "vb".to_owned()));
}
#[test]
fn test_s310_multiple_comparisons_accumulate() {
    let (ex, _dir) = setup();
    ex.execute("COMPARE QUERY PLAN COST v1 WITH v2").unwrap();
    ex.execute("COMPARE QUERY PLAN COST v3 WITH v4").unwrap();
    assert_eq!(ex.s310_plan_cost_comparisons.lock_safe().len(), 2);
}

#[test]
fn test_s311_show_replica_lag_alerts_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("SET REPLICA LAG ALERT THRESHOLD FOR orders TO 5s")
        .unwrap();
    let r = ex.execute("SHOW REPLICA LAG ALERTS").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s311_clear_replica_lag_alert_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("SET REPLICA LAG ALERT THRESHOLD FOR orders TO 5s")
        .unwrap();
    let r = ex.execute("CLEAR REPLICA LAG ALERT FOR orders").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
    assert!(ex.s311_replica_lag_alerts.lock_safe().is_empty());
}

// ── S312 — Index selectivity stats ───────────────────────────────────────
#[test]
fn test_s312_compute_selectivity_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("COMPUTE INDEX SELECTIVITY FOR idx_orders")
        .unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s312_show_selectivity_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("COMPUTE INDEX SELECTIVITY FOR idx_orders")
        .unwrap();
    let r = ex.execute("SHOW INDEX SELECTIVITY").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s312_clear_selectivity_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("COMPUTE INDEX SELECTIVITY FOR idx_orders")
        .unwrap();
    let r = ex.execute("CLEAR INDEX SELECTIVITY").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
    assert!(ex.s312_index_selectivity.lock_safe().is_empty());
}
#[test]
fn test_s312_selectivity_key_stored() {
    let (ex, _dir) = setup();
    ex.execute("COMPUTE INDEX SELECTIVITY FOR idx_a").unwrap();
    assert!(ex.s312_index_selectivity.lock_safe().contains_key("idx_a"));
}
#[test]
fn test_s312_multiple_selectivities() {
    let (ex, _dir) = setup();
    ex.execute("COMPUTE INDEX SELECTIVITY FOR idx_a").unwrap();
    ex.execute("COMPUTE INDEX SELECTIVITY FOR idx_b").unwrap();
    assert_eq!(ex.s312_index_selectivity.lock_safe().len(), 2);
}

#[test]
fn test_s313_reset_global_row_limit_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("SET GLOBAL QUERY ROW LIMIT 5000").unwrap();
    ex.execute("RESET GLOBAL QUERY ROW LIMIT").unwrap();
    assert!(ex.s313_global_row_limit.lock_safe().is_none());
}
#[test]
fn test_s313_default_row_limit_is_none() {
    let (ex, _dir) = setup();
    assert!(ex.s313_global_row_limit.lock_safe().is_none());
}

// ── S314 — Table vacuum scheduling ───────────────────────────────────────
#[test]
fn test_s314_schedule_vacuum_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("SCHEDULE VACUUM FOR orders AT '2026-01-01 00:00'")
        .unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s314_show_vacuum_schedule_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("SCHEDULE VACUUM FOR orders AT '2026-01-01 00:00'")
        .unwrap();
    let r = ex.execute("SHOW VACUUM SCHEDULE").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s314_cancel_vacuum_schedule_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("SCHEDULE VACUUM FOR orders AT '2026-01-01 00:00'")
        .unwrap();
    let r = ex.execute("CANCEL VACUUM SCHEDULE FOR orders").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
    assert!(ex.s314_vacuum_schedule.lock_safe().is_empty());
}
#[test]
fn test_s314_schedule_value_stored() {
    let (ex, _dir) = setup();
    ex.execute("SCHEDULE VACUUM FOR orders AT '2026-02-02 00:00'")
        .unwrap();
    assert_eq!(
        ex.s314_vacuum_schedule.lock_safe().get("orders").unwrap(),
        "2026-02-02 00:00"
    );
}
#[test]
fn test_s314_multiple_schedules() {
    let (ex, _dir) = setup();
    ex.execute("SCHEDULE VACUUM FOR orders AT '2026-01-01 00:00'")
        .unwrap();
    ex.execute("SCHEDULE VACUUM FOR users AT '2026-01-02 00:00'")
        .unwrap();
    assert_eq!(ex.s314_vacuum_schedule.lock_safe().len(), 2);
}

// ── S315 — Schema object dependencies ────────────────────────────────────
#[test]
fn test_s315_show_object_dependencies_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("SHOW OBJECT DEPENDENCIES FOR public.orders")
        .unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s315_show_dependent_objects_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("SHOW DEPENDENT OBJECTS FOR public.orders")
        .unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s315_clear_dependency_cache_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("CLEAR DEPENDENCY CACHE").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s315_clear_dependency_cache_increments_counter() {
    let (ex, _dir) = setup();
    ex.execute("CLEAR DEPENDENCY CACHE").unwrap();
    assert_eq!(*ex.s315_dependency_cache_clears.lock_safe(), 1);
}
#[test]
fn test_s315_clear_dependency_cache_multiple_times() {
    let (ex, _dir) = setup();
    ex.execute("CLEAR DEPENDENCY CACHE").unwrap();
    ex.execute("CLEAR DEPENDENCY CACHE").unwrap();
    assert_eq!(*ex.s315_dependency_cache_clears.lock_safe(), 2);
}

#[test]
fn test_s316_show_connection_tags_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("SET CONNECTION TAG 'reporting'").unwrap();
    let r = ex.execute("SHOW CONNECTION TAGS").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s316_clear_connection_tag_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("SET CONNECTION TAG 'reporting'").unwrap();
    let r = ex.execute("CLEAR CONNECTION TAG").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
    assert!(ex.s316_connection_tags.lock_safe().is_empty());
}

// ── S317 — Table data skew detection ─────────────────────────────────────
#[test]
fn test_s317_detect_data_skew_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex.execute("DETECT DATA SKEW FOR orders.total").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s317_show_data_skew_report_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("DETECT DATA SKEW FOR orders.total").unwrap();
    let r = ex.execute("SHOW DATA SKEW REPORT").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s317_clear_data_skew_report_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("DETECT DATA SKEW FOR orders.total").unwrap();
    let r = ex.execute("CLEAR DATA SKEW REPORT").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
    assert!(ex.s317_data_skew_report.lock_safe().is_empty());
}
#[test]
fn test_s317_report_key_stored() {
    let (ex, _dir) = setup();
    ex.execute("DETECT DATA SKEW FOR orders.total").unwrap();
    assert!(ex.s317_data_skew_report.lock_safe().contains_key("orders"));
}
#[test]
fn test_s317_multiple_skew_reports() {
    let (ex, _dir) = setup();
    ex.execute("DETECT DATA SKEW FOR orders.total").unwrap();
    ex.execute("DETECT DATA SKEW FOR users.age").unwrap();
    assert_eq!(ex.s317_data_skew_report.lock_safe().len(), 2);
}

#[test]
fn test_s318_show_rebuild_triggers_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("SET INDEX REBUILD TRIGGER idx_orders AT BLOAT 50")
        .unwrap();
    let r = ex.execute("SHOW INDEX REBUILD TRIGGERS").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s318_drop_rebuild_trigger_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("SET INDEX REBUILD TRIGGER idx_orders AT BLOAT 50")
        .unwrap();
    let r = ex.execute("DROP INDEX REBUILD TRIGGER idx_orders").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
    assert!(ex.s318_index_rebuild_triggers.lock_safe().is_empty());
}

// ── S319 — Query plan cache eviction ─────────────────────────────────────
#[test]
fn test_s319_evict_plan_cache_entry_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("EVICT QUERY PLAN CACHE ENTRY 'SELECT 1'")
        .unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s319_show_plan_cache_evictions_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("EVICT QUERY PLAN CACHE ENTRY 'SELECT 1'")
        .unwrap();
    let r = ex.execute("SHOW PLAN CACHE EVICTIONS").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s319_clear_plan_cache_evictions_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("EVICT QUERY PLAN CACHE ENTRY 'SELECT 1'")
        .unwrap();
    let r = ex.execute("CLEAR PLAN CACHE EVICTIONS").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
    assert!(ex.s319_plan_cache_evictions.lock_safe().is_empty());
}
#[test]
fn test_s319_eviction_value_stored() {
    let (ex, _dir) = setup();
    ex.execute("EVICT QUERY PLAN CACHE ENTRY 'SELECT 2'")
        .unwrap();
    assert_eq!(ex.s319_plan_cache_evictions.lock_safe()[0], "SELECT 2");
}
#[test]
fn test_s319_multiple_evictions_accumulate() {
    let (ex, _dir) = setup();
    ex.execute("EVICT QUERY PLAN CACHE ENTRY 'SELECT 1'")
        .unwrap();
    ex.execute("EVICT QUERY PLAN CACHE ENTRY 'SELECT 2'")
        .unwrap();
    assert_eq!(ex.s319_plan_cache_evictions.lock_safe().len(), 2);
}

#[test]
fn test_s320_show_write_throttles_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("SET TABLE orders WRITE THROTTLE 1000 PER SECOND")
        .unwrap();
    let r = ex.execute("SHOW WRITE THROTTLES").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s320_drop_write_throttle_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("SET TABLE orders WRITE THROTTLE 1000 PER SECOND")
        .unwrap();
    let r = ex.execute("DROP WRITE THROTTLE FOR orders").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
    assert!(ex.s320_write_throttles.lock_safe().is_empty());
}

// ── S321 — Column cardinality estimation ─────────────────────────────────
#[test]
fn test_s321_estimate_cardinality_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex.execute("ESTIMATE CARDINALITY FOR orders.total").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s321_show_cardinality_estimates_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("ESTIMATE CARDINALITY FOR orders.total").unwrap();
    let r = ex.execute("SHOW CARDINALITY ESTIMATES").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s321_clear_cardinality_estimates_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("ESTIMATE CARDINALITY FOR orders.total").unwrap();
    let r = ex.execute("CLEAR CARDINALITY ESTIMATES").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
    assert!(ex.s321_cardinality_estimates.lock_safe().is_empty());
}
#[test]
fn test_s321_estimate_key_stored() {
    let (ex, _dir) = setup();
    ex.execute("ESTIMATE CARDINALITY FOR orders.total").unwrap();
    assert!(
        ex.s321_cardinality_estimates
            .lock_safe()
            .contains_key(&("orders".to_owned(), "total".to_owned()))
    );
}
#[test]
fn test_s321_multiple_estimates_accumulate() {
    let (ex, _dir) = setup();
    ex.execute("ESTIMATE CARDINALITY FOR orders.total").unwrap();
    ex.execute("ESTIMATE CARDINALITY FOR orders.name").unwrap();
    assert_eq!(ex.s321_cardinality_estimates.lock_safe().len(), 2);
}

// ── S322 — Query session variables v2 ────────────────────────────────────
#[test]
fn test_s322_set_session_var2_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("SET SESSION VAR2 mode = 'fast'").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s322_show_session_vars2_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("SET SESSION VAR2 mode = 'fast'").unwrap();
    let r = ex.execute("SHOW SESSION VARS2").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s322_unset_session_var2_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("SET SESSION VAR2 mode = 'fast'").unwrap();
    let r = ex.execute("UNSET SESSION VAR2 mode").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
    assert!(ex.s322_session_vars2.lock_safe().is_empty());
}
#[test]
fn test_s322_var_value_stored() {
    let (ex, _dir) = setup();
    ex.execute("SET SESSION VAR2 lang = 'rust'").unwrap();
    assert_eq!(
        ex.s322_session_vars2.lock_safe().get("lang").unwrap(),
        "rust"
    );
}
#[test]
fn test_s322_multiple_vars() {
    let (ex, _dir) = setup();
    ex.execute("SET SESSION VAR2 a = '1'").unwrap();
    ex.execute("SET SESSION VAR2 b = '2'").unwrap();
    assert_eq!(ex.s322_session_vars2.lock_safe().len(), 2);
}

// ── S323 — Table backup scheduling ───────────────────────────────────────
#[test]
fn test_s323_schedule_backup_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("SCHEDULE BACKUP FOR orders EVERY '1d'").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s323_show_backup_schedule_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("SCHEDULE BACKUP FOR orders EVERY '1d'").unwrap();
    let r = ex.execute("SHOW BACKUP SCHEDULE").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s323_cancel_backup_schedule_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("SCHEDULE BACKUP FOR orders EVERY '1d'").unwrap();
    let r = ex.execute("CANCEL BACKUP SCHEDULE FOR orders").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
    assert!(ex.s323_backup_schedule.lock_safe().is_empty());
}
#[test]
fn test_s323_schedule_value_stored() {
    let (ex, _dir) = setup();
    ex.execute("SCHEDULE BACKUP FOR orders EVERY '2d'").unwrap();
    assert_eq!(
        ex.s323_backup_schedule.lock_safe().get("orders").unwrap(),
        "2d"
    );
}
#[test]
fn test_s323_multiple_schedules() {
    let (ex, _dir) = setup();
    ex.execute("SCHEDULE BACKUP FOR orders EVERY '1d'").unwrap();
    ex.execute("SCHEDULE BACKUP FOR users EVERY '7d'").unwrap();
    assert_eq!(ex.s323_backup_schedule.lock_safe().len(), 2);
}

#[test]
fn test_s324_show_fragmentation_alerts_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("SET INDEX FRAGMENTATION ALERT idx_orders AT 30")
        .unwrap();
    let r = ex.execute("SHOW INDEX FRAGMENTATION ALERTS").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s324_clear_fragmentation_alert_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("SET INDEX FRAGMENTATION ALERT idx_orders AT 30")
        .unwrap();
    let r = ex
        .execute("CLEAR INDEX FRAGMENTATION ALERT FOR idx_orders")
        .unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
    assert!(ex.s324_index_fragmentation_alerts.lock_safe().is_empty());
}

#[test]
fn test_s325_reset_result_set_size_limit_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("SET RESULT SET SIZE LIMIT 5MB").unwrap();
    ex.execute("RESET RESULT SET SIZE LIMIT").unwrap();
    assert!(ex.s325_result_set_size_limit.lock_safe().is_none());
}
#[test]
fn test_s325_default_limit_is_none() {
    let (ex, _dir) = setup();
    assert!(ex.s325_result_set_size_limit.lock_safe().is_none());
}

// ── S326 — Table hot key detection ───────────────────────────────────────
#[test]
fn test_s326_detect_hot_keys_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex.execute("DETECT HOT KEYS FOR orders").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s326_show_hot_key_report_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("DETECT HOT KEYS FOR orders").unwrap();
    let r = ex.execute("SHOW HOT KEY REPORT").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s326_clear_hot_key_report_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("DETECT HOT KEYS FOR orders").unwrap();
    let r = ex.execute("CLEAR HOT KEY REPORT").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
    assert!(ex.s326_hot_key_report.lock_safe().is_empty());
}
#[test]
fn test_s326_report_key_stored() {
    let (ex, _dir) = setup();
    ex.execute("DETECT HOT KEYS FOR orders").unwrap();
    assert!(ex.s326_hot_key_report.lock_safe().contains_key("orders"));
}
#[test]
fn test_s326_multiple_reports() {
    let (ex, _dir) = setup();
    ex.execute("DETECT HOT KEYS FOR orders").unwrap();
    ex.execute("DETECT HOT KEYS FOR users").unwrap();
    assert_eq!(ex.s326_hot_key_report.lock_safe().len(), 2);
}

// ── S327 — Schema change approval workflow ───────────────────────────────
#[test]
fn test_s327_request_approval_returns_id() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("REQUEST SCHEMA CHANGE APPROVAL FOR public DESC 'add column'")
        .unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s327_approve_schema_change_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("REQUEST SCHEMA CHANGE APPROVAL FOR public DESC 'add column'")
        .unwrap();
    let r = ex.execute("APPROVE SCHEMA CHANGE 1").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s327_approve_unknown_change_errors() {
    let (ex, _dir) = setup();
    let r = ex.execute("APPROVE SCHEMA CHANGE 999").unwrap();
    assert!(matches!(r, QueryResult::Error(_)));
}
#[test]
fn test_s327_show_pending_changes_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("REQUEST SCHEMA CHANGE APPROVAL FOR public DESC 'add column'")
        .unwrap();
    let r = ex.execute("SHOW PENDING SCHEMA CHANGES").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 1);
    } else {
        panic!();
    }
}
#[test]
fn test_s327_approved_change_not_pending() {
    let (ex, _dir) = setup();
    ex.execute("REQUEST SCHEMA CHANGE APPROVAL FOR public DESC 'add column'")
        .unwrap();
    ex.execute("APPROVE SCHEMA CHANGE 1").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW PENDING SCHEMA CHANGES").unwrap() {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}

// ── S328 — Query cost anomaly detection ──────────────────────────────────
#[test]
fn test_s328_detect_cost_anomalies_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("DETECT QUERY COST ANOMALIES").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s328_show_cost_anomalies_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW COST ANOMALIES").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s328_clear_cost_anomalies_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("CLEAR COST ANOMALIES").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s328_detect_anomalies_empty_by_default() {
    let (ex, _dir) = setup();
    ex.execute("DETECT QUERY COST ANOMALIES").unwrap();
    assert!(ex.s328_cost_anomalies.lock_safe().is_empty());
}
#[test]
fn test_s328_clear_after_detect() {
    let (ex, _dir) = setup();
    ex.execute("DETECT QUERY COST ANOMALIES").unwrap();
    ex.execute("CLEAR COST ANOMALIES").unwrap();
    assert!(ex.s328_cost_anomalies.lock_safe().is_empty());
}

// ── S329 — Table read replica promotion ──────────────────────────────────
#[test]
fn test_s329_promote_read_replica_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("PROMOTE READ REPLICA 'host1:5432'").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s329_show_promotion_history_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("PROMOTE READ REPLICA 'host1:5432'").unwrap();
    let r = ex.execute("SHOW REPLICA PROMOTION HISTORY").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s329_clear_promotion_history_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("PROMOTE READ REPLICA 'host1:5432'").unwrap();
    let r = ex.execute("CLEAR REPLICA PROMOTION HISTORY").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
    assert!(ex.s329_replica_promotions.lock_safe().is_empty());
}
#[test]
fn test_s329_promotion_value_stored() {
    let (ex, _dir) = setup();
    ex.execute("PROMOTE READ REPLICA 'host2:5432'").unwrap();
    assert_eq!(ex.s329_replica_promotions.lock_safe()[0], "host2:5432");
}
#[test]
fn test_s329_multiple_promotions_accumulate() {
    let (ex, _dir) = setup();
    ex.execute("PROMOTE READ REPLICA 'host1:5432'").unwrap();
    ex.execute("PROMOTE READ REPLICA 'host2:5432'").unwrap();
    assert_eq!(ex.s329_replica_promotions.lock_safe().len(), 2);
}

// ── S330 — Index usage heatmap ────────────────────────────────────────────
#[test]
fn test_s330_show_index_usage_heatmap_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW INDEX USAGE HEATMAP").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s330_refresh_index_usage_heatmap_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("REFRESH INDEX USAGE HEATMAP").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s330_clear_index_usage_heatmap_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("CLEAR INDEX USAGE HEATMAP").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s330_heatmap_empty_by_default() {
    let (ex, _dir) = setup();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW INDEX USAGE HEATMAP").unwrap() {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}
#[test]
fn test_s330_clear_after_refresh() {
    let (ex, _dir) = setup();
    ex.execute("REFRESH INDEX USAGE HEATMAP").unwrap();
    ex.execute("CLEAR INDEX USAGE HEATMAP").unwrap();
    assert!(ex.s330_index_usage_heatmap.lock_safe().is_empty());
}

#[test]
fn test_s331_reset_batch_retry_policy_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("SET BATCH RETRY POLICY max=3 backoff=200ms")
        .unwrap();
    ex.execute("RESET BATCH RETRY POLICY").unwrap();
    assert!(ex.s331_batch_retry_policy.lock_safe().is_none());
}
#[test]
fn test_s331_default_policy_is_none() {
    let (ex, _dir) = setup();
    assert!(ex.s331_batch_retry_policy.lock_safe().is_none());
}

// ── S332 — Table storage compression stats ───────────────────────────────
#[test]
fn test_s332_show_compression_stats_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("SHOW STORAGE COMPRESSION STATS FOR orders")
        .unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s332_refresh_compression_stats_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("REFRESH STORAGE COMPRESSION STATS FOR orders")
        .unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s332_clear_compression_stats_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("REFRESH STORAGE COMPRESSION STATS FOR orders")
        .unwrap();
    let r = ex.execute("CLEAR STORAGE COMPRESSION STATS").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
    assert!(ex.s332_storage_compression_stats.lock_safe().is_empty());
}
#[test]
fn test_s332_stats_stored_after_refresh() {
    let (ex, _dir) = setup();
    ex.execute("REFRESH STORAGE COMPRESSION STATS FOR orders")
        .unwrap();
    assert!(
        ex.s332_storage_compression_stats
            .lock_safe()
            .contains_key("orders")
    );
}
#[test]
fn test_s332_unknown_table_returns_default() {
    let (ex, _dir) = setup();
    if let QueryResult::Rows { rows, .. } = ex
        .execute("SHOW STORAGE COMPRESSION STATS FOR unknown_tbl")
        .unwrap()
    {
        assert_eq!(rows[0][1], "UNKNOWN");
    } else {
        panic!();
    }
}

// ── S333 — Schema drift detection ────────────────────────────────────────
#[test]
fn test_s333_detect_schema_drift_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("DETECT SCHEMA DRIFT BETWEEN dev AND prod")
        .unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s333_show_drift_report_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("DETECT SCHEMA DRIFT BETWEEN dev AND prod")
        .unwrap();
    let r = ex.execute("SHOW SCHEMA DRIFT REPORT").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s333_clear_drift_report_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("DETECT SCHEMA DRIFT BETWEEN dev AND prod")
        .unwrap();
    let r = ex.execute("CLEAR SCHEMA DRIFT REPORT").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
    assert!(ex.s333_schema_drift_report.lock_safe().is_none());
}
#[test]
fn test_s333_default_report_is_none() {
    let (ex, _dir) = setup();
    assert!(ex.s333_schema_drift_report.lock_safe().is_none());
}
#[test]
fn test_s333_report_contains_schemas() {
    let (ex, _dir) = setup();
    ex.execute("DETECT SCHEMA DRIFT BETWEEN dev AND prod")
        .unwrap();
    let report = ex.s333_schema_drift_report.lock_safe().clone().unwrap();
    assert!(report.contains("dev") && report.contains("prod"));
}

// ── S334 — Query plan stability tracking ─────────────────────────────────
#[test]
fn test_s334_track_plan_stability_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("TRACK QUERY PLAN STABILITY FOR 'SELECT 1'")
        .unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s334_show_plan_stability_report_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("TRACK QUERY PLAN STABILITY FOR 'SELECT 1'")
        .unwrap();
    let r = ex.execute("SHOW PLAN STABILITY REPORT").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s334_clear_plan_stability_report_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("TRACK QUERY PLAN STABILITY FOR 'SELECT 1'")
        .unwrap();
    let r = ex.execute("CLEAR PLAN STABILITY REPORT").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
    assert!(ex.s334_plan_stability_report.lock_safe().is_empty());
}
#[test]
fn test_s334_tracked_query_key_stored() {
    let (ex, _dir) = setup();
    ex.execute("TRACK QUERY PLAN STABILITY FOR 'SELECT 2'")
        .unwrap();
    assert!(
        ex.s334_plan_stability_report
            .lock_safe()
            .contains_key("SELECT 2")
    );
}
#[test]
fn test_s334_multiple_tracked_queries() {
    let (ex, _dir) = setup();
    ex.execute("TRACK QUERY PLAN STABILITY FOR 'SELECT 1'")
        .unwrap();
    ex.execute("TRACK QUERY PLAN STABILITY FOR 'SELECT 2'")
        .unwrap();
    assert_eq!(ex.s334_plan_stability_report.lock_safe().len(), 2);
}

// ── S335 — Table connection draining ─────────────────────────────────────
#[test]
fn test_s335_drain_connections_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("DRAIN CONNECTIONS FOR orders").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s335_show_draining_tables_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("DRAIN CONNECTIONS FOR orders").unwrap();
    let r = ex.execute("SHOW DRAINING TABLES").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s335_cancel_drain_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("DRAIN CONNECTIONS FOR orders").unwrap();
    let r = ex.execute("CANCEL DRAIN FOR orders").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
    assert!(ex.s335_draining_tables.lock_safe().is_empty());
}
#[test]
fn test_s335_table_marked_draining() {
    let (ex, _dir) = setup();
    ex.execute("DRAIN CONNECTIONS FOR orders").unwrap();
    assert!(ex.s335_draining_tables.lock_safe().contains("orders"));
}
#[test]
fn test_s335_multiple_draining_tables() {
    let (ex, _dir) = setup();
    ex.execute("DRAIN CONNECTIONS FOR orders").unwrap();
    ex.execute("DRAIN CONNECTIONS FOR users").unwrap();
    assert_eq!(ex.s335_draining_tables.lock_safe().len(), 2);
}

#[test]
fn test_s336_reset_resource_limits_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("SET INDEX BUILD MEMORY LIMIT 256MB").unwrap();
    ex.execute("RESET INDEX BUILD RESOURCE LIMITS").unwrap();
    assert!(ex.s336_index_build_resource_limits.lock_safe().is_none());
}
#[test]
fn test_s336_default_limit_is_none() {
    let (ex, _dir) = setup();
    assert!(ex.s336_index_build_resource_limits.lock_safe().is_none());
}

#[test]
fn test_s337_show_streaming_status_off_default() {
    let (ex, _dir) = setup();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW STREAMING V2 STATUS").unwrap() {
        assert_eq!(rows[0][0], "OFF");
    } else {
        panic!();
    }
}
#[test]
fn test_s337_toggle_streaming_v2() {
    let (ex, _dir) = setup();
    ex.execute("SET STREAMING V2 MODE ON").unwrap();
    ex.execute("SET STREAMING V2 MODE OFF").unwrap();
    assert!(!*ex.s337_streaming_v2_mode.lock_safe());
}

// ── S338 — Table shard split/merge ───────────────────────────────────────
#[test]
fn test_s338_split_shard_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("SPLIT SHARD FOR orders AT KEY 'k100'").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s338_merge_shards_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("MERGE SHARDS FOR orders").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s338_show_history_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("SPLIT SHARD FOR orders AT KEY 'k100'").unwrap();
    let r = ex.execute("SHOW SHARD SPLIT MERGE HISTORY").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s338_history_records_split_and_merge() {
    let (ex, _dir) = setup();
    ex.execute("SPLIT SHARD FOR orders AT KEY 'k100'").unwrap();
    ex.execute("MERGE SHARDS FOR orders").unwrap();
    assert_eq!(ex.s338_shard_split_merge_history.lock_safe().len(), 2);
}
#[test]
fn test_s338_split_event_format() {
    let (ex, _dir) = setup();
    ex.execute("SPLIT SHARD FOR orders AT KEY 'k1'").unwrap();
    assert!(ex.s338_shard_split_merge_history.lock_safe()[0].contains("SPLIT"));
}

// ── S339 — Schema object tagging ─────────────────────────────────────────
#[test]
fn test_s339_tag_schema_object_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("TAG SCHEMA OBJECT public.orders WITH 'pii'")
        .unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s339_show_object_tags_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("TAG SCHEMA OBJECT public.orders WITH 'pii'")
        .unwrap();
    let r = ex.execute("SHOW OBJECT TAGS").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s339_remove_tag_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("TAG SCHEMA OBJECT public.orders WITH 'pii'")
        .unwrap();
    let r = ex.execute("REMOVE TAG FROM public.orders").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
    assert!(ex.s339_object_tags.lock_safe().is_empty());
}
#[test]
fn test_s339_tag_value_stored() {
    let (ex, _dir) = setup();
    ex.execute("TAG SCHEMA OBJECT public.orders WITH 'sensitive'")
        .unwrap();
    assert_eq!(
        ex.s339_object_tags
            .lock_safe()
            .get("public.orders")
            .unwrap()[0],
        "sensitive"
    );
}
#[test]
fn test_s339_multiple_tags_on_same_object() {
    let (ex, _dir) = setup();
    ex.execute("TAG SCHEMA OBJECT public.orders WITH 'pii'")
        .unwrap();
    ex.execute("TAG SCHEMA OBJECT public.orders WITH 'sensitive'")
        .unwrap();
    assert_eq!(
        ex.s339_object_tags
            .lock_safe()
            .get("public.orders")
            .unwrap()
            .len(),
        2
    );
}

// ── S340 — 340-sprint checkpoint ─────────────────────────────────────────
#[test]
fn test_s340_show_sprint_checkpoint_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW SPRINT CHECKPOINT 340").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s340_select_from_checkpoint_table() {
    let (ex, _dir) = setup();
    let r = ex.execute("SELECT * FROM vv_checkpoint_340").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s340_show_total_sql_command_count() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW TOTAL SQL COMMAND COUNT").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s340_checkpoint_value_is_340() {
    let (ex, _dir) = setup();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW SPRINT CHECKPOINT 340").unwrap() {
        assert_eq!(rows[0][0], "340");
    } else {
        panic!();
    }
}
#[test]
fn test_s340_checkpoint_status_complete() {
    let (ex, _dir) = setup();
    if let QueryResult::Rows { rows, .. } = ex.execute("SELECT * FROM vv_checkpoint_340").unwrap() {
        assert_eq!(rows[0][1], "COMPLETE");
    } else {
        panic!();
    }
}

#[test]
fn test_s341_status_default_off() {
    let (ex, _dir) = setup();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW QUERY TEXT REDACTION STATUS").unwrap()
    {
        assert_eq!(rows[0][0], "OFF");
    } else {
        panic!();
    }
}
#[test]
fn test_s341_toggle_roundtrip() {
    let (ex, _dir) = setup();
    ex.execute("SET QUERY TEXT REDACTION ON").unwrap();
    ex.execute("SET QUERY TEXT REDACTION OFF").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW QUERY TEXT REDACTION STATUS").unwrap()
    {
        assert_eq!(rows[0][0], "OFF");
    } else {
        panic!();
    }
}

#[test]
fn test_s342_reset_codec() {
    let (ex, _dir) = setup();
    ex.execute("SET COLUMN CODEC FOR t.col TO ZSTD").unwrap();
    ex.execute("RESET COLUMN CODEC FOR t.col").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW COLUMN CODECS").unwrap() {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}
#[test]
fn test_s342_reset_nonexistent_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("RESET COLUMN CODEC FOR t.missing").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}

// ── S343 — index concurrent build mode ───────────────────────────────────
#[test]
fn test_s343_create_index_concurrently() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("CREATE INDEX idx1 ON t(col) CONCURRENTLY")
        .unwrap();
    assert!(matches!(r, QueryResult::Created));
}
#[test]
fn test_s343_show_builds() {
    let (ex, _dir) = setup();
    ex.execute("CREATE INDEX idx1 ON t(col) CONCURRENTLY")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW CONCURRENT INDEX BUILDS").unwrap() {
        assert_eq!(rows[0][0], "idx1");
    } else {
        panic!();
    }
}
#[test]
fn test_s343_cancel_build() {
    let (ex, _dir) = setup();
    ex.execute("CREATE INDEX idx1 ON t(col) CONCURRENTLY")
        .unwrap();
    ex.execute("CANCEL CONCURRENT INDEX BUILD idx1").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW CONCURRENT INDEX BUILDS").unwrap() {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}
#[test]
fn test_s343_empty_builds_initially() {
    let (ex, _dir) = setup();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW CONCURRENT INDEX BUILDS").unwrap() {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}
#[test]
fn test_s343_cancel_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("CANCEL CONCURRENT INDEX BUILD idx_x").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}

#[test]
fn test_s344_default_encoding() {
    let (ex, _dir) = setup();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW RESULT ENCODING").unwrap() {
        assert_eq!(rows[0][0], "UTF8");
    } else {
        panic!();
    }
}
#[test]
fn test_s344_reset_encoding() {
    let (ex, _dir) = setup();
    ex.execute("SET RESULT ENCODING LATIN1").unwrap();
    ex.execute("RESET RESULT ENCODING").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW RESULT ENCODING").unwrap() {
        assert_eq!(rows[0][0], "UTF8");
    } else {
        panic!();
    }
}
#[test]
fn test_s344_reset_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("RESET RESULT ENCODING").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}

// ── S345 — table storage class migration ─────────────────────────────────
#[test]
fn test_s345_migrate_table() {
    let (ex, _dir) = setup();
    let r = ex.execute("MIGRATE TABLE t STORAGE CLASS TO SSD").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s345_show_migrations() {
    let (ex, _dir) = setup();
    ex.execute("MIGRATE TABLE t STORAGE CLASS TO SSD").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW STORAGE CLASS MIGRATIONS").unwrap() {
        assert_eq!(rows[0], vec!["t".to_owned(), "SSD".to_owned()]);
    } else {
        panic!();
    }
}
#[test]
fn test_s345_cancel_migration() {
    let (ex, _dir) = setup();
    ex.execute("MIGRATE TABLE t STORAGE CLASS TO SSD").unwrap();
    ex.execute("CANCEL STORAGE CLASS MIGRATION FOR t").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW STORAGE CLASS MIGRATIONS").unwrap() {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}
#[test]
fn test_s345_empty_initially() {
    let (ex, _dir) = setup();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW STORAGE CLASS MIGRATIONS").unwrap() {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}
#[test]
fn test_s345_cancel_nonexistent_ok() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("CANCEL STORAGE CLASS MIGRATION FOR missing")
        .unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}

// ── S346 — schema baseline snapshots ──────────────────────────────────────
#[test]
fn test_s346_create_baseline() {
    let (ex, _dir) = setup();
    let r = ex.execute("CREATE SCHEMA BASELINE bl1 FOR s1").unwrap();
    assert!(matches!(r, QueryResult::Created));
}
#[test]
fn test_s346_show_baselines() {
    let (ex, _dir) = setup();
    ex.execute("CREATE SCHEMA BASELINE bl1 FOR s1").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW SCHEMA BASELINES").unwrap() {
        assert_eq!(rows[0], vec!["bl1".to_owned(), "s1".to_owned()]);
    } else {
        panic!();
    }
}
#[test]
fn test_s346_drop_baseline() {
    let (ex, _dir) = setup();
    ex.execute("CREATE SCHEMA BASELINE bl1 FOR s1").unwrap();
    let r = ex.execute("DROP SCHEMA BASELINE bl1").unwrap();
    assert!(matches!(r, QueryResult::Dropped));
}
#[test]
fn test_s346_dropped_not_shown() {
    let (ex, _dir) = setup();
    ex.execute("CREATE SCHEMA BASELINE bl1 FOR s1").unwrap();
    ex.execute("DROP SCHEMA BASELINE bl1").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW SCHEMA BASELINES").unwrap() {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}
#[test]
fn test_s346_empty_initially() {
    let (ex, _dir) = setup();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW SCHEMA BASELINES").unwrap() {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}

// ── S347 — query execution timeline ──────────────────────────────────────
#[test]
fn test_s347_show_timeline_for() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("SHOW EXECUTION TIMELINE FOR 'SELECT 1'")
        .unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s347_timeline_summary() {
    let (ex, _dir) = setup();
    ex.execute("SHOW EXECUTION TIMELINE FOR 'SELECT 1'")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW EXECUTION TIMELINE SUMMARY").unwrap() {
        assert_eq!(rows[0][0], "1");
    } else {
        panic!();
    }
}
#[test]
fn test_s347_clear_timeline() {
    let (ex, _dir) = setup();
    ex.execute("SHOW EXECUTION TIMELINE FOR 'SELECT 1'")
        .unwrap();
    ex.execute("CLEAR EXECUTION TIMELINE").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW EXECUTION TIMELINE SUMMARY").unwrap() {
        assert_eq!(rows[0][0], "0");
    } else {
        panic!();
    }
}
#[test]
fn test_s347_summary_zero_initially() {
    let (ex, _dir) = setup();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW EXECUTION TIMELINE SUMMARY").unwrap() {
        assert_eq!(rows[0][0], "0");
    } else {
        panic!();
    }
}
#[test]
fn test_s347_multiple_entries() {
    let (ex, _dir) = setup();
    ex.execute("SHOW EXECUTION TIMELINE FOR 'SELECT 1'")
        .unwrap();
    ex.execute("SHOW EXECUTION TIMELINE FOR 'SELECT 2'")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW EXECUTION TIMELINE SUMMARY").unwrap() {
        assert_eq!(rows[0][0], "2");
    } else {
        panic!();
    }
}

#[test]
fn test_s348_drop_quota() {
    let (ex, _dir) = setup();
    ex.execute("SET CONNECTION QUOTA FOR t TO 50").unwrap();
    let r = ex.execute("DROP CONNECTION QUOTA FOR t").unwrap();
    assert!(matches!(r, QueryResult::Dropped));
}
#[test]
fn test_s348_dropped_not_shown() {
    let (ex, _dir) = setup();
    ex.execute("SET CONNECTION QUOTA FOR t TO 50").unwrap();
    ex.execute("DROP CONNECTION QUOTA FOR t").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW CONNECTION QUOTAS").unwrap() {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}
#[test]
fn test_s348_empty_initially() {
    let (ex, _dir) = setup();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW CONNECTION QUOTAS").unwrap() {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}

// ── S349 — column top-k values ────────────────────────────────────────────
#[test]
fn test_s349_compute_topk() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("COMPUTE TOP K VALUES FOR t.col LIMIT 10")
        .unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s349_show_topk() {
    let (ex, _dir) = setup();
    ex.execute("COMPUTE TOP K VALUES FOR t.col LIMIT 10")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW TOP K VALUES FOR t.col").unwrap() {
        assert_eq!(rows[0], vec!["t.col".to_owned(), "10".to_owned()]);
    } else {
        panic!();
    }
}
#[test]
fn test_s349_show_topk_default_zero() {
    let (ex, _dir) = setup();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW TOP K VALUES FOR t.other").unwrap() {
        assert_eq!(rows[0][1], "0");
    } else {
        panic!();
    }
}
#[test]
fn test_s349_clear_topk() {
    let (ex, _dir) = setup();
    ex.execute("COMPUTE TOP K VALUES FOR t.col LIMIT 10")
        .unwrap();
    let r = ex.execute("CLEAR TOP K VALUES").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s349_cleared_resets_to_zero() {
    let (ex, _dir) = setup();
    ex.execute("COMPUTE TOP K VALUES FOR t.col LIMIT 10")
        .unwrap();
    ex.execute("CLEAR TOP K VALUES").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW TOP K VALUES FOR t.col").unwrap() {
        assert_eq!(rows[0][1], "0");
    } else {
        panic!();
    }
}

// ── S350 — query plan cache warming ──────────────────────────────────────
#[test]
fn test_s350_warm_cache() {
    let (ex, _dir) = setup();
    let r = ex.execute("WARM QUERY PLAN CACHE WITH 'SELECT 1'").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s350_show_warm_status() {
    let (ex, _dir) = setup();
    ex.execute("WARM QUERY PLAN CACHE WITH 'SELECT 1'").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW PLAN CACHE WARM STATUS").unwrap() {
        assert_eq!(rows[0][0], "SELECT 1");
    } else {
        panic!();
    }
}
#[test]
fn test_s350_clear_warm_status() {
    let (ex, _dir) = setup();
    ex.execute("WARM QUERY PLAN CACHE WITH 'SELECT 1'").unwrap();
    ex.execute("CLEAR PLAN CACHE WARM STATUS").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW PLAN CACHE WARM STATUS").unwrap() {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}
#[test]
fn test_s350_empty_initially() {
    let (ex, _dir) = setup();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW PLAN CACHE WARM STATUS").unwrap() {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}
#[test]
fn test_s350_multiple_warms() {
    let (ex, _dir) = setup();
    ex.execute("WARM QUERY PLAN CACHE WITH 'SELECT 1'").unwrap();
    ex.execute("WARM QUERY PLAN CACHE WITH 'SELECT 2'").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW PLAN CACHE WARM STATUS").unwrap() {
        assert_eq!(rows.len(), 2);
    } else {
        panic!();
    }
}

#[test]
fn test_s351_reset_preference() {
    let (ex, _dir) = setup();
    ex.execute("SET READ PREFERENCE FOR t TO NEAREST").unwrap();
    ex.execute("RESET READ PREFERENCE FOR t").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW READ PREFERENCES").unwrap() {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}
#[test]
fn test_s351_empty_initially() {
    let (ex, _dir) = setup();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW READ PREFERENCES").unwrap() {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}
#[test]
fn test_s351_reset_nonexistent_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("RESET READ PREFERENCE FOR missing").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}

// ── S352 — index online rebuild ──────────────────────────────────────────
#[test]
fn test_s352_rebuild_online() {
    let (ex, _dir) = setup();
    let r = ex.execute("REBUILD INDEX idx1 ONLINE").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s352_show_status() {
    let (ex, _dir) = setup();
    ex.execute("REBUILD INDEX idx1 ONLINE").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW ONLINE REBUILD STATUS").unwrap() {
        assert_eq!(rows[0][0], "idx1");
    } else {
        panic!();
    }
}
#[test]
fn test_s352_cancel_rebuild() {
    let (ex, _dir) = setup();
    ex.execute("REBUILD INDEX idx1 ONLINE").unwrap();
    ex.execute("CANCEL ONLINE REBUILD FOR idx1").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW ONLINE REBUILD STATUS").unwrap() {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}
#[test]
fn test_s352_empty_initially() {
    let (ex, _dir) = setup();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW ONLINE REBUILD STATUS").unwrap() {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}
#[test]
fn test_s352_cancel_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("CANCEL ONLINE REBUILD FOR missing").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}

#[test]
fn test_s353_default_off() {
    let (ex, _dir) = setup();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW RESULT CHECKSUM STATUS").unwrap() {
        assert_eq!(rows[0][0], "OFF");
    } else {
        panic!();
    }
}
#[test]
fn test_s353_toggle_roundtrip() {
    let (ex, _dir) = setup();
    ex.execute("SET RESULT CHECKSUM ON").unwrap();
    ex.execute("SET RESULT CHECKSUM OFF").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW RESULT CHECKSUM STATUS").unwrap() {
        assert_eq!(rows[0][0], "OFF");
    } else {
        panic!();
    }
}

#[test]
fn test_s354_drop_policy() {
    let (ex, _dir) = setup();
    ex.execute("SET AUTOSCALE POLICY FOR t MIN 1 MAX 10")
        .unwrap();
    let r = ex.execute("DROP AUTOSCALE POLICY FOR t").unwrap();
    assert!(matches!(r, QueryResult::Dropped));
}
#[test]
fn test_s354_dropped_not_shown() {
    let (ex, _dir) = setup();
    ex.execute("SET AUTOSCALE POLICY FOR t MIN 1 MAX 10")
        .unwrap();
    ex.execute("DROP AUTOSCALE POLICY FOR t").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW AUTOSCALE POLICIES").unwrap() {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}
#[test]
fn test_s354_empty_initially() {
    let (ex, _dir) = setup();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW AUTOSCALE POLICIES").unwrap() {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}

// ── S355 — schema object versioning ──────────────────────────────────────
#[test]
fn test_s355_tag_version() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("TAG SCHEMA OBJECT VERSION s.obj AS 'v1'")
        .unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s355_show_versions() {
    let (ex, _dir) = setup();
    ex.execute("TAG SCHEMA OBJECT VERSION s.obj AS 'v1'")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW OBJECT VERSIONS FOR s.obj").unwrap() {
        assert_eq!(rows[0][0], "v1");
    } else {
        panic!();
    }
}
#[test]
fn test_s355_multiple_versions() {
    let (ex, _dir) = setup();
    ex.execute("TAG SCHEMA OBJECT VERSION s.obj AS 'v1'")
        .unwrap();
    ex.execute("TAG SCHEMA OBJECT VERSION s.obj AS 'v2'")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW OBJECT VERSIONS FOR s.obj").unwrap() {
        assert_eq!(rows.len(), 2);
    } else {
        panic!();
    }
}
#[test]
fn test_s355_clear_versions() {
    let (ex, _dir) = setup();
    ex.execute("TAG SCHEMA OBJECT VERSION s.obj AS 'v1'")
        .unwrap();
    ex.execute("CLEAR OBJECT VERSIONS FOR s.obj").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW OBJECT VERSIONS FOR s.obj").unwrap() {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}
#[test]
fn test_s355_empty_initially() {
    let (ex, _dir) = setup();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW OBJECT VERSIONS FOR s.other").unwrap()
    {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}

#[test]
fn test_s356_clear_client_id() {
    let (ex, _dir) = setup();
    ex.execute("SET CLIENT ID 'app-name'").unwrap();
    ex.execute("CLEAR CLIENT ID").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW CLIENT ID").unwrap() {
        assert_eq!(rows[0][0], "");
    } else {
        panic!();
    }
}
#[test]
fn test_s356_default_empty() {
    let (ex, _dir) = setup();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW CLIENT ID").unwrap() {
        assert_eq!(rows[0][0], "");
    } else {
        panic!();
    }
}

// ── S357 — table skew rebalancing ────────────────────────────────────────
#[test]
fn test_s357_rebalance_skew() {
    let (ex, _dir) = setup();
    let r = ex.execute("REBALANCE SKEW FOR t").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s357_show_history() {
    let (ex, _dir) = setup();
    ex.execute("REBALANCE SKEW FOR t").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW SKEW REBALANCE HISTORY").unwrap() {
        assert_eq!(rows[0], vec!["t".to_owned(), "1".to_owned()]);
    } else {
        panic!();
    }
}
#[test]
fn test_s357_multiple_rebalances() {
    let (ex, _dir) = setup();
    ex.execute("REBALANCE SKEW FOR t").unwrap();
    ex.execute("REBALANCE SKEW FOR t").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW SKEW REBALANCE HISTORY").unwrap() {
        assert_eq!(rows[0][1], "2");
    } else {
        panic!();
    }
}
#[test]
fn test_s357_cancel_rebalance() {
    let (ex, _dir) = setup();
    ex.execute("REBALANCE SKEW FOR t").unwrap();
    ex.execute("CANCEL SKEW REBALANCE FOR t").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW SKEW REBALANCE HISTORY").unwrap() {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}
#[test]
fn test_s357_empty_initially() {
    let (ex, _dir) = setup();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW SKEW REBALANCE HISTORY").unwrap() {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}

// ── S358 — index build progress notifications ────────────────────────────
#[test]
fn test_s358_subscribe() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("SUBSCRIBE INDEX BUILD NOTIFICATIONS FOR idx1")
        .unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s358_show_subscriptions() {
    let (ex, _dir) = setup();
    ex.execute("SUBSCRIBE INDEX BUILD NOTIFICATIONS FOR idx1")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW INDEX BUILD SUBSCRIPTIONS").unwrap() {
        assert_eq!(rows[0][0], "idx1");
    } else {
        panic!();
    }
}
#[test]
fn test_s358_unsubscribe() {
    let (ex, _dir) = setup();
    ex.execute("SUBSCRIBE INDEX BUILD NOTIFICATIONS FOR idx1")
        .unwrap();
    ex.execute("UNSUBSCRIBE INDEX BUILD NOTIFICATIONS FOR idx1")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW INDEX BUILD SUBSCRIPTIONS").unwrap() {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}
#[test]
fn test_s358_empty_initially() {
    let (ex, _dir) = setup();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW INDEX BUILD SUBSCRIPTIONS").unwrap() {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}
#[test]
fn test_s358_unsubscribe_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("UNSUBSCRIBE INDEX BUILD NOTIFICATIONS FOR missing")
        .unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}

// ── S359 — query plan explain diff ───────────────────────────────────────
#[test]
fn test_s359_diff_explain() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("DIFF EXPLAIN 'SELECT 1' AGAINST 'SELECT 2'")
        .unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s359_diff_contains_both() {
    let (ex, _dir) = setup();
    if let QueryResult::Rows { rows, .. } = ex
        .execute("DIFF EXPLAIN 'SELECT 1' AGAINST 'SELECT 2'")
        .unwrap()
    {
        assert!(rows[0][0].contains("SELECT 1") && rows[0][0].contains("SELECT 2"));
    } else {
        panic!();
    }
}
#[test]
fn test_s359_show_diffs() {
    let (ex, _dir) = setup();
    ex.execute("DIFF EXPLAIN 'SELECT 1' AGAINST 'SELECT 2'")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW EXPLAIN DIFFS").unwrap() {
        assert_eq!(rows.len(), 1);
    } else {
        panic!();
    }
}
#[test]
fn test_s359_clear_diffs() {
    let (ex, _dir) = setup();
    ex.execute("DIFF EXPLAIN 'SELECT 1' AGAINST 'SELECT 2'")
        .unwrap();
    ex.execute("CLEAR EXPLAIN DIFFS").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW EXPLAIN DIFFS").unwrap() {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}
#[test]
fn test_s359_empty_initially() {
    let (ex, _dir) = setup();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW EXPLAIN DIFFS").unwrap() {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}

#[test]
fn test_s360_reset_buffer_size() {
    let (ex, _dir) = setup();
    ex.execute("SET WRITE BUFFER SIZE FOR t TO 64MB").unwrap();
    ex.execute("RESET WRITE BUFFER SIZE FOR t").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW WRITE BUFFER SIZES").unwrap() {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}
#[test]
fn test_s360_empty_initially() {
    let (ex, _dir) = setup();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW WRITE BUFFER SIZES").unwrap() {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}
#[test]
fn test_s360_reset_nonexistent_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("RESET WRITE BUFFER SIZE FOR missing").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}

// ── S361 — column distinct count estimation ──────────────────────────────
#[test]
fn test_s361_estimate_distinct_count() {
    let (ex, _dir) = setup();
    let r = ex.execute("ESTIMATE DISTINCT COUNT FOR t.col").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s361_show_estimates() {
    let (ex, _dir) = setup();
    ex.execute("ESTIMATE DISTINCT COUNT FOR t.col").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW DISTINCT COUNT ESTIMATES").unwrap() {
        assert_eq!(rows[0][0], "t.col");
    } else {
        panic!();
    }
}
#[test]
fn test_s361_clear_estimates() {
    let (ex, _dir) = setup();
    ex.execute("ESTIMATE DISTINCT COUNT FOR t.col").unwrap();
    ex.execute("CLEAR DISTINCT COUNT ESTIMATES").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW DISTINCT COUNT ESTIMATES").unwrap() {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}
#[test]
fn test_s361_empty_initially() {
    let (ex, _dir) = setup();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW DISTINCT COUNT ESTIMATES").unwrap() {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}
#[test]
fn test_s361_estimate_missing_table_zero() {
    let (ex, _dir) = setup();
    if let QueryResult::Rows { rows, .. } = ex
        .execute("ESTIMATE DISTINCT COUNT FOR missing.col")
        .unwrap()
    {
        assert_eq!(rows[0][1], "0");
    } else {
        panic!();
    }
}

// ── S362 — query global variables ────────────────────────────────────────
#[test]
fn test_s362_set_global_var() {
    let (ex, _dir) = setup();
    let r = ex.execute("SET GLOBAL VAR name = 'value'").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s362_show_global_vars() {
    let (ex, _dir) = setup();
    ex.execute("SET GLOBAL VAR name = 'value'").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW GLOBAL VARS").unwrap() {
        assert_eq!(rows[0], vec!["name".to_owned(), "value".to_owned()]);
    } else {
        panic!();
    }
}
#[test]
fn test_s362_unset_global_var() {
    let (ex, _dir) = setup();
    ex.execute("SET GLOBAL VAR name = 'value'").unwrap();
    ex.execute("UNSET GLOBAL VAR name").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW GLOBAL VARS").unwrap() {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}
#[test]
fn test_s362_empty_initially() {
    let (ex, _dir) = setup();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW GLOBAL VARS").unwrap() {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}
#[test]
fn test_s362_overwrite_var() {
    let (ex, _dir) = setup();
    ex.execute("SET GLOBAL VAR name = 'first'").unwrap();
    ex.execute("SET GLOBAL VAR name = 'second'").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW GLOBAL VARS").unwrap() {
        assert_eq!(rows[0][1], "second");
    } else {
        panic!();
    }
}

// ── S363 — table incremental backup ──────────────────────────────────────
#[test]
fn test_s363_backup_incremental() {
    let (ex, _dir) = setup();
    let r = ex.execute("BACKUP TABLE t INCREMENTAL").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s363_show_backups() {
    let (ex, _dir) = setup();
    ex.execute("BACKUP TABLE t INCREMENTAL").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW INCREMENTAL BACKUPS").unwrap() {
        assert_eq!(rows[0][0], "t");
    } else {
        panic!();
    }
}
#[test]
fn test_s363_clear_backups() {
    let (ex, _dir) = setup();
    ex.execute("BACKUP TABLE t INCREMENTAL").unwrap();
    ex.execute("CLEAR INCREMENTAL BACKUPS").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW INCREMENTAL BACKUPS").unwrap() {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}
#[test]
fn test_s363_empty_initially() {
    let (ex, _dir) = setup();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW INCREMENTAL BACKUPS").unwrap() {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}
#[test]
fn test_s363_multiple_backups() {
    let (ex, _dir) = setup();
    ex.execute("BACKUP TABLE t INCREMENTAL").unwrap();
    ex.execute("BACKUP TABLE t2 INCREMENTAL").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW INCREMENTAL BACKUPS").unwrap() {
        assert_eq!(rows.len(), 2);
    } else {
        panic!();
    }
}

#[test]
fn test_s364_reset_window() {
    let (ex, _dir) = setup();
    ex.execute("SET INDEX MAINTENANCE WINDOW '02:00-04:00'")
        .unwrap();
    ex.execute("RESET INDEX MAINTENANCE WINDOW").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW INDEX MAINTENANCE WINDOW").unwrap() {
        assert_eq!(rows[0][0], "");
    } else {
        panic!();
    }
}
#[test]
fn test_s364_default_empty() {
    let (ex, _dir) = setup();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW INDEX MAINTENANCE WINDOW").unwrap() {
        assert_eq!(rows[0][0], "");
    } else {
        panic!();
    }
}
#[test]
fn test_s364_reset_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("RESET INDEX MAINTENANCE WINDOW").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}

// ── S365 — query result pagination tokens ────────────────────────────────
#[test]
fn test_s365_generate_token() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("GENERATE PAGINATION TOKEN FOR 'SELECT 1'")
        .unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s365_show_tokens() {
    let (ex, _dir) = setup();
    ex.execute("GENERATE PAGINATION TOKEN FOR 'SELECT 1'")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW PAGINATION TOKENS").unwrap() {
        assert_eq!(rows.len(), 1);
    } else {
        panic!();
    }
}
#[test]
fn test_s365_revoke_token() {
    let (ex, _dir) = setup();
    let token = if let QueryResult::Rows { rows, .. } = ex
        .execute("GENERATE PAGINATION TOKEN FOR 'SELECT 1'")
        .unwrap()
    {
        rows[0][0].clone()
    } else {
        panic!()
    };
    let r = ex
        .execute(&format!("REVOKE PAGINATION TOKEN '{}'", token))
        .unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s365_revoked_not_shown() {
    let (ex, _dir) = setup();
    let token = if let QueryResult::Rows { rows, .. } = ex
        .execute("GENERATE PAGINATION TOKEN FOR 'SELECT 1'")
        .unwrap()
    {
        rows[0][0].clone()
    } else {
        panic!()
    };
    ex.execute(&format!("REVOKE PAGINATION TOKEN '{}'", token))
        .unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW PAGINATION TOKENS").unwrap() {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}
#[test]
fn test_s365_empty_initially() {
    let (ex, _dir) = setup();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW PAGINATION TOKENS").unwrap() {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}

// ── S366 — table access pattern classification ───────────────────────────
#[test]
fn test_s366_classify() {
    let (ex, _dir) = setup();
    let r = ex.execute("CLASSIFY ACCESS PATTERN FOR t").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s366_show_classes() {
    let (ex, _dir) = setup();
    ex.execute("CLASSIFY ACCESS PATTERN FOR t").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW ACCESS PATTERN CLASSES").unwrap() {
        assert_eq!(rows[0][0], "t");
    } else {
        panic!();
    }
}
#[test]
fn test_s366_clear_classes() {
    let (ex, _dir) = setup();
    ex.execute("CLASSIFY ACCESS PATTERN FOR t").unwrap();
    ex.execute("CLEAR ACCESS PATTERN CLASSES").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW ACCESS PATTERN CLASSES").unwrap() {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}
#[test]
fn test_s366_empty_initially() {
    let (ex, _dir) = setup();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW ACCESS PATTERN CLASSES").unwrap() {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}
#[test]
fn test_s366_classify_result_mixed() {
    let (ex, _dir) = setup();
    if let QueryResult::Rows { rows, .. } = ex.execute("CLASSIFY ACCESS PATTERN FOR t").unwrap() {
        assert_eq!(rows[0][1], "MIXED");
    } else {
        panic!();
    }
}

// ── S367 — schema change dry-run ─────────────────────────────────────────
#[test]
fn test_s367_dry_run() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("DRY RUN SCHEMA CHANGE 'ALTER TABLE t ADD COLUMN c INT'")
        .unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s367_show_results() {
    let (ex, _dir) = setup();
    ex.execute("DRY RUN SCHEMA CHANGE 'ALTER TABLE t ADD COLUMN c INT'")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW DRY RUN RESULTS").unwrap() {
        assert_eq!(rows.len(), 1);
    } else {
        panic!();
    }
}
#[test]
fn test_s367_clear_results() {
    let (ex, _dir) = setup();
    ex.execute("DRY RUN SCHEMA CHANGE 'ALTER TABLE t ADD COLUMN c INT'")
        .unwrap();
    ex.execute("CLEAR DRY RUN RESULTS").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW DRY RUN RESULTS").unwrap() {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}
#[test]
fn test_s367_empty_initially() {
    let (ex, _dir) = setup();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW DRY RUN RESULTS").unwrap() {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}
#[test]
fn test_s367_result_valid() {
    let (ex, _dir) = setup();
    if let QueryResult::Rows { rows, .. } = ex
        .execute("DRY RUN SCHEMA CHANGE 'ALTER TABLE t ADD COLUMN c INT'")
        .unwrap()
    {
        assert_eq!(rows[0][1], "VALID");
    } else {
        panic!();
    }
}

// ── S368 — query latency percentiles ─────────────────────────────────────
#[test]
fn test_s368_show_default() {
    let (ex, _dir) = setup();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW QUERY LATENCY PERCENTILES").unwrap() {
        assert_eq!(rows[0][0], "p50=0,p95=0,p99=0");
    } else {
        panic!();
    }
}
#[test]
fn test_s368_refresh() {
    let (ex, _dir) = setup();
    let r = ex.execute("REFRESH QUERY LATENCY PERCENTILES").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s368_show_after_refresh() {
    let (ex, _dir) = setup();
    ex.execute("REFRESH QUERY LATENCY PERCENTILES").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW QUERY LATENCY PERCENTILES").unwrap() {
        assert_eq!(rows[0][0], "p50=1,p95=5,p99=10");
    } else {
        panic!();
    }
}
#[test]
fn test_s368_clear() {
    let (ex, _dir) = setup();
    ex.execute("REFRESH QUERY LATENCY PERCENTILES").unwrap();
    ex.execute("CLEAR QUERY LATENCY PERCENTILES").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW QUERY LATENCY PERCENTILES").unwrap() {
        assert_eq!(rows[0][0], "p50=0,p95=0,p99=0");
    } else {
        panic!();
    }
}
#[test]
fn test_s368_clear_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("CLEAR QUERY LATENCY PERCENTILES").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}

// ── S369 — table replica failover history ────────────────────────────────
#[test]
fn test_s369_record_failover() {
    let (ex, _dir) = setup();
    let r = ex.execute("RECORD REPLICA FAILOVER FOR t").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s369_show_history() {
    let (ex, _dir) = setup();
    ex.execute("RECORD REPLICA FAILOVER FOR t").unwrap();
    if let QueryResult::Rows { rows, .. } =
        ex.execute("SHOW REPLICA FAILOVER HISTORY FOR t").unwrap()
    {
        assert_eq!(rows.len(), 1);
    } else {
        panic!();
    }
}
#[test]
fn test_s369_multiple_failovers() {
    let (ex, _dir) = setup();
    ex.execute("RECORD REPLICA FAILOVER FOR t").unwrap();
    ex.execute("RECORD REPLICA FAILOVER FOR t").unwrap();
    if let QueryResult::Rows { rows, .. } =
        ex.execute("SHOW REPLICA FAILOVER HISTORY FOR t").unwrap()
    {
        assert_eq!(rows.len(), 2);
    } else {
        panic!();
    }
}
#[test]
fn test_s369_clear_history() {
    let (ex, _dir) = setup();
    ex.execute("RECORD REPLICA FAILOVER FOR t").unwrap();
    ex.execute("CLEAR REPLICA FAILOVER HISTORY").unwrap();
    if let QueryResult::Rows { rows, .. } =
        ex.execute("SHOW REPLICA FAILOVER HISTORY FOR t").unwrap()
    {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}
#[test]
fn test_s369_empty_initially() {
    let (ex, _dir) = setup();
    if let QueryResult::Rows { rows, .. } =
        ex.execute("SHOW REPLICA FAILOVER HISTORY FOR t").unwrap()
    {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}

// ── S370 — index usage recommendations v2 ────────────────────────────────
#[test]
fn test_s370_recommend() {
    let (ex, _dir) = setup();
    let r = ex.execute("RECOMMEND INDEX USAGE FOR t").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s370_show_recommendations() {
    let (ex, _dir) = setup();
    ex.execute("RECOMMEND INDEX USAGE FOR t").unwrap();
    if let QueryResult::Rows { rows, .. } =
        ex.execute("SHOW INDEX USAGE RECOMMENDATIONS V2").unwrap()
    {
        assert_eq!(rows[0][0], "t");
    } else {
        panic!();
    }
}
#[test]
fn test_s370_clear_recommendations() {
    let (ex, _dir) = setup();
    ex.execute("RECOMMEND INDEX USAGE FOR t").unwrap();
    ex.execute("CLEAR INDEX USAGE RECOMMENDATIONS V2").unwrap();
    if let QueryResult::Rows { rows, .. } =
        ex.execute("SHOW INDEX USAGE RECOMMENDATIONS V2").unwrap()
    {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}
#[test]
fn test_s370_empty_initially() {
    let (ex, _dir) = setup();
    if let QueryResult::Rows { rows, .. } =
        ex.execute("SHOW INDEX USAGE RECOMMENDATIONS V2").unwrap()
    {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}
#[test]
fn test_s370_recommendation_text() {
    let (ex, _dir) = setup();
    if let QueryResult::Rows { rows, .. } = ex.execute("RECOMMEND INDEX USAGE FOR t").unwrap() {
        assert_eq!(rows[0][1], "ADD INDEX ON PRIMARY KEY");
    } else {
        panic!();
    }
}

#[test]
fn test_s371_default_off() {
    let (ex, _dir) = setup();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW BATCH DEDUP STATUS").unwrap() {
        assert_eq!(rows[0][0], "OFF");
    } else {
        panic!();
    }
}
#[test]
fn test_s371_toggle_roundtrip() {
    let (ex, _dir) = setup();
    ex.execute("SET BATCH DEDUP ON").unwrap();
    ex.execute("SET BATCH DEDUP OFF").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW BATCH DEDUP STATUS").unwrap() {
        assert_eq!(rows[0][0], "OFF");
    } else {
        panic!();
    }
}

// ── S372 — table storage tiering rules ───────────────────────────────────
#[test]
fn test_s372_create_rule() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("CREATE STORAGE TIERING RULE r1 FOR t AFTER '30d' TO COLD")
        .unwrap();
    assert!(matches!(r, QueryResult::Created));
}
#[test]
fn test_s372_show_rules() {
    let (ex, _dir) = setup();
    ex.execute("CREATE STORAGE TIERING RULE r1 FOR t AFTER '30d' TO COLD")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW STORAGE TIERING RULES").unwrap() {
        assert_eq!(rows[0][0], "r1");
    } else {
        panic!();
    }
}
#[test]
fn test_s372_drop_rule() {
    let (ex, _dir) = setup();
    ex.execute("CREATE STORAGE TIERING RULE r1 FOR t AFTER '30d' TO COLD")
        .unwrap();
    let r = ex.execute("DROP STORAGE TIERING RULE r1").unwrap();
    assert!(matches!(r, QueryResult::Dropped));
}
#[test]
fn test_s372_dropped_not_shown() {
    let (ex, _dir) = setup();
    ex.execute("CREATE STORAGE TIERING RULE r1 FOR t AFTER '30d' TO COLD")
        .unwrap();
    ex.execute("DROP STORAGE TIERING RULE r1").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW STORAGE TIERING RULES").unwrap() {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}
#[test]
fn test_s372_empty_initially() {
    let (ex, _dir) = setup();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW STORAGE TIERING RULES").unwrap() {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}

// ── S373 — schema consistency check ──────────────────────────────────────
#[test]
fn test_s373_check_consistency() {
    let (ex, _dir) = setup();
    let r = ex.execute("CHECK SCHEMA CONSISTENCY FOR s1").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s373_show_report() {
    let (ex, _dir) = setup();
    ex.execute("CHECK SCHEMA CONSISTENCY FOR s1").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW SCHEMA CONSISTENCY REPORT").unwrap() {
        assert!(rows[0][0].contains("s1"));
    } else {
        panic!();
    }
}
#[test]
fn test_s373_default_report() {
    let (ex, _dir) = setup();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW SCHEMA CONSISTENCY REPORT").unwrap() {
        assert_eq!(rows[0][0], "no report");
    } else {
        panic!();
    }
}
#[test]
fn test_s373_clear_report() {
    let (ex, _dir) = setup();
    ex.execute("CHECK SCHEMA CONSISTENCY FOR s1").unwrap();
    ex.execute("CLEAR SCHEMA CONSISTENCY REPORT").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW SCHEMA CONSISTENCY REPORT").unwrap() {
        assert_eq!(rows[0][0], "no report");
    } else {
        panic!();
    }
}
#[test]
fn test_s373_clear_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("CLEAR SCHEMA CONSISTENCY REPORT").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}

// ── S374 — query plan regression detection ───────────────────────────────
#[test]
fn test_s374_detect_regression() {
    let (ex, _dir) = setup();
    let r = ex.execute("DETECT PLAN REGRESSION FOR 'SELECT 1'").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s374_show_regressions() {
    let (ex, _dir) = setup();
    ex.execute("DETECT PLAN REGRESSION FOR 'SELECT 1'").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW PLAN REGRESSIONS").unwrap() {
        assert_eq!(rows.len(), 1);
    } else {
        panic!();
    }
}
#[test]
fn test_s374_clear_regressions() {
    let (ex, _dir) = setup();
    ex.execute("DETECT PLAN REGRESSION FOR 'SELECT 1'").unwrap();
    ex.execute("CLEAR PLAN REGRESSIONS").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW PLAN REGRESSIONS").unwrap() {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}
#[test]
fn test_s374_empty_initially() {
    let (ex, _dir) = setup();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW PLAN REGRESSIONS").unwrap() {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}
#[test]
fn test_s374_not_regressed() {
    let (ex, _dir) = setup();
    if let QueryResult::Rows { rows, .. } =
        ex.execute("DETECT PLAN REGRESSION FOR 'SELECT 1'").unwrap()
    {
        assert_eq!(rows[0][1], "false");
    } else {
        panic!();
    }
}

#[test]
fn test_s375_drop_affinity() {
    let (ex, _dir) = setup();
    ex.execute("SET CONNECTION AFFINITY FOR t TO NODE node1")
        .unwrap();
    let r = ex.execute("DROP CONNECTION AFFINITY FOR t").unwrap();
    assert!(matches!(r, QueryResult::Dropped));
}
#[test]
fn test_s375_dropped_not_shown() {
    let (ex, _dir) = setup();
    ex.execute("SET CONNECTION AFFINITY FOR t TO NODE node1")
        .unwrap();
    ex.execute("DROP CONNECTION AFFINITY FOR t").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW CONNECTION AFFINITIES").unwrap() {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}
#[test]
fn test_s375_empty_initially() {
    let (ex, _dir) = setup();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW CONNECTION AFFINITIES").unwrap() {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}

#[test]
fn test_s376_default_throttle() {
    let (ex, _dir) = setup();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW INDEX BUILD THROTTLE").unwrap() {
        assert_eq!(rows[0][0], "100");
    } else {
        panic!();
    }
}
#[test]
fn test_s376_reset_throttle() {
    let (ex, _dir) = setup();
    ex.execute("SET INDEX BUILD THROTTLE 50 PERCENT").unwrap();
    ex.execute("RESET INDEX BUILD THROTTLE").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW INDEX BUILD THROTTLE").unwrap() {
        assert_eq!(rows[0][0], "100");
    } else {
        panic!();
    }
}
#[test]
fn test_s376_reset_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("RESET INDEX BUILD THROTTLE").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}

// ── S377 — query result set versioning ───────────────────────────────────
#[test]
fn test_s377_tag_version() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("TAG RESULT SET VERSION FOR 'SELECT 1' AS 'v1'")
        .unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s377_show_versions() {
    let (ex, _dir) = setup();
    ex.execute("TAG RESULT SET VERSION FOR 'SELECT 1' AS 'v1'")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW RESULT SET VERSIONS").unwrap() {
        assert_eq!(rows[0], vec!["SELECT 1".to_owned(), "v1".to_owned()]);
    } else {
        panic!();
    }
}
#[test]
fn test_s377_clear_versions() {
    let (ex, _dir) = setup();
    ex.execute("TAG RESULT SET VERSION FOR 'SELECT 1' AS 'v1'")
        .unwrap();
    ex.execute("CLEAR RESULT SET VERSIONS").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW RESULT SET VERSIONS").unwrap() {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}
#[test]
fn test_s377_empty_initially() {
    let (ex, _dir) = setup();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW RESULT SET VERSIONS").unwrap() {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}
#[test]
fn test_s377_overwrite_version() {
    let (ex, _dir) = setup();
    ex.execute("TAG RESULT SET VERSION FOR 'SELECT 1' AS 'v1'")
        .unwrap();
    ex.execute("TAG RESULT SET VERSION FOR 'SELECT 1' AS 'v2'")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW RESULT SET VERSIONS").unwrap() {
        assert_eq!(rows[0][1], "v2");
    } else {
        panic!();
    }
}

// ── S378 — table shard key rebalancing ───────────────────────────────────
#[test]
fn test_s378_rebalance() {
    let (ex, _dir) = setup();
    let r = ex.execute("REBALANCE SHARD KEY FOR t").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s378_show_history() {
    let (ex, _dir) = setup();
    ex.execute("REBALANCE SHARD KEY FOR t").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW SHARD KEY REBALANCE HISTORY").unwrap()
    {
        assert_eq!(rows[0], vec!["t".to_owned(), "1".to_owned()]);
    } else {
        panic!();
    }
}
#[test]
fn test_s378_cancel_rebalance() {
    let (ex, _dir) = setup();
    ex.execute("REBALANCE SHARD KEY FOR t").unwrap();
    ex.execute("CANCEL SHARD KEY REBALANCE FOR t").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW SHARD KEY REBALANCE HISTORY").unwrap()
    {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}
#[test]
fn test_s378_empty_initially() {
    let (ex, _dir) = setup();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW SHARD KEY REBALANCE HISTORY").unwrap()
    {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}
#[test]
fn test_s378_multiple_rebalances() {
    let (ex, _dir) = setup();
    ex.execute("REBALANCE SHARD KEY FOR t").unwrap();
    ex.execute("REBALANCE SHARD KEY FOR t").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW SHARD KEY REBALANCE HISTORY").unwrap()
    {
        assert_eq!(rows[0][1], "2");
    } else {
        panic!();
    }
}

#[test]
fn test_s379_clear_ownership() {
    let (ex, _dir) = setup();
    ex.execute("SET OWNER FOR s.obj TO user1").unwrap();
    ex.execute("CLEAR OBJECT OWNERSHIP FOR s.obj").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW OBJECT OWNERSHIP").unwrap() {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}
#[test]
fn test_s379_empty_initially() {
    let (ex, _dir) = setup();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW OBJECT OWNERSHIP").unwrap() {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}

// ── S380 — 380-sprint checkpoint ─────────────────────────────────────────
#[test]
fn test_s380_show_sprint_checkpoint_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW SPRINT CHECKPOINT 380").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s380_select_from_checkpoint_table() {
    let (ex, _dir) = setup();
    let r = ex.execute("SELECT * FROM vv_checkpoint_380").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s380_show_engine_build_info() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW ENGINE BUILD INFO").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s380_checkpoint_value_is_380() {
    let (ex, _dir) = setup();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW SPRINT CHECKPOINT 380").unwrap() {
        assert_eq!(rows[0][0], "380");
    } else {
        panic!();
    }
}
#[test]
fn test_s380_checkpoint_status_complete() {
    let (ex, _dir) = setup();
    if let QueryResult::Rows { rows, .. } = ex.execute("SELECT * FROM vv_checkpoint_380").unwrap() {
        assert_eq!(rows[0][1], "COMPLETE");
    } else {
        panic!();
    }
}
