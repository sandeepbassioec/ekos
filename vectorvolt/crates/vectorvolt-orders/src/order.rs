//! Order domain types.

use serde::{Deserialize, Serialize};

/// Current lifecycle state of an order.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OrderStatus {
    Pending,
    Confirmed,
    Shipped,
    Delivered,
    Cancelled,
}

/// A single line item in an order.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderLine {
    pub sku: String,
    pub qty: u32,
    pub unit_price_cents: i64,
}

impl OrderLine {
    /// Line total in cents.
    pub fn total_cents(&self) -> i64 {
        self.unit_price_cents * self.qty as i64
    }
}

/// A customer order with one or more line items.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: u64,
    pub customer_id: u64,
    pub lines: Vec<OrderLine>,
    pub status: OrderStatus,
    pub created_at_ns: u64,
    pub total_cents: i64,
}
