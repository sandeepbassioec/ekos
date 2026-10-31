//! Persistent order store with state-machine transitions.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};
use std::time::{SystemTime, UNIX_EPOCH};

use parking_lot::{Mutex, RwLock};

use crate::error::{OrderError, OrderResult};
use crate::order::{Order, OrderLine, OrderStatus};
use crate::persist;

/// Persistent order store.
pub struct OrderStore {
    path: PathBuf,
    store: Arc<RwLock<std::collections::HashMap<u64, Order>>>,
    next_id: AtomicU64,
    /// Serialises `persist`: writers share one `.tmp`, and the snapshot is
    /// taken under this lock so an older state never overwrites a newer one.
    persist_lock: Mutex<()>,
}

impl OrderStore {
    /// Open (or create) the order store at `dir`.
    pub fn open(dir: &Path) -> OrderResult<Self> {
        std::fs::create_dir_all(dir)?;
        let path = dir.join("orders.json");
        let store: std::collections::HashMap<u64, Order> = match persist::read_committed(&path)? {
            Some(raw) => serde_json::from_slice(&raw)?,
            None => std::collections::HashMap::new(),
        };
        let next_id = store.keys().copied().max().unwrap_or(0) + 1;
        Ok(Self {
            path,
            store: Arc::new(RwLock::new(store)),
            next_id: AtomicU64::new(next_id),
            persist_lock: Mutex::new(()),
        })
    }

    fn persist(&self) -> OrderResult<()> {
        let _guard = self.persist_lock.lock();
        let raw = serde_json::to_vec(&*self.store.read())?;
        persist::write_atomic(&self.path, &raw)?;
        Ok(())
    }

    fn now_ns() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64
    }

    /// Create a new order.  Computes `total_cents` from the lines.
    pub fn create(&self, customer_id: u64, lines: Vec<OrderLine>) -> OrderResult<u64> {
        let id = self.next_id.fetch_add(1, AtomicOrdering::Relaxed);
        let total_cents = lines.iter().map(|l| l.total_cents()).sum();
        let order = Order {
            id,
            customer_id,
            lines,
            status: OrderStatus::Pending,
            created_at_ns: Self::now_ns(),
            total_cents,
        };
        self.store.write().insert(id, order);
        self.persist()?;
        Ok(id)
    }

    /// Fetch an order by id.
    pub fn get(&self, id: u64) -> OrderResult<Option<Order>> {
        Ok(self.store.read().get(&id).cloned())
    }

    fn transition(&self, id: u64, from: &[OrderStatus], to: OrderStatus) -> OrderResult<()> {
        let mut store = self.store.write();
        let order = store.get_mut(&id).ok_or(OrderError::NotFound(id))?;
        if !from.contains(&order.status) {
            return Err(OrderError::InvalidTransition(format!("{:?}", order.status)));
        }
        order.status = to;
        drop(store);
        self.persist()
    }

    pub fn confirm(&self, id: u64) -> OrderResult<()> {
        self.transition(id, &[OrderStatus::Pending], OrderStatus::Confirmed)
    }

    pub fn ship(&self, id: u64) -> OrderResult<()> {
        self.transition(id, &[OrderStatus::Confirmed], OrderStatus::Shipped)
    }

    pub fn deliver(&self, id: u64) -> OrderResult<()> {
        self.transition(id, &[OrderStatus::Shipped], OrderStatus::Delivered)
    }

    pub fn cancel(&self, id: u64) -> OrderResult<()> {
        self.transition(
            id,
            &[OrderStatus::Pending, OrderStatus::Confirmed],
            OrderStatus::Cancelled,
        )
    }

    /// Return all orders for `customer_id`.
    pub fn orders_for_customer(&self, customer_id: u64) -> OrderResult<Vec<Order>> {
        Ok(self
            .store
            .read()
            .values()
            .filter(|o| o.customer_id == customer_id)
            .cloned()
            .collect())
    }

    /// Sum of `total_cents` for all `Delivered` orders.
    pub fn total_revenue_cents(&self) -> OrderResult<i64> {
        Ok(self
            .store
            .read()
            .values()
            .filter(|o| o.status == OrderStatus::Delivered)
            .map(|o| o.total_cents)
            .sum())
    }
}
