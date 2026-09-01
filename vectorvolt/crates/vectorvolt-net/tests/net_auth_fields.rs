//! Tests for Layer 310.
use vectorvolt_net::auth::{
    AuthConfig, AuthError, AuthState, NONCE_LEN, RESPONSE_LEN, Scope, Secret, client_response,
    compute_response,
};

fn secret() -> Secret {
    Secret::new(vec![0u8; 32]).unwrap()
}

#[test]
fn nonce_len_is_32() {
    assert_eq!(NONCE_LEN, 32);
}
#[test]
fn response_len_is_32() {
    assert_eq!(RESPONSE_LEN, 32);
}
#[test]
fn scope_default_readonly() {
    assert_eq!(Scope::default(), Scope::ReadOnly);
}
#[test]
fn scope_allows_self() {
    assert!(Scope::Admin.allows(Scope::Admin));
}
#[test]
fn scope_admin_allows_readwrite() {
    assert!(Scope::Admin.allows(Scope::ReadWrite));
}
#[test]
fn scope_readonly_not_admin() {
    assert!(!Scope::ReadOnly.allows(Scope::Admin));
}
#[test]
fn secret_too_short_errors() {
    assert!(Secret::new(vec![0u8; 5]).is_err());
}
#[test]
fn secret_min_len_ok() {
    assert!(Secret::new(vec![0u8; Secret::MIN_LEN]).is_ok());
}
#[test]
fn secret_debug_redacted() {
    let s = secret();
    assert!(format!("{s:?}").contains("redacted"));
}
#[test]
fn auth_config_disabled() {
    let c = AuthConfig::disabled();
    assert!(!c.is_enabled());
}
#[test]
fn auth_config_with_key_enabled() {
    let c = AuthConfig::disabled().with_key("k", secret());
    assert!(c.is_enabled());
}
#[test]
fn compute_response_deterministic() {
    let s = secret();
    let n = [1u8; NONCE_LEN];
    assert_eq!(compute_response(&s, &n), compute_response(&s, &n));
}
#[test]
fn client_response_matches_compute() {
    let s = secret();
    let n = [2u8; NONCE_LEN];
    assert_eq!(
        compute_response(&s, &n),
        client_response(&vec![0u8; 32], &n)
    );
}
