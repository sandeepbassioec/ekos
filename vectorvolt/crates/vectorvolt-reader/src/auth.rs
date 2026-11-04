//! pgwire authentication: SCRAM-SHA-256 (RFC 5802 / RFC 7677, as PostgreSQL
//! speaks it), MD5 and cleartext password.
//!
//! The server holds one password (`vvctl serve --secret`). For SCRAM it is
//! turned into a [`ScramVerifier`] once — random per-server salt, 4096
//! PBKDF2 iterations — and only the derived StoredKey / ServerKey are kept,
//! so the cleartext password is never needed per connection. MD5 and
//! cleartext necessarily keep the password itself (MD5's hash depends on the
//! connecting user name).
//!
//! Everything here is pure (no I/O): the session drives the message exchange.

use std::fmt;
use std::str::FromStr;

use hmac::{Hmac, Mac};
use rand::RngCore;
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

type HmacSha256 = Hmac<Sha256>;

/// SASL mechanism name advertised in AuthenticationSASL.
pub const SCRAM_SHA_256: &str = "SCRAM-SHA-256";

/// PBKDF2 iteration count for the server's SCRAM verifier (PostgreSQL's
/// default `scram_iterations`).
pub const SCRAM_ITERATIONS: u32 = 4096;

/// Random salt length for the server's SCRAM verifier (PostgreSQL uses 16).
const SCRAM_SALT_LEN: usize = 16;

/// Random bytes in the server half of the SCRAM nonce (PostgreSQL uses 18).
const SCRAM_NONCE_LEN: usize = 18;

// ── authentication method ─────────────────────────────────────────────────────

/// How pgwire clients prove they know the password.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AuthMethod {
    /// SCRAM-SHA-256 (AuthenticationSASL). The default.
    #[default]
    ScramSha256,
    /// AuthenticationMD5Password: `md5` + md5(md5(password + user) + salt).
    Md5,
    /// AuthenticationCleartextPassword (only sensible over TLS).
    Password,
}

impl AuthMethod {
    /// The name accepted by [`FromStr`] (and `vvctl serve --pgwire-auth`).
    pub fn as_str(self) -> &'static str {
        match self {
            AuthMethod::ScramSha256 => "scram",
            AuthMethod::Md5 => "md5",
            AuthMethod::Password => "password",
        }
    }
}

impl fmt::Display for AuthMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for AuthMethod {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "scram" | "scram-sha-256" | "scram_sha_256" => Ok(AuthMethod::ScramSha256),
            "md5" => Ok(AuthMethod::Md5),
            "password" | "cleartext" | "plain" => Ok(AuthMethod::Password),
            other => Err(format!(
                "unknown pgwire auth method `{other}` (expected scram, md5 or password)"
            )),
        }
    }
}

/// The server-side credential a session checks clients against.
#[derive(Clone)]
pub enum Credentials {
    /// No authentication.
    None,
    /// Cleartext password exchange.
    Cleartext(String),
    /// MD5 exchange (needs the password: the hash includes the user name).
    Md5(String),
    /// SCRAM-SHA-256 against a precomputed verifier.
    Scram(std::sync::Arc<ScramVerifier>),
}

impl fmt::Debug for Credentials {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Never print secrets.
        f.write_str(match self {
            Credentials::None => "Credentials::None",
            Credentials::Cleartext(_) => "Credentials::Cleartext(..)",
            Credentials::Md5(_) => "Credentials::Md5(..)",
            Credentials::Scram(_) => "Credentials::Scram(..)",
        })
    }
}

impl Credentials {
    /// Build the credential for `password` under `method` (`None` = no auth).
    /// For SCRAM this derives the verifier now, with a fresh random salt.
    pub fn new(password: Option<&str>, method: AuthMethod) -> Self {
        match (password, method) {
            (None, _) => Credentials::None,
            (Some(pw), AuthMethod::Password) => Credentials::Cleartext(pw.to_owned()),
            (Some(pw), AuthMethod::Md5) => Credentials::Md5(pw.to_owned()),
            (Some(pw), AuthMethod::ScramSha256) => {
                Credentials::Scram(std::sync::Arc::new(ScramVerifier::generate(pw)))
            }
        }
    }
}

// ── primitives ────────────────────────────────────────────────────────────────

/// HMAC-SHA-256.
pub fn hmac_sha256(key: &[u8], data: &[u8]) -> [u8; 32] {
    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC accepts any key length");
    mac.update(data);
    mac.finalize().into_bytes().into()
}

/// SHA-256.
pub fn sha256(data: &[u8]) -> [u8; 32] {
    Sha256::digest(data).into()
}

/// SCRAM `Hi()` = PBKDF2-HMAC-SHA-256 with a single 32-byte output block.
pub fn pbkdf2_hmac_sha256(password: &[u8], salt: &[u8], iterations: u32) -> [u8; 32] {
    let mac = HmacSha256::new_from_slice(password).expect("HMAC accepts any key length");
    let mut first = mac.clone();
    first.update(salt);
    first.update(&1u32.to_be_bytes());
    let mut u: [u8; 32] = first.finalize().into_bytes().into();
    let mut out = u;
    for _ in 1..iterations.max(1) {
        let mut m = mac.clone();
        m.update(&u);
        u = m.finalize().into_bytes().into();
        for (o, b) in out.iter_mut().zip(u.iter()) {
            *o ^= b;
        }
    }
    out
}

const B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Standard base64 (RFC 4648, padded).
pub fn base64_encode(data: &[u8]) -> String {
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b = [
            chunk[0],
            *chunk.get(1).unwrap_or(&0),
            *chunk.get(2).unwrap_or(&0),
        ];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        out.push(B64[(n >> 18) as usize & 63] as char);
        out.push(B64[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 {
            B64[(n >> 6) as usize & 63] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            B64[n as usize & 63] as char
        } else {
            '='
        });
    }
    out
}

/// Strict standard base64 decoding (padding required). `None` on bad input.
pub fn base64_decode(s: &str) -> Option<Vec<u8>> {
    let s = s.as_bytes();
    if !s.len().is_multiple_of(4) {
        return None;
    }
    fn val(c: u8) -> Option<u32> {
        Some(match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            _ => return None,
        } as u32)
    }
    let mut out = Vec::with_capacity(s.len() / 4 * 3);
    let n_chunks = s.len() / 4;
    for (i, c) in s.chunks(4).enumerate() {
        let last = i + 1 == n_chunks;
        let pad = c.iter().rev().take_while(|&&b| b == b'=').count();
        if pad > 2 || (pad > 0 && !last) {
            return None;
        }
        let mut n = 0u32;
        for &b in &c[..4 - pad] {
            n = (n << 6) | val(b)?;
        }
        n <<= 6 * pad as u32;
        out.push((n >> 16) as u8);
        if pad < 2 {
            out.push((n >> 8) as u8);
        }
        if pad < 1 {
            out.push(n as u8);
        }
    }
    Some(out)
}

/// Lower-case hex.
fn hex(bytes: &[u8]) -> String {
    const H: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push(H[(b >> 4) as usize] as char);
        s.push(H[(b & 15) as usize] as char);
    }
    s
}

/// MD5 (RFC 1321). Only for the PostgreSQL MD5 password exchange.
pub fn md5(data: &[u8]) -> [u8; 16] {
    const S: [u32; 64] = [
        7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 5, 9, 14, 20, 5, 9, 14, 20, 5,
        9, 14, 20, 5, 9, 14, 20, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 6, 10,
        15, 21, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21,
    ];
    let k: [u32; 64] =
        std::array::from_fn(|i| ((i as f64 + 1.0).sin().abs() * 4_294_967_296.0) as u32);
    let mut h: [u32; 4] = [0x6745_2301, 0xefcd_ab89, 0x98ba_dcfe, 0x1032_5476];
    let mut msg = data.to_vec();
    let bit_len = (data.len() as u64).wrapping_mul(8);
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bit_len.to_le_bytes());
    for block in msg.chunks(64) {
        let m: [u32; 16] = std::array::from_fn(|i| {
            u32::from_le_bytes([
                block[4 * i],
                block[4 * i + 1],
                block[4 * i + 2],
                block[4 * i + 3],
            ])
        });
        let [mut a, mut b, mut c, mut d] = h;
        for i in 0..64 {
            let (f, g) = match i / 16 {
                0 => ((b & c) | (!b & d), i),
                1 => ((d & b) | (!d & c), (5 * i + 1) % 16),
                2 => (b ^ c ^ d, (3 * i + 5) % 16),
                _ => (c ^ (b | !d), (7 * i) % 16),
            };
            let f = f.wrapping_add(a).wrapping_add(k[i]).wrapping_add(m[g]);
            a = d;
            d = c;
            c = b;
            b = b.wrapping_add(f.rotate_left(S[i]));
        }
        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
    }
    let mut out = [0u8; 16];
    for (i, w) in h.iter().enumerate() {
        out[4 * i..4 * i + 4].copy_from_slice(&w.to_le_bytes());
    }
    out
}

/// The PasswordMessage an MD5 client sends:
/// `"md5" + hex(md5(hex(md5(password + user)) + salt))`.
pub fn md5_password_response(password: &str, user: &str, salt: &[u8; 4]) -> String {
    let inner = hex(&md5(format!("{password}{user}").as_bytes()));
    let mut outer = inner.into_bytes();
    outer.extend_from_slice(salt);
    format!("md5{}", hex(&md5(&outer)))
}

/// Constant-time check of an MD5 PasswordMessage.
pub fn md5_verify(password: &str, user: &str, salt: &[u8; 4], supplied: &str) -> bool {
    let expected = md5_password_response(password, user, salt);
    bool::from(expected.as_bytes().ct_eq(supplied.as_bytes()))
}

/// Fresh random bytes from the OS RNG.
pub fn random_bytes<const N: usize>() -> [u8; N] {
    let mut b = [0u8; N];
    rand::rngs::OsRng.fill_bytes(&mut b);
    b
}

// ── SCRAM-SHA-256 ─────────────────────────────────────────────────────────────

/// The server's SCRAM-SHA-256 verifier for one password:
/// `StoredKey = H(HMAC(SaltedPassword, "Client Key"))`,
/// `ServerKey = HMAC(SaltedPassword, "Server Key")`.
pub struct ScramVerifier {
    salt: Vec<u8>,
    iterations: u32,
    stored_key: [u8; 32],
    server_key: [u8; 32],
}

impl ScramVerifier {
    /// Derive a verifier with a fresh random salt and [`SCRAM_ITERATIONS`].
    pub fn generate(password: &str) -> Self {
        let salt: [u8; SCRAM_SALT_LEN] = random_bytes();
        Self::new(password, &salt, SCRAM_ITERATIONS)
    }

    /// Derive a verifier from an explicit salt / iteration count.
    ///
    /// SASLprep is not applied: for ASCII passwords (the `--secret` hex
    /// strings) it is the identity, and PostgreSQL itself falls back to the
    /// raw bytes when SASLprep does not apply.
    pub fn new(password: &str, salt: &[u8], iterations: u32) -> Self {
        let salted = pbkdf2_hmac_sha256(password.as_bytes(), salt, iterations);
        let client_key = hmac_sha256(&salted, b"Client Key");
        Self {
            salt: salt.to_vec(),
            iterations,
            stored_key: sha256(&client_key),
            server_key: hmac_sha256(&salted, b"Server Key"),
        }
    }

    pub fn salt(&self) -> &[u8] {
        &self.salt
    }

    pub fn iterations(&self) -> u32 {
        self.iterations
    }

    pub fn stored_key(&self) -> &[u8; 32] {
        &self.stored_key
    }

    pub fn server_key(&self) -> &[u8; 32] {
        &self.server_key
    }
}

/// Why a SCRAM exchange failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScramError {
    /// Malformed or unsupported message (SQLSTATE 08P01).
    Protocol(String),
    /// The client asked for something we do not support (SQLSTATE 0A000).
    Unsupported(String),
    /// Wrong password (SQLSTATE 28P01).
    BadProof,
}

impl ScramError {
    /// SQLSTATE for the FATAL error sent to the client.
    pub fn sqlstate(&self) -> &'static str {
        match self {
            ScramError::Protocol(_) => "08P01",
            ScramError::Unsupported(_) => "0A000",
            ScramError::BadProof => "28P01",
        }
    }
}

impl fmt::Display for ScramError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScramError::Protocol(m) => write!(f, "malformed SCRAM message: {m}"),
            ScramError::Unsupported(m) => f.write_str(m),
            ScramError::BadProof => f.write_str("password authentication failed"),
        }
    }
}

/// Server state between the client-first and client-final messages.
pub struct ScramServer {
    gs2_header: String,
    client_first_bare: String,
    server_first: String,
    nonce: String,
}

fn proto(msg: &str) -> ScramError {
    ScramError::Protocol(msg.to_owned())
}

/// Characters allowed in a SCRAM nonce: printable ASCII except ','.
fn valid_nonce(n: &str) -> bool {
    !n.is_empty() && n.bytes().all(|b| (0x21..=0x7e).contains(&b) && b != b',')
}

impl ScramServer {
    /// Process the client-first-message (the SASLInitialResponse data) and
    /// return the server state plus the server-first-message to send in
    /// AuthenticationSASLContinue. `server_nonce_bytes` become the
    /// (base64-encoded) server half of the nonce. Channel binding is never
    /// offered, so `p=` is refused while `n` and `y` are accepted.
    pub fn start(
        verifier: &ScramVerifier,
        client_first: &[u8],
        server_nonce_bytes: &[u8],
    ) -> Result<(Self, String), ScramError> {
        let msg = std::str::from_utf8(client_first).map_err(|_| proto("not UTF-8"))?;
        // gs2-header = gs2-cbind-flag "," [ authzid ] ","
        let (cbind, rest) = msg
            .split_once(',')
            .ok_or_else(|| proto("missing GS2 header"))?;
        match cbind {
            // Client does not support channel binding.
            "n" => {}
            // Client supports channel binding but thinks the server does not.
            // True: SCRAM-SHA-256-PLUS is never advertised (not even over
            // TLS), so this is not a downgrade and is accepted.
            "y" => {}
            c if c.starts_with("p=") => {
                return Err(ScramError::Unsupported(
                    "channel binding is not supported by this server \
                     (connect with channel_binding=disable or prefer)"
                        .to_owned(),
                ));
            }
            _ => return Err(proto("invalid channel-binding flag")),
        }
        let (authzid, bare) = rest
            .split_once(',')
            .ok_or_else(|| proto("missing GS2 header"))?;
        if !authzid.is_empty() {
            return Err(ScramError::Unsupported(
                "SCRAM authorization identity (a=) is not supported".to_owned(),
            ));
        }
        let gs2_header = format!("{cbind},,");
        // client-first-message-bare = [reserved-mext ","] username "," nonce ["," extensions]
        let mut parts = bare.split(',');
        let user = parts.next().unwrap_or("");
        if user.starts_with("m=") {
            return Err(ScramError::Unsupported(
                "SCRAM mandatory extensions are not supported".to_owned(),
            ));
        }
        // The user name is taken from the startup packet (as PostgreSQL
        // does); `n=` is usually empty and is otherwise ignored.
        if !user.starts_with("n=") {
            return Err(proto("expected n= attribute"));
        }
        let client_nonce = parts
            .next()
            .and_then(|p| p.strip_prefix("r="))
            .ok_or_else(|| proto("expected r= attribute"))?;
        if !valid_nonce(client_nonce) {
            return Err(proto("invalid client nonce"));
        }
        let nonce = format!("{client_nonce}{}", base64_encode(server_nonce_bytes));
        let server_first = format!(
            "r={nonce},s={},i={}",
            base64_encode(verifier.salt()),
            verifier.iterations()
        );
        Ok((
            Self {
                gs2_header,
                client_first_bare: bare.to_owned(),
                server_first: server_first.clone(),
                nonce,
            },
            server_first,
        ))
    }

    /// Like [`start`](Self::start) with a fresh random server nonce.
    pub fn start_random(
        verifier: &ScramVerifier,
        client_first: &[u8],
    ) -> Result<(Self, String), ScramError> {
        let nonce: [u8; SCRAM_NONCE_LEN] = random_bytes();
        Self::start(verifier, client_first, &nonce)
    }

    /// Verify the client-final-message (the SASLResponse data). On success
    /// returns the server-final-message (`v=<ServerSignature>`) to send in
    /// AuthenticationSASLFinal. The proof check is constant time.
    pub fn finish(
        self,
        verifier: &ScramVerifier,
        client_final: &[u8],
    ) -> Result<String, ScramError> {
        let msg = std::str::from_utf8(client_final).map_err(|_| proto("not UTF-8"))?;
        // client-final-message = channel-binding "," nonce ["," extensions] "," proof
        let (without_proof, proof_b64) = msg
            .rsplit_once(",p=")
            .ok_or_else(|| proto("missing proof"))?;
        let mut parts = without_proof.split(',');
        let cbind = parts
            .next()
            .and_then(|p| p.strip_prefix("c="))
            .ok_or_else(|| proto("expected c= attribute"))?;
        let cbind = base64_decode(cbind).ok_or_else(|| proto("invalid channel-binding data"))?;
        if cbind != self.gs2_header.as_bytes() {
            return Err(proto("channel-binding data does not match the GS2 header"));
        }
        let nonce = parts
            .next()
            .and_then(|p| p.strip_prefix("r="))
            .ok_or_else(|| proto("expected r= attribute"))?;
        if !bool::from(nonce.as_bytes().ct_eq(self.nonce.as_bytes())) {
            return Err(proto("nonce mismatch"));
        }
        let proof = base64_decode(proof_b64).ok_or_else(|| proto("invalid proof encoding"))?;
        if proof.len() != 32 {
            return Err(proto("invalid proof length"));
        }
        let auth_message = format!(
            "{},{},{}",
            self.client_first_bare, self.server_first, without_proof
        );
        let client_signature = hmac_sha256(verifier.stored_key(), auth_message.as_bytes());
        let mut client_key = [0u8; 32];
        for (i, k) in client_key.iter_mut().enumerate() {
            *k = proof[i] ^ client_signature[i];
        }
        if !bool::from(sha256(&client_key).ct_eq(verifier.stored_key())) {
            return Err(ScramError::BadProof);
        }
        let server_signature = hmac_sha256(verifier.server_key(), auth_message.as_bytes());
        Ok(format!("v={}", base64_encode(&server_signature)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_roundtrip() {
        for (raw, enc) in [
            (&b""[..], ""),
            (b"f", "Zg=="),
            (b"fo", "Zm8="),
            (b"foo", "Zm9v"),
            (b"foob", "Zm9vYg=="),
            (b"fooba", "Zm9vYmE="),
            (b"foobar", "Zm9vYmFy"),
            (b"n,,", "biws"),
        ] {
            assert_eq!(base64_encode(raw), enc);
            assert_eq!(base64_decode(enc).as_deref(), Some(raw));
        }
        for bad in ["Zg=", "Z===", "Zg==Zg==", "Zm9*", "===="] {
            assert_eq!(base64_decode(bad), None, "{bad}");
        }
    }

    #[test]
    fn md5_known_vectors() {
        assert_eq!(hex(&md5(b"")), "d41d8cd98f00b204e9800998ecf8427e");
        assert_eq!(hex(&md5(b"abc")), "900150983cd24fb0d6963f7d28e17f72");
        assert_eq!(
            hex(&md5(b"The quick brown fox jumps over the lazy dog")),
            "9e107d9d372bb6826bd81d3542a419d6"
        );
        // Multi-block input.
        assert_eq!(
            hex(&md5(
                b"12345678901234567890123456789012345678901234567890123456789012345678901234567890"
            )),
            "57edf4a22be3c955ac49da2e2107b67a"
        );
    }

    #[test]
    fn pbkdf2_rfc7914_vector() {
        // RFC 7914 §11, PBKDF2-HMAC-SHA256 (P="passwd", S="salt", c=1).
        let out = pbkdf2_hmac_sha256(b"passwd", b"salt", 1);
        assert_eq!(
            hex(&out),
            "55ac046e56e3089fec1691c22544b605f94185216dde0465e68b9d57c20dacbc"
        );
    }

    /// RFC 7677 §3 test vector: user "user", password "pencil",
    /// salt W22ZaJ0SNY7soEsUEjb6gQ==, 4096 iterations.
    #[test]
    fn scram_rfc7677_vector() {
        let salt = base64_decode("W22ZaJ0SNY7soEsUEjb6gQ==").unwrap();
        let v = ScramVerifier::new("pencil", &salt, 4096);
        let client_first = b"n,,n=user,r=rOprNGfwEbeRWgbNEkqO";
        let server_nonce_suffix = "%hvYDpWUa2RaTCAfuxFIlj)hNlF$k0";
        // Our server nonce is base64 of random bytes; to reproduce the RFC
        // exchange, build the state by hand with the RFC's nonce.
        let (mut st, _) = ScramServer::start(&v, client_first, b"x").unwrap();
        st.nonce = format!("rOprNGfwEbeRWgbNEkqO{server_nonce_suffix}");
        st.server_first = format!("r={},s=W22ZaJ0SNY7soEsUEjb6gQ==,i=4096", st.nonce);
        let client_final = "c=biws,r=rOprNGfwEbeRWgbNEkqO%hvYDpWUa2RaTCAfuxFIlj)hNlF$k0,\
                            p=dHzbZapWIk4jUhN+Ute9ytag9zjfMHgsqmmiz7AndVQ=";
        let server_final = st.finish(&v, client_final.as_bytes()).unwrap();
        assert_eq!(
            server_final,
            "v=6rriTRBi23WpRR/wtup+mMhUZUn/dB5nLTJRsjl95G4="
        );
    }

    #[test]
    fn scram_rejects_channel_binding_and_bad_proof() {
        let v = ScramVerifier::new("pencil", b"salt", 4096);
        let err = ScramServer::start(&v, b"p=tls-server-end-point,,n=,r=abc", b"n")
            .err()
            .unwrap();
        assert_eq!(err.sqlstate(), "0A000");
        assert!(ScramServer::start(&v, b"y,,n=,r=abc", b"n").is_ok());
        assert!(ScramServer::start(&v, b"n,a=bob,n=,r=abc", b"n").is_err());
        assert!(ScramServer::start(&v, b"garbage", b"n").is_err());
        let (st, _) = ScramServer::start(&v, b"n,,n=,r=abc", b"n").unwrap();
        let nonce = st.nonce.clone();
        let bad = format!("c=biws,r={nonce},p={}", base64_encode(&[0u8; 32]));
        assert_eq!(st.finish(&v, bad.as_bytes()), Err(ScramError::BadProof));
    }

    #[test]
    fn auth_method_parse() {
        assert_eq!("scram".parse(), Ok(AuthMethod::ScramSha256));
        assert_eq!("SCRAM-SHA-256".parse(), Ok(AuthMethod::ScramSha256));
        assert_eq!("md5".parse(), Ok(AuthMethod::Md5));
        assert_eq!("password".parse(), Ok(AuthMethod::Password));
        assert!("trust".parse::<AuthMethod>().is_err());
        assert_eq!(AuthMethod::default(), AuthMethod::ScramSha256);
    }

    #[test]
    fn md5_response_format() {
        let r = md5_password_response("secret", "alice", &[1, 2, 3, 4]);
        assert!(r.starts_with("md5") && r.len() == 35);
        assert!(md5_verify("secret", "alice", &[1, 2, 3, 4], &r));
        assert!(!md5_verify("secret", "bob", &[1, 2, 3, 4], &r));
        assert!(!md5_verify("secret", "alice", &[1, 2, 3, 5], &r));
    }
}
