use std::sync::Arc;
use tempfile::TempDir;
use vectorvolt_orders::{OrderError, OrderLine, OrderStatus, OrderStore};

fn setup() -> (OrderStore, TempDir) {
    let dir = TempDir::new().unwrap();
    let store = OrderStore::open(dir.path()).unwrap();
    (store, dir)
}

fn line(sku: &str, qty: u32, price: i64) -> OrderLine {
    OrderLine { sku: sku.into(), qty, unit_price_cents: price }
}

#[test]
fn create_order_computes_total() {
    let (store, _dir) = setup();
    let id = store.create(1, vec![line("A", 2, 500), line("B", 1, 300)]).unwrap();
    let order = store.get(id).unwrap().unwrap();
    assert_eq!(order.total_cents, 1300); // 2*500 + 1*300
}

#[test]
fn get_order() {
    let (store, _dir) = setup();
    let id = store.create(1, vec![line("X", 1, 100)]).unwrap();
    let order = store.get(id).unwrap();
    assert!(order.is_some());
}

#[test]
fn confirm_transitions_to_confirmed() {
    let (store, _dir) = setup();
    let id = store.create(1, vec![line("A", 1, 100)]).unwrap();
    store.confirm(id).unwrap();
    let order = store.get(id).unwrap().unwrap();
    assert_eq!(order.status, OrderStatus::Confirmed);
}

#[test]
fn ship_transitions_to_shipped() {
    let (store, _dir) = setup();
    let id = store.create(1, vec![line("A", 1, 100)]).unwrap();
    store.confirm(id).unwrap();
    store.ship(id).unwrap();
    let order = store.get(id).unwrap().unwrap();
    assert_eq!(order.status, OrderStatus::Shipped);
}

#[test]
fn deliver_transitions_to_delivered() {
    let (store, _dir) = setup();
    let id = store.create(1, vec![line("A", 1, 100)]).unwrap();
    store.confirm(id).unwrap();
    store.ship(id).unwrap();
    store.deliver(id).unwrap();
    let order = store.get(id).unwrap().unwrap();
    assert_eq!(order.status, OrderStatus::Delivered);
}

#[test]
fn cancel_from_pending_ok() {
    let (store, _dir) = setup();
    let id = store.create(1, vec![line("A", 1, 100)]).unwrap();
    store.cancel(id).unwrap();
    let order = store.get(id).unwrap().unwrap();
    assert_eq!(order.status, OrderStatus::Cancelled);
}

#[test]
fn cancel_from_shipped_returns_err() {
    let (store, _dir) = setup();
    let id = store.create(1, vec![line("A", 1, 100)]).unwrap();
    store.confirm(id).unwrap();
    store.ship(id).unwrap();
    let result = store.cancel(id);
    assert!(matches!(result, Err(OrderError::InvalidTransition(_))));
}

#[test]
fn orders_for_customer_returns_correct() {
    let (store, _dir) = setup();
    store.create(10, vec![line("A", 1, 100)]).unwrap();
    store.create(10, vec![line("B", 2, 200)]).unwrap();
    store.create(20, vec![line("C", 1, 50)]).unwrap();
    let orders = store.orders_for_customer(10).unwrap();
    assert_eq!(orders.len(), 2);
}

#[test]
fn total_revenue_cents_sums_delivered_orders() {
    let (store, _dir) = setup();
    let id1 = store.create(1, vec![line("A", 1, 1000)]).unwrap();
    let id2 = store.create(1, vec![line("B", 1, 2000)]).unwrap();
    for id in [id1, id2] {
        store.confirm(id).unwrap();
        store.ship(id).unwrap();
        store.deliver(id).unwrap();
    }
    assert_eq!(store.total_revenue_cents().unwrap(), 3000);
}

#[test]
fn multiple_order_lines() {
    let (store, _dir) = setup();
    let lines = vec![line("X", 1, 100), line("Y", 3, 200), line("Z", 2, 50)];
    let id = store.create(1, lines).unwrap();
    let order = store.get(id).unwrap().unwrap();
    assert_eq!(order.lines.len(), 3);
    assert_eq!(order.total_cents, 100 + 600 + 100);
}

#[test]
fn invalid_status_transition_returns_err() {
    let (store, _dir) = setup();
    let id = store.create(1, vec![line("A", 1, 100)]).unwrap();
    // Can't ship a Pending order (must confirm first).
    let result = store.ship(id);
    assert!(matches!(result, Err(OrderError::InvalidTransition(_))));
}

#[test]
fn fifty_concurrent_order_creates() {
    let dir = TempDir::new().unwrap();
    let store = Arc::new(OrderStore::open(dir.path()).unwrap());
    let mut handles = Vec::new();
    for _ in 0..50 {
        let s = Arc::clone(&store);
        handles.push(std::thread::spawn(move || {
            s.create(1, vec![line("A", 1, 100)]).unwrap()
        }));
    }
    let mut ids: Vec<u64> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), 50);
}

#[test]
fn get_nonexistent_returns_none() {
    let (store, _dir) = setup();
    assert!(store.get(9999).unwrap().is_none());
}
