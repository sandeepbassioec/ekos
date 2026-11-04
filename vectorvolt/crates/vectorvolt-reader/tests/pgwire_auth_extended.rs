//! D2: pgwire SCRAM-SHA-256 / MD5 authentication and the extended query
//! protocol, driven by a hand-rolled client speaking raw protocol 3.0.

use std::net::SocketAddr;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use vectorvolt_reader::auth::{
    AuthMethod, base64_decode, base64_encode, hmac_sha256, md5_password_response,
    pbkdf2_hmac_sha256, sha256,
};
use vectorvolt_reader::server::{PgConfig, PgServer};

const PASSWORD: &str = "s3cret-pass";

/// Start a server (serving until the test ends) and return its address.
async fn start(dir: &std::path::Path, password: Option<&str>, method: AuthMethod) -> SocketAddr {
    let config = PgConfig::new("127.0.0.1:0".parse().unwrap(), dir.to_path_buf());
    let server = PgServer::new(config)
        .unwrap()
        .with_auth_method(method)
        .with_password(password.map(str::to_owned));
    let (listener, addr) = server.bind().await.unwrap();
    tokio::spawn(server.serve_listener_until(listener, addr, std::future::pending()));
    addr
}

fn startup_msg(user: &str) -> Vec<u8> {
    let mut params = Vec::new();
    for (k, v) in [("user", user), ("database", "vectorvolt")] {
        params.extend_from_slice(k.as_bytes());
        params.push(0);
        params.extend_from_slice(v.as_bytes());
        params.push(0);
    }
    params.push(0);
    let mut buf = ((8 + params.len()) as i32).to_be_bytes().to_vec();
    buf.extend_from_slice(&(3i32 << 16).to_be_bytes());
    buf.extend_from_slice(&params);
    buf
}

/// A typed frontend message.
fn msg(t: u8, body: &[u8]) -> Vec<u8> {
    let mut v = vec![t];
    v.extend_from_slice(&((4 + body.len()) as i32).to_be_bytes());
    v.extend_from_slice(body);
    v
}

fn cstr(s: &str) -> Vec<u8> {
    let mut v = s.as_bytes().to_vec();
    v.push(0);
    v
}

/// Read one backend message (type byte + body).
async fn read_msg(s: &mut TcpStream) -> (u8, Vec<u8>) {
    let mut hdr = [0u8; 5];
    s.read_exact(&mut hdr).await.unwrap();
    let len = i32::from_be_bytes([hdr[1], hdr[2], hdr[3], hdr[4]]) as usize;
    let mut body = vec![0u8; len - 4];
    s.read_exact(&mut body).await.unwrap();
    (hdr[0], body)
}

/// The value of field `f` of an ErrorResponse / NoticeResponse body.
fn field(body: &[u8], f: u8) -> String {
    let mut i = 0;
    while i < body.len() && body[i] != 0 {
        let end = i + 1 + body[i + 1..].iter().position(|&b| b == 0).unwrap();
        if body[i] == f {
            return String::from_utf8_lossy(&body[i + 1..end]).into_owned();
        }
        i = end + 1;
    }
    String::new()
}

fn auth_code(body: &[u8]) -> i32 {
    i32::from_be_bytes([body[0], body[1], body[2], body[3]])
}

/// After AuthenticationOk: skip ParameterStatus until ReadyForQuery.
async fn expect_ready(s: &mut TcpStream) {
    loop {
        let (t, body) = read_msg(s).await;
        match t {
            b'S' | b'K' | b'N' => continue,
            b'Z' => return,
            other => panic!(
                "unexpected {:?} before ReadyForQuery: {}",
                other as char,
                field(&body, b'M')
            ),
        }
    }
}

/// Minimal SCRAM-SHA-256 client. Returns the stream after AuthenticationOk
/// + ReadyForQuery, or the SQLSTATE of the ErrorResponse that refused it.
async fn scram_connect(addr: SocketAddr, password: &str, gs2: &str) -> Result<TcpStream, String> {
    let mut s = TcpStream::connect(addr).await.unwrap();
    s.write_all(&startup_msg("alice")).await.unwrap();
    let (t, body) = read_msg(&mut s).await;
    assert_eq!(t, b'R');
    assert_eq!(auth_code(&body), 10, "expected AuthenticationSASL");
    let mechs: Vec<&[u8]> = body[4..].split(|&b| b == 0).collect();
    assert!(mechs.contains(&&b"SCRAM-SHA-256"[..]), "{mechs:?}");
    assert!(!mechs.contains(&&b"SCRAM-SHA-256-PLUS"[..]));

    let client_nonce = "fyko+d2lbbFgONRv9qkxdawL";
    let client_first_bare = format!("n=,r={client_nonce}");
    let client_first = format!("{gs2}{client_first_bare}");
    let mut init = cstr("SCRAM-SHA-256");
    init.extend_from_slice(&(client_first.len() as i32).to_be_bytes());
    init.extend_from_slice(client_first.as_bytes());
    s.write_all(&msg(b'p', &init)).await.unwrap();

    let (t, body) = read_msg(&mut s).await;
    if t == b'E' {
        return Err(field(&body, b'C'));
    }
    assert_eq!(t, b'R');
    assert_eq!(auth_code(&body), 11, "expected AuthenticationSASLContinue");
    let server_first = String::from_utf8(body[4..].to_vec()).unwrap();
    let mut nonce = "";
    let mut salt = Vec::new();
    let mut iters = 0u32;
    for attr in server_first.split(',') {
        match attr.split_at(2) {
            ("r=", v) => nonce = v,
            ("s=", v) => salt = base64_decode(v).unwrap(),
            ("i=", v) => iters = v.parse().unwrap(),
            _ => {}
        }
    }
    assert!(nonce.starts_with(client_nonce) && nonce.len() > client_nonce.len());
    assert_eq!(iters, 4096);
    assert!(!salt.is_empty());

    let salted = pbkdf2_hmac_sha256(password.as_bytes(), &salt, iters);
    let client_key = hmac_sha256(&salted, b"Client Key");
    let stored_key = sha256(&client_key);
    let without_proof = format!("c={},r={nonce}", base64_encode(gs2.as_bytes()));
    let auth_message = format!("{client_first_bare},{server_first},{without_proof}");
    let signature = hmac_sha256(&stored_key, auth_message.as_bytes());
    let proof: Vec<u8> = client_key
        .iter()
        .zip(signature.iter())
        .map(|(a, b)| a ^ b)
        .collect();
    let client_final = format!("{without_proof},p={}", base64_encode(&proof));
    s.write_all(&msg(b'p', client_final.as_bytes()))
        .await
        .unwrap();

    let (t, body) = read_msg(&mut s).await;
    if t == b'E' {
        return Err(field(&body, b'C'));
    }
    assert_eq!(t, b'R');
    assert_eq!(auth_code(&body), 12, "expected AuthenticationSASLFinal");
    let server_key = hmac_sha256(&salted, b"Server Key");
    let expected = format!(
        "v={}",
        base64_encode(&hmac_sha256(&server_key, auth_message.as_bytes()))
    );
    assert_eq!(String::from_utf8(body[4..].to_vec()).unwrap(), expected);

    let (t, body) = read_msg(&mut s).await;
    assert_eq!(
        (t, auth_code(&body)),
        (b'R', 0),
        "expected AuthenticationOk"
    );
    expect_ready(&mut s).await;
    Ok(s)
}

/// Run a simple query and collect (type, body) until ReadyForQuery.
async fn simple(s: &mut TcpStream, sql: &str) -> Vec<(u8, Vec<u8>)> {
    s.write_all(&msg(b'Q', &cstr(sql))).await.unwrap();
    until_ready(s).await
}

async fn until_ready(s: &mut TcpStream) -> Vec<(u8, Vec<u8>)> {
    let mut out = Vec::new();
    loop {
        let m = read_msg(s).await;
        let done = m.0 == b'Z';
        out.push(m);
        if done {
            return out;
        }
    }
}

fn types(msgs: &[(u8, Vec<u8>)]) -> String {
    msgs.iter().map(|(t, _)| *t as char).collect()
}

/// Cell values of every DataRow.
fn data_rows(msgs: &[(u8, Vec<u8>)]) -> Vec<Vec<String>> {
    msgs.iter()
        .filter(|(t, _)| *t == b'D')
        .map(|(_, b)| {
            let n = i16::from_be_bytes([b[0], b[1]]) as usize;
            let mut i = 2;
            let mut row = Vec::new();
            for _ in 0..n {
                let len = i32::from_be_bytes([b[i], b[i + 1], b[i + 2], b[i + 3]]);
                i += 4;
                if len < 0 {
                    row.push("NULL".to_owned());
                } else {
                    row.push(String::from_utf8_lossy(&b[i..i + len as usize]).into_owned());
                    i += len as usize;
                }
            }
            row
        })
        .collect()
}

/// Column names of a RowDescription body.
fn row_desc_columns(body: &[u8]) -> Vec<String> {
    let n = i16::from_be_bytes([body[0], body[1]]) as usize;
    let mut i = 2;
    let mut cols = Vec::new();
    for _ in 0..n {
        let end = i + body[i..].iter().position(|&b| b == 0).unwrap();
        cols.push(String::from_utf8_lossy(&body[i..end]).into_owned());
        i = end + 1 + 18;
    }
    cols
}

// ── extended-protocol message builders ───────────────────────────────────────

fn parse(name: &str, sql: &str, oids: &[u32]) -> Vec<u8> {
    let mut b = cstr(name);
    b.extend(cstr(sql));
    b.extend_from_slice(&(oids.len() as i16).to_be_bytes());
    for o in oids {
        b.extend_from_slice(&o.to_be_bytes());
    }
    msg(b'P', &b)
}

fn bind(portal: &str, stmt: &str, formats: &[i16], params: &[Option<&[u8]>]) -> Vec<u8> {
    let mut b = cstr(portal);
    b.extend(cstr(stmt));
    b.extend_from_slice(&(formats.len() as i16).to_be_bytes());
    for f in formats {
        b.extend_from_slice(&f.to_be_bytes());
    }
    b.extend_from_slice(&(params.len() as i16).to_be_bytes());
    for p in params {
        match p {
            None => b.extend_from_slice(&(-1i32).to_be_bytes()),
            Some(v) => {
                b.extend_from_slice(&(v.len() as i32).to_be_bytes());
                b.extend_from_slice(v);
            }
        }
    }
    b.extend_from_slice(&0i16.to_be_bytes()); // all results text
    msg(b'B', &b)
}

fn describe(kind: u8, name: &str) -> Vec<u8> {
    let mut b = vec![kind];
    b.extend(cstr(name));
    msg(b'D', &b)
}

fn execute(portal: &str, max_rows: i32) -> Vec<u8> {
    let mut b = cstr(portal);
    b.extend_from_slice(&max_rows.to_be_bytes());
    msg(b'E', &b)
}

fn close(kind: u8, name: &str) -> Vec<u8> {
    let mut b = vec![kind];
    b.extend(cstr(name));
    msg(b'C', &b)
}

fn sync() -> Vec<u8> {
    msg(b'S', &[])
}

/// A no-auth session with table `t(pk, val)` holding three rows.
async fn ready_with_table(dir: &std::path::Path) -> TcpStream {
    let addr = start(dir, None, AuthMethod::default()).await;
    let mut s = TcpStream::connect(addr).await.unwrap();
    s.write_all(&startup_msg("postgres")).await.unwrap();
    let (t, body) = read_msg(&mut s).await;
    assert_eq!((t, auth_code(&body)), (b'R', 0));
    expect_ready(&mut s).await;
    for sql in [
        "CREATE TABLE t (pk U64, val U64)",
        "INSERT INTO t (pk, val) VALUES (1, 10)",
        "INSERT INTO t (pk, val) VALUES (2, 20)",
        "INSERT INTO t (pk, val) VALUES (3, 30)",
    ] {
        let r = simple(&mut s, sql).await;
        assert!(!types(&r).contains('E'), "{sql}: {}", types(&r));
    }
    s
}

// ── SCRAM ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn scram_is_default_and_accepts_correct_password() {
    let dir = tempfile::tempdir().unwrap();
    let addr = start(dir.path(), Some(PASSWORD), AuthMethod::default()).await;
    let mut s = scram_connect(addr, PASSWORD, "n,,").await.expect("auth ok");
    let r = simple(&mut s, "CREATE TABLE a (pk U64, val U64)").await;
    assert!(!types(&r).contains('E'), "{}", types(&r));
}

#[tokio::test]
async fn scram_rejects_wrong_password() {
    let dir = tempfile::tempdir().unwrap();
    let addr = start(dir.path(), Some(PASSWORD), AuthMethod::ScramSha256).await;
    let err = scram_connect(addr, "not-the-password", "n,,")
        .await
        .expect_err("must be refused");
    assert_eq!(err, "28P01");
}

#[tokio::test]
async fn scram_channel_binding_flags() {
    let dir = tempfile::tempdir().unwrap();
    let addr = start(dir.path(), Some(PASSWORD), AuthMethod::ScramSha256).await;
    // 'y': client supports binding but the server does not offer it — fine.
    scram_connect(addr, PASSWORD, "y,,").await.expect("y,, ok");
    // 'p=': channel binding requested — not supported, refused.
    let err = scram_connect(addr, PASSWORD, "p=tls-server-end-point,,")
        .await
        .expect_err("p= must be refused");
    assert_eq!(err, "0A000");
}

#[tokio::test]
async fn scram_rejects_cleartext_password_message() {
    let dir = tempfile::tempdir().unwrap();
    let addr = start(dir.path(), Some(PASSWORD), AuthMethod::ScramSha256).await;
    let mut s = TcpStream::connect(addr).await.unwrap();
    s.write_all(&startup_msg("alice")).await.unwrap();
    let (_, body) = read_msg(&mut s).await;
    assert_eq!(auth_code(&body), 10);
    // A plain PasswordMessage is not a SASLInitialResponse.
    s.write_all(&msg(b'p', &cstr(PASSWORD))).await.unwrap();
    let (t, body) = read_msg(&mut s).await;
    assert_eq!(t, b'E');
    assert_ne!(field(&body, b'C'), "");
}

// ── MD5 ──────────────────────────────────────────────────────────────────────

async fn md5_connect(addr: SocketAddr, user: &str, password: &str) -> (u8, Vec<u8>) {
    let mut s = TcpStream::connect(addr).await.unwrap();
    s.write_all(&startup_msg(user)).await.unwrap();
    let (t, body) = read_msg(&mut s).await;
    assert_eq!(t, b'R');
    assert_eq!(auth_code(&body), 5, "expected AuthenticationMD5Password");
    let salt: [u8; 4] = body[4..8].try_into().unwrap();
    let resp = md5_password_response(password, user, &salt);
    s.write_all(&msg(b'p', &cstr(&resp))).await.unwrap();
    read_msg(&mut s).await
}

#[tokio::test]
async fn md5_accepts_correct_password() {
    let dir = tempfile::tempdir().unwrap();
    let addr = start(dir.path(), Some(PASSWORD), AuthMethod::Md5).await;
    let (t, body) = md5_connect(addr, "bob", PASSWORD).await;
    assert_eq!((t, auth_code(&body)), (b'R', 0), "{}", field(&body, b'M'));
}

#[tokio::test]
async fn md5_rejects_wrong_password() {
    let dir = tempfile::tempdir().unwrap();
    let addr = start(dir.path(), Some(PASSWORD), AuthMethod::Md5).await;
    let (t, body) = md5_connect(addr, "bob", "wrong").await;
    assert_eq!(t, b'E');
    assert_eq!(field(&body, b'C'), "28P01");
}

#[tokio::test]
async fn auth_requires_user_name() {
    let dir = tempfile::tempdir().unwrap();
    let addr = start(dir.path(), Some(PASSWORD), AuthMethod::ScramSha256).await;
    let mut s = TcpStream::connect(addr).await.unwrap();
    s.write_all(&startup_msg("")).await.unwrap();
    let (t, body) = read_msg(&mut s).await;
    assert_eq!(t, b'E');
    assert_eq!(field(&body, b'C'), "28000");
}

// ── extended query protocol ──────────────────────────────────────────────────

#[tokio::test]
async fn extended_parse_bind_execute_with_parameter() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = ready_with_table(dir.path()).await;

    let mut batch = parse("by_pk", "SELECT * FROM t WHERE pk = $1", &[]);
    batch.extend(describe(b'S', "by_pk"));
    batch.extend(bind("", "by_pk", &[], &[Some(b"2")]));
    batch.extend(describe(b'P', ""));
    batch.extend(execute("", 0));
    batch.extend(sync());
    s.write_all(&batch).await.unwrap();
    let r = until_ready(&mut s).await;
    assert_eq!(types(&r), "1tT2TDCZ", "{r:?}");
    // ParameterDescription: one parameter, reported as text (OID 25).
    let pd = &r[1].1;
    assert_eq!(i16::from_be_bytes([pd[0], pd[1]]), 1);
    assert_eq!(u32::from_be_bytes([pd[2], pd[3], pd[4], pd[5]]), 25);
    assert_eq!(row_desc_columns(&r[2].1), vec!["pk", "val"]);
    assert_eq!(row_desc_columns(&r[4].1), vec!["pk", "val"]);
    assert_eq!(data_rows(&r), vec![vec!["2".to_owned(), "20".to_owned()]]);
    let tag = String::from_utf8_lossy(&r[6].1);
    assert!(tag.starts_with("SELECT 1"), "{tag}");

    // The named statement is reusable with another value, typed int4 binary.
    let mut batch = parse("typed", "SELECT * FROM t WHERE pk = $1", &[23]);
    batch.extend(bind("p1", "typed", &[1], &[Some(&3i32.to_be_bytes())]));
    batch.extend(execute("p1", 0));
    batch.extend(bind("", "by_pk", &[], &[Some(b"1")]));
    batch.extend(execute("", 0));
    batch.extend(sync());
    s.write_all(&batch).await.unwrap();
    let r = until_ready(&mut s).await;
    assert_eq!(types(&r), "12DC2DCZ", "{r:?}");
    assert_eq!(
        data_rows(&r),
        vec![
            vec!["3".to_owned(), "30".to_owned()],
            vec!["1".to_owned(), "10".to_owned()]
        ]
    );
}

#[tokio::test]
async fn extended_insert_has_no_data_and_command_complete() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = ready_with_table(dir.path()).await;
    let mut batch = parse("", "INSERT INTO t (pk, val) VALUES ($1, $2)", &[20, 20]);
    batch.extend(describe(b'S', ""));
    batch.extend(bind("", "", &[], &[Some(b"4"), Some(b"40")]));
    batch.extend(describe(b'P', ""));
    batch.extend(execute("", 0));
    batch.extend(sync());
    s.write_all(&batch).await.unwrap();
    let r = until_ready(&mut s).await;
    assert_eq!(types(&r), "1tn2nCZ", "{r:?}");
    let r = simple(&mut s, "SELECT * FROM t WHERE pk = 4").await;
    assert_eq!(data_rows(&r), vec![vec!["4".to_owned(), "40".to_owned()]]);
}

#[tokio::test]
async fn extended_row_limit_suspends_portal() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = ready_with_table(dir.path()).await;
    let mut batch = parse("", "SELECT * FROM t", &[]);
    batch.extend(bind("cur", "", &[], &[]));
    batch.extend(execute("cur", 2));
    batch.extend(execute("cur", 2));
    batch.extend(close(b'P', "cur"));
    batch.extend(sync());
    s.write_all(&batch).await.unwrap();
    let r = until_ready(&mut s).await;
    assert_eq!(types(&r), "12DDsDC3Z", "{r:?}");
    assert_eq!(data_rows(&r).len(), 3);
    let tag = String::from_utf8_lossy(&r[6].1);
    assert!(tag.starts_with("SELECT 3"), "{tag}");
}

#[tokio::test]
async fn extended_error_skips_until_sync_then_recovers() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = ready_with_table(dir.path()).await;
    // Bind to a statement that does not exist: ErrorResponse, then the
    // Execute (and everything else) up to Sync is ignored.
    let mut batch = bind("", "missing", &[], &[]);
    batch.extend(execute("", 0));
    batch.extend(parse("", "SELECT * FROM t", &[]));
    batch.extend(sync());
    s.write_all(&batch).await.unwrap();
    let r = until_ready(&mut s).await;
    assert_eq!(types(&r), "EZ", "{r:?}");
    assert_eq!(field(&r[0].1, b'C'), "26000");

    // An executor error during Execute behaves the same way.
    let mut batch = parse("", "SELECT * FROM no_such_table WHERE pk = $1", &[]);
    batch.extend(bind("", "", &[], &[Some(b"1")]));
    batch.extend(execute("", 0));
    batch.extend(execute("", 0));
    batch.extend(sync());
    s.write_all(&batch).await.unwrap();
    let r = until_ready(&mut s).await;
    assert_eq!(types(&r), "12EZ", "{r:?}");

    // Wrong parameter count.
    let mut batch = parse("", "SELECT * FROM t WHERE pk = $1", &[]);
    batch.extend(bind("", "", &[], &[]));
    batch.extend(sync());
    s.write_all(&batch).await.unwrap();
    let r = until_ready(&mut s).await;
    assert_eq!(types(&r), "1EZ", "{r:?}");
    assert_eq!(field(&r[1].1, b'C'), "08P01");

    // A quote in a text parameter is refused rather than spliced in.
    let mut batch = parse("", "SELECT * FROM t WHERE pk = $1", &[]);
    batch.extend(bind("", "", &[], &[Some(b"1' OR '1'='1")]));
    batch.extend(sync());
    s.write_all(&batch).await.unwrap();
    let r = until_ready(&mut s).await;
    assert_eq!(types(&r), "1EZ", "{r:?}");
    assert_eq!(field(&r[1].1, b'C'), "22P02");

    // Duplicate named statement.
    let mut batch = parse("dup", "SELECT * FROM t", &[]);
    batch.extend(parse("dup", "SELECT * FROM t", &[]));
    batch.extend(sync());
    s.write_all(&batch).await.unwrap();
    let r = until_ready(&mut s).await;
    assert_eq!(types(&r), "1EZ", "{r:?}");
    assert_eq!(field(&r[1].1, b'C'), "42P05");

    // After the errors the session works normally, extended and simple.
    let mut batch = close(b'S', "dup");
    batch.extend(parse("dup", "SELECT * FROM t WHERE pk = $1", &[]));
    batch.extend(bind("", "dup", &[], &[Some(b"3")]));
    batch.extend(execute("", 0));
    batch.extend(sync());
    s.write_all(&batch).await.unwrap();
    let r = until_ready(&mut s).await;
    assert_eq!(types(&r), "312DCZ", "{r:?}");
    assert_eq!(data_rows(&r), vec![vec!["3".to_owned(), "30".to_owned()]]);
    let r = simple(&mut s, "SELECT * FROM t").await;
    assert_eq!(data_rows(&r).len(), 3);
}

#[tokio::test]
async fn extended_flush_sends_pending_without_ready() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = ready_with_table(dir.path()).await;
    let mut batch = parse("", "SELECT * FROM t", &[]);
    batch.extend(msg(b'H', &[]));
    s.write_all(&batch).await.unwrap();
    let (t, _) = read_msg(&mut s).await;
    assert_eq!(t, b'1');
    s.write_all(&sync()).await.unwrap();
    let (t, _) = read_msg(&mut s).await;
    assert_eq!(t, b'Z');
}

#[tokio::test]
async fn extended_over_authenticated_scram_session() {
    let dir = tempfile::tempdir().unwrap();
    let addr = start(dir.path(), Some(PASSWORD), AuthMethod::ScramSha256).await;
    let mut s = scram_connect(addr, PASSWORD, "n,,").await.unwrap();
    let r = simple(&mut s, "CREATE TABLE u (pk U64, val U64)").await;
    assert!(!types(&r).contains('E'), "{}", types(&r));
    let mut batch = parse("", "INSERT INTO u (pk, val) VALUES ($1, $2)", &[]);
    batch.extend(bind("", "", &[], &[Some(b"7"), Some(b"70")]));
    batch.extend(execute("", 0));
    batch.extend(sync());
    s.write_all(&batch).await.unwrap();
    let r = until_ready(&mut s).await;
    assert_eq!(types(&r), "12CZ", "{r:?}");
}
