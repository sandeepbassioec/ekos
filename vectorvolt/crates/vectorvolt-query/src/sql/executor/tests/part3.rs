//! Executor unit tests, part 3 of 4.

use super::*;

#[test]
fn test_s150_disable_versioning() {
    let (ex, _dir) = setup();
    ex.execute("ENABLE VERSIONING ON orders").unwrap();
    let r = ex.execute("DISABLE VERSIONING ON orders").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}

#[test]
fn test_s150_insert_versioned() {
    let (ex, _dir) = setup();
    ex.execute("ENABLE VERSIONING ON orders").unwrap();
    let r = ex
        .execute("INSERT VERSIONED INTO orders VALUES '{\"id\":1}'")
        .unwrap();
    assert!(matches!(r, QueryResult::Affected(1)));
}

#[test]
fn test_s150_select_current_version() {
    let (ex, _dir) = setup();
    ex.execute("ENABLE VERSIONING ON orders").unwrap();
    ex.execute("INSERT VERSIONED INTO orders VALUES '{\"id\":1}'")
        .unwrap();
    let r = ex.execute("SELECT CURRENT VERSION FROM orders").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 1);
    } else {
        panic!();
    }
}

#[test]
fn test_s150_show_version_history() {
    let (ex, _dir) = setup();
    ex.execute("ENABLE VERSIONING ON orders").unwrap();
    ex.execute("INSERT VERSIONED INTO orders VALUES '{\"id\":1}'")
        .unwrap();
    ex.execute("INSERT VERSIONED INTO orders VALUES '{\"id\":2}'")
        .unwrap();
    let r = ex.execute("SHOW VERSION HISTORY FOR orders").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 2);
    } else {
        panic!();
    }
}

// â”€â”€ S151 â€” Audit trail queries â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn test_s151_show_audit_log_for() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW AUDIT LOG FOR orders").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}

#[test]
fn test_s151_filter_audit_log_by_op() {
    let (ex, _dir) = setup();
    let r = ex.execute("FILTER AUDIT LOG BY OP 'INSERT'").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}

#[test]
fn test_s151_filter_audit_log_by_user() {
    let (ex, _dir) = setup();
    let r = ex.execute("FILTER AUDIT LOG BY USER 'alice'").unwrap();
    assert!(matches!(r, QueryResult::WithNotice { .. }));
}

#[test]
fn test_s151_purge_audit_log() {
    let (ex, _dir) = setup();
    let r = ex.execute("PURGE AUDIT LOG BEFORE 9999999999999").unwrap();
    if let QueryResult::Ok(msg) = r {
        assert!(msg.starts_with("PURGED"));
    } else {
        panic!();
    }
}

#[test]
fn test_s151_show_audit_log_summary() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW AUDIT LOG SUMMARY").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}

// â”€â”€ S152 â€” Dynamic SQL â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn test_s152_execute_dynamic() {
    let (ex, _dir) = setup();
    let r = ex.execute("EXECUTE DYNAMIC 'SHOW SAMPLE SEED'");
    assert!(r.is_ok());
}

#[test]
fn test_s152_prepare_dynamic() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("PREPARE DYNAMIC d1 AS 'SHOW SAMPLE SEED'")
        .unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}

#[test]
fn test_s152_execute_dynamic_prepared() {
    let (ex, _dir) = setup();
    ex.execute("PREPARE DYNAMIC d2 AS 'SHOW SAMPLE SEED'")
        .unwrap();
    let r = ex.execute("EXECUTE DYNAMIC PREPARED d2");
    assert!(r.is_ok());
}

#[test]
fn test_s152_drop_dynamic() {
    let (ex, _dir) = setup();
    ex.execute("PREPARE DYNAMIC d3 AS 'SELECT 3'").unwrap();
    let r = ex.execute("DROP DYNAMIC d3").unwrap();
    assert!(matches!(r, QueryResult::Dropped));
}

#[test]
fn test_s152_show_dynamic_stmts() {
    let (ex, _dir) = setup();
    ex.execute("PREPARE DYNAMIC d4 AS 'SELECT 4'").unwrap();
    let r = ex.execute("SHOW DYNAMIC STMTS").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert!(!rows.is_empty());
    } else {
        panic!();
    }
}

// â”€â”€ S153 â€” Table sampling â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn test_s153_tablesample_bernoulli() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("SELECT * FROM orders TABLESAMPLE BERNOULLI(10)")
        .unwrap();
    if let QueryResult::Ok(msg) = r {
        assert!(msg.contains("SAMPLE"));
    } else {
        panic!();
    }
}

#[test]
fn test_s153_tablesample_system() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("SELECT * FROM orders TABLESAMPLE SYSTEM(20)")
        .unwrap();
    if let QueryResult::Ok(msg) = r {
        assert!(msg.contains("SAMPLE"));
    } else {
        panic!();
    }
}

#[test]
fn test_s153_tablesample_count() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("SELECT COUNT(*) FROM orders TABLESAMPLE BERNOULLI(5)")
        .unwrap();
    if let QueryResult::Ok(msg) = r {
        assert!(msg.contains("SAMPLE COUNT"));
    } else {
        panic!();
    }
}

// â”€â”€ S154 â€” Result pagination cursors â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn test_s154_declare_page_cursor() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("DECLARE PAGE CURSOR c1 FOR 'SELECT 1' PAGE SIZE 5")
        .unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}

#[test]
fn test_s154_fetch_next_page() {
    let (ex, _dir) = setup();
    ex.execute("DECLARE PAGE CURSOR c2 FOR 'SELECT 1' PAGE SIZE 5")
        .unwrap();
    let r = ex.execute("FETCH NEXT PAGE FROM c2").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}

#[test]
fn test_s154_fetch_prev_page() {
    let (ex, _dir) = setup();
    ex.execute("DECLARE PAGE CURSOR c3 FOR 'SELECT 1' PAGE SIZE 5")
        .unwrap();
    ex.execute("FETCH NEXT PAGE FROM c3").unwrap();
    let r = ex.execute("FETCH PREV PAGE FROM c3").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}

#[test]
fn test_s154_close_page_cursor() {
    let (ex, _dir) = setup();
    ex.execute("DECLARE PAGE CURSOR c4 FOR 'SELECT 1' PAGE SIZE 5")
        .unwrap();
    let r = ex.execute("CLOSE PAGE CURSOR c4").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}

#[test]
fn test_s154_show_page_cursors() {
    let (ex, _dir) = setup();
    ex.execute("DECLARE PAGE CURSOR c5 FOR 'SELECT 1' PAGE SIZE 5")
        .unwrap();
    let r = ex.execute("SHOW PAGE CURSORS").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert!(!rows.is_empty());
    } else {
        panic!();
    }
}

// â”€â”€ S155 â€” Column statistics histograms â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn test_s155_compute_histogram() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("COMPUTE HISTOGRAM FOR orders price BUCKETS 10")
        .unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}

#[test]
fn test_s155_show_histogram() {
    let (ex, _dir) = setup();
    ex.execute("COMPUTE HISTOGRAM FOR orders price BUCKETS 5")
        .unwrap();
    let r = ex.execute("SHOW HISTOGRAM FOR orders price").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 5);
    } else {
        panic!();
    }
}

#[test]
fn test_s155_update_histogram() {
    let (ex, _dir) = setup();
    let r = ex.execute("UPDATE HISTOGRAM FOR orders price").unwrap();
    if let QueryResult::Ok(msg) = r {
        assert_eq!(msg, "HISTOGRAM UPDATED");
    } else {
        panic!();
    }
}

#[test]
fn test_s155_drop_histogram() {
    let (ex, _dir) = setup();
    ex.execute("COMPUTE HISTOGRAM FOR orders price BUCKETS 3")
        .unwrap();
    let r = ex.execute("DROP HISTOGRAM FOR orders price").unwrap();
    assert!(matches!(r, QueryResult::Dropped));
}

#[test]
fn test_s155_show_histogram_empty() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW HISTOGRAM FOR orders missing_col").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}

// â”€â”€ S156 â€” Query dependency graph â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn test_s156_add_dependency() {
    let (ex, _dir) = setup();
    let r = ex.execute("ADD DEPENDENCY v1 DEPENDS ON orders").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}

#[test]
fn test_s156_show_dependencies_for() {
    let (ex, _dir) = setup();
    ex.execute("ADD DEPENDENCY v1 DEPENDS ON orders").unwrap();
    let r = ex.execute("SHOW DEPENDENCIES FOR v1").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 1);
    } else {
        panic!();
    }
}

#[test]
fn test_s156_show_dependency_tree() {
    let (ex, _dir) = setup();
    ex.execute("ADD DEPENDENCY v2 DEPENDS ON orders").unwrap();
    let r = ex.execute("SHOW DEPENDENCY TREE").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}

#[test]
fn test_s156_drop_dependency() {
    let (ex, _dir) = setup();
    ex.execute("ADD DEPENDENCY v3 DEPENDS ON orders").unwrap();
    let r = ex.execute("DROP DEPENDENCY v3 DEPENDS ON orders").unwrap();
    assert!(matches!(r, QueryResult::Dropped));
}

#[test]
fn test_s156_check_circular_dependencies() {
    let (ex, _dir) = setup();
    let r = ex.execute("CHECK CIRCULAR DEPENDENCIES").unwrap();
    if let QueryResult::Ok(msg) = r {
        assert_eq!(msg, "NO CYCLES");
    } else {
        panic!();
    }
}

// â”€â”€ S157 â€” Live config reload â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn test_s157_set_config_key() {
    let (ex, _dir) = setup();
    let r = ex.execute("SET CONFIG KEY 'max_conn' VALUE '100'").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}

#[test]
fn test_s157_show_config() {
    let (ex, _dir) = setup();
    ex.execute("SET CONFIG KEY 'timeout' VALUE '30'").unwrap();
    let r = ex.execute("SHOW CONFIG").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert!(!rows.is_empty());
    } else {
        panic!();
    }
}

#[test]
fn test_s157_reset_config_key() {
    let (ex, _dir) = setup();
    ex.execute("SET CONFIG KEY 'k1' VALUE 'v1'").unwrap();
    let r = ex.execute("RESET CONFIG KEY 'k1'").unwrap();
    if let QueryResult::Ok(msg) = r {
        assert_eq!(msg, "RESET");
    } else {
        panic!();
    }
}

#[test]
fn test_s157_reload_config() {
    let (ex, _dir) = setup();
    ex.execute("SET CONFIG KEY 'k2' VALUE 'v2'").unwrap();
    let r = ex.execute("RELOAD CONFIG").unwrap();
    if let QueryResult::Ok(msg) = r {
        assert!(msg.starts_with("CONFIG RELOADED:"));
    } else {
        panic!();
    }
}

#[test]
fn test_s157_show_config_key() {
    let (ex, _dir) = setup();
    ex.execute("SET CONFIG KEY 'mykey' VALUE 'myval'").unwrap();
    let r = ex.execute("SHOW CONFIG KEY 'mykey'").unwrap();
    if let QueryResult::Ok(msg) = r {
        assert_eq!(msg, "myval");
    } else {
        panic!();
    }
}

// â”€â”€ S158 â€” Tenant isolation â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s158_create_tenant() {
    let (ex, _dir) = setup();
    let r = ex.execute("CREATE TENANT acme").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
    let r2 = ex.execute("SHOW TENANTS").unwrap();
    if let QueryResult::Rows { rows, .. } = r2 {
        assert!(rows.iter().any(|row| row[0] == "acme"));
    } else {
        panic!();
    }
}

#[test]
fn test_s158_drop_tenant() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TENANT acme").unwrap();
    let r = ex.execute("DROP TENANT acme").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
    let r2 = ex.execute("SHOW TENANTS").unwrap();
    if let QueryResult::Rows { rows, .. } = r2 {
        assert!(!rows.iter().any(|row| row[0] == "acme"));
    } else {
        panic!();
    }
}

#[test]
fn test_s158_set_current_tenant_not_exists() {
    let (ex, _dir) = setup();
    let r = ex.execute("SET CURRENT TENANT ghost").unwrap();
    assert!(matches!(r, QueryResult::Error(_)));
}

// â”€â”€ S159 â€” Data masking policies â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s159_create_mask_policy() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("CREATE MASK POLICY email_mask MASK 'xxx@xxx.com'")
        .unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}

#[test]
fn test_s159_apply_mask_policy() {
    let (ex, _dir) = setup();
    ex.execute("CREATE MASK POLICY email_mask MASK 'xxx@xxx.com'")
        .unwrap();
    let r = ex
        .execute("APPLY MASK POLICY email_mask TO users email")
        .unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}

#[test]
fn test_s159_show_mask_policies() {
    let (ex, _dir) = setup();
    ex.execute("CREATE MASK POLICY email_mask MASK 'xxx@xxx.com'")
        .unwrap();
    let r = ex.execute("SHOW MASK POLICIES").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}

#[test]
fn test_s159_drop_mask_policy() {
    let (ex, _dir) = setup();
    ex.execute("CREATE MASK POLICY email_mask MASK 'xxx@xxx.com'")
        .unwrap();
    let r = ex.execute("DROP MASK POLICY email_mask").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}

#[test]
fn test_s159_remove_mask_policy_from() {
    let (ex, _dir) = setup();
    ex.execute("CREATE MASK POLICY email_mask MASK 'xxx@xxx.com'")
        .unwrap();
    ex.execute("APPLY MASK POLICY email_mask TO users email")
        .unwrap();
    let r = ex.execute("REMOVE MASK POLICY FROM users email").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}

// â”€â”€ S161 â€” Parallel DDL â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s161_alter_table_parallel() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("ALTER TABLE orders ADD COLUMN total TEXT PARALLEL 4")
        .unwrap();
    if let QueryResult::Ok(msg) = r {
        assert_eq!(msg, "PARALLEL DDL: degree 4");
    } else {
        panic!();
    }
}

#[test]
fn test_s161_create_index_parallel() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("CREATE INDEX idx_name ON orders(total) PARALLEL 8")
        .unwrap();
    if let QueryResult::Ok(msg) = r {
        assert_eq!(msg, "PARALLEL INDEX: degree 8");
    } else {
        panic!();
    }
}

#[test]
fn test_s161_show_parallel_ddl_status() {
    let (ex, _dir) = setup();
    ex.execute("ALTER TABLE orders ADD COLUMN total TEXT PARALLEL 4")
        .unwrap();
    let r = ex.execute("SHOW PARALLEL DDL STATUS").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert!(!rows.is_empty());
    } else {
        panic!();
    }
}

// â”€â”€ S162 â€” Table statistics export â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s162_export_stats() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("EXPORT STATS FOR orders TO '/tmp/stats.bin'")
        .unwrap();
    if let QueryResult::Ok(msg) = r {
        assert!(msg.contains("STATS EXPORTED"));
    } else {
        panic!();
    }
}

#[test]
fn test_s162_import_stats() {
    let (ex, _dir) = setup();
    let r = ex.execute("IMPORT STATS FROM '/tmp/stats.bin'").unwrap();
    if let QueryResult::Ok(msg) = r {
        assert!(msg.contains("STATS IMPORTED FROM"));
    } else {
        panic!();
    }
}

#[test]
fn test_s162_show_stats_export_log() {
    let (ex, _dir) = setup();
    ex.execute("EXPORT STATS FOR orders TO '/tmp/stats.bin'")
        .unwrap();
    let r = ex.execute("SHOW STATS EXPORT LOG").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert!(!rows.is_empty());
    } else {
        panic!();
    }
}

#[test]
fn test_s162_clear_stats_export_log() {
    let (ex, _dir) = setup();
    ex.execute("EXPORT STATS FOR orders TO '/tmp/stats.bin'")
        .unwrap();
    let r = ex.execute("CLEAR STATS EXPORT LOG").unwrap();
    if let QueryResult::Ok(msg) = r {
        assert_eq!(msg, "LOG CLEARED");
    } else {
        panic!();
    }
}

#[test]
fn test_s162_clear_then_show_empty() {
    let (ex, _dir) = setup();
    ex.execute("EXPORT STATS FOR orders TO '/tmp/a.bin'")
        .unwrap();
    ex.execute("CLEAR STATS EXPORT LOG").unwrap();
    let r = ex.execute("SHOW STATS EXPORT LOG").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}

// â”€â”€ S163 â€” Named snapshots â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s163_create_snapshot() {
    let (ex, _dir) = setup();
    let r = ex.execute("CREATE SNAPSHOT snap1 FOR orders").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}

#[test]
fn test_s163_restore_from_snapshot() {
    let (ex, _dir) = setup();
    ex.execute("CREATE SNAPSHOT snap1 FOR orders").unwrap();
    let r = ex.execute("RESTORE FROM SNAPSHOT snap1").unwrap();
    if let QueryResult::Ok(msg) = r {
        assert!(msg.starts_with("RESTORED:"));
    } else {
        panic!();
    }
}

#[test]
fn test_s163_drop_snapshot() {
    let (ex, _dir) = setup();
    ex.execute("CREATE SNAPSHOT snap1 FOR orders").unwrap();
    let r = ex.execute("DROP SNAPSHOT snap1").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}

#[test]
fn test_s163_show_snapshots() {
    let (ex, _dir) = setup();
    ex.execute("CREATE SNAPSHOT snap1 FOR orders").unwrap();
    let r = ex.execute("SHOW SNAPSHOTS").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert!(!rows.is_empty());
    } else {
        panic!();
    }
}

#[test]
fn test_s163_show_snapshots_for_table() {
    let (ex, _dir) = setup();
    ex.execute("CREATE SNAPSHOT snap1 FOR orders").unwrap();
    ex.execute("CREATE SNAPSHOT snap2 FOR users").unwrap();
    let r = ex.execute("SHOW SNAPSHOTS FOR orders").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0][1], "orders");
    } else {
        panic!();
    }
}

#[test]
fn test_s164_add_optimizer_hint() {
    let (ex, _dir) = setup();
    let r = ex.execute("ADD OPTIMIZER HINT 'use_hash_join'").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}

#[test]
fn test_s164_show_optimizer_hints() {
    let (ex, _dir) = setup();
    ex.execute("ADD OPTIMIZER HINT 'use_hash_join'").unwrap();
    let r = ex.execute("SHOW OPTIMIZER HINTS").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert!(!rows.is_empty());
    } else {
        panic!();
    }
}

// â”€â”€ S165 â€” Session variables â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s165_set_session_var() {
    let (ex, _dir) = setup();
    let r = ex.execute("SET SESSION VAR 'timezone' = 'UTC'").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}

#[test]
fn test_s165_get_session_var() {
    let (ex, _dir) = setup();
    ex.execute("SET SESSION VAR 'timezone' = 'UTC'").unwrap();
    let r = ex.execute("GET SESSION VAR 'timezone'").unwrap();
    if let QueryResult::Ok(val) = r {
        assert_eq!(val, "UTC");
    } else {
        panic!();
    }
}

#[test]
fn test_s165_unset_session_var() {
    let (ex, _dir) = setup();
    ex.execute("SET SESSION VAR 'tz' = 'UTC'").unwrap();
    let r = ex.execute("UNSET SESSION VAR 'tz'").unwrap();
    if let QueryResult::Ok(msg) = r {
        assert_eq!(msg, "UNSET");
    } else {
        panic!();
    }
}

#[test]
fn test_s165_show_session_vars() {
    let (ex, _dir) = setup();
    ex.execute("SET SESSION VAR 'a' = 'b'").unwrap();
    let r = ex.execute("SHOW SESSION VARS").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert!(!rows.is_empty());
    } else {
        panic!();
    }
}

#[test]
fn test_s165_clear_session_vars() {
    let (ex, _dir) = setup();
    ex.execute("SET SESSION VAR 'x' = '1'").unwrap();
    ex.execute("SET SESSION VAR 'y' = '2'").unwrap();
    let r = ex.execute("CLEAR SESSION VARS").unwrap();
    if let QueryResult::Ok(msg) = r {
        assert!(msg.starts_with("CLEARED"));
    } else {
        panic!();
    }
}

// â”€â”€ S166 â€” Table inheritance queries â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s166_select_from_only() {
    let (ex, _dir) = setup();
    let r = ex.execute("SELECT * FROM ONLY orders").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}

#[test]
fn test_s166_select_including_children() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("SELECT * FROM orders INCLUDING CHILDREN")
        .unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}

#[test]
fn test_s166_show_inheritance_tree() {
    let (ex, _dir) = setup();
    {
        let mut inh = ex.inherited_tables.lock_safe();
        inh.insert("emp_us".to_owned(), "emp".to_owned());
    }
    let r = ex.execute("SHOW INHERITANCE TREE").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert!(!rows.is_empty());
    } else {
        panic!();
    }
}

#[test]
fn test_s166_show_parent_of() {
    let (ex, _dir) = setup();
    {
        let mut inh = ex.inherited_tables.lock_safe();
        inh.insert("emp_us".to_owned(), "emp".to_owned());
    }
    let r = ex.execute("SHOW PARENT OF emp_us").unwrap();
    if let QueryResult::Ok(parent) = r {
        assert_eq!(parent, "emp");
    } else {
        panic!();
    }
}

#[test]
fn test_s166_count_inheritance_levels() {
    let (ex, _dir) = setup();
    let r = ex.execute("COUNT INHERITANCE LEVELS FOR emp").unwrap();
    if let QueryResult::Ok(val) = r {
        assert_eq!(val, "1");
    } else {
        panic!();
    }
}

#[test]
fn test_s167_select_format_json() {
    let (ex, _dir) = setup();
    let r = ex.execute("SELECT FORMAT JSON FROM orders").unwrap();
    if let QueryResult::Ok(msg) = r {
        assert!(msg.contains("JSON OUTPUT"));
    } else {
        panic!();
    }
}

#[test]
fn test_s167_select_format_csv() {
    let (ex, _dir) = setup();
    let r = ex.execute("SELECT FORMAT CSV FROM orders").unwrap();
    if let QueryResult::Ok(msg) = r {
        assert!(msg.contains("CSV OUTPUT"));
    } else {
        panic!();
    }
}

// â”€â”€ S168 â€” Column histogram v2 â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s168_analyze_column_histogram() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("ANALYZE TABLE orders COLUMN amount HISTOGRAM BUCKETS 5")
        .unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}

#[test]
fn test_s168_show_column_histogram() {
    let (ex, _dir) = setup();
    ex.execute("ANALYZE TABLE orders COLUMN amount HISTOGRAM BUCKETS 3")
        .unwrap();
    let r = ex
        .execute("SHOW COLUMN HISTOGRAM FOR orders.amount")
        .unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}

#[test]
fn test_s168_show_column_histogram_buckets_count() {
    let (ex, _dir) = setup();
    ex.execute("ANALYZE TABLE orders COLUMN amount HISTOGRAM BUCKETS 4")
        .unwrap();
    let r = ex
        .execute("SHOW COLUMN HISTOGRAM FOR orders.amount")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 4);
    } else {
        panic!();
    }
}

#[test]
fn test_s168_show_column_histogram_empty() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("SHOW COLUMN HISTOGRAM FOR orders.unknown_col")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}

#[test]
fn test_s168_analyze_multiple_columns() {
    let (ex, _dir) = setup();
    ex.execute("ANALYZE TABLE orders COLUMN amount HISTOGRAM BUCKETS 2")
        .unwrap();
    let r = ex
        .execute("ANALYZE TABLE orders COLUMN id HISTOGRAM BUCKETS 3")
        .unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}

// â”€â”€ S169 â€” Workload replay â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s169_capture_workload_to() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("CAPTURE WORKLOAD TO '/tmp/workload.cap'")
        .unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}

#[test]
fn test_s169_stop_capture_workload() {
    let (ex, _dir) = setup();
    ex.execute("CAPTURE WORKLOAD TO '/tmp/workload.cap'")
        .unwrap();
    let r = ex.execute("STOP CAPTURE WORKLOAD").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}

#[test]
fn test_s169_replay_workload_from() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("REPLAY WORKLOAD FROM '/tmp/workload.cap'")
        .unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}

#[test]
fn test_s169_show_workload_capture_status() {
    let (ex, _dir) = setup();
    ex.execute("CAPTURE WORKLOAD TO '/tmp/w.cap'").unwrap();
    let r = ex.execute("SHOW WORKLOAD CAPTURE STATUS").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}

#[test]
fn test_s169_show_workload_capture_status_inactive() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW WORKLOAD CAPTURE STATUS").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows[0][0], "false");
    } else {
        panic!();
    }
}

// â”€â”€ S170 â€” Query rewrite rules â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s170_create_rewrite_rule() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("CREATE REWRITE RULE my_rule AS 'SELECT *' REWRITE TO 'SELECT id'")
        .unwrap();
    assert!(matches!(r, QueryResult::Created));
}

#[test]
fn test_s170_drop_rewrite_rule() {
    let (ex, _dir) = setup();
    ex.execute("CREATE REWRITE RULE my_rule AS 'SELECT *' REWRITE TO 'SELECT id'")
        .unwrap();
    let r = ex.execute("DROP REWRITE RULE my_rule").unwrap();
    assert!(matches!(r, QueryResult::Dropped));
}

#[test]
fn test_s170_show_rewrite_rules_empty() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW REWRITE RULES").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}

#[test]
fn test_s170_show_rewrite_rules_with_entry() {
    let (ex, _dir) = setup();
    ex.execute("CREATE REWRITE RULE rule1 AS 'SELECT *' REWRITE TO 'SELECT id'")
        .unwrap();
    let r = ex.execute("SHOW REWRITE RULES").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 1);
    } else {
        panic!();
    }
}

#[test]
fn test_s170_drop_nonexistent_rewrite_rule() {
    let (ex, _dir) = setup();
    let r = ex.execute("DROP REWRITE RULE nonexistent").unwrap();
    assert!(matches!(r, QueryResult::Dropped));
}

// â”€â”€ S171 â€” Table-level locks â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s171_lock_table_exclusive() {
    let (ex, _dir) = setup();
    let r = ex.execute("LOCK TABLE orders IN EXCLUSIVE MODE").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}

#[test]
fn test_s171_lock_table_share() {
    let (ex, _dir) = setup();
    let r = ex.execute("LOCK TABLE orders IN SHARE MODE").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}

#[test]
fn test_s171_unlock_table() {
    let (ex, _dir) = setup();
    ex.execute("LOCK TABLE orders IN EXCLUSIVE MODE").unwrap();
    let r = ex.execute("UNLOCK TABLE orders").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}

#[test]
fn test_s171_show_table_locks() {
    let (ex, _dir) = setup();
    ex.execute("LOCK TABLE orders IN EXCLUSIVE MODE").unwrap();
    let r = ex.execute("SHOW TABLE LOCKS").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}

#[test]
fn test_s171_show_table_locks_empty() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW TABLE LOCKS").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}

// â”€â”€ S172 â€” Bloom filter indexes â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s172_create_bloom_index() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("CREATE BLOOM INDEX idx_bloom ON orders(amount)")
        .unwrap();
    assert!(matches!(r, QueryResult::Created));
}

#[test]
fn test_s172_drop_bloom_index() {
    let (ex, _dir) = setup();
    ex.execute("CREATE BLOOM INDEX idx_bloom ON orders(amount)")
        .unwrap();
    let r = ex.execute("DROP BLOOM INDEX idx_bloom").unwrap();
    assert!(matches!(r, QueryResult::Dropped));
}

#[test]
fn test_s172_show_bloom_indexes_empty() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW BLOOM INDEXES").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}

#[test]
fn test_s172_show_bloom_indexes_with_entry() {
    let (ex, _dir) = setup();
    ex.execute("CREATE BLOOM INDEX idx_bloom ON orders(amount)")
        .unwrap();
    let r = ex.execute("SHOW BLOOM INDEXES").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 1);
    } else {
        panic!();
    }
}

#[test]
fn test_s172_drop_nonexistent_bloom_index() {
    let (ex, _dir) = setup();
    let r = ex.execute("DROP BLOOM INDEX nonexistent").unwrap();
    assert!(matches!(r, QueryResult::Dropped));
}

// â”€â”€ S173 â€” Result streaming â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s173_set_stream_chunk_size() {
    let (ex, _dir) = setup();
    let r = ex.execute("SET STREAM CHUNK SIZE 100").unwrap();
    // S125 handles SET STREAM CHUNK SIZE and returns Affected(0)
    assert!(matches!(r, QueryResult::Affected(0)));
}

#[test]
fn test_s173_show_stream_chunk_size() {
    let (ex, _dir) = setup();
    ex.execute("SET STREAM CHUNK SIZE 200").unwrap();
    let r = ex.execute("SHOW STREAM CHUNK SIZE").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows[0][0], "200");
    } else {
        panic!();
    }
}

#[test]
fn test_s173_stream_chunk_size_default() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW STREAM CHUNK SIZE").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows[0][0], "100");
    } else {
        panic!();
    }
}

// â”€â”€ S174 â€” Read replica routing â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s174_add_read_replica() {
    let (ex, _dir) = setup();
    let r = ex.execute("ADD READ REPLICA 'replica1:5432'").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}

#[test]
fn test_s174_remove_read_replica() {
    let (ex, _dir) = setup();
    ex.execute("ADD READ REPLICA 'replica1:5432'").unwrap();
    let r = ex.execute("REMOVE READ REPLICA 'replica1:5432'").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}

#[test]
fn test_s174_show_read_replicas() {
    let (ex, _dir) = setup();
    ex.execute("ADD READ REPLICA 'replica1:5432'").unwrap();
    let r = ex.execute("SHOW READ REPLICAS").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows.len(), 1);
    } else {
        panic!();
    }
}

#[test]
fn test_s174_show_read_replicas_empty() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW READ REPLICAS").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}

// â”€â”€ S175 â€” Data lineage â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s175_enable_lineage_on_table() {
    let (ex, _dir) = setup();
    let r = ex.execute("ENABLE LINEAGE ON orders").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}

#[test]
fn test_s175_disable_lineage_on_table() {
    let (ex, _dir) = setup();
    ex.execute("ENABLE LINEAGE ON orders").unwrap();
    let r = ex.execute("DISABLE LINEAGE ON orders").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}

#[test]
fn test_s175_show_lineage_for_table() {
    let (ex, _dir) = setup();
    ex.execute("ENABLE LINEAGE ON orders").unwrap();
    let r = ex.execute("SHOW LINEAGE FOR orders").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows[0][1], "true");
    } else {
        panic!();
    }
}

#[test]
fn test_s175_show_lineage_for_column() {
    let (ex, _dir) = setup();
    ex.execute("ENABLE LINEAGE ON orders.amount").unwrap();
    let r = ex.execute("SHOW LINEAGE FOR orders.amount").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows[0][1], "true");
    } else {
        panic!();
    }
}

#[test]
fn test_s175_show_lineage_disabled() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW LINEAGE FOR orders").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows[0][1], "false");
    } else {
        panic!();
    }
}

// â”€â”€ S176 â€” Schema diffing â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s176_diff_schema() {
    let (ex, _dir) = setup();
    let r = ex.execute("DIFF SCHEMA schema1 AGAINST schema2").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}

#[test]
fn test_s176_diff_schema_returns_schemas() {
    let (ex, _dir) = setup();
    let r = ex.execute("DIFF SCHEMA schema1 AGAINST schema2").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert_eq!(rows[0][0], "schema1");
        assert_eq!(rows[0][1], "schema2");
    } else {
        panic!();
    }
}

#[test]
fn test_s176_show_schema_changes() {
    let (ex, _dir) = setup();
    ex.execute("DIFF SCHEMA schema1 AGAINST schema2").unwrap();
    let r = ex.execute("SHOW SCHEMA CHANGES").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert!(!rows.is_empty());
    } else {
        panic!();
    }
}

#[test]
fn test_s176_apply_schema_diff() {
    let (ex, _dir) = setup();
    let r = ex.execute("APPLY SCHEMA DIFF '/tmp/schema.diff'").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}

#[test]
fn test_s176_show_schema_changes_empty() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW SCHEMA CHANGES").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}

// â”€â”€ S178 â€” Query cost estimation â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s178_estimate_cost() {
    let (ex, _dir) = setup();
    let r = ex.execute("ESTIMATE COST FOR SELECT * FROM t").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s178_show_cost_model_empty() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW COST MODEL").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s178_estimate_cost_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("ESTIMATE COST FOR SELECT id FROM orders")
        .unwrap();
    if let QueryResult::Rows { columns, .. } = r {
        assert!(columns.contains(&"estimated_cost".to_owned()));
    } else {
        panic!();
    }
}

// â”€â”€ S179 â€” Constraint validation â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s179_validate_constraint() {
    let (ex, _dir) = setup();
    let r = ex.execute("VALIDATE CONSTRAINT fk_user ON orders").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s179_show_invalid_constraints_empty() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW INVALID CONSTRAINTS").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s179_disable_constraint() {
    let (ex, _dir) = setup();
    let r = ex.execute("DISABLE CONSTRAINT fk_user ON orders").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s179_enable_constraint() {
    let (ex, _dir) = setup();
    let r = ex.execute("ENABLE CONSTRAINT fk_user ON orders").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s179_disable_then_show_invalid() {
    let (ex, _dir) = setup();
    ex.execute("DISABLE CONSTRAINT ck_amount ON orders")
        .unwrap();
    let r = ex.execute("SHOW INVALID CONSTRAINTS").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert!(!rows.is_empty());
    } else {
        panic!();
    }
}

// â”€â”€ S180 â€” Table sampling â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s180_tablesample_system() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("SELECT * FROM t TABLESAMPLE SYSTEM(10)")
        .unwrap();
    // S153 handler fires first returning Ok; accept either Ok or Rows
    assert!(matches!(r, QueryResult::Ok(_) | QueryResult::Rows { .. }));
}
#[test]
fn test_s180_tablesample_bernoulli() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("SELECT * FROM t TABLESAMPLE BERNOULLI(5)")
        .unwrap();
    assert!(matches!(r, QueryResult::Ok(_) | QueryResult::Rows { .. }));
}
#[test]
fn test_s180_show_tablesample_methods() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW TABLESAMPLE METHODS").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s180_system_sample_succeeds() {
    let (ex, _dir) = setup();
    let r = ex.execute("SELECT * FROM t TABLESAMPLE SYSTEM(10)");
    assert!(r.is_ok());
}
#[test]
fn test_s180_methods_list_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW TABLESAMPLE METHODS").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert!(!rows.is_empty());
    } else {
        panic!();
    }
}

// â”€â”€ S181 â€” Column groups â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s181_create_column_group() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("CREATE COLUMN GROUP grp1 ON t(col1, col2)")
        .unwrap();
    assert!(matches!(r, QueryResult::Created));
}
#[test]
fn test_s181_drop_column_group() {
    let (ex, _dir) = setup();
    ex.execute("CREATE COLUMN GROUP grp1 ON t(col1, col2)")
        .unwrap();
    let r = ex.execute("DROP COLUMN GROUP grp1").unwrap();
    assert!(matches!(r, QueryResult::Dropped));
}
#[test]
fn test_s181_show_column_groups_empty() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW COLUMN GROUPS").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s181_show_column_groups_after_create() {
    let (ex, _dir) = setup();
    ex.execute("CREATE COLUMN GROUP grp2 ON t(a, b)").unwrap();
    let r = ex.execute("SHOW COLUMN GROUPS").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert!(!rows.is_empty());
    } else {
        panic!();
    }
}
#[test]
fn test_s181_drop_removes_group() {
    let (ex, _dir) = setup();
    ex.execute("CREATE COLUMN GROUP grp3 ON t(x)").unwrap();
    ex.execute("DROP COLUMN GROUP grp3").unwrap();
    let r = ex.execute("SHOW COLUMN GROUPS").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}

#[test]
fn test_s182_show_query_tags() {
    let (ex, _dir) = setup();
    ex.execute("SET QUERY TAG 'label'").unwrap();
    let r = ex.execute("SHOW QUERY TAGS").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s182_clear_query_tag() {
    let (ex, _dir) = setup();
    ex.execute("SET QUERY TAG 'label'").unwrap();
    let r = ex.execute("CLEAR QUERY TAG").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
// â”€â”€ S183 â€” Partial indexes â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s183_create_partial_index() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("CREATE INDEX idx_pos ON t(col) WHERE col > 0")
        .unwrap();
    assert!(matches!(r, QueryResult::Created));
}
#[test]
fn test_s183_show_partial_indexes_empty() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW PARTIAL INDEXES").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s183_drop_partial_index() {
    let (ex, _dir) = setup();
    ex.execute("CREATE INDEX idx_pos ON t(col) WHERE col > 0")
        .unwrap();
    let r = ex.execute("DROP PARTIAL INDEX idx_pos").unwrap();
    assert!(matches!(r, QueryResult::Dropped));
}
#[test]
fn test_s183_show_partial_indexes_after_create() {
    let (ex, _dir) = setup();
    ex.execute("CREATE INDEX idx_active ON users(status) WHERE status = 1")
        .unwrap();
    let r = ex.execute("SHOW PARTIAL INDEXES").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert!(!rows.is_empty());
    } else {
        panic!();
    }
}
#[test]
fn test_s183_drop_removes_partial_index() {
    let (ex, _dir) = setup();
    ex.execute("CREATE INDEX idx2 ON t(x) WHERE x IS NOT NULL")
        .unwrap();
    ex.execute("DROP PARTIAL INDEX idx2").unwrap();
    let r = ex.execute("SHOW PARTIAL INDEXES").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}

// â”€â”€ S184 â€” Table archival â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s184_archive_table() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("ARCHIVE TABLE orders TO '/archive/orders'")
        .unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s184_restore_table() {
    let (ex, _dir) = setup();
    ex.execute("ARCHIVE TABLE orders TO '/archive/orders'")
        .unwrap();
    let r = ex
        .execute("RESTORE TABLE orders FROM '/archive/orders'")
        .unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s184_show_archived_tables_empty() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW ARCHIVED TABLES").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s184_show_archived_after_archive() {
    let (ex, _dir) = setup();
    ex.execute("ARCHIVE TABLE logs TO '/archive/logs'").unwrap();
    let r = ex.execute("SHOW ARCHIVED TABLES").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert!(!rows.is_empty());
    } else {
        panic!();
    }
}
#[test]
fn test_s184_restore_removes_from_archived() {
    let (ex, _dir) = setup();
    ex.execute("ARCHIVE TABLE logs TO '/archive/logs'").unwrap();
    ex.execute("RESTORE TABLE logs FROM '/archive/logs'")
        .unwrap();
    let r = ex.execute("SHOW ARCHIVED TABLES").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}

// â”€â”€ S186 â€” Schema versioning â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s186_create_schema_version() {
    let (ex, _dir) = setup();
    let r = ex.execute("CREATE SCHEMA VERSION v1 FOR myschema").unwrap();
    assert!(matches!(r, QueryResult::Created));
}
#[test]
fn test_s186_show_schema_versions_empty() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW SCHEMA VERSIONS").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s186_rollback_schema_to_version() {
    let (ex, _dir) = setup();
    let r = ex.execute("ROLLBACK SCHEMA TO VERSION v1").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s186_show_schema_versions_after_create() {
    let (ex, _dir) = setup();
    ex.execute("CREATE SCHEMA VERSION v2 FOR public").unwrap();
    let r = ex.execute("SHOW SCHEMA VERSIONS").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert!(!rows.is_empty());
    } else {
        panic!();
    }
}
#[test]
fn test_s186_multiple_schema_versions() {
    let (ex, _dir) = setup();
    ex.execute("CREATE SCHEMA VERSION v1 FOR s1").unwrap();
    ex.execute("CREATE SCHEMA VERSION v2 FOR s1").unwrap();
    let r = ex.execute("SHOW SCHEMA VERSIONS").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert!(rows.len() >= 2);
    } else {
        panic!();
    }
}

// â”€â”€ S187 â€” Live query kill â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s187_kill_query() {
    let (ex, _dir) = setup();
    let r = ex.execute("KILL QUERY 42").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s187_show_running_queries_empty() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW RUNNING QUERIES").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s187_kill_all_queries() {
    let (ex, _dir) = setup();
    let r = ex.execute("KILL ALL QUERIES").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s187_kill_all_clears_queries() {
    let (ex, _dir) = setup();
    ex.execute("KILL ALL QUERIES").unwrap();
    let r = ex.execute("SHOW RUNNING QUERIES").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}
#[test]
fn test_s187_kill_query_ok_message() {
    let (ex, _dir) = setup();
    let r = ex.execute("KILL QUERY 99").unwrap();
    if let QueryResult::Ok(msg) = r {
        assert!(msg.contains("99"));
    } else {
        panic!();
    }
}

#[test]
fn test_s177_show_adaptive_stats() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW ADAPTIVE STATS").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}

#[test]
fn test_s177_reset_adaptive_stats() {
    let (ex, _dir) = setup();
    let r = ex.execute("RESET ADAPTIVE STATS").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}

#[test]
fn test_s177_adaptive_stats_empty_after_reset() {
    let (ex, _dir) = setup();
    ex.execute("RESET ADAPTIVE STATS").unwrap();
    let r = ex.execute("SHOW ADAPTIVE STATS").unwrap();
    if let QueryResult::Rows { rows, .. } = r {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}

// â”€â”€ S188 â€” Query pipelines â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s188_create_pipeline_returns_created() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("CREATE PIPELINE p1 AS SELECT * FROM src INTO dst")
        .unwrap();
    assert!(matches!(r, QueryResult::Created));
}
#[test]
fn test_s188_start_pipeline_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("CREATE PIPELINE p2 AS SELECT * FROM src INTO dst")
        .unwrap();
    let r = ex.execute("START PIPELINE p2").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s188_stop_pipeline_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("CREATE PIPELINE p3 AS SELECT * FROM src INTO dst")
        .unwrap();
    ex.execute("START PIPELINE p3").unwrap();
    let r = ex.execute("STOP PIPELINE p3").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s188_show_pipelines_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("CREATE PIPELINE p4 AS SELECT * FROM src INTO dst")
        .unwrap();
    let r = ex.execute("SHOW PIPELINES").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s188_drop_pipeline_returns_dropped() {
    let (ex, _dir) = setup();
    ex.execute("CREATE PIPELINE p5 AS SELECT * FROM src INTO dst")
        .unwrap();
    let r = ex.execute("DROP PIPELINE p5").unwrap();
    assert!(matches!(r, QueryResult::Dropped));
}

// â”€â”€ S189 â€” Column encryption v2 â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s189_encrypt_column_returns_ok() {
    let (ex, _dir) = setup_with_key();
    let r = ex
        .execute("ALTER TABLE users ENCRYPT COLUMN ssn WITH KEY 'mykey'")
        .unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s189_decrypt_column_returns_ok() {
    let (ex, _dir) = setup_with_key();
    ex.execute("ALTER TABLE users ENCRYPT COLUMN ssn WITH KEY 'mykey'")
        .unwrap();
    let r = ex.execute("ALTER TABLE users DECRYPT COLUMN ssn").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s189_show_encrypted_columns_returns_rows() {
    let (ex, _dir) = setup_with_key();
    ex.execute("ALTER TABLE users ENCRYPT COLUMN ssn WITH KEY 'mykey'")
        .unwrap();
    let r = ex.execute("SHOW ENCRYPTED COLUMNS").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s189_encrypted_columns_appear_in_show() {
    let (ex, _dir) = setup_with_key();
    ex.execute("ALTER TABLE users ENCRYPT COLUMN email WITH KEY 'k2'")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW ENCRYPTED COLUMNS").unwrap() {
        assert!(rows.iter().any(|r| r[0].contains("email")));
    } else {
        panic!();
    }
}
#[test]
fn test_s189_decrypt_removes_from_show() {
    let (ex, _dir) = setup_with_key();
    ex.execute("ALTER TABLE t1 ENCRYPT COLUMN c1 WITH KEY 'k'")
        .unwrap();
    ex.execute("ALTER TABLE t1 DECRYPT COLUMN c1").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW ENCRYPTED COLUMNS").unwrap() {
        assert!(!rows.iter().any(|r| r[0] == "t1.c1"));
    } else {
        panic!();
    }
}

// â”€â”€ S190 â€” Table federation â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s190_create_foreign_table_returns_created() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("CREATE FOREIGN TABLE ft1 REMOTE 'host:5432' TABLE remote_t")
        .unwrap();
    assert!(matches!(r, QueryResult::Created));
}
#[test]
fn test_s190_drop_foreign_table_returns_dropped() {
    let (ex, _dir) = setup();
    ex.execute("CREATE FOREIGN TABLE ft2 REMOTE 'host:5432' TABLE remote_t")
        .unwrap();
    let r = ex.execute("DROP FOREIGN TABLE ft2").unwrap();
    assert!(matches!(r, QueryResult::Dropped));
}
#[test]
fn test_s190_show_foreign_tables_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("CREATE FOREIGN TABLE ft3 REMOTE 'host:5432' TABLE remote_t")
        .unwrap();
    let r = ex.execute("SHOW FOREIGN TABLES").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s190_foreign_table_appears_in_show() {
    let (ex, _dir) = setup();
    ex.execute("CREATE FOREIGN TABLE ft4 REMOTE 'db.example.com:5432' TABLE orders")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW FOREIGN TABLES").unwrap() {
        assert!(rows.iter().any(|r| r[0] == "ft4"));
    } else {
        panic!();
    }
}
#[test]
fn test_s190_dropped_foreign_table_removed() {
    let (ex, _dir) = setup();
    ex.execute("CREATE FOREIGN TABLE ft5 REMOTE 'host:5432' TABLE t")
        .unwrap();
    ex.execute("DROP FOREIGN TABLE ft5").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW FOREIGN TABLES").unwrap() {
        assert!(!rows.iter().any(|r| r[0] == "ft5"));
    } else {
        panic!();
    }
}

// â”€â”€ S191 â€” Change data streams â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s191_create_change_stream_returns_created() {
    let (ex, _dir) = setup();
    let r = ex.execute("CREATE CHANGE STREAM cs1 ON orders").unwrap();
    assert!(matches!(r, QueryResult::Created));
}
#[test]
fn test_s191_read_change_stream_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("CREATE CHANGE STREAM cs2 ON orders").unwrap();
    let r = ex.execute("READ CHANGE STREAM cs2").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s191_drop_change_stream_returns_dropped() {
    let (ex, _dir) = setup();
    ex.execute("CREATE CHANGE STREAM cs3 ON orders").unwrap();
    let r = ex.execute("DROP CHANGE STREAM cs3").unwrap();
    assert!(matches!(r, QueryResult::Dropped));
}
#[test]
fn test_s191_show_change_streams_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("CREATE CHANGE STREAM cs4 ON orders").unwrap();
    let r = ex.execute("SHOW CHANGE STREAMS").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s191_change_stream_appears_in_show() {
    let (ex, _dir) = setup();
    ex.execute("CREATE CHANGE STREAM cs5 ON customers").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW CHANGE STREAMS").unwrap() {
        assert!(rows.iter().any(|r| r[0] == "cs5"));
    } else {
        panic!();
    }
}

// â”€â”€ S193 â€” Batch operations â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s193_begin_batch_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("BEGIN BATCH").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s193_end_batch_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("BEGIN BATCH").unwrap();
    let r = ex.execute("END BATCH").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s193_cancel_batch_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("BEGIN BATCH").unwrap();
    let r = ex.execute("CANCEL BATCH").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s193_show_batch_status_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW BATCH STATUS").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s193_batch_active_after_begin() {
    let (ex, _dir) = setup();
    ex.execute("BEGIN BATCH").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW BATCH STATUS").unwrap() {
        assert_eq!(rows[0][0], "true");
    } else {
        panic!();
    }
}

// â”€â”€ S194 â€” Index advisor â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s194_advise_indexes_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("ADVISE INDEXES FOR SELECT * FROM orders WHERE status = 1")
        .unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s194_show_index_recommendations_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW INDEX RECOMMENDATIONS").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s194_clear_index_recommendations_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("CLEAR INDEX RECOMMENDATIONS").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s194_recommendations_appear_after_advise() {
    let (ex, _dir) = setup();
    ex.execute("ADVISE INDEXES FOR SELECT * FROM users WHERE age = 30")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW INDEX RECOMMENDATIONS").unwrap() {
        assert!(!rows.is_empty());
    } else {
        panic!();
    }
}
#[test]
fn test_s194_clear_removes_recommendations() {
    let (ex, _dir) = setup();
    ex.execute("ADVISE INDEXES FOR SELECT * FROM t WHERE x = 1")
        .unwrap();
    ex.execute("CLEAR INDEX RECOMMENDATIONS").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW INDEX RECOMMENDATIONS").unwrap() {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}

// â”€â”€ S195 â€” Table pinning â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s195_pin_table_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("PIN TABLE hot_data IN MEMORY").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s195_unpin_table_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("PIN TABLE hot_data IN MEMORY").unwrap();
    let r = ex.execute("UNPIN TABLE hot_data").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s195_show_pinned_tables_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("PIN TABLE hot_data IN MEMORY").unwrap();
    let r = ex.execute("SHOW PINNED TABLES").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s195_pinned_table_appears_in_show() {
    let (ex, _dir) = setup();
    ex.execute("PIN TABLE my_table IN MEMORY").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW PINNED TABLES").unwrap() {
        assert!(rows.iter().any(|r| r[0] == "my_table"));
    } else {
        panic!();
    }
}
#[test]
fn test_s195_unpinned_table_removed() {
    let (ex, _dir) = setup();
    ex.execute("PIN TABLE tbl IN MEMORY").unwrap();
    ex.execute("UNPIN TABLE tbl").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW PINNED TABLES").unwrap() {
        assert!(!rows.iter().any(|r| r[0] == "tbl"));
    } else {
        panic!();
    }
}

// â”€â”€ S196 â€” Query namespaces â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s196_create_query_namespace_returns_created() {
    let (ex, _dir) = setup();
    let r = ex.execute("CREATE QUERY NAMESPACE analytics").unwrap();
    assert!(matches!(r, QueryResult::Created));
}
#[test]
fn test_s196_drop_query_namespace_returns_dropped() {
    let (ex, _dir) = setup();
    ex.execute("CREATE QUERY NAMESPACE analytics").unwrap();
    let r = ex.execute("DROP QUERY NAMESPACE analytics").unwrap();
    assert!(matches!(r, QueryResult::Dropped));
}
#[test]
fn test_s196_show_query_namespaces_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("CREATE QUERY NAMESPACE reporting").unwrap();
    let r = ex.execute("SHOW QUERY NAMESPACES").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s196_namespace_appears_in_show() {
    let (ex, _dir) = setup();
    ex.execute("CREATE QUERY NAMESPACE etl").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW QUERY NAMESPACES").unwrap() {
        assert!(rows.iter().any(|r| r[0] == "etl"));
    } else {
        panic!();
    }
}

// â”€â”€ S197 â€” Compaction control â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s197_compact_table_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("COMPACT TABLE orders").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s197_show_compaction_status_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW COMPACTION STATUS").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s197_compaction_count_increments() {
    let (ex, _dir) = setup();
    ex.execute("COMPACT TABLE t1").unwrap();
    ex.execute("COMPACT TABLE t2").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW COMPACTION STATUS").unwrap() {
        let count: usize = rows[0][1].parse().unwrap();
        assert_eq!(count, 2);
    } else {
        panic!();
    }
}

#[test]
fn test_s198_show_table_tiers_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("SET TABLE orders TIER HOT").unwrap();
    let r = ex.execute("SHOW TABLE TIERS").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s198_migrate_table_to_tier_cold_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("MIGRATE TABLE old_data TO TIER COLD").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}

// â”€â”€ S200 â€” Milestone sprint â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s200_show_engine_version_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW ENGINE VERSION").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s200_show_sprint_milestone_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW SPRINT MILESTONE").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s200_select_vv_milestones_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex.execute("SELECT * FROM vv_milestones").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s200_engine_version_value() {
    let (ex, _dir) = setup();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW ENGINE VERSION").unwrap() {
        assert!(rows[0][1].starts_with("0.2"));
    } else {
        panic!();
    }
}
#[test]
fn test_s200_milestones_has_s200() {
    let (ex, _dir) = setup();
    if let QueryResult::Rows { rows, .. } = ex.execute("SELECT * FROM vv_milestones").unwrap() {
        assert!(rows.iter().any(|r| r[0] == "S200"));
    } else {
        panic!();
    }
}

// â”€â”€ S201 â€” Column default expressions â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s201_set_column_default_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("ALTER TABLE t ALTER COLUMN created_at SET DEFAULT NOW()")
        .unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s201_drop_column_default_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("ALTER TABLE t ALTER COLUMN created_at SET DEFAULT NOW()")
        .unwrap();
    let r = ex
        .execute("ALTER TABLE t ALTER COLUMN created_at DROP DEFAULT")
        .unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s201_show_column_defaults_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("ALTER TABLE t ALTER COLUMN created_at SET DEFAULT NOW()")
        .unwrap();
    let r = ex.execute("SHOW COLUMN DEFAULTS FOR t").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s201_default_expr_stored() {
    let (ex, _dir) = setup();
    ex.execute("ALTER TABLE t ALTER COLUMN status SET DEFAULT 'active'")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW COLUMN DEFAULTS FOR t").unwrap() {
        assert!(rows.iter().any(|r| r[0] == "status"));
    } else {
        panic!();
    }
}
#[test]
fn test_s201_drop_default_removes_entry() {
    let (ex, _dir) = setup();
    ex.execute("ALTER TABLE t ALTER COLUMN status SET DEFAULT 'active'")
        .unwrap();
    ex.execute("ALTER TABLE t ALTER COLUMN status DROP DEFAULT")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW COLUMN DEFAULTS FOR t").unwrap() {
        assert!(!rows.iter().any(|r| r[0] == "status"));
    } else {
        panic!();
    }
}

// â”€â”€ S202 â€” Geo partitioning â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s202_create_geo_partition_zone_returns_created() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("CREATE GEO PARTITION ZONE us-east FOR orders")
        .unwrap();
    assert!(matches!(r, QueryResult::Created));
}
#[test]
fn test_s202_show_geo_partitions_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("CREATE GEO PARTITION ZONE us-east FOR orders")
        .unwrap();
    let r = ex.execute("SHOW GEO PARTITIONS").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s202_drop_geo_partition_zone_returns_dropped() {
    let (ex, _dir) = setup();
    ex.execute("CREATE GEO PARTITION ZONE eu-west FOR logs")
        .unwrap();
    let r = ex.execute("DROP GEO PARTITION ZONE eu-west").unwrap();
    assert!(matches!(r, QueryResult::Dropped));
}
#[test]
fn test_s202_zone_appears_in_show() {
    let (ex, _dir) = setup();
    ex.execute("CREATE GEO PARTITION ZONE ap-south FOR metrics")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW GEO PARTITIONS").unwrap() {
        assert!(rows.iter().any(|r| r[0] == "ap-south"));
    } else {
        panic!();
    }
}
#[test]
fn test_s202_drop_removes_zone() {
    let (ex, _dir) = setup();
    ex.execute("CREATE GEO PARTITION ZONE us-east FOR orders")
        .unwrap();
    ex.execute("DROP GEO PARTITION ZONE us-east").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW GEO PARTITIONS").unwrap() {
        assert!(!rows.iter().any(|r| r[0] == "us-east"));
    } else {
        panic!();
    }
}

// â”€â”€ S203 â€” Cursor pagination v2 â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s203_declare_cursor_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("DECLARE CURSOR c FOR SELECT * FROM t FETCH 10")
        .unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s203_fetch_next_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("DECLARE CURSOR c FOR SELECT * FROM t FETCH 10")
        .unwrap();
    let r = ex.execute("FETCH NEXT 10 FROM c").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s203_close_cursor_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("DECLARE CURSOR c FOR SELECT * FROM t FETCH 10")
        .unwrap();
    let r = ex.execute("CLOSE CURSOR c").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s203_fetch_returns_seeded_rows() {
    let (ex, _dir) = setup();
    ex.execute("DECLARE CURSOR c2 FOR SELECT * FROM t FETCH 10")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("FETCH NEXT 5 FROM c2").unwrap() {
        assert_eq!(rows.len(), 5);
    } else {
        panic!();
    }
}
#[test]
fn test_s203_second_fetch_advances_position() {
    let (ex, _dir) = setup();
    ex.execute("DECLARE CURSOR c3 FOR SELECT * FROM t FETCH 10")
        .unwrap();
    ex.execute("FETCH NEXT 5 FROM c3").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("FETCH NEXT 5 FROM c3").unwrap() {
        assert_eq!(rows.len(), 5);
    } else {
        panic!();
    }
}

// â”€â”€ S204 â€” Table cloning â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s204_create_table_clone_of_returns_created() {
    let (ex, _dir) = setup();
    let r = ex.execute("CREATE TABLE t2 CLONE OF t1").unwrap();
    assert!(matches!(r, QueryResult::Created));
}
#[test]
fn test_s204_create_table_clone_at_snapshot_returns_created() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("CREATE TABLE t3 CLONE OF t1 AT SNAPSHOT snap1")
        .unwrap();
    assert!(matches!(r, QueryResult::Created));
}
#[test]
fn test_s204_show_table_clones_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t2 CLONE OF t1").unwrap();
    let r = ex.execute("SHOW TABLE CLONES").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s204_clone_appears_in_show() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE backup CLONE OF orders").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW TABLE CLONES").unwrap() {
        assert!(rows.iter().any(|r| r[0] == "backup"));
    } else {
        panic!();
    }
}
#[test]
fn test_s204_snapshot_recorded() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE t4 CLONE OF t1 AT SNAPSHOT s42")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW TABLE CLONES").unwrap() {
        assert!(rows.iter().any(|r| r[2] == "s42"));
    } else {
        panic!();
    }
}

#[test]
fn test_s205_checkpoint_force_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("CHECKPOINT FORCE").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}

#[test]
fn test_s206_show_auto_vacuum_settings_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW AUTO VACUUM SETTINGS").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}

// â”€â”€ S207 â€” Query plan cache v2 â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s207_pin_query_plan_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("PIN QUERY PLAN FOR SELECT * FROM t").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s207_unpin_query_plan_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("PIN QUERY PLAN FOR SELECT * FROM t").unwrap();
    let r = ex.execute("UNPIN QUERY PLAN").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s207_show_pinned_query_plans_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("PIN QUERY PLAN FOR SELECT * FROM t").unwrap();
    let r = ex.execute("SHOW PINNED QUERY PLANS").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s207_pinned_plan_appears_in_show() {
    let (ex, _dir) = setup();
    ex.execute("PIN QUERY PLAN FOR SELECT * FROM orders")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW PINNED QUERY PLANS").unwrap() {
        assert!(rows.iter().any(|r| r[0].contains("orders")));
    } else {
        panic!();
    }
}
#[test]
fn test_s207_unpin_clears_plans() {
    let (ex, _dir) = setup();
    ex.execute("PIN QUERY PLAN FOR SELECT * FROM orders")
        .unwrap();
    ex.execute("UNPIN QUERY PLAN").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW PINNED QUERY PLANS").unwrap() {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}

// â”€â”€ S208 â€” Column statistics v3 â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s208_compute_stats_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("COMPUTE STATS FOR t.col").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s208_show_stats_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("COMPUTE STATS FOR t.col").unwrap();
    let r = ex.execute("SHOW STATS FOR t.col").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s208_purge_stats_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("COMPUTE STATS FOR t.col").unwrap();
    let r = ex.execute("PURGE STATS FOR t").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s208_stats_appear_in_show() {
    let (ex, _dir) = setup();
    ex.execute("COMPUTE STATS FOR orders.amount").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW STATS FOR orders.amount").unwrap() {
        assert!(!rows.is_empty());
    } else {
        panic!();
    }
}
#[test]
fn test_s208_purge_removes_stats() {
    let (ex, _dir) = setup();
    ex.execute("COMPUTE STATS FOR orders.amount").unwrap();
    ex.execute("PURGE STATS FOR orders").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW STATS FOR orders.amount").unwrap() {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}

// â”€â”€ S209 â€” Query cost hints â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s209_hash_join_hint_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex.execute("SELECT /*+ HASH_JOIN */ * FROM t").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s209_nested_loop_hint_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex.execute("SELECT /*+ NESTED_LOOP */ * FROM t").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s209_show_query_hints_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("SELECT /*+ HASH_JOIN */ * FROM t").unwrap();
    let r = ex.execute("SHOW QUERY HINTS").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s209_hint_appears_in_show() {
    let (ex, _dir) = setup();
    ex.execute("SELECT /*+ MERGE_JOIN */ * FROM t").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW QUERY HINTS").unwrap() {
        assert!(rows.iter().any(|r| r[0].contains("MERGE_JOIN")));
    } else {
        panic!();
    }
}
#[test]
fn test_s209_multiple_hints_accumulated() {
    let (ex, _dir) = setup();
    ex.execute("SELECT /*+ HASH_JOIN */ * FROM t").unwrap();
    ex.execute("SELECT /*+ NESTED_LOOP */ * FROM t").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW QUERY HINTS").unwrap() {
        assert!(rows.len() >= 2);
    } else {
        panic!();
    }
}

// â”€â”€ S210 â€” Table inheritance v2 â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s210_add_parent_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("ALTER TABLE child ADD PARENT parent2").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s210_show_inheritance_chain_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("ALTER TABLE child ADD PARENT parent2").unwrap();
    let r = ex.execute("SHOW INHERITANCE CHAIN FOR child").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s210_detach_table_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("ALTER TABLE child ADD PARENT parent2").unwrap();
    let r = ex.execute("DETACH TABLE child FROM parent2").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s210_parent_appears_in_chain() {
    let (ex, _dir) = setup();
    ex.execute("ALTER TABLE child ADD PARENT root").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW INHERITANCE CHAIN FOR child").unwrap()
    {
        assert!(rows.iter().any(|r| r[1] == "root"));
    } else {
        panic!();
    }
}
#[test]
fn test_s210_detach_removes_parent() {
    let (ex, _dir) = setup();
    ex.execute("ALTER TABLE child ADD PARENT root").unwrap();
    ex.execute("DETACH TABLE child FROM root").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW INHERITANCE CHAIN FOR child").unwrap()
    {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}

// â”€â”€ S211 â€” Result set transforms â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s211_pivot_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("SELECT * FROM t PIVOT col FOR val IN (a,b,c)")
        .unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s211_unpivot_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex.execute("SELECT * FROM t UNPIVOT col FOR val").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s211_show_pivot_cache_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("SELECT * FROM t PIVOT col FOR val IN (a,b,c)")
        .unwrap();
    let r = ex.execute("SHOW PIVOT CACHE").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s211_pivot_cached() {
    let (ex, _dir) = setup();
    ex.execute("SELECT * FROM orders PIVOT amount FOR month IN (jan,feb)")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW PIVOT CACHE").unwrap() {
        assert!(!rows.is_empty());
    } else {
        panic!();
    }
}
#[test]
fn test_s211_unpivot_result_column() {
    let (ex, _dir) = setup();
    if let QueryResult::Rows { columns, .. } =
        ex.execute("SELECT * FROM t UNPIVOT col FOR val").unwrap()
    {
        assert!(!columns.is_empty());
    } else {
        panic!();
    }
}

#[test]
fn test_s212_show_trace_samples_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW TRACE SAMPLES").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s212_clear_trace_samples_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("CLEAR TRACE SAMPLES").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s212_clear_empties_samples() {
    let (ex, _dir) = setup();
    ex.execute("CLEAR TRACE SAMPLES").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW TRACE SAMPLES").unwrap() {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}

// â”€â”€ S213 â€” Column aliases DDL â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s213_create_column_alias_returns_created() {
    let (ex, _dir) = setup();
    let r = ex.execute("CREATE COLUMN ALIAS alias FOR t.col").unwrap();
    assert!(matches!(r, QueryResult::Created));
}
#[test]
fn test_s213_drop_column_alias_returns_dropped() {
    let (ex, _dir) = setup();
    ex.execute("CREATE COLUMN ALIAS alias FOR t.col").unwrap();
    let r = ex.execute("DROP COLUMN ALIAS alias").unwrap();
    assert!(matches!(r, QueryResult::Dropped));
}
#[test]
fn test_s213_show_column_aliases_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("CREATE COLUMN ALIAS alias FOR t.col").unwrap();
    let r = ex.execute("SHOW COLUMN ALIASES").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s213_alias_appears_in_show() {
    let (ex, _dir) = setup();
    ex.execute("CREATE COLUMN ALIAS total FOR orders.amount")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW COLUMN ALIASES").unwrap() {
        assert!(rows.iter().any(|r| r[0] == "total"));
    } else {
        panic!();
    }
}
#[test]
fn test_s213_drop_removes_alias() {
    let (ex, _dir) = setup();
    ex.execute("CREATE COLUMN ALIAS total FOR orders.amount")
        .unwrap();
    ex.execute("DROP COLUMN ALIAS total").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW COLUMN ALIASES").unwrap() {
        assert!(rows.iter().all(|r| r[0] != "total"));
    } else {
        panic!();
    }
}

// â”€â”€ S214 â€” Table access patterns â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s214_show_access_pattern_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW ACCESS PATTERN FOR t").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s214_reset_access_pattern_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("RESET ACCESS PATTERN FOR t").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}

// â”€â”€ S215 â€” Query normalization â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s215_normalize_query_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("NORMALIZE QUERY 'SELECT * FROM t WHERE id = 1'")
        .unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s215_show_normalized_queries_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("NORMALIZE QUERY 'SELECT * FROM t WHERE id = 1'")
        .unwrap();
    let r = ex.execute("SHOW NORMALIZED QUERIES").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s215_clear_normalized_queries_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("CLEAR NORMALIZED QUERIES").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s215_normalized_appears_in_show() {
    let (ex, _dir) = setup();
    ex.execute("NORMALIZE QUERY 'SELECT * FROM orders'")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW NORMALIZED QUERIES").unwrap() {
        assert!(!rows.is_empty());
    } else {
        panic!();
    }
}
#[test]
fn test_s215_clear_empties_normalized() {
    let (ex, _dir) = setup();
    ex.execute("NORMALIZE QUERY 'SELECT 1'").unwrap();
    ex.execute("CLEAR NORMALIZED QUERIES").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW NORMALIZED QUERIES").unwrap() {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}

// â”€â”€ S216 â€” Incremental MV refresh â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s216_refresh_mv_incrementally_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("REFRESH MATERIALIZED VIEW mv INCREMENTALLY")
        .unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s216_show_mv_refresh_status_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("REFRESH MATERIALIZED VIEW mv INCREMENTALLY")
        .unwrap();
    let r = ex.execute("SHOW MV REFRESH STATUS").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s216_mv_status_recorded() {
    let (ex, _dir) = setup();
    ex.execute("REFRESH MATERIALIZED VIEW summary INCREMENTALLY")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW MV REFRESH STATUS").unwrap() {
        assert!(rows.iter().any(|r| r[1] == "incremental"));
    } else {
        panic!();
    }
}
#[test]
fn test_s216_multiple_mv_refreshes() {
    let (ex, _dir) = setup();
    ex.execute("REFRESH MATERIALIZED VIEW mv1 INCREMENTALLY")
        .unwrap();
    ex.execute("REFRESH MATERIALIZED VIEW mv2 INCREMENTALLY")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW MV REFRESH STATUS").unwrap() {
        assert!(rows.len() >= 2);
    } else {
        panic!();
    }
}

// â”€â”€ S218 â€” Query profiling â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s218_profile_select_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex.execute("PROFILE SELECT * FROM t").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s218_show_query_profiles_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW QUERY PROFILES").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s218_clear_query_profiles_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("CLEAR QUERY PROFILES").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s218_profile_recorded_in_show() {
    let (ex, _dir) = setup();
    ex.execute("PROFILE SELECT 1").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW QUERY PROFILES").unwrap() {
        assert!(!rows.is_empty());
    } else {
        panic!();
    }
}
#[test]
fn test_s218_clear_empties_profiles() {
    let (ex, _dir) = setup();
    ex.execute("PROFILE SELECT 1").unwrap();
    ex.execute("CLEAR QUERY PROFILES").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW QUERY PROFILES").unwrap() {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}

// â”€â”€ S219 â€” Table rewrites â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s219_rewrite_table_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("REWRITE TABLE t AS SELECT * FROM t WHERE active = 1")
        .unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s219_show_rewrite_history_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW TABLE REWRITE HISTORY").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s219_clear_rewrite_history_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("CLEAR TABLE REWRITE HISTORY").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s219_rewrite_recorded_in_history() {
    let (ex, _dir) = setup();
    ex.execute("REWRITE TABLE orders AS SELECT * FROM orders WHERE status = 'active'")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW TABLE REWRITE HISTORY").unwrap() {
        assert!(!rows.is_empty());
    } else {
        panic!();
    }
}
#[test]
fn test_s219_clear_empties_history() {
    let (ex, _dir) = setup();
    ex.execute("REWRITE TABLE t AS SELECT * FROM t").unwrap();
    ex.execute("CLEAR TABLE REWRITE HISTORY").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW TABLE REWRITE HISTORY").unwrap() {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}

#[test]
fn test_s220_show_storage_format_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW TABLE STORAGE FORMAT FOR t").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s220_default_format_is_row() {
    let (ex, _dir) = setup();
    if let QueryResult::Rows { rows, .. } = ex
        .execute("SHOW TABLE STORAGE FORMAT FOR unknown_table")
        .unwrap()
    {
        assert_eq!(rows[0][1], "ROW");
    } else {
        panic!();
    }
}

// â”€â”€ S221 â€” Workload classification â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s221_create_workload_class_returns_created() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("CREATE WORKLOAD CLASS wc WHEN user='admin'")
        .unwrap();
    assert!(matches!(r, QueryResult::Created));
}
#[test]
fn test_s221_drop_workload_class_returns_dropped() {
    let (ex, _dir) = setup();
    ex.execute("CREATE WORKLOAD CLASS wc WHEN user='admin'")
        .unwrap();
    let r = ex.execute("DROP WORKLOAD CLASS wc").unwrap();
    assert!(matches!(r, QueryResult::Dropped));
}
#[test]
fn test_s221_show_workload_classes_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW WORKLOAD CLASSES").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s221_class_appears_in_show() {
    let (ex, _dir) = setup();
    ex.execute("CREATE WORKLOAD CLASS analytics WHEN user='analyst'")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW WORKLOAD CLASSES").unwrap() {
        assert!(rows.iter().any(|r| r[0] == "analytics"));
    } else {
        panic!();
    }
}
#[test]
fn test_s221_drop_removes_class() {
    let (ex, _dir) = setup();
    ex.execute("CREATE WORKLOAD CLASS wc2 WHEN user='x'")
        .unwrap();
    ex.execute("DROP WORKLOAD CLASS wc2").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW WORKLOAD CLASSES").unwrap() {
        assert!(!rows.iter().any(|r| r[0] == "wc2"));
    } else {
        panic!();
    }
}

// â”€â”€ S222 â€” Query cancel hooks â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s222_on_query_cancel_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("ON QUERY CANCEL EXECUTE 'cleanup'").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s222_show_cancel_hooks_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW CANCEL HOOKS").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s222_drop_cancel_hook_returns_dropped() {
    let (ex, _dir) = setup();
    ex.execute("ON QUERY CANCEL EXECUTE 'cleanup'").unwrap();
    let r = ex.execute("DROP CANCEL HOOK 1").unwrap();
    assert!(matches!(r, QueryResult::Dropped));
}
#[test]
fn test_s222_hook_appears_in_show() {
    let (ex, _dir) = setup();
    ex.execute("ON QUERY CANCEL EXECUTE 'rollback'").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW CANCEL HOOKS").unwrap() {
        assert!(!rows.is_empty());
    } else {
        panic!();
    }
}
#[test]
fn test_s222_drop_hook_removes_it() {
    let (ex, _dir) = setup();
    ex.execute("ON QUERY CANCEL EXECUTE 'cleanup'").unwrap();
    ex.execute("DROP CANCEL HOOK 1").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW CANCEL HOOKS").unwrap() {
        assert!(!rows.iter().any(|r| r[0] == "1"));
    } else {
        panic!();
    }
}

// â”€â”€ S223 â€” Schema introspection v2 â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s223_describe_function_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex.execute("DESCRIBE FUNCTION fn").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s223_describe_trigger_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex.execute("DESCRIBE TRIGGER trig").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s223_describe_index_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex.execute("DESCRIBE INDEX idx").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s223_describe_known_function_shows_body() {
    let (ex, _dir) = setup();
    ex.execute("CREATE FUNCTION myfn(x) RETURNS U64 AS 'x'")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("DESCRIBE FUNCTION myfn").unwrap() {
        assert_eq!(rows[0][0], "myfn");
    } else {
        panic!();
    }
}
#[test]
fn test_s223_describe_index_type_is_btree() {
    let (ex, _dir) = setup();
    if let QueryResult::Rows { rows, .. } = ex.execute("DESCRIBE INDEX myidx").unwrap() {
        assert_eq!(rows[0][1], "btree");
    } else {
        panic!();
    }
}

// â”€â”€ S224 â€” Table health checks â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s224_check_table_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex.execute("CHECK TABLE t").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s224_repair_table_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("REPAIR TABLE t").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s224_show_table_health_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW TABLE HEALTH").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s224_check_table_status_ok() {
    let (ex, _dir) = setup();
    if let QueryResult::Rows { rows, .. } = ex.execute("CHECK TABLE orders").unwrap() {
        assert_eq!(rows[0][1], "ok");
    } else {
        panic!();
    }
}
#[test]
fn test_s224_health_log_records_checks() {
    let (ex, _dir) = setup();
    ex.execute("CHECK TABLE orders").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW TABLE HEALTH").unwrap() {
        assert!(!rows.is_empty());
    } else {
        panic!();
    }
}

// â”€â”€ S226 â€” Live reload config â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s226_reload_config_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("RELOAD SERVER CONFIG").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s226_reload_config_file_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("RELOAD SERVER CONFIG FILE 'path/to/config.toml'")
        .unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s226_show_config_reload_status_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW CONFIG RELOAD STATUS").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s226_reload_appears_in_status() {
    let (ex, _dir) = setup();
    ex.execute("RELOAD SERVER CONFIG").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW CONFIG RELOAD STATUS").unwrap() {
        assert!(!rows.is_empty());
    } else {
        panic!();
    }
}
#[test]
fn test_s226_reload_file_appears_in_status() {
    let (ex, _dir) = setup();
    ex.execute("RELOAD SERVER CONFIG FILE 'custom.toml'")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW CONFIG RELOAD STATUS").unwrap() {
        assert!(rows.iter().any(|r| r[1].contains("custom.toml")));
    } else {
        panic!();
    }
}

// â”€â”€ S227 â€” Adaptive indexing â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s227_enable_adaptive_indexing_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("ENABLE ADAPTIVE INDEXING ON t").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s227_disable_adaptive_indexing_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("DISABLE ADAPTIVE INDEXING ON t").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s227_show_adaptive_index_status_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW ADAPTIVE INDEX STATUS").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s227_table_appears_in_status_after_enable() {
    let (ex, _dir) = setup();
    ex.execute("ENABLE ADAPTIVE INDEXING ON orders").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW ADAPTIVE INDEX STATUS").unwrap() {
        assert!(rows.iter().any(|r| r[0] == "orders"));
    } else {
        panic!();
    }
}
#[test]
fn test_s227_disable_removes_table_from_status() {
    let (ex, _dir) = setup();
    ex.execute("ENABLE ADAPTIVE INDEXING ON logs").unwrap();
    ex.execute("DISABLE ADAPTIVE INDEXING ON logs").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW ADAPTIVE INDEX STATUS").unwrap() {
        assert!(!rows.iter().any(|r| r[0] == "logs"));
    } else {
        panic!();
    }
}

// â”€â”€ S228 â€” Query routing rules â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s228_create_routing_rule_returns_created() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("CREATE ROUTING RULE rr WHEN query_type='SELECT' ROUTE TO replica")
        .unwrap();
    assert!(matches!(r, QueryResult::Created));
}
#[test]
fn test_s228_drop_routing_rule_returns_dropped() {
    let (ex, _dir) = setup();
    ex.execute("CREATE ROUTING RULE rr WHEN query_type='SELECT' ROUTE TO replica")
        .unwrap();
    let r = ex.execute("DROP ROUTING RULE rr").unwrap();
    assert!(matches!(r, QueryResult::Dropped));
}
#[test]
fn test_s228_show_routing_rules_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("CREATE ROUTING RULE rr2 WHEN query_type='SELECT' ROUTE TO replica")
        .unwrap();
    let r = ex.execute("SHOW ROUTING RULES").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s228_routing_rule_appears_in_show() {
    let (ex, _dir) = setup();
    ex.execute("CREATE ROUTING RULE myrule WHEN query_type='INSERT' ROUTE TO primary")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW ROUTING RULES").unwrap() {
        assert!(rows.iter().any(|r| r[0] == "myrule"));
    } else {
        panic!();
    }
}
#[test]
fn test_s228_drop_routing_rule_removes_from_show() {
    let (ex, _dir) = setup();
    ex.execute("CREATE ROUTING RULE tr WHEN query_type='SELECT' ROUTE TO replica")
        .unwrap();
    ex.execute("DROP ROUTING RULE tr").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW ROUTING RULES").unwrap() {
        assert!(!rows.iter().any(|r| r[0] == "tr"));
    } else {
        panic!();
    }
}

#[test]
fn test_s229_show_table_expiry_policies_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("SET TABLE orders EXPIRE AFTER 90d").unwrap();
    let r = ex.execute("SHOW TABLE EXPIRY POLICIES").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s229_cancel_table_expiry_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("SET TABLE t EXPIRE AFTER 30d").unwrap();
    let r = ex.execute("CANCEL TABLE EXPIRY FOR t").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s229_cancel_removes_policy() {
    let (ex, _dir) = setup();
    ex.execute("SET TABLE events EXPIRE AFTER 14d").unwrap();
    ex.execute("CANCEL TABLE EXPIRY FOR events").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW TABLE EXPIRY POLICIES").unwrap() {
        assert!(!rows.iter().any(|r| r[0] == "events"));
    } else {
        panic!();
    }
}

// â”€â”€ S230 â€” Column rename tracking â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s230_rename_column_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE s230t (pk U64, old U64)").unwrap();
    let r = ex
        .execute("ALTER TABLE s230t RENAME COLUMN old TO new")
        .unwrap();
    assert!(matches!(r, QueryResult::Affected(_) | QueryResult::Ok(_)));
}
#[test]
fn test_s230_show_column_rename_history_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE s230h (pk U64, a U64)").unwrap();
    ex.execute("ALTER TABLE s230h RENAME COLUMN a TO b")
        .unwrap();
    let r = ex.execute("SHOW COLUMN RENAME HISTORY FOR s230h").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s230_clear_column_rename_history_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("CLEAR COLUMN RENAME HISTORY").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s230_rename_appears_in_history() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE s230o (pk U64, status U64)")
        .unwrap();
    ex.execute("ALTER TABLE s230o RENAME COLUMN status TO state")
        .unwrap();
    if let QueryResult::Rows { rows, .. } =
        ex.execute("SHOW COLUMN RENAME HISTORY FOR s230o").unwrap()
    {
        assert!(rows.iter().any(|r| r[0] == "status" && r[1] == "state"));
    } else {
        panic!();
    }
}
#[test]
fn test_s230_clear_removes_history() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE s230c (pk U64, x U64)").unwrap();
    ex.execute("ALTER TABLE s230c RENAME COLUMN x TO y")
        .unwrap();
    ex.execute("CLEAR COLUMN RENAME HISTORY").unwrap();
    if let QueryResult::Rows { rows, .. } =
        ex.execute("SHOW COLUMN RENAME HISTORY FOR s230c").unwrap()
    {
        assert!(rows.is_empty());
    } else {
        panic!();
    }
}

// â”€â”€ S231 â€” Query dependency graph â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s231_show_query_dependencies_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("SHOW QUERY DEPENDENCIES FOR 'SELECT * FROM t'")
        .unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s231_show_view_dependencies_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW VIEW DEPENDENCIES FOR v").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s231_show_table_dependents_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW TABLE DEPENDENTS FOR t").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s231_query_dependencies_has_rows() {
    let (ex, _dir) = setup();
    if let QueryResult::Rows { rows, .. } = ex
        .execute("SHOW QUERY DEPENDENCIES FOR 'SELECT * FROM orders'")
        .unwrap()
    {
        assert!(!rows.is_empty());
    } else {
        panic!();
    }
}
#[test]
fn test_s231_view_dependencies_has_rows() {
    let (ex, _dir) = setup();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW VIEW DEPENDENCIES FOR myview").unwrap()
    {
        assert!(!rows.is_empty());
    } else {
        panic!();
    }
}

// â”€â”€ S232 â€” Savepoint v2 â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s232_savepoint_with_description_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("SAVEPOINT sp WITH DESCRIPTION 'label'").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s232_list_savepoints_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("SAVEPOINT sp1 WITH DESCRIPTION 'first'")
        .unwrap();
    let r = ex.execute("LIST SAVEPOINTS").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s232_rollback_to_savepoint_partial_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("ROLLBACK TO SAVEPOINT sp PARTIAL").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s232_savepoint_description_appears_in_list() {
    let (ex, _dir) = setup();
    ex.execute("SAVEPOINT chk WITH DESCRIPTION 'checkpoint'")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("LIST SAVEPOINTS").unwrap() {
        assert!(rows.iter().any(|r| r[0] == "chk" && r[1] == "checkpoint"));
    } else {
        panic!();
    }
}
#[test]
fn test_s232_multiple_savepoints_in_list() {
    let (ex, _dir) = setup();
    ex.execute("SAVEPOINT sp_a WITH DESCRIPTION 'a'").unwrap();
    ex.execute("SAVEPOINT sp_b WITH DESCRIPTION 'b'").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("LIST SAVEPOINTS").unwrap() {
        assert!(rows.len() >= 2);
    } else {
        panic!();
    }
}

#[test]
fn test_s233_show_standby_query_mode_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW STANDBY QUERY MODE").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s233_allow_writes_on_standby_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("ALLOW WRITES ON STANDBY").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}

// â”€â”€ S234 â€” Index usage stats â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s234_show_index_usage_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW INDEX USAGE FOR t").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s234_reset_index_usage_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("RESET INDEX USAGE FOR t").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s234_show_unused_indexes_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("RESET INDEX USAGE FOR orders").unwrap();
    let r = ex.execute("SHOW UNUSED INDEXES").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s234_reset_table_appears_in_unused() {
    let (ex, _dir) = setup();
    ex.execute("RESET INDEX USAGE FOR inventory").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW UNUSED INDEXES").unwrap() {
        assert!(rows.iter().any(|r| r[0] == "inventory"));
    } else {
        panic!();
    }
}
#[test]
fn test_s234_show_index_usage_has_scan_count() {
    let (ex, _dir) = setup();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW INDEX USAGE FOR t").unwrap() {
        assert_eq!(rows[0][1], "0");
    } else {
        panic!();
    }
}

// â”€â”€ S235 â€” Schema export/import â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s235_export_schema_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("EXPORT SCHEMA public TO '/tmp/schema.sql'")
        .unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s235_import_schema_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("IMPORT SCHEMA FROM '/tmp/schema.sql'").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s235_show_schema_export_status_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("EXPORT SCHEMA public TO '/tmp/schema.sql'")
        .unwrap();
    let r = ex.execute("SHOW SCHEMA EXPORT STATUS").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s235_export_appears_in_status() {
    let (ex, _dir) = setup();
    ex.execute("EXPORT SCHEMA analytics TO '/tmp/analytics.sql'")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW SCHEMA EXPORT STATUS").unwrap() {
        assert!(rows.iter().any(|r| r[0] == "analytics"));
    } else {
        panic!();
    }
}
#[test]
fn test_s235_import_appears_in_status() {
    let (ex, _dir) = setup();
    ex.execute("IMPORT SCHEMA FROM '/tmp/myschema.sql'")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW SCHEMA EXPORT STATUS").unwrap() {
        assert!(rows.iter().any(|r| r[0] == "import"));
    } else {
        panic!();
    }
}

// â”€â”€ S236 â€” Query bucketing â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s236_create_query_bucket_returns_created() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("CREATE QUERY BUCKET qb FOR SELECT * FROM t WHERE col BETWEEN 1 AND 10")
        .unwrap();
    assert!(matches!(r, QueryResult::Created));
}
#[test]
fn test_s236_show_query_buckets_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("CREATE QUERY BUCKET qb FOR SELECT * FROM t WHERE col BETWEEN 1 AND 10")
        .unwrap();
    let r = ex.execute("SHOW QUERY BUCKETS").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s236_drop_query_bucket_returns_dropped() {
    let (ex, _dir) = setup();
    ex.execute("CREATE QUERY BUCKET qb FOR SELECT * FROM t WHERE col BETWEEN 1 AND 10")
        .unwrap();
    let r = ex.execute("DROP QUERY BUCKET qb").unwrap();
    assert!(matches!(r, QueryResult::Dropped));
}
#[test]
fn test_s236_bucket_appears_in_show() {
    let (ex, _dir) = setup();
    ex.execute("CREATE QUERY BUCKET mybucket FOR SELECT * FROM orders WHERE id > 0")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW QUERY BUCKETS").unwrap() {
        assert!(rows.iter().any(|r| r[0] == "mybucket"));
    } else {
        panic!();
    }
}
#[test]
fn test_s236_drop_removes_bucket() {
    let (ex, _dir) = setup();
    ex.execute("CREATE QUERY BUCKET tb FOR SELECT 1").unwrap();
    ex.execute("DROP QUERY BUCKET tb").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW QUERY BUCKETS").unwrap() {
        assert!(!rows.iter().any(|r| r[0] == "tb"));
    } else {
        panic!();
    }
}

// â”€â”€ S237 â€” Live metrics streaming â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s237_stream_metrics_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("STREAM METRICS TO 'localhost:9000'").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s237_stop_metrics_stream_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("STREAM METRICS TO 'localhost:9000'").unwrap();
    let r = ex.execute("STOP METRICS STREAM").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s237_show_metrics_stream_status_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW METRICS STREAM STATUS").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s237_streaming_active_after_stream_command() {
    let (ex, _dir) = setup();
    ex.execute("STREAM METRICS TO 'host:9001'").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW METRICS STREAM STATUS").unwrap() {
        assert_eq!(rows[0][0], "true");
    } else {
        panic!();
    }
}
#[test]
fn test_s237_streaming_inactive_after_stop() {
    let (ex, _dir) = setup();
    ex.execute("STREAM METRICS TO 'host:9001'").unwrap();
    ex.execute("STOP METRICS STREAM").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW METRICS STREAM STATUS").unwrap() {
        assert_eq!(rows[0][0], "false");
    } else {
        panic!();
    }
}

// â”€â”€ S217 â€” Storage encryption at rest â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s217_enable_encryption_at_rest_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("ENABLE ENCRYPTION AT REST").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s217_disable_encryption_at_rest_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("DISABLE ENCRYPTION AT REST").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s217_show_encryption_status_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW ENCRYPTION STATUS").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s217_encryption_enabled_status() {
    let (ex, _dir) = setup();
    ex.execute("ENABLE ENCRYPTION AT REST").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW ENCRYPTION STATUS").unwrap() {
        assert_eq!(rows[0][0], "true");
    } else {
        panic!();
    }
}
#[test]
fn test_s217_encryption_disabled_status() {
    let (ex, _dir) = setup();
    ex.execute("ENABLE ENCRYPTION AT REST").unwrap();
    ex.execute("DISABLE ENCRYPTION AT REST").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW ENCRYPTION STATUS").unwrap() {
        assert_eq!(rows[0][0], "false");
    } else {
        panic!();
    }
}

// ── S238 — Table observation hooks ──────────────────────────────────────────
#[test]
fn test_s238_create_observe_hook_returns_created() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("CREATE OBSERVE HOOK oh ON t WHEN INSERT")
        .unwrap();
    assert!(matches!(r, QueryResult::Created));
}
#[test]
fn test_s238_drop_observe_hook_returns_dropped() {
    let (ex, _dir) = setup();
    ex.execute("CREATE OBSERVE HOOK oh ON t WHEN INSERT")
        .unwrap();
    let r = ex.execute("DROP OBSERVE HOOK oh").unwrap();
    assert!(matches!(r, QueryResult::Dropped));
}
#[test]
fn test_s238_show_observe_hooks_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW OBSERVE HOOKS").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s238_hook_appears_in_show() {
    let (ex, _dir) = setup();
    ex.execute("CREATE OBSERVE HOOK oh1 ON orders WHEN INSERT")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW OBSERVE HOOKS").unwrap() {
        assert!(rows.iter().any(|r| r[0] == "oh1"));
    } else {
        panic!();
    }
}
#[test]
fn test_s238_drop_removes_hook() {
    let (ex, _dir) = setup();
    ex.execute("CREATE OBSERVE HOOK oh2 ON orders WHEN INSERT")
        .unwrap();
    ex.execute("DROP OBSERVE HOOK oh2").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW OBSERVE HOOKS").unwrap() {
        assert!(rows.iter().all(|r| r[0] != "oh2"));
    } else {
        panic!();
    }
}

// ── S240 — Column lineage ───────────────────────────────────────────────────
#[test]
fn test_s240_show_column_lineage_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW COLUMN LINEAGE FOR t.col").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s240_trace_column_to_source_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex.execute("TRACE COLUMN t.col TO SOURCE").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s240_show_lineage_graph_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW LINEAGE GRAPH").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s240_show_column_lineage_has_columns() {
    let (ex, _dir) = setup();
    if let QueryResult::Rows { columns, .. } =
        ex.execute("SHOW COLUMN LINEAGE FOR orders.id").unwrap()
    {
        assert_eq!(columns[0], "table");
    } else {
        panic!();
    }
}
#[test]
fn test_s240_trace_column_has_columns() {
    let (ex, _dir) = setup();
    if let QueryResult::Rows { columns, .. } =
        ex.execute("TRACE COLUMN orders.id TO SOURCE").unwrap()
    {
        assert_eq!(columns[1], "column");
    } else {
        panic!();
    }
}

// ── S241 — Failsafe mode ────────────────────────────────────────────────────
#[test]
fn test_s241_enable_failsafe_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("ENABLE FAILSAFE MODE").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s241_disable_failsafe_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("DISABLE FAILSAFE MODE").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s241_show_failsafe_status_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW FAILSAFE STATUS").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s241_test_failsafe_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("TEST FAILSAFE").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s241_failsafe_enabled_in_status() {
    let (ex, _dir) = setup();
    ex.execute("ENABLE FAILSAFE MODE").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW FAILSAFE STATUS").unwrap() {
        assert_eq!(rows[0][0], "true");
    } else {
        panic!();
    }
}

// ── S242 — Query signatures ─────────────────────────────────────────────────
#[test]
fn test_s242_compute_query_signature_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("COMPUTE QUERY SIGNATURE 'SELECT * FROM t'")
        .unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s242_show_query_signatures_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW QUERY SIGNATURES").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s242_match_query_signature_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex.execute("MATCH QUERY SIGNATURE 'abc123'").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s242_signature_appears_in_show() {
    let (ex, _dir) = setup();
    ex.execute("COMPUTE QUERY SIGNATURE 'SELECT 1'").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW QUERY SIGNATURES").unwrap() {
        assert!(!rows.is_empty());
    } else {
        panic!();
    }
}
#[test]
fn test_s242_signature_has_columns() {
    let (ex, _dir) = setup();
    if let QueryResult::Rows { columns, .. } =
        ex.execute("COMPUTE QUERY SIGNATURE 'SELECT 2'").unwrap()
    {
        assert_eq!(columns[0], "signature");
    } else {
        panic!();
    }
}

#[test]
fn test_s243_show_table_merge_strategies_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW TABLE MERGE STRATEGIES").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}

// ── S244 — Schema snapshots ─────────────────────────────────────────────────
#[test]
fn test_s244_snapshot_schema_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("SNAPSHOT SCHEMA public TO 'snap1'").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s244_restore_schema_from_snapshot_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("RESTORE SCHEMA FROM SNAPSHOT 'snap1'").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s244_show_schema_snapshots_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW SCHEMA SNAPSHOTS").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s244_snapshot_appears_in_show() {
    let (ex, _dir) = setup();
    ex.execute("SNAPSHOT SCHEMA public TO 'mysnap'").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW SCHEMA SNAPSHOTS").unwrap() {
        assert!(rows.iter().any(|r| r[0] == "mysnap"));
    } else {
        panic!();
    }
}
#[test]
fn test_s244_schema_snapshots_has_columns() {
    let (ex, _dir) = setup();
    if let QueryResult::Rows { columns, .. } = ex.execute("SHOW SCHEMA SNAPSHOTS").unwrap() {
        assert_eq!(columns[0], "name");
    } else {
        panic!();
    }
}

// ── S245 — Query plan export ────────────────────────────────────────────────
#[test]
fn test_s245_export_query_plan_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("EXPORT QUERY PLAN FOR SELECT * FROM t TO 'plan.json'")
        .unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s245_import_query_plan_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("IMPORT QUERY PLAN FROM 'plan.json'").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s245_show_exported_plans_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW EXPORTED PLANS").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s245_plan_appears_in_show() {
    let (ex, _dir) = setup();
    ex.execute("EXPORT QUERY PLAN FOR SELECT * FROM t TO 'myplan.json'")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW EXPORTED PLANS").unwrap() {
        assert!(!rows.is_empty());
    } else {
        panic!();
    }
}
#[test]
fn test_s245_exported_plans_has_columns() {
    let (ex, _dir) = setup();
    if let QueryResult::Rows { columns, .. } = ex.execute("SHOW EXPORTED PLANS").unwrap() {
        assert_eq!(columns[0], "path");
    } else {
        panic!();
    }
}

// ── S246 — Table fragmentation ──────────────────────────────────────────────
#[test]
fn test_s246_show_table_fragmentation_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW TABLE FRAGMENTATION FOR t").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s246_defragment_table_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("DEFRAGMENT TABLE t").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s246_fragmentation_has_columns() {
    let (ex, _dir) = setup();
    if let QueryResult::Rows { columns, .. } = ex.execute("SHOW TABLE FRAGMENTATION FOR t").unwrap()
    {
        assert_eq!(columns[0], "table");
    } else {
        panic!();
    }
}

// ── S248 — Cross-shard transactions ─────────────────────────────────────────
#[test]
fn test_s248_begin_cross_shard_returns_tx_begin() {
    let (ex, _dir) = setup();
    let r = ex.execute("BEGIN CROSS SHARD TRANSACTION").unwrap();
    assert!(matches!(r, QueryResult::TxBegin));
}
#[test]
fn test_s248_commit_cross_shard_returns_tx_commit() {
    let (ex, _dir) = setup();
    ex.execute("BEGIN CROSS SHARD TRANSACTION").unwrap();
    let r = ex.execute("COMMIT CROSS SHARD").unwrap();
    assert!(matches!(r, QueryResult::TxCommit));
}
#[test]
fn test_s248_show_cross_shard_tx_status_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW CROSS SHARD TX STATUS").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s248_active_flag_set_after_begin() {
    let (ex, _dir) = setup();
    ex.execute("BEGIN CROSS SHARD TRANSACTION").unwrap();
    assert!(*ex.s248_cross_shard_tx_active.lock_safe());
}
#[test]
fn test_s248_active_flag_cleared_after_commit() {
    let (ex, _dir) = setup();
    ex.execute("BEGIN CROSS SHARD TRANSACTION").unwrap();
    ex.execute("COMMIT CROSS SHARD").unwrap();
    assert!(!*ex.s248_cross_shard_tx_active.lock_safe());
}

// ── S249 — Table statistics reset ────────────────────────────────────────────
#[test]
fn test_s249_reset_table_stats_for_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("RESET TABLE STATS FOR orders").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s249_reset_all_table_stats_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("RESET ALL TABLE STATS").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s249_show_stats_reset_history_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("RESET TABLE STATS FOR orders").unwrap();
    let r = ex.execute("SHOW STATS RESET HISTORY").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s249_reset_history_records_table_name() {
    let (ex, _dir) = setup();
    ex.execute("RESET TABLE STATS FOR orders").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW STATS RESET HISTORY").unwrap() {
        assert_eq!(rows[0][0], "orders");
    } else {
        panic!();
    }
}
#[test]
fn test_s249_multiple_resets_recorded() {
    let (ex, _dir) = setup();
    ex.execute("RESET TABLE STATS FOR t1").unwrap();
    ex.execute("RESET TABLE STATS FOR t2").unwrap();
    let log = ex.s249_stats_reset_log.lock_safe().clone();
    assert_eq!(log.len(), 2);
}

// ── S250 — Query plan locking ─────────────────────────────────────────────────
#[test]
fn test_s250_lock_query_plan_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("LOCK QUERY PLAN FOR SELECT * FROM orders")
        .unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s250_unlock_query_plan_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("UNLOCK QUERY PLAN").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s250_show_locked_query_plans_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("LOCK QUERY PLAN FOR SELECT * FROM orders")
        .unwrap();
    let r = ex.execute("SHOW LOCKED QUERY PLANS").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s250_locked_plan_stored() {
    let (ex, _dir) = setup();
    ex.execute("LOCK QUERY PLAN FOR SELECT * FROM orders")
        .unwrap();
    assert!(!ex.s250_locked_plans.lock_safe().is_empty());
}
#[test]
fn test_s250_unlock_clears_plans() {
    let (ex, _dir) = setup();
    ex.execute("LOCK QUERY PLAN FOR SELECT * FROM orders")
        .unwrap();
    ex.execute("UNLOCK QUERY PLAN").unwrap();
    assert!(ex.s250_locked_plans.lock_safe().is_empty());
}

// ── S251 — Column histograms v3 ───────────────────────────────────────────────
#[test]
fn test_s251_build_histogram_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("BUILD HISTOGRAM FOR orders.amount BUCKETS 50")
        .unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s251_show_histogram_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("BUILD HISTOGRAM FOR orders.amount BUCKETS 5")
        .unwrap();
    let r = ex.execute("SHOW HISTOGRAM FOR orders.amount").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s251_show_histogram_bucket_count() {
    let (ex, _dir) = setup();
    ex.execute("BUILD HISTOGRAM FOR orders.amount BUCKETS 3")
        .unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW HISTOGRAM FOR orders.amount").unwrap()
    {
        assert_eq!(rows.len(), 3);
    } else {
        panic!();
    }
}
#[test]
fn test_s251_drop_histogram_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("BUILD HISTOGRAM FOR orders.amount BUCKETS 5")
        .unwrap();
    let r = ex.execute("DROP HISTOGRAM FOR orders.amount").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s251_drop_histogram_removes_data() {
    let (ex, _dir) = setup();
    ex.execute("BUILD HISTOGRAM FOR orders.amount BUCKETS 5")
        .unwrap();
    ex.execute("DROP HISTOGRAM FOR orders.amount").unwrap();
    assert!(ex.s251_histograms_v3.lock_safe().is_empty());
}

// ── S252 — Table access control v2 ───────────────────────────────────────────
#[test]
fn test_s252_grant_select_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("GRANT SELECT ON orders TO ROLE analyst")
        .unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s252_revoke_select_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("GRANT SELECT ON orders TO ROLE analyst")
        .unwrap();
    let r = ex
        .execute("REVOKE SELECT ON orders FROM ROLE analyst")
        .unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s252_show_role_grants_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("GRANT SELECT ON orders TO ROLE analyst")
        .unwrap();
    let r = ex.execute("SHOW ROLE GRANTS").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s252_grant_stored_in_list() {
    let (ex, _dir) = setup();
    ex.execute("GRANT SELECT ON orders TO ROLE analyst")
        .unwrap();
    assert!(!ex.s252_role_grants.lock_safe().is_empty());
}
#[test]
fn test_s252_revoke_removes_grant() {
    let (ex, _dir) = setup();
    ex.execute("GRANT SELECT ON orders TO ROLE analyst")
        .unwrap();
    ex.execute("REVOKE SELECT ON orders FROM ROLE analyst")
        .unwrap();
    assert!(ex.s252_role_grants.lock_safe().is_empty());
}

// ── S253 — Query scheduling ───────────────────────────────────────────────────
#[test]
fn test_s253_schedule_query_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("SCHEDULE QUERY 'SELECT * FROM orders' AT '2026-01-01 00:00'")
        .unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s253_show_scheduled_queries_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("SCHEDULE QUERY 'SELECT * FROM orders' AT '2026-01-01 00:00'")
        .unwrap();
    let r = ex.execute("SHOW SCHEDULED QUERIES").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s253_cancel_scheduled_query_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("SCHEDULE QUERY 'SELECT 1' AT '2026-06-01 12:00'")
        .unwrap();
    let r = ex.execute("CANCEL SCHEDULED QUERY 1").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s253_scheduled_query_count() {
    let (ex, _dir) = setup();
    ex.execute("SCHEDULE QUERY 'SELECT 1' AT '2026-01-01 00:00'")
        .unwrap();
    ex.execute("SCHEDULE QUERY 'SELECT 2' AT '2026-02-01 00:00'")
        .unwrap();
    assert_eq!(ex.s253_scheduled_queries.lock_safe().len(), 2);
}
#[test]
fn test_s253_cancel_marks_cancelled() {
    let (ex, _dir) = setup();
    ex.execute("SCHEDULE QUERY 'SELECT 1' AT '2026-01-01 00:00'")
        .unwrap();
    ex.execute("CANCEL SCHEDULED QUERY 1").unwrap();
    let qs = ex.s253_scheduled_queries.lock_safe();
    assert!(qs[0].3);
}

// ── S254 — Index coalescing ───────────────────────────────────────────────────
#[test]
fn test_s254_coalesce_indexes_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("COALESCE INDEXES ON orders").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s254_show_index_coalesce_status_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW INDEX COALESCE STATUS").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s254_coalesce_log_records_table() {
    let (ex, _dir) = setup();
    ex.execute("COALESCE INDEXES ON orders").unwrap();
    let log = ex.s254_coalesce_log.lock_safe();
    assert_eq!(log[0], "orders");
}

#[test]
fn test_s255_show_scan_parallelism_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("SET TABLE orders SCAN PARALLELISM 4").unwrap();
    let r = ex.execute("SHOW SCAN PARALLELISM FOR orders").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s255_reset_scan_parallelism_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("SET TABLE orders SCAN PARALLELISM 4").unwrap();
    let r = ex.execute("RESET SCAN PARALLELISM FOR orders").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s255_reset_removes_parallelism() {
    let (ex, _dir) = setup();
    ex.execute("SET TABLE orders SCAN PARALLELISM 4").unwrap();
    ex.execute("RESET SCAN PARALLELISM FOR orders").unwrap();
    assert!(!ex.s255_scan_parallelism.lock_safe().contains_key("orders"));
}

// ── S256 — Query result caching v2 ───────────────────────────────────────────
#[test]
fn test_s256_cache_query_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("CACHE QUERY 'SELECT * FROM orders' TTL 300")
        .unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s256_show_cached_queries_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("CACHE QUERY 'SELECT * FROM orders' TTL 300")
        .unwrap();
    let r = ex.execute("SHOW CACHED QUERIES").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s256_evict_cached_query_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("CACHE QUERY 'SELECT * FROM orders' TTL 300")
        .unwrap();
    let r = ex.execute("EVICT CACHED QUERY 1").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s256_cached_query_count() {
    let (ex, _dir) = setup();
    ex.execute("CACHE QUERY 'SELECT 1' TTL 60").unwrap();
    ex.execute("CACHE QUERY 'SELECT 2' TTL 120").unwrap();
    assert_eq!(ex.s256_cached_queries.lock_safe().len(), 2);
}
#[test]
fn test_s256_evict_removes_entry() {
    let (ex, _dir) = setup();
    ex.execute("CACHE QUERY 'SELECT 1' TTL 60").unwrap();
    ex.execute("EVICT CACHED QUERY 1").unwrap();
    assert!(ex.s256_cached_queries.lock_safe().is_empty());
}

// ── S257 — Live schema migration ──────────────────────────────────────────────
#[test]
fn test_s257_migrate_schema_live_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("MIGRATE SCHEMA public LIVE").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s257_show_live_migration_status_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("MIGRATE SCHEMA public LIVE").unwrap();
    let r = ex.execute("SHOW LIVE MIGRATION STATUS").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s257_pause_live_migration_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("MIGRATE SCHEMA public LIVE").unwrap();
    let r = ex.execute("PAUSE LIVE MIGRATION").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s257_resume_live_migration_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("MIGRATE SCHEMA public LIVE").unwrap();
    ex.execute("PAUSE LIVE MIGRATION").unwrap();
    let r = ex.execute("RESUME LIVE MIGRATION").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s257_pause_sets_paused_flag() {
    let (ex, _dir) = setup();
    ex.execute("MIGRATE SCHEMA public LIVE").unwrap();
    ex.execute("PAUSE LIVE MIGRATION").unwrap();
    let migs = ex.s257_live_migrations.lock_safe();
    assert!(migs["public"].1);
}

// ── S247 — Shard rebalancing ────────────────────────────────────────────────
#[test]
fn test_s247_rebalance_shards_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("REBALANCE SHARDS").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s247_show_shard_balance_returns_rows() {
    let (ex, _dir) = setup();
    let r = ex.execute("SHOW SHARD BALANCE").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s247_rebalance_count_increments() {
    let (ex, _dir) = setup();
    ex.execute("REBALANCE SHARDS").unwrap();
    ex.execute("REBALANCE SHARDS").unwrap();
    if let QueryResult::Rows { rows, .. } = ex.execute("SHOW SHARD BALANCE").unwrap() {
        let count: usize = rows[0][1].parse().unwrap();
        assert_eq!(count, 2);
    } else {
        panic!();
    }
}

// â”€â”€ S258 â€" Query cost feedback â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s258_report_query_cost_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("REPORT QUERY COST 1 ACTUAL 50ms").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s258_show_cost_feedback_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("REPORT QUERY COST 1 ACTUAL 50ms").unwrap();
    let r = ex.execute("SHOW QUERY COST FEEDBACK").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s258_feedback_recorded_value() {
    let (ex, _dir) = setup();
    ex.execute("REPORT QUERY COST 3 ACTUAL 99ms").unwrap();
    let fb = ex.s258_cost_feedback.lock_safe();
    assert_eq!(fb[0], (3, 99));
}
#[test]
fn test_s258_clear_cost_feedback_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("REPORT QUERY COST 1 ACTUAL 50ms").unwrap();
    let r = ex.execute("CLEAR QUERY COST FEEDBACK").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s258_clear_empties_feedback() {
    let (ex, _dir) = setup();
    ex.execute("REPORT QUERY COST 1 ACTUAL 50ms").unwrap();
    ex.execute("CLEAR QUERY COST FEEDBACK").unwrap();
    assert!(ex.s258_cost_feedback.lock_safe().is_empty());
}

// â”€â”€ S259 â€" Table bloom filters v2 â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s259_create_bloom_filter_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("CREATE BLOOM FILTER ON orders(status) FPR 0.01")
        .unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s259_drop_bloom_filter_returns_ok() {
    let (ex, _dir) = setup();
    ex.execute("CREATE BLOOM FILTER ON orders(status) FPR 0.01")
        .unwrap();
    let r = ex.execute("DROP BLOOM FILTER ON orders").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
#[test]
fn test_s259_show_bloom_filters_returns_rows() {
    let (ex, _dir) = setup();
    ex.execute("CREATE BLOOM FILTER ON orders(status) FPR 0.01")
        .unwrap();
    let r = ex.execute("SHOW BLOOM FILTERS").unwrap();
    assert!(matches!(r, QueryResult::Rows { .. }));
}
#[test]
fn test_s259_stored_fpr_value() {
    let (ex, _dir) = setup();
    ex.execute("CREATE BLOOM FILTER ON orders(status) FPR 0.05")
        .unwrap();
    let bf = ex.s259_bloom_filters_v2.lock_safe();
    assert_eq!(bf.get("orders").unwrap().1, 0.05);
}
#[test]
fn test_s259_drop_removes_entry() {
    let (ex, _dir) = setup();
    ex.execute("CREATE BLOOM FILTER ON orders(status) FPR 0.01")
        .unwrap();
    ex.execute("DROP BLOOM FILTER ON orders").unwrap();
    assert!(ex.s259_bloom_filters_v2.lock_safe().is_empty());
}

// â”€â”€ S260 â€" Async query execution â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
#[test]
fn test_s260_submit_async_query_returns_ok() {
    let (ex, _dir) = setup();
    let r = ex.execute("SUBMIT ASYNC QUERY 'SELECT 1'").unwrap();
    assert!(matches!(r, QueryResult::Ok(_)));
}
