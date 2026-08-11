//! `vectorvolt-http` — axum-based HTTP/REST gateway for VectorVolt.
//!
//! ## Endpoints
//!
//! | Method | Path | Operation |
//! |--------|------|-----------|
//! | POST   | `/tables/{table}/rows/{pk}` | Insert row |
//! | GET    | `/tables/{table}/rows/{pk}` | Get row by pk |
//! | DELETE | `/tables/{table}/rows/{pk}` | Delete row |
//! | GET    | `/tables/{table}/scan` | Range scan (`?col=&lo=&hi=`) |
//! | GET    | `/tables/{table}/secondary` | Secondary lookup (`?col=&val=`) |
//! | GET    | `/tables` | List all tables |
//! | GET    | `/tables/{table}/schema` | Describe table schema |
//! | POST   | `/tables/{table}` | Create table (body `{"columns":[...]}`) |
//! | DELETE | `/tables/{table}` | Drop table |

pub mod handlers;
pub mod server;
pub mod types;

pub use server::{HttpServer, HttpServerConfig};
