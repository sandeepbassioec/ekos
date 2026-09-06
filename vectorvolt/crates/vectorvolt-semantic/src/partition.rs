//! Semantic partition routing (VV-PRJ-010..013, VV-PRJ-020, VV-PRJ-021).

use chrono::{DateTime, Datelike, Utc};
use rhai::{Engine, Scope};
use std::collections::HashMap;
use thiserror::Error;

use crate::config::PartitionConfig;

#[derive(Debug, Error)]
pub enum PartitionError {
    #[error("shard_count must be > 0")]
    ZeroShardCount,
    #[error("unknown shard key '{0}'")]
    UnknownKey(String),
    #[error("rhai eval: {0}")]
    RhaiEval(String),
}

pub type PartitionResult<T> = Result<T, PartitionError>;

/// Strategy selector (VV-PRJ-010).
pub enum PartitionStrategy {
    None,
    Date(DateRangeRouter),
    Customer(CustomerRouter),
    Expression(Box<ExpressionRouter>),
    Explicit(ExplicitShardMap),
}

impl PartitionStrategy {
    /// Build a strategy from YAML config.
    pub fn from_config(cfg: &PartitionConfig) -> PartitionResult<Self> {
        use crate::config::PartitionStrategyKind::*;
        Ok(match cfg.strategy {
            None => PartitionStrategy::None,
            Date => PartitionStrategy::Date(DateRangeRouter::new(
                cfg.granularity.as_deref().unwrap_or("month"),
            )),
            Customer | Hash => {
                let shards = cfg.shard_count.unwrap_or(8);
                if shards == 0 {
                    return Err(PartitionError::ZeroShardCount);
                }
                PartitionStrategy::Customer(CustomerRouter::new(shards))
            }
            Expression => {
                let shards = cfg.shard_count.unwrap_or(8);
                let formula = cfg.formula.clone().unwrap_or_else(|| "0".into());
                PartitionStrategy::Expression(Box::new(ExpressionRouter::new(formula, shards)))
            }
            Explicit => PartitionStrategy::Explicit(ExplicitShardMap::new(cfg.shard_map.clone())),
        })
    }
}

/// Dispatches to the underlying strategy (VV-PRJ-010 `SemanticPartitionKey` trait).
pub trait SemanticRouter: Send + Sync {
    /// Route an i64 value (e.g. timestamp epoch seconds or integer FK) to a bucket.
    fn route_i64(&self, value: i64) -> PartitionResult<usize>;
    /// Route a string value (e.g. region, tenant) to a shard index.
    fn route_str(&self, value: &str) -> PartitionResult<usize>;
}

// ── DateRangeRouter ──────────────────────────────────────────────────────────

/// Maps Unix timestamp → partition bucket by year/month/day (VV-PRJ-011).
pub struct DateRangeRouter {
    granularity: Granularity,
}

enum Granularity {
    Year,
    Month,
    Day,
}

impl DateRangeRouter {
    pub fn new(gran: &str) -> Self {
        let granularity = match gran {
            "year" => Granularity::Year,
            "day" => Granularity::Day,
            _ => Granularity::Month,
        };
        Self { granularity }
    }

    pub fn bucket(&self, ts_secs: i64) -> usize {
        let dt = DateTime::<Utc>::from_timestamp(ts_secs, 0).unwrap_or_default();
        match self.granularity {
            Granularity::Year => dt.year() as usize,
            Granularity::Month => dt.year() as usize * 12 + (dt.month0() as usize),
            Granularity::Day => dt.year() as usize * 366 + dt.ordinal0() as usize,
        }
    }

    /// Given a date range predicate, return the set of buckets that overlap it.
    pub fn buckets_for_range(&self, start_ts: i64, end_ts: i64) -> Vec<usize> {
        let start = self.bucket(start_ts);
        let end = self.bucket(end_ts);
        (start..=end).collect()
    }
}

impl SemanticRouter for DateRangeRouter {
    fn route_i64(&self, value: i64) -> PartitionResult<usize> {
        Ok(self.bucket(value))
    }
    fn route_str(&self, _: &str) -> PartitionResult<usize> {
        Ok(0)
    }
}

// ── CustomerRouter ───────────────────────────────────────────────────────────

/// Fibonacci hash on an FK column → shard index (VV-PRJ-012).
pub struct CustomerRouter {
    shard_count: usize,
}

impl CustomerRouter {
    pub fn new(shard_count: usize) -> Self {
        Self { shard_count }
    }
}

/// Fibonacci hashing constant (same as AggregateRouter).
const FIBONACCI: u64 = 11_400_714_819_323_198_485;

impl SemanticRouter for CustomerRouter {
    fn route_i64(&self, value: i64) -> PartitionResult<usize> {
        if self.shard_count == 0 {
            return Err(PartitionError::ZeroShardCount);
        }
        let h = (value as u64).wrapping_mul(FIBONACCI);
        Ok(
            (h >> (64 - usize::BITS + usize::leading_zeros(self.shard_count))) as usize
                % self.shard_count,
        )
    }
    fn route_str(&self, value: &str) -> PartitionResult<usize> {
        let mut h: u64 = 0xcbf29ce484222325;
        for b in value.bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(0x100000001b3);
        }
        Ok((h % self.shard_count as u64) as usize)
    }
}

// ── ExpressionRouter ─────────────────────────────────────────────────────────

/// Evaluates a Rhai expression over a column value, result mod shard_count (VV-PRJ-020).
pub struct ExpressionRouter {
    formula: String,
    shard_count: usize,
    engine: Engine,
}

impl ExpressionRouter {
    pub fn new(formula: String, shard_count: usize) -> Self {
        let mut engine = Engine::new();
        engine.set_max_operations(1_000);
        Self {
            formula,
            shard_count,
            engine,
        }
    }

    pub fn route(&self, value: i64) -> PartitionResult<usize> {
        if self.shard_count == 0 {
            return Err(PartitionError::ZeroShardCount);
        }
        let mut scope = Scope::new();
        scope.push("value", value);
        let result: i64 = self
            .engine
            .eval_expression_with_scope::<i64>(&mut scope, &self.formula)
            .map_err(|e| PartitionError::RhaiEval(e.to_string()))?;
        Ok((result.unsigned_abs() as usize) % self.shard_count)
    }
}

impl SemanticRouter for ExpressionRouter {
    fn route_i64(&self, value: i64) -> PartitionResult<usize> {
        self.route(value)
    }
    fn route_str(&self, _: &str) -> PartitionResult<usize> {
        Ok(0)
    }
}

// ── ExplicitShardMap ─────────────────────────────────────────────────────────

/// Discrete value → shard index table (VV-PRJ-021).
pub struct ExplicitShardMap {
    map: HashMap<String, usize>,
}

impl ExplicitShardMap {
    pub fn new(map: HashMap<String, usize>) -> Self {
        Self { map }
    }
}

impl SemanticRouter for ExplicitShardMap {
    fn route_i64(&self, value: i64) -> PartitionResult<usize> {
        self.route_str(&value.to_string())
    }
    fn route_str(&self, value: &str) -> PartitionResult<usize> {
        self.map
            .get(value)
            .copied()
            .ok_or_else(|| PartitionError::UnknownKey(value.to_string()))
    }
}

// ── PartitionPruner ──────────────────────────────────────────────────────────

/// Skips partitions whose bucket cannot match a predicate (VV-PRJ-013).
///
/// Tracks which buckets have been written to so query time can skip all others.
pub struct PartitionPruner {
    /// Set of bucket indices that have received at least one row.
    written: std::collections::BTreeSet<usize>,
}

impl PartitionPruner {
    pub fn new() -> Self {
        Self {
            written: Default::default(),
        }
    }

    /// Record that a row was written to `bucket`.
    pub fn record_write(&mut self, bucket: usize) {
        self.written.insert(bucket);
    }

    /// Return only the `candidate_buckets` that have been written to.
    pub fn prune(&self, candidate_buckets: &[usize]) -> Vec<usize> {
        candidate_buckets
            .iter()
            .copied()
            .filter(|b| self.written.contains(b))
            .collect()
    }

    /// Number of buckets with data.
    pub fn written_count(&self) -> usize {
        self.written.len()
    }
}

impl Default for PartitionPruner {
    fn default() -> Self {
        Self::new()
    }
}
