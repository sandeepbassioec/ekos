//! Tests for Layer 310.
use vectorvolt_net::auth::{AuthConfig, AuthError, AuthState, Scope, Secret};

fn secret() -> Secret {
    Secret::new(vec![0u8; 32]).unwrap()
}
