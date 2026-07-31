//! Challenge–response authentication (VV-SEC-001/002/003).
//!
//! ## Why not a bearer token
//!
//! The obvious design is "client sends an API key, server compares it". Over
//! plaintext TCP that is close to worthless: anyone who can read one packet has
//! the key forever, and every subsequent connection hands it over again.
//!
//! So the shared secret **never crosses the wire**. The server sends a random
//! nonce; the client proves it knows the secret by returning
//! `SHA3-256(secret ‖ nonce)`. An observer sees a nonce and a hash, neither of
//! which is reusable — the next connection gets a different nonce.
//!
//! This is strictly better than a bearer token on an untrusted network, and it is
//! **still not a substitute for TLS**:
//!
//! - the session that follows is plaintext, so data is readable in transit;
//! - there is no server authentication, so a man in the middle can relay the
//!   handshake and then own the session.
//!
//! Auth answers "who is this", TLS answers "is anyone listening". They are
//! different problems and this module only solves the first. Saying otherwise
//! would be worse than shipping nothing, because it invites the deployment it
//! cannot protect.
//!
//! ## Why plain SHA3 rather than HMAC
//!
//! `H(secret ‖ message)` is unsafe with SHA-2 because Merkle–Damgård constructions
//! admit length-extension: knowing `H(secret ‖ m)` lets an attacker compute
//! `H(secret ‖ m ‖ padding ‖ m2)` without the secret. SHA-3 is a sponge and has no
//! such property, so the prefix construction is sound and HMAC's extra block is
//! unnecessary. That is a real cryptographic distinction and not a shortcut — with
//! SHA-256 here, HMAC would be mandatory.

use std::collections::HashMap;

use sha3::{Digest, Sha3_256};
use subtle::ConstantTimeEq;

/// Nonce length. 32 bytes makes a repeat cryptographically implausible, which is
/// what stops a captured response being replayed against a future challenge.
pub const NONCE_LEN: usize = 32;

/// Response length — a SHA3-256 digest.
pub const RESPONSE_LEN: usize = 32;

/// Failed attempts before a connection is dropped.
///
/// Online brute force is only viable if a client gets unlimited guesses on one
/// socket. Three, then the connection closes and the attacker pays a TCP handshake
/// per attempt.
pub const MAX_AUTH_ATTEMPTS: u32 = 3;

/// What an authenticated key is allowed to do (VV-SEC-004).
///
/// A total order rather than a permission set: each level contains the one below
/// it. Independent flags would be more expressive and would also make it possible
/// to configure "may compact but may not read", which is not a coherent thing to
/// want and would be a footgun rather than a feature.
///
/// The split is by BLAST RADIUS, not by verb:
///
/// | Scope | May do | Why the line is here |
/// |---|---|---|
/// | `ReadOnly` | Get, RangeScan, Stats | Cannot change or destroy anything |
/// | `ReadWrite` | + Insert, Delete | Changes rows; a mistake is recoverable per row |
/// | `Admin` | + Flush, Compact | Rewrites and deletes FILES; a mistake is not row-scoped |
///
/// Compact is the reason `Admin` exists. It rewrites every partition in a job and
/// deletes the originals — a single call from a misconfigured client can rewrite a
/// whole table. That is not something an ordinary writer credential should be able
/// to trigger.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum Scope {
    /// Read data. The safe default for a new credential.
    #[default]
    ReadOnly,
    /// Read and modify rows.
    ReadWrite,
    /// Everything, including operations that rewrite or delete files.
    Admin,
}

impl Scope {
    /// True if this scope permits everything `required` needs.
    pub fn allows(self, required: Scope) -> bool {
        self >= required
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Scope::ReadOnly => "read-only",
            Scope::ReadWrite => "read-write",
            Scope::Admin => "admin",
        }
    }
}

/// A shared secret.
///
/// `Debug` is implemented manually to print nothing. Secrets end up in logs
/// through a derived `Debug` on some enclosing struct far more often than through
/// anyone printing them deliberately.
#[derive(Clone, PartialEq, Eq)]
pub struct Secret(Vec<u8>);

impl Secret {
    /// Minimum secret length, in bytes.
    ///
    /// 16 bytes of real entropy is beyond offline brute force. This does not
    /// measure entropy — it cannot — so a 16-character English password passes
    /// while being far weaker than the length implies. It is a floor against
    /// obvious mistakes, not a strength check, and the docs say so rather than
    /// implying the server validated anything meaningful.
    pub const MIN_LEN: usize = 16;

    pub fn new(bytes: impl Into<Vec<u8>>) -> Result<Self, AuthError> {
        let bytes = bytes.into();
        if bytes.len() < Self::MIN_LEN {
            return Err(AuthError::SecretTooShort {
                len: bytes.len(),
                min: Self::MIN_LEN,
            });
        }
        Ok(Secret(bytes))
    }

    fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl std::fmt::Debug for Secret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Secret(<redacted, {} bytes>)", self.0.len())
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum AuthError {
    #[error("secret is {len} bytes; at least {min} are required")]
    SecretTooShort { len: usize, min: usize },
    #[error("unknown key id")]
    UnknownKeyId,
    #[error("response does not match the challenge")]
    BadResponse,
    #[error("no challenge has been issued on this connection")]
    NoChallenge,
    #[error("too many failed attempts")]
    TooManyAttempts,
    #[error("operation requires {required} access; this key has {held}")]
    InsufficientScope {
        required: &'static str,
        held: &'static str,
    },
}

/// Who may connect.
///
/// Empty means authentication is **disabled** — every connection is treated as
/// authorised. That is the right default for an embedded or single-process
/// deployment and the wrong one for anything reachable, so
/// [`AuthConfig::is_enabled`] exists for a server to log which mode it is in
/// rather than leaving it implicit.
#[derive(Debug, Clone, Default)]
pub struct AuthConfig {
    credentials: HashMap<String, (Secret, Scope)>,
}

impl AuthConfig {
    /// No credentials: authentication disabled.
    pub fn disabled() -> Self {
        Self::default()
    }

    /// Add a key. Defaults to [`Scope::ReadOnly`] — the least-privilege default,
    /// so forgetting to specify a scope under-grants rather than over-grants.
    pub fn with_key(mut self, key_id: impl Into<String>, secret: Secret) -> Self {
        self.credentials
            .insert(key_id.into(), (secret, Scope::ReadOnly));
        self
    }

    /// Add a key with an explicit scope.
    pub fn with_scoped_key(
        mut self,
        key_id: impl Into<String>,
        secret: Secret,
        scope: Scope,
    ) -> Self {
        self.credentials.insert(key_id.into(), (secret, scope));
        self
    }

    pub fn is_enabled(&self) -> bool {
        !self.credentials.is_empty()
    }

    fn entry_for(&self, key_id: &str) -> Option<&(Secret, Scope)> {
        self.credentials.get(key_id)
    }
}

/// The expected response to `nonce` for `secret`.
pub fn compute_response(secret: &Secret, nonce: &[u8]) -> [u8; RESPONSE_LEN] {
    let mut h = Sha3_256::new();
    h.update(secret.as_bytes());
    h.update(nonce);
    let out = h.finalize();
    let mut resp = [0u8; RESPONSE_LEN];
    resp.copy_from_slice(&out);
    resp
}

/// Compute a response as a client, from raw secret bytes.
///
/// Separate from [`compute_response`] so a client is not forced through
/// [`Secret::new`]'s length check — a server rejecting a short secret at
/// configuration time is useful; a client refusing to attempt a login the
/// operator configured is not, and would just produce a confusing local error
/// instead of a clear server-side rejection.
pub fn client_response(secret: &[u8], nonce: &[u8]) -> [u8; RESPONSE_LEN] {
    let mut h = Sha3_256::new();
    h.update(secret);
    h.update(nonce);
    let out = h.finalize();
    let mut resp = [0u8; RESPONSE_LEN];
    resp.copy_from_slice(&out);
    resp
}

/// Per-connection authentication state.
#[derive(Debug)]
pub struct AuthState {
    /// The nonce issued to this connection, if any.
    challenge: Option<[u8; NONCE_LEN]>,
    authenticated: bool,
    attempts: u32,
    key_id: Option<String>,
    scope: Scope,
}

impl AuthState {
    /// A connection that has not yet authenticated.
    pub fn new(config: &AuthConfig) -> Self {
        AuthState {
            challenge: None,
            // With no credentials configured, every connection is authorised from
            // the start. The alternative — refusing everything — would make an
            // unconfigured server silently useless rather than openly open.
            authenticated: !config.is_enabled(),
            attempts: 0,
            key_id: None,
            // With auth disabled every connection is already authorised, so it
            // gets full access — anything less would break the embedded case while
            // providing no security, since there is nobody to distinguish.
            scope: if config.is_enabled() {
                Scope::ReadOnly
            } else {
                Scope::Admin
            },
        }
    }

    pub fn is_authenticated(&self) -> bool {
        self.authenticated
    }

    /// The authenticated key id, for audit logging.
    pub fn key_id(&self) -> Option<&str> {
        self.key_id.as_deref()
    }

    /// What this connection is allowed to do.
    pub fn scope(&self) -> Scope {
        self.scope
    }

    /// Check the connection may perform an operation needing `required`.
    ///
    /// The error names both what was needed and what is held. An authorisation
    /// failure is almost always a misconfiguration rather than an attack, and
    /// "forbidden" with no detail turns a one-minute fix into an afternoon. It
    /// leaks only the caller's OWN scope, which they can determine anyway by
    /// trying operations.
    pub fn require(&self, required: Scope) -> Result<(), AuthError> {
        if self.scope.allows(required) {
            Ok(())
        } else {
            Err(AuthError::InsufficientScope {
                required: required.as_str(),
                held: self.scope.as_str(),
            })
        }
    }

    /// Issue a fresh challenge, replacing any previous one.
    ///
    /// Replacing matters: a client that requests a second challenge invalidates
    /// the first, so exactly one nonce is ever live per connection and a response
    /// cannot be prepared against a nonce that has since been superseded.
    pub fn new_challenge(&mut self) -> [u8; NONCE_LEN] {
        let mut nonce = [0u8; NONCE_LEN];
        // OS entropy. A predictable nonce defeats the whole scheme — an attacker
        // who can guess the next one precomputes the response offline from a
        // single captured pair. This is why a counter or a timestamp is not
        // acceptable here even though both are unique.
        getrandom::fill(&mut nonce).expect("OS entropy source unavailable");
        self.challenge = Some(nonce);
        nonce
    }

    /// Verify a response.
    ///
    /// The comparison is constant-time. A byte-by-byte early-exit compare leaks
    /// how many leading bytes matched through timing, which turns a 2^256 search
    /// into 32 sequential 256-way searches.
    ///
    /// An unknown key id still performs a comparison against a dummy secret, so
    /// "no such key" and "wrong secret" take the same time and cannot be
    /// distinguished by an attacker probing for valid key ids.
    pub fn verify(
        &mut self,
        config: &AuthConfig,
        key_id: &str,
        response: &[u8],
    ) -> Result<(), AuthError> {
        if self.attempts >= MAX_AUTH_ATTEMPTS {
            return Err(AuthError::TooManyAttempts);
        }
        self.attempts += 1;

        let Some(nonce) = self.challenge else {
            return Err(AuthError::NoChallenge);
        };

        let known = config.entry_for(key_id);
        // A fixed dummy so the unknown-key path does the same work.
        let dummy = (Secret(vec![0u8; Secret::MIN_LEN]), Scope::ReadOnly);
        let (secret, scope) = known.unwrap_or(&dummy);
        let expected = compute_response(secret, &nonce);

        let matches: bool = expected.ct_eq(response).into();
        if !matches || known.is_none() {
            // Deliberately the same error either way. Telling a caller which key
            // ids exist is free reconnaissance.
            return Err(AuthError::BadResponse);
        }

        // Single use. Without this, a captured response could be replayed against
        // the same still-live nonce.
        self.challenge = None;
        self.authenticated = true;
        self.key_id = Some(key_id.to_string());
        self.scope = *scope;
        Ok(())
    }

    /// True once the connection should be dropped for repeated failures.
    pub fn is_locked_out(&self) -> bool {
        self.attempts >= MAX_AUTH_ATTEMPTS && !self.authenticated
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn secret() -> Secret {
        Secret::new(b"correct-horse-battery-staple".to_vec()).unwrap()
    }

    fn config() -> AuthConfig {
        AuthConfig::disabled().with_key("app1", secret())
    }

    #[test]
    fn a_correct_response_authenticates() {
        let cfg = config();
        let mut st = AuthState::new(&cfg);
        assert!(!st.is_authenticated(), "must start unauthenticated");

        let nonce = st.new_challenge();
        let resp = client_response(b"correct-horse-battery-staple", &nonce);
        st.verify(&cfg, "app1", &resp).unwrap();

        assert!(st.is_authenticated());
        assert_eq!(st.key_id(), Some("app1"));
    }

    #[test]
    fn a_wrong_secret_is_rejected() {
        let cfg = config();
        let mut st = AuthState::new(&cfg);
        let nonce = st.new_challenge();
        let resp = client_response(b"wrong-secret-wrong-secret", &nonce);
        assert_eq!(st.verify(&cfg, "app1", &resp), Err(AuthError::BadResponse));
        assert!(!st.is_authenticated());
    }

    /// An unknown key id gives the SAME error as a wrong secret.
    ///
    /// Distinguishing them tells an attacker which key ids exist, which is the
    /// first half of the problem solved for free.
    #[test]
    fn an_unknown_key_is_indistinguishable_from_a_wrong_secret() {
        let cfg = config();
        let mut st = AuthState::new(&cfg);
        let nonce = st.new_challenge();
        // A response that would be correct if the key existed.
        let resp = client_response(b"correct-horse-battery-staple", &nonce);
        assert_eq!(
            st.verify(&cfg, "nosuchkey", &resp),
            Err(AuthError::BadResponse)
        );
        assert!(!st.is_authenticated());
    }

    /// **A captured response cannot be replayed.**
    ///
    /// The nonce is consumed on success and replaced on every new challenge, so a
    /// response is valid exactly once, for exactly one connection.
    #[test]
    fn responses_cannot_be_replayed() {
        let cfg = config();

        // Capture a valid exchange.
        let mut first = AuthState::new(&cfg);
        let nonce = first.new_challenge();
        let captured = client_response(b"correct-horse-battery-staple", &nonce);
        first.verify(&cfg, "app1", &captured).unwrap();

        // Replay onto a fresh connection: its nonce differs, so the response fails.
        let mut second = AuthState::new(&cfg);
        let _ = second.new_challenge();
        assert_eq!(
            second.verify(&cfg, "app1", &captured),
            Err(AuthError::BadResponse),
            "a captured response authenticated a second connection"
        );

        // And it cannot be reused on the ORIGINAL connection either, because the
        // nonce was consumed.
        assert_eq!(
            first.verify(&cfg, "app1", &captured),
            Err(AuthError::NoChallenge)
        );
    }

    /// Nonces do not repeat.
    ///
    /// A repeated nonce makes a captured response replayable, which is the one
    /// thing this scheme exists to prevent.
    #[test]
    fn nonces_are_unique_across_many_challenges() {
        let cfg = config();
        let mut seen = std::collections::HashSet::new();
        for _ in 0..2000 {
            let mut st = AuthState::new(&cfg);
            assert!(seen.insert(st.new_challenge()), "a nonce repeated");
        }
        // And they are not obviously structured — an all-zero or counter-like
        // nonce would pass the uniqueness check above while being predictable.
        let mut st = AuthState::new(&cfg);
        let n = st.new_challenge();
        assert_ne!(n, [0u8; NONCE_LEN], "nonce was all zeros");
        assert!(
            n.iter().collect::<std::collections::HashSet<_>>().len() > 4,
            "nonce has suspiciously low variety: {n:?}"
        );
    }

    /// Requesting a new challenge invalidates the previous one.
    #[test]
    fn a_new_challenge_supersedes_the_old_one() {
        let cfg = config();
        let mut st = AuthState::new(&cfg);
        let first = st.new_challenge();
        let stale = client_response(b"correct-horse-battery-staple", &first);
        let second = st.new_challenge();
        assert_ne!(first, second);
        assert_eq!(st.verify(&cfg, "app1", &stale), Err(AuthError::BadResponse));
    }

    #[test]
    fn responding_without_a_challenge_fails() {
        let cfg = config();
        let mut st = AuthState::new(&cfg);
        assert_eq!(
            st.verify(&cfg, "app1", &[0u8; RESPONSE_LEN]),
            Err(AuthError::NoChallenge)
        );
    }

    /// Repeated failures lock the connection out.
    #[test]
    fn attempts_are_limited() {
        let cfg = config();
        let mut st = AuthState::new(&cfg);
        for _ in 0..MAX_AUTH_ATTEMPTS {
            let nonce = st.new_challenge();
            let bad = client_response(b"nope-nope-nope-nope-nope", &nonce);
            assert_eq!(st.verify(&cfg, "app1", &bad), Err(AuthError::BadResponse));
        }
        assert!(st.is_locked_out());
        // Even a CORRECT response is now refused.
        let nonce = st.new_challenge();
        let good = client_response(b"correct-horse-battery-staple", &nonce);
        assert_eq!(
            st.verify(&cfg, "app1", &good),
            Err(AuthError::TooManyAttempts)
        );
        assert!(!st.is_authenticated());
    }

    /// A response of the wrong length is rejected, not padded or truncated.
    #[test]
    fn malformed_response_lengths_are_rejected() {
        let cfg = config();
        for len in [0usize, 1, 16, 31, 33, 64] {
            let mut st = AuthState::new(&cfg);
            let _ = st.new_challenge();
            assert_eq!(
                st.verify(&cfg, "app1", &vec![0u8; len]),
                Err(AuthError::BadResponse),
                "a {len}-byte response was not rejected"
            );
        }
    }

    /// With no credentials configured, connections are authorised immediately.
    ///
    /// Openly open rather than silently useless — and `is_enabled` lets the server
    /// say which mode it is in.
    #[test]
    fn no_credentials_means_authentication_is_disabled() {
        let cfg = AuthConfig::disabled();
        assert!(!cfg.is_enabled());
        let st = AuthState::new(&cfg);
        assert!(st.is_authenticated());

        assert!(config().is_enabled());
        assert!(!AuthState::new(&config()).is_authenticated());
    }

    #[test]
    fn short_secrets_are_refused_at_configuration_time() {
        assert!(matches!(
            Secret::new(b"short".to_vec()),
            Err(AuthError::SecretTooShort { .. })
        ));
        assert!(Secret::new(vec![0u8; Secret::MIN_LEN]).is_ok());
    }

    /// A secret must not appear in debug output.
    ///
    /// Secrets leak into logs through a derived `Debug` on an enclosing struct far
    /// more often than through anyone printing them on purpose.
    #[test]
    fn secrets_are_redacted_in_debug_output() {
        let s = secret();
        let printed = format!("{s:?}");
        assert!(
            !printed.contains("correct-horse"),
            "secret leaked: {printed}"
        );
        assert!(printed.contains("redacted"));

        let cfg = config();
        let printed = format!("{cfg:?}");
        assert!(
            !printed.contains("correct-horse"),
            "secret leaked through AuthConfig: {printed}"
        );
    }

    /// The response depends on both the secret and the nonce.
    #[test]
    fn the_response_binds_secret_and_nonce_together() {
        let a = Secret::new(vec![1u8; 32]).unwrap();
        let b = Secret::new(vec![2u8; 32]).unwrap();
        let n1 = [3u8; NONCE_LEN];
        let n2 = [4u8; NONCE_LEN];

        assert_ne!(compute_response(&a, &n1), compute_response(&b, &n1));
        assert_ne!(compute_response(&a, &n1), compute_response(&a, &n2));
        // Deterministic for the same inputs.
        assert_eq!(compute_response(&a, &n1), compute_response(&a, &n1));
        // And the client path agrees with the server path.
        assert_eq!(compute_response(&a, &n1), client_response(&[1u8; 32], &n1));
    }
}
