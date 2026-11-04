//! Semantic Partitioning + Materialized Read Models + External Cache Tier (S22).
//!
//! VV-PRJ-010..021, VV-CFG-040..041

pub mod cache;
pub mod config;
pub mod partition;
pub mod read_model;
pub mod reducer;

pub use cache::ReadModelSnapshot;
pub use cache::{InProcessReadCache, ReadCache, RedisCacheError, RedisReadCache};
pub use config::{
    CacheConfig, InProcessCacheConfig, PartitionConfig, ReadModelConfig, RedisCacheConfig,
    ReducerKind,
};
pub use partition::{
    CustomerRouter, DateRangeRouter, ExplicitShardMap, ExpressionRouter, PartitionPruner,
    PartitionStrategy, SemanticRouter,
};
pub use read_model::ReadModelRegistry;
pub use reducer::{BuiltinReducer, HttpCallbackReducer, RhaiReducer, SemanticReducer};
