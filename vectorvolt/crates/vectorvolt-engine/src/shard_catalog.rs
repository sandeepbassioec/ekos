//! ShardCatalog — multi-table TxnManager registry (VV-ENG-006).
//!
//! [`ShardCatalog`] is the single in-process coordinator that owns one
//! [`TxnManager`] per table. It is the entry point the network server (S4)
//! will route requests through.
//!
//! Design constraints (per architecture decisions):
//! - Self-hosted — no ZooKeeper / etcd dependency.
//! - Each table has its own WAL directory and flush directory.
//! - Partition count per table is set at creation; supports
//!   [`AggregateRouter`] for co-located aggregates.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use vectorvolt_wal::{WalConfig, WalWriter};

use crate::aggregate_root::AggregateRouter;
use crate::error::{EngineError, EngineResult};
use crate::schema::TableSchema;
use crate::schema_registry::SchemaRegistry;
use vectorvolt_txn::TxnManager;

// ── TableEntry ────────────────────────────────────────────────────────────────

/// Everything the catalog owns for one table.
pub struct TableEntry {
    pub manager: Arc<TxnManager>,
    pub schema: TableSchema,
    pub partition_count: usize,
    pub aggregate_router: Option<AggregateRouter>,
}

// ── ShardCatalog ──────────────────────────────────────────────────────────────

/// In-process registry of [`TxnManager`] instances, one per table.
pub struct ShardCatalog {
    base_dir: PathBuf,
    tables: RwLock<HashMap<String, Arc<TableEntry>>>,
    registry: Option<SchemaRegistry>,
}

impl ShardCatalog {
    /// Create a catalog rooted at `base_dir` (no schema persistence).
    pub fn new(base_dir: impl Into<PathBuf>) -> Self {
        Self {
            base_dir: base_dir.into(),
            tables: RwLock::new(HashMap::new()),
            registry: None,
        }
    }

    /// Create a catalog with an embedded [`SchemaRegistry`] for automatic
    /// schema persistence on `create_table` / `drop_table`.
    ///
    /// `registry_dir` is the directory where `.schema` files are stored.
    /// It is created if it does not exist.
    pub fn with_registry(
        base_dir: impl Into<PathBuf>,
        registry_dir: impl Into<PathBuf>,
    ) -> EngineResult<Self> {
        let registry_dir = registry_dir.into();
        std::fs::create_dir_all(&registry_dir).map_err(EngineError::Io)?;
        Ok(Self {
            base_dir: base_dir.into(),
            tables: RwLock::new(HashMap::new()),
            registry: Some(SchemaRegistry::new(registry_dir)),
        })
    }

    /// Whether this catalog has an embedded schema registry.
    pub fn has_registry(&self) -> bool {
        self.registry.is_some()
    }

    /// Reload all tables whose schemas are persisted in the embedded registry.
    ///
    /// Returns the number of tables restored.  Returns `Ok(0)` when no registry
    /// is attached.  Schemas already loaded in-process are skipped.
    pub fn restore_all(
        &self,
        flush_threshold: usize,
        partition_count: usize,
    ) -> EngineResult<usize> {
        let Some(ref reg) = self.registry else {
            return Ok(0);
        };
        let all = reg.load_all()?;
        let mut count = 0;
        for (name, schema) in all {
            if self.contains(&name) {
                continue;
            }
            self.create_table_inner(&name, schema, flush_threshold, partition_count, false)?;
            count += 1;
        }
        Ok(count)
    }

    /// Create and register a new table.
    ///
    /// Creates `<base_dir>/<name>/wal/` and `<base_dir>/<name>/data/`.
    /// When an embedded registry is attached the schema is auto-persisted.
    /// Returns an error if the table already exists.
    pub fn create_table(
        &self,
        name: impl Into<String>,
        schema: TableSchema,
        flush_threshold: usize,
        partition_count: usize,
    ) -> EngineResult<Arc<TableEntry>> {
        let name = name.into();
        self.create_table_inner(&name, schema, flush_threshold, partition_count, true)
    }

    fn create_table_inner(
        &self,
        name: &str,
        schema: TableSchema,
        flush_threshold: usize,
        partition_count: usize,
        persist: bool,
    ) -> EngineResult<Arc<TableEntry>> {
        {
            let tables = self.tables.read().unwrap();
            if tables.contains_key(name) {
                return Err(EngineError::TableAlreadyExists(name.to_owned()));
            }
        }

        let table_dir = self.base_dir.join(name);
        let wal_dir = table_dir.join("wal");
        let data_dir = table_dir.join("data");
        std::fs::create_dir_all(&wal_dir).map_err(EngineError::Io)?;
        std::fs::create_dir_all(&data_dir).map_err(EngineError::Io)?;

        let wal = WalWriter::open(WalConfig::default().with_dir(&wal_dir).with_fsync(false))
            .map_err(EngineError::Wal)?;

        let col_names: Vec<String> = schema.names();
        let manager = Arc::new(
            TxnManager::new(wal, col_names, &data_dir, flush_threshold)
                .with_partition_count(partition_count),
        );

        if persist && let Some(ref reg) = self.registry {
            reg.save(name, &schema)?;
        }

        let entry = Arc::new(TableEntry {
            manager,
            schema,
            partition_count,
            aggregate_router: None,
        });

        self.tables
            .write()
            .unwrap()
            .insert(name.to_owned(), Arc::clone(&entry));
        Ok(entry)
    }

    /// Register an [`AggregateRouter`] for a table that is an aggregate root.
    ///
    /// The router is stored alongside the root table's entry.
    pub fn set_aggregate_router(
        &self,
        root_table: &str,
        router: AggregateRouter,
    ) -> EngineResult<()> {
        let mut tables = self.tables.write().unwrap();
        let entry = tables
            .get(root_table)
            .ok_or_else(|| EngineError::TableNotFound(root_table.to_owned()))?;

        let updated = Arc::new(TableEntry {
            manager: Arc::clone(&entry.manager),
            schema: entry.schema.clone(),
            partition_count: entry.partition_count,
            aggregate_router: Some(router),
        });
        tables.insert(root_table.to_owned(), updated);
        Ok(())
    }

    /// Return the [`TableEntry`] for `name`.
    pub fn get(&self, name: &str) -> EngineResult<Arc<TableEntry>> {
        self.tables
            .read()
            .unwrap()
            .get(name)
            .cloned()
            .ok_or_else(|| EngineError::TableNotFound(name.to_owned()))
    }

    /// Return the [`TxnManager`] for `name`.
    pub fn manager(&self, name: &str) -> EngineResult<Arc<TxnManager>> {
        self.get(name).map(|e| Arc::clone(&e.manager))
    }

    /// Drop a table, removing it from the in-process registry.
    ///
    /// When an embedded [`SchemaRegistry`] is attached the persisted schema
    /// is also removed.  Data directories on disk are NOT deleted.
    pub fn drop_table(&self, name: &str) -> EngineResult<()> {
        self.tables
            .write()
            .unwrap()
            .remove(name)
            .map(|_| ())
            .ok_or_else(|| EngineError::TableNotFound(name.to_owned()))?;
        if let Some(ref reg) = self.registry && reg.exists(name) {
            reg.remove(name)?;
        }
        Ok(())
    }

    /// List all registered table names.
    pub fn list_tables(&self) -> Vec<String> {
        let mut names: Vec<String> = self.tables.read().unwrap().keys().cloned().collect();
        names.sort();
        names
    }

    /// Return `true` if `name` is a registered table.
    pub fn contains(&self, name: &str) -> bool {
        self.tables.read().unwrap().contains_key(name)
    }

    /// Number of registered tables.
    pub fn table_count(&self) -> usize {
        self.tables.read().unwrap().len()
    }

    /// Base directory for this catalog.
    pub fn base_dir(&self) -> &Path {
        &self.base_dir
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    use crate::aggregate_root::AggregateRoot;
    use crate::schema::{ColumnDef, TableSchema};

    fn two_col_schema() -> TableSchema {
        TableSchema::new(vec![ColumnDef::u64("pk"), ColumnDef::u64("val")]).unwrap()
    }

    fn make_catalog() -> (ShardCatalog, TempDir) {
        let dir = TempDir::new().unwrap();
        (ShardCatalog::new(dir.path()), dir)
    }

    #[test]
    fn create_table_succeeds() {
        let (cat, _d) = make_catalog();
        cat.create_table("orders", two_col_schema(), 1000, 1)
            .unwrap();
        assert!(cat.contains("orders"));
    }

    #[test]
    fn create_duplicate_table_errors() {
        let (cat, _d) = make_catalog();
        cat.create_table("orders", two_col_schema(), 1000, 1)
            .unwrap();
        assert!(
            cat.create_table("orders", two_col_schema(), 1000, 1)
                .is_err()
        );
    }

    #[test]
    fn get_unknown_table_errors() {
        let (cat, _d) = make_catalog();
        assert!(cat.get("nonexistent").is_err());
    }

    #[test]
    fn drop_table_removes_from_registry() {
        let (cat, _d) = make_catalog();
        cat.create_table("t1", two_col_schema(), 1000, 1).unwrap();
        cat.drop_table("t1").unwrap();
        assert!(!cat.contains("t1"));
    }

    #[test]
    fn drop_nonexistent_table_errors() {
        let (cat, _d) = make_catalog();
        assert!(cat.drop_table("ghost").is_err());
    }

    #[test]
    fn list_tables_sorted() {
        let (cat, _d) = make_catalog();
        cat.create_table("zebra", two_col_schema(), 1000, 1)
            .unwrap();
        cat.create_table("alpha", two_col_schema(), 1000, 1)
            .unwrap();
        assert_eq!(cat.list_tables(), vec!["alpha", "zebra"]);
    }

    #[test]
    fn table_count_correct() {
        let (cat, _d) = make_catalog();
        assert_eq!(cat.table_count(), 0);
        cat.create_table("t1", two_col_schema(), 1000, 1).unwrap();
        cat.create_table("t2", two_col_schema(), 1000, 1).unwrap();
        assert_eq!(cat.table_count(), 2);
    }

    #[test]
    fn manager_insert_and_get_roundtrip() {
        let (cat, _d) = make_catalog();
        cat.create_table("users", two_col_schema(), 1000, 1)
            .unwrap();
        let mgr = cat.manager("users").unwrap();
        mgr.insert(1, vec![1, 100]).unwrap();
        assert_eq!(mgr.get(1).unwrap(), Some(vec![1, 100]));
    }

    #[test]
    fn set_aggregate_router_roundtrip() {
        let (cat, _d) = make_catalog();
        cat.create_table("orders", two_col_schema(), 1000, 2)
            .unwrap();
        let root = AggregateRoot::new("orders").with_member("order_lines", 1);
        let router = AggregateRouter::new(root, 2);
        cat.set_aggregate_router("orders", router).unwrap();
        let entry = cat.get("orders").unwrap();
        assert!(entry.aggregate_router.is_some());
    }

    #[test]
    fn set_aggregate_router_unknown_table_errors() {
        let (cat, _d) = make_catalog();
        let root = AggregateRoot::new("orders");
        let router = AggregateRouter::new(root, 1);
        assert!(cat.set_aggregate_router("orders", router).is_err());
    }

    #[test]
    fn partition_count_stored_in_entry() {
        let (cat, _d) = make_catalog();
        cat.create_table("t", two_col_schema(), 1000, 4).unwrap();
        assert_eq!(cat.get("t").unwrap().partition_count, 4);
    }

    #[test]
    fn base_dir_accessible() {
        let dir = TempDir::new().unwrap();
        let cat = ShardCatalog::new(dir.path());
        assert_eq!(cat.base_dir(), dir.path());
    }
}
