//! S22 integration tests — VV-PRJ-010..021, VV-CFG-040..041

use std::collections::HashMap;
use std::sync::Arc;

use vectorvolt_semantic::config::PartitionStrategyKind;
use vectorvolt_semantic::{
    BuiltinReducer, CacheConfig, CustomerRouter, DateRangeRouter, ExplicitShardMap,
    ExpressionRouter, InProcessReadCache, PartitionConfig, PartitionPruner, PartitionStrategy,
    ReadCache, ReadModelConfig, ReadModelRegistry, ReducerKind, RhaiReducer, SemanticReducer,
    SemanticRouter,
};
use vectorvolt_txn::Mutation;

// ── VV-PRJ-011: DateRangeRouter ───────────────────────────────────────────────

#[test]
fn date_bucket_month_granularity() {
    let router = DateRangeRouter::new("month");
    // 2025-01-15 epoch ~ 1736899200
    let bucket_jan = router.bucket(1736899200);
    let bucket_feb = router.bucket(1739577600); // 2025-02-15
    assert_ne!(bucket_jan, bucket_feb);
}

#[test]
fn date_bucket_year_granularity_same_year() {
    let router = DateRangeRouter::new("year");
    let jan = router.bucket(1736899200); // Jan 2025
    let dec = router.bucket(1766534400); // Dec 2025
    assert_eq!(jan, dec, "same year → same bucket");
}

#[test]
fn date_prune_reduces_partitions_scanned() {
    let router = DateRangeRouter::new("month");
    let mut pruner = PartitionPruner::new();
    // One representative timestamp per month in 2025 (1st of each month UTC)
    let month_starts: &[i64] = &[
        1735689600, // 2025-01-01
        1738368000, // 2025-02-01
        1740787200, // 2025-03-01
        1743465600, // 2025-04-01
        1746057600, // 2025-05-01
        1748736000, // 2025-06-01
        1751328000, // 2025-07-01
        1754006400, // 2025-08-01
        1756684800, // 2025-09-01
        1759276800, // 2025-10-01
        1761955200, // 2025-11-01
        1764547200, // 2025-12-01
    ];
    for ts in month_starts {
        pruner.record_write(router.bucket(*ts));
    }
    assert_eq!(pruner.written_count(), 12);
    // Query: January 2025 only — should scan exactly 1 partition
    let jan_start = 1735689600i64;
    let jan_end = 1738367999i64;
    let candidates = router.buckets_for_range(jan_start, jan_end);
    let scanned = pruner.prune(&candidates);
    assert_eq!(
        scanned.len(),
        1,
        "only January bucket should survive pruning"
    );
}

// ── VV-PRJ-012: CustomerRouter ───────────────────────────────────────────────

#[test]
fn customer_router_consistent_hash() {
    let router = CustomerRouter::new(8);
    let shard_a = router.route_i64(42).unwrap();
    let shard_b = router.route_i64(42).unwrap();
    assert_eq!(shard_a, shard_b, "same FK → same shard always");
}

#[test]
fn customer_router_within_bounds() {
    let router = CustomerRouter::new(4);
    for pk in 0i64..100 {
        let s = router.route_i64(pk).unwrap();
        assert!(s < 4);
    }
}

// ── VV-PRJ-020: ExpressionRouter ─────────────────────────────────────────────

#[test]
fn expression_router_formula_mod_shard() {
    // formula: value / 100000  → bucket = (amount / $1000) % 4
    let router = ExpressionRouter::new("value / 100000".into(), 4);
    let shard = router.route_i64(250_000).unwrap();
    // 250000 / 100000 = 2, 2 % 4 = 2
    assert_eq!(shard, 2);
}

#[test]
fn expression_router_deterministic() {
    let router = ExpressionRouter::new("value * 3".into(), 8);
    let s1 = router.route_i64(7).unwrap();
    let s2 = router.route_i64(7).unwrap();
    assert_eq!(s1, s2);
}

// ── VV-PRJ-021: ExplicitShardMap ─────────────────────────────────────────────

#[test]
fn explicit_shard_map_routes_by_region() {
    let mut map = HashMap::new();
    map.insert("EU".into(), 0usize);
    map.insert("US".into(), 1usize);
    map.insert("APAC".into(), 2usize);
    let router = ExplicitShardMap::new(map);
    assert_eq!(router.route_str("EU").unwrap(), 0);
    assert_eq!(router.route_str("US").unwrap(), 1);
    assert_eq!(router.route_str("APAC").unwrap(), 2);
}

#[test]
fn explicit_shard_map_unknown_key_errors() {
    let router = ExplicitShardMap::new(HashMap::new());
    assert!(router.route_str("JP").is_err());
}

// ── VV-PRJ-013: PartitionPruner ──────────────────────────────────────────────

#[test]
fn pruner_skips_empty_buckets() {
    let mut pruner = PartitionPruner::new();
    pruner.record_write(3);
    pruner.record_write(7);
    let result = pruner.prune(&[1, 3, 5, 7, 9]);
    assert_eq!(result, vec![3, 7]);
}

// ── VV-CFG-040: PartitionConfig YAML round-trip ──────────────────────────────

#[test]
fn partition_config_yaml_round_trip() {
    let yaml = r#"
strategy: date
column: created_at
granularity: month
"#;
    let cfg: PartitionConfig = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(cfg.strategy, PartitionStrategyKind::Date);
    assert_eq!(cfg.granularity.as_deref(), Some("month"));
}

#[test]
fn partition_config_explicit_shard_map() {
    let yaml = r#"
strategy: explicit
column: region
shard_map:
  EU: 0
  US: 1
"#;
    let cfg: PartitionConfig = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(cfg.shard_map["EU"], 0);
    assert_eq!(cfg.shard_map["US"], 1);
}

// ── VV-PRJ-014: ReadModelRegistry ───────────────────────────────────────────

#[test]
fn registry_register_and_query() {
    let reg = ReadModelRegistry::new();
    reg.register(Box::new(BuiltinReducer::count("order_count")), None, None);
    reg.apply(
        "order_count",
        "cust:1",
        &Mutation::Insert {
            pk: 1,
            values: vec![1],
        },
    );
    reg.apply(
        "order_count",
        "cust:1",
        &Mutation::Insert {
            pk: 2,
            values: vec![1],
        },
    );
    let snap = reg.query("order_count").unwrap();
    assert_eq!(snap["cust:1"], serde_json::json!(2));
}

#[test]
fn registry_stop_pauses_updates() {
    let reg = ReadModelRegistry::new();
    reg.register(Box::new(BuiltinReducer::count("cnt")), None, None);
    reg.apply(
        "cnt",
        "k",
        &Mutation::Insert {
            pk: 1,
            values: vec![],
        },
    );
    reg.stop("cnt");
    reg.apply(
        "cnt",
        "k",
        &Mutation::Insert {
            pk: 2,
            values: vec![],
        },
    );
    let snap = reg.query("cnt").unwrap();
    // Only 1 event landed before stop
    assert_eq!(snap["k"], serde_json::json!(1));
}

// ── VV-PRJ-015: InProcessReadCache ───────────────────────────────────────────

#[test]
fn cache_set_get_hit() {
    let cache = InProcessReadCache::new(100, 60);
    let mut snap = HashMap::new();
    snap.insert("total".into(), serde_json::json!(42));
    cache.set("k1", Arc::new(snap));
    let got = cache.get("k1").unwrap();
    assert_eq!(got["total"], serde_json::json!(42));
}

#[test]
fn cache_invalidate_removes_entry() {
    let cache = InProcessReadCache::new(100, 60);
    cache.set("k2", Arc::new(HashMap::new()));
    cache.invalidate("k2");
    assert!(cache.get("k2").is_none());
}

// ── VV-PRJ-018: RhaiReducer ──────────────────────────────────────────────────

#[test]
fn rhai_reducer_count_custom() {
    let script = r#"
fn reduce(state, row) {
    let cnt = if state.contains("count") { state["count"] } else { 0 };
    state["count"] = cnt + 1;
    state
}
"#;
    let mut reducer = RhaiReducer::from_source("rolling_count", script);
    reducer
        .apply(
            "cust:1",
            &Mutation::Insert {
                pk: 1,
                values: vec![100],
            },
        )
        .unwrap();
    reducer
        .apply(
            "cust:1",
            &Mutation::Insert {
                pk: 2,
                values: vec![200],
            },
        )
        .unwrap();
    let snap = reducer.snapshot();
    // The count for key cust:1 should be 2
    assert!(snap.contains_key("cust:1"));
}

// ── VV-CFG-041: CacheConfig YAML ─────────────────────────────────────────────

#[test]
fn cache_config_yaml_round_trip() {
    let yaml = r#"
in_process:
  max_entries: 10000
  ttl_s: 60
redis:
  url: redis://localhost:6379
  ttl_s: 300
  key_prefix: "vv:rm:"
"#;
    let cfg: CacheConfig = serde_yaml::from_str(yaml).unwrap();
    let ip = cfg.in_process.unwrap();
    assert_eq!(ip.max_entries, 10000);
    assert_eq!(ip.ttl_s, 60);
    let redis = cfg.redis.unwrap();
    assert_eq!(redis.url, "redis://localhost:6379");
}

// ── VV-PRJ-017: ReadModelConfig YAML ─────────────────────────────────────────

#[test]
fn read_model_config_yaml_parse() {
    let yaml = r#"
name: monthly_order_totals
source: orders
reducer: sum
group_by: month(created_at)
cache:
  in_process:
    max_entries: 10000
    ttl_s: 60
"#;
    let cfg: ReadModelConfig = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(cfg.name, "monthly_order_totals");
    assert_eq!(cfg.reducer, ReducerKind::Sum);
}

// ── VV-PRJ-017: PartitionStrategy::from_config ────────────────────────────────

#[test]
fn partition_strategy_from_config_date() {
    let cfg = PartitionConfig {
        strategy: PartitionStrategyKind::Date,
        granularity: Some("month".into()),
        ..Default::default()
    };
    let _ = PartitionStrategy::from_config(&cfg).unwrap();
}

#[test]
fn partition_strategy_from_config_expression() {
    let cfg = PartitionConfig {
        strategy: PartitionStrategyKind::Expression,
        formula: Some("value / 1000".into()),
        shard_count: Some(4),
        ..Default::default()
    };
    let _ = PartitionStrategy::from_config(&cfg).unwrap();
}
