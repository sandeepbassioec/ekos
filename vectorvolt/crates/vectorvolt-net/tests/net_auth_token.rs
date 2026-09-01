//! Tests for Layer 311: Secret, AuthConfig, AuthState, tokens (VV-NET-311).

use vectorvolt_net::auth::{AuthConfig, AuthError, AuthState, NONCE_LEN, Scope, Secret};

fn secret() -> Secret {
    Secret::new(b"supersecretkey1234".to_vec()).unwrap()
}

#[test]
fn secret_new_min_len_ok() {
    let s = Secret::new(vec![0u8; Secret::MIN_LEN]);
    assert!(s.is_ok());
}

#[test]
fn secret_new_too_short_err() {
    let s = Secret::new(vec![0u8; 4]);
    assert!(matches!(s, Err(AuthError::SecretTooShort { .. })));
}

#[test]
fn secret_min_len_is_16() {
    assert_eq!(Secret::MIN_LEN, 16);
}

#[test]
fn auth_config_disabled_not_enabled() {
    assert!(!AuthConfig::disabled().is_enabled());
}

#[test]
fn auth_config_with_key_enabled() {
    let cfg = AuthConfig::disabled().with_key("k1", secret());
    assert!(cfg.is_enabled());
}

#[test]
fn auth_config_with_scoped_key_enabled() {
    let cfg = AuthConfig::disabled().with_scoped_key("k1", secret(), Scope::Admin);
    assert!(cfg.is_enabled());
}

#[test]
fn scope_readonly_default() {
    let s: Scope = Default::default();
    assert_eq!(s, Scope::ReadOnly);
}

#[test]
fn scope_admin_allows_all() {
    assert!(Scope::Admin.allows(Scope::ReadOnly));
    assert!(Scope::Admin.allows(Scope::ReadWrite));
    assert!(Scope::Admin.allows(Scope::Admin));
}

#[test]
fn scope_readonly_only_allows_readonly() {
    assert!(Scope::ReadOnly.allows(Scope::ReadOnly));
    assert!(!Scope::ReadOnly.allows(Scope::ReadWrite));
}

#[test]
fn scope_as_str_readonly() {
    assert_eq!(Scope::ReadOnly.as_str(), "read-only");
}

#[test]
fn scope_as_str_admin() {
    assert_eq!(Scope::Admin.as_str(), "admin");
}

#[test]
fn nonce_len_is_32() {
    assert_eq!(NONCE_LEN, 32);
}

#[test]
fn auth_state_not_authenticated_initially() {
    let cfg = AuthConfig::disabled().with_key("k", secret());
    let state = AuthState::new(&cfg);
    assert!(!state.is_authenticated());
}
