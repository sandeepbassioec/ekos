//! Aggregate Root Partitioning (VV-ENG-023).
//!
//! Routes every member-table row to the **same partition** as its root aggregate
//! row, so a complete aggregate (Order + OrderLine + Payment) always lives on
//! one shard. This enables single-shard ACID without Saga or 2PC.
//!
//! ## How it works
//!
//! 1. One table is designated the **root** (e.g. `orders`). Its rows are
//!    partitioned normally via [`select_partition`].
//! 2. Every **member** table (e.g. `order_lines`, `payments`) declares which
//!    column index holds the foreign key back to the root.
//! 3. When inserting a member row, call [`AggregateRouter::partition_for_member`]
//!    with the FK value — it returns the same partition index the root row uses.
//!
//! ## Example
//!
//! ```
//! use vectorvolt_engine::aggregate_root::{AggregateRoot, AggregateRouter};
//!
//! let root = AggregateRoot::new("orders")
//!     .with_member("order_lines", 1)   // column[1] = order_id FK
//!     .with_member("payments", 1);     // column[1] = order_id FK
//!
//! let router = AggregateRouter::new(root, 4);
//!
//! let order_pk = 42u64;
//! let line_fk  = 42u64;   // same value
//!
//! assert_eq!(
//!     router.partition_for_root(order_pk),
//!     router.partition_for_member("order_lines", line_fk).unwrap(),
//! );
//! ```

use std::collections::HashMap;

use vectorvolt_txn::select_partition;

// ── AggregateRoot definition ──────────────────────────────────────────────────

/// Declares an aggregate root and its member tables.
#[derive(Debug, Clone, PartialEq)]
pub struct AggregateRoot {
    /// Name of the root table (e.g. `"orders"`).
    pub root_table: String,
    /// member table name → column index that holds the FK to the root PK.
    pub members: HashMap<String, usize>,
}

impl AggregateRoot {
    pub fn new(root_table: impl Into<String>) -> Self {
        Self {
            root_table: root_table.into(),
            members: HashMap::new(),
        }
    }

    /// Register `table` as a member. `fk_col` is the column index whose value
    /// equals the root PK.
    pub fn with_member(mut self, table: impl Into<String>, fk_col: usize) -> Self {
        self.members.insert(table.into(), fk_col);
        self
    }

    /// Return the FK column index for `table`, or `None` if it is not a member.
    pub fn fk_col(&self, table: &str) -> Option<usize> {
        self.members.get(table).copied()
    }

    /// Return `true` if `table` is either the root or a registered member.
    pub fn contains_table(&self, table: &str) -> bool {
        table == self.root_table || self.members.contains_key(table)
    }
}

// ── AggregateRouter ───────────────────────────────────────────────────────────

/// Routes rows to partitions, keeping an aggregate on a single shard.
#[derive(Debug, Clone)]
pub struct AggregateRouter {
    pub root: AggregateRoot,
    pub partition_count: usize,
}

impl AggregateRouter {
    pub fn new(root: AggregateRoot, partition_count: usize) -> Self {
        Self {
            root,
            partition_count: partition_count.max(1),
        }
    }

    /// Partition for a root row. Uses standard Fibonacci hash on `root_pk`.
    #[inline]
    pub fn partition_for_root(&self, root_pk: u64) -> usize {
        select_partition(root_pk, self.partition_count)
    }

    /// Partition for a member row identified by its FK value (which equals
    /// the root PK). Returns `None` if `table` is not a registered member.
    #[inline]
    pub fn partition_for_member(&self, table: &str, fk_value: u64) -> Option<usize> {
        if self.root.fk_col(table).is_some() {
            Some(select_partition(fk_value, self.partition_count))
        } else {
            None
        }
    }

    /// Route any row in the aggregate: root or member.
    ///
    /// Returns `None` if `table` is not part of this aggregate.
    pub fn partition_for_table(&self, table: &str, pk_or_fk: u64) -> Option<usize> {
        if table == self.root.root_table {
            Some(self.partition_for_root(pk_or_fk))
        } else {
            self.partition_for_member(table, pk_or_fk)
        }
    }

    /// Verify that two rows (root + member) will land on the same partition.
    ///
    /// Returns `true` when `member_fk == root_pk` AND both hash identically
    /// (always true since they use the same value).
    pub fn same_shard(&self, root_pk: u64, member_table: &str, member_fk: u64) -> bool {
        match self.partition_for_member(member_table, member_fk) {
            Some(mp) => self.partition_for_root(root_pk) == mp,
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_router(n: usize) -> AggregateRouter {
        let root = AggregateRoot::new("orders")
            .with_member("order_lines", 1)
            .with_member("payments", 2);
        AggregateRouter::new(root, n)
    }

    #[test]
    fn root_and_member_same_partition_when_fk_equals_pk() {
        let r = make_router(4);
        for pk in 0u64..50 {
            assert_eq!(
                r.partition_for_root(pk),
                r.partition_for_member("order_lines", pk).unwrap()
            );
        }
    }

    #[test]
    fn unknown_member_returns_none() {
        let r = make_router(4);
        assert!(r.partition_for_member("invoices", 1).is_none());
    }

    #[test]
    fn partition_for_table_root() {
        let r = make_router(4);
        assert_eq!(
            r.partition_for_table("orders", 7),
            Some(r.partition_for_root(7))
        );
    }

    #[test]
    fn partition_for_table_member() {
        let r = make_router(4);
        assert_eq!(
            r.partition_for_table("payments", 7),
            r.partition_for_member("payments", 7)
        );
    }

    #[test]
    fn partition_for_table_unknown_returns_none() {
        let r = make_router(4);
        assert!(r.partition_for_table("unknown_table", 1).is_none());
    }

    #[test]
    fn same_shard_true_when_fk_equals_pk() {
        let r = make_router(4);
        for pk in 0u64..50 {
            assert!(r.same_shard(pk, "order_lines", pk));
        }
    }

    #[test]
    fn same_shard_false_for_different_fk() {
        let r = make_router(8);
        // Find a pk and fk that route to different partitions.
        let (pk, fk) = (0u64..1000)
            .flat_map(|p| (0u64..1000).map(move |f| (p, f)))
            .find(|(p, f)| select_partition(*p, 8) != select_partition(*f, 8))
            .unwrap();
        assert!(!r.same_shard(pk, "order_lines", fk));
    }

    #[test]
    fn contains_table_root() {
        let r = make_router(4);
        assert!(r.root.contains_table("orders"));
    }

    #[test]
    fn contains_table_member() {
        let r = make_router(4);
        assert!(r.root.contains_table("payments"));
    }

    #[test]
    fn contains_table_false_for_unknown() {
        let r = make_router(4);
        assert!(!r.root.contains_table("invoices"));
    }

    #[test]
    fn fk_col_returns_correct_index() {
        let r = make_router(4);
        assert_eq!(r.root.fk_col("order_lines"), Some(1));
        assert_eq!(r.root.fk_col("payments"), Some(2));
        assert_eq!(r.root.fk_col("orders"), None);
    }

    #[test]
    fn single_partition_always_zero() {
        let r = make_router(1);
        for pk in 0u64..20 {
            assert_eq!(r.partition_for_root(pk), 0);
            assert_eq!(r.partition_for_member("payments", pk), Some(0));
        }
    }

    #[test]
    fn with_member_builder_is_fluent() {
        let root = AggregateRoot::new("orders")
            .with_member("lines", 1)
            .with_member("payments", 2)
            .with_member("refunds", 3);
        assert_eq!(root.members.len(), 3);
    }
}
