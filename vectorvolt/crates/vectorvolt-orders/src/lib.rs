//! `vectorvolt-orders` — Order Management state-machine reference domain.

pub mod error;
pub mod order;
mod persist;
pub mod store;

pub use error::{OrderError, OrderResult};
pub use order::{Order, OrderLine, OrderStatus};
pub use store::OrderStore;
