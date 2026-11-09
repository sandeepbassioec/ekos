/// Tests for `vvctl serve --encrypt-at-rest` (VV-SEC-006).
///
/// Scope of this fix: `schema.txt` (the single-table metadata file) is
/// transparently AES-256-GCM encrypted when `--encrypt-at-rest` is set, using
/// `vectorvolt_crypto::{EncryptedWriter, EncryptedReader}` and a master key
/// resolved from `VECTORVOLT_MASTER_KEY`. Table WALs are encrypted in both
/// single-table and `--multi` mode (PP-51), and the pgwire listener gets a
/// derived SQL column-encryption key (A2). Row data in `.vvsst` partitions is
/// NOT covered yet (see the doc comment on `ServeArgs::encrypt_at_rest`).
use std::path::PathBuf;
use std::sync::Mutex;

use tempfile::TempDir;
use vvctl::commands::serve::{ServeArgs, bind};

// `VECTORVOLT_MASTER_KEY` is process-global env state; serialize tests that
// touch it so they don't race each other.
static ENV_LOCK: Mutex<()> = Mutex::new(());

fn base_args(dir: &TempDir, encrypt_at_rest: bool) -> ServeArgs {
    ServeArgs {
        dir: dir.path().to_path_buf(),
        columns: vec!["pk".into(), "val".into()],
        addr: "127.0.0.1:0".parse().unwrap(),
        max_connections: 32,
        secret: None,
        key_id: "default".into(),
        key_scope: None,
        key_file: None,
        cert: None,
        key: None,
        multi: false,
        config: None,
        master_config: None,
        sinks_config: None,
        flush_threshold: None,
        durability: None,
        max_partitions: None,
        catalog: false,
        restore_catalog: false,
        http_addr: None,
        http_token: None,
        transport: "tcp".into(),
        xdp_iface: "lo".into(),
        xdp_queue: 0,
        xdp_ring_size: 256,
        grpc_port: 50051,
        no_grpc: false,
        grpc_host: std::net::IpAddr::from([127, 0, 0, 1]),
        grpc_token: None,
        tpm_unseal: false,
        encrypt_at_rest,
        metrics_port: 0,
        metrics_host: std::net::IpAddr::from([127, 0, 0, 1]),
        pgwire_port: None,
        cluster_bind: None,
        cluster_seeds: vec![],
        cluster_node_id: None,
        cluster_status_port: 7900,
        listener: Default::default(),
    }
}

fn set_test_key() {
    // Arbitrary but fixed 32-byte key (64 hex chars) for reproducibility.
    unsafe {
        std::env::set_var(
            "VECTORVOLT_MASTER_KEY",
            "891cbeb0ca01a43cadf845ce73c7756c5e4ce5e63e831132fe44c1803c473c35",
        );
    }
}

fn clear_key() {
    unsafe {
        std::env::remove_var("VECTORVOLT_MASTER_KEY");
    }
}

fn schema_path(dir: &TempDir) -> PathBuf {
    dir.path().join("schema.txt")
}

#[test]
fn encrypt_at_rest_off_by_default() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    clear_key();
    let dir = TempDir::new().unwrap();
    assert!(!base_args(&dir, false).encrypt_at_rest);
}

#[test]
fn encrypt_at_rest_without_key_fails_fast() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    clear_key();
    let dir = TempDir::new().unwrap();
    let args = base_args(&dir, true);
    let err = match bind(&args) {
        Ok(_) => panic!("must fail without a master key"),
        Err(e) => e,
    };
    assert!(
        err.to_string().contains("master key"),
        "unexpected error: {err}"
    );
}

#[test]
fn encrypt_at_rest_schema_file_is_not_plaintext_on_disk() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    set_test_key();
    let dir = TempDir::new().unwrap();
    let args = base_args(&dir, true);

    let bound = bind(&args).expect("bind should succeed with a master key set");
    assert_eq!(bound.schema, vec!["pk".to_string(), "val".to_string()]);
    assert!(bound.encrypt_at_rest);

    let raw = std::fs::read(schema_path(&dir)).expect("schema.txt must exist");
    // The plaintext schema is short ASCII ("pk\nval") — if encryption is a
    // no-op this substring would be trivially present in the raw bytes.
    let raw_str = String::from_utf8_lossy(&raw);
    assert!(
        !raw_str.contains("pk") && !raw_str.contains("val"),
        "schema.txt on disk contains plaintext column names — encryption did not apply: {raw_str:?}"
    );

    clear_key();
}

#[test]
fn encrypt_at_rest_round_trips_through_bind() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    set_test_key();
    let dir = TempDir::new().unwrap();

    // First bind creates the encrypted schema.txt.
    {
        let args = base_args(&dir, true);
        let bound = bind(&args).expect("create");
        assert_eq!(bound.schema, vec!["pk".to_string(), "val".to_string()]);
    }

    // Second bind (schema.txt already exists) must decrypt it correctly.
    {
        let args = base_args(&dir, true);
        let bound = bind(&args).expect("reopen");
        assert_eq!(bound.schema, vec!["pk".to_string(), "val".to_string()]);
    }

    clear_key();
}

#[test]
fn encrypt_at_rest_wrong_key_fails_to_decrypt() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    set_test_key();
    let dir = TempDir::new().unwrap();
    {
        let args = base_args(&dir, true);
        bind(&args).expect("create with key A");
    }

    unsafe {
        std::env::set_var(
            "VECTORVOLT_MASTER_KEY",
            "ac8795ba9b94370dbdd53f308f7c22099ae3af377a0f6433bb50855aa1f70dec",
        );
    }
    let args = base_args(&dir, true);
    let err = match bind(&args) {
        Ok(_) => panic!("must fail to decrypt with the wrong key"),
        Err(e) => e,
    };
    assert!(
        err.to_string().to_lowercase().contains("decrypt"),
        "unexpected error: {err}"
    );

    clear_key();
}

// ── --multi --encrypt-at-rest (PP-51 / C1) ────────────────────────────────────

/// Distinctive value the plaintext WAL encoding writes verbatim (LE u64).
const MARKER: u64 = 0xDEAD_BEEF_CAFE_F00D;

fn multi_args(dir: &TempDir, encrypt_at_rest: bool) -> ServeArgs {
    let mut args = base_args(dir, encrypt_at_rest);
    args.multi = true;
    args.columns = vec![];
    args
}

fn wal_bytes(dir: &std::path::Path) -> Vec<u8> {
    let mut bytes = Vec::new();
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_file() {
            bytes.extend(std::fs::read(&path).unwrap());
        }
    }
    bytes
}

fn contains_marker(bytes: &[u8]) -> bool {
    bytes.windows(8).any(|w| w == MARKER.to_le_bytes())
}

/// Bind `args`, serve on a background thread, run `f` against the wire
/// address, then shut the server down and wait for it to release the data.
fn with_running_server(args: &ServeArgs, f: impl FnOnce(std::net::SocketAddr)) {
    let bound = bind(args).expect("bind");
    let addr = bound.server.local_addr().expect("local_addr");
    let shutdown = bound.server.shutdown_handle();
    let server = bound.server;
    let handle = std::thread::spawn(move || {
        let _ = server.serve();
    });
    f(addr);
    shutdown.shutdown();
    handle.join().unwrap();
}

#[test]
fn multi_encrypt_at_rest_round_trips_a_row_and_encrypts_the_wal() {
    use vectorvolt_net::Client;
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    set_test_key();
    let dir = TempDir::new().unwrap();

    {
        let bound = bind(&multi_args(&dir, true)).expect("--multi --encrypt-at-rest must bind");
        assert!(bound.encrypt_at_rest);
        assert!(bound.master_key.is_some());
        assert!(bound.registry.as_ref().unwrap().is_encrypted());
    }

    with_running_server(&multi_args(&dir, true), |addr| {
        let mut c = Client::connect(addr).unwrap();
        c.create_table("secrets", vec!["pk".into(), "val".into()])
            .unwrap();
        c.insert(1, vec![1, MARKER]).unwrap();
        assert_eq!(c.get(1).unwrap(), Some(vec![1, MARKER]));
    });

    let wal = wal_bytes(&dir.path().join("secrets").join("wal"));
    assert!(!wal.is_empty(), "WAL must have been written");
    assert!(
        !contains_marker(&wal),
        "marker found in plaintext in the table WAL: --multi did not encrypt"
    );

    // Restart with the key: the row is recovered from the encrypted WAL.
    with_running_server(&multi_args(&dir, true), |addr| {
        let mut c = Client::connect(addr).unwrap();
        c.use_table("secrets").unwrap();
        assert_eq!(c.get(1).unwrap(), Some(vec![1, MARKER]));
    });

    // Restart without encryption: the encrypted table must not open silently empty.
    clear_key();
    assert!(
        bind(&multi_args(&dir, false)).is_err(),
        "restoring an encrypted catalog without the key must fail"
    );
}

#[test]
fn multi_encrypt_at_rest_without_key_fails_fast() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    clear_key();
    let dir = TempDir::new().unwrap();
    let err = match bind(&multi_args(&dir, true)) {
        Ok(_) => panic!("must fail without a master key"),
        Err(e) => e,
    };
    assert!(
        err.to_string().contains("master key"),
        "unexpected error: {err}"
    );
}

#[test]
fn single_table_encrypt_at_rest_encrypts_the_wal() {
    use vectorvolt_net::Client;
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    set_test_key();
    let dir = TempDir::new().unwrap();
    with_running_server(&base_args(&dir, true), |addr| {
        let mut c = Client::connect(addr).unwrap();
        c.insert(1, vec![1, MARKER]).unwrap();
    });
    let wal = wal_bytes(&dir.path().join("wal"));
    assert!(!wal.is_empty(), "WAL must have been written");
    assert!(!contains_marker(&wal), "single-table WAL is plaintext");
    with_running_server(&base_args(&dir, true), |addr| {
        let mut c = Client::connect(addr).unwrap();
        assert_eq!(c.get(1).unwrap(), Some(vec![1, MARKER]));
    });
    clear_key();
}

// ── SQL column encryption over pgwire (A2) ────────────────────────────────────

#[test]
fn derived_column_key_is_deterministic_and_domain_separated() {
    use vectorvolt_crypto::KeyStore;
    use vvctl::commands::serve::derive_column_key;
    let master =
        KeyStore::from_hex("891cbeb0ca01a43cadf845ce73c7756c5e4ce5e63e831132fe44c1803c473c35")
            .unwrap();
    let a = derive_column_key(&master).unwrap();
    let b = derive_column_key(&master).unwrap();
    assert_eq!(a.as_bytes(), b.as_bytes());
    assert_ne!(a.as_bytes(), master.as_bytes());
}

/// Read whole backend messages until ReadyForQuery ('Z').
async fn read_until_ready(stream: &mut tokio::net::TcpStream) -> Vec<u8> {
    use tokio::io::AsyncReadExt;
    let mut all = Vec::new();
    let mut pos = 0;
    let mut chunk = [0u8; 4096];
    loop {
        while all.len() >= pos + 5 {
            let tag = all[pos];
            let len = i32::from_be_bytes(all[pos + 1..pos + 5].try_into().unwrap()) as usize;
            if all.len() < pos + 1 + len {
                break;
            }
            pos += 1 + len;
            if tag == b'Z' {
                return all;
            }
        }
        let n = tokio::time::timeout(std::time::Duration::from_secs(5), stream.read(&mut chunk))
            .await
            .expect("timed out waiting for ReadyForQuery")
            .unwrap();
        if n == 0 {
            return all;
        }
        all.extend_from_slice(&chunk[..n]);
    }
}

/// Run `queries` over pgwire against the pgwire server `vvctl serve` builds
/// for `args` (same registry, same derived column key), returning each reply
/// as lossy text.
fn pgwire_replies(args: &ServeArgs, queries: &[&str]) -> Vec<String> {
    use bytes::BufMut;
    use tokio::io::AsyncWriteExt;
    use vectorvolt_reader::server::PgConfig;
    use vvctl::commands::serve::{build_pg_server, derive_column_key};

    let bound = bind(args).expect("bind");
    let registry = bound.registry.clone().expect("multi mode registry");
    let column_key = bound
        .master_key
        .as_deref()
        .map(|k| derive_column_key(k).unwrap());

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async move {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        drop(listener);
        let server = build_pg_server(
            PgConfig::new(addr, PathBuf::new()),
            registry,
            None,
            Default::default(),
            column_key,
        );
        let (tx, rx) = tokio::sync::oneshot::channel::<()>();
        let serve = tokio::spawn(async move {
            let _ = server
                .serve_until(async move {
                    let _ = rx.await;
                })
                .await;
        });

        let mut stream = None;
        for _ in 0..100 {
            if let Ok(s) = tokio::net::TcpStream::connect(addr).await {
                stream = Some(s);
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
        let mut stream = stream.expect("connect to pgwire");

        let params = b"user\0postgres\0database\0vectorvolt\0\0";
        let mut buf = bytes::BytesMut::new();
        buf.put_i32((8 + params.len()) as i32);
        buf.put_i32(3 << 16);
        buf.put(params.as_ref());
        stream.write_all(&buf).await.unwrap();
        read_until_ready(&mut stream).await;

        let mut replies = Vec::new();
        for sql in queries {
            let payload = format!("{sql}\0");
            let mut msg = bytes::BytesMut::new();
            msg.put_u8(b'Q');
            msg.put_i32((4 + payload.len()) as i32);
            msg.put(payload.as_bytes());
            stream.write_all(&msg).await.unwrap();
            let reply = read_until_ready(&mut stream).await;
            replies.push(String::from_utf8_lossy(&reply).into_owned());
        }
        drop(stream);
        let _ = tx.send(());
        let _ = serve.await;
        replies
    })
}

#[test]
fn pgwire_insert_encrypted_works_with_encrypt_at_rest() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    set_test_key();
    let dir = TempDir::new().unwrap();
    let replies = pgwire_replies(
        &multi_args(&dir, true),
        &[
            "CREATE ENCRYPTED COLUMN emp salary KEY HINT 'hk'",
            "INSERT ENCRYPTED INTO emp salary KEY 'hk' VALUE 'hello-secret' PK 1",
            "SELECT DECRYPT salary FROM emp KEY 'hk' PK 1",
        ],
    );
    clear_key();
    for r in &replies[..2] {
        assert!(
            !r.contains("not configured"),
            "column key must be set: {r:?}"
        );
    }
    assert!(
        replies[2].contains("hello-secret"),
        "SELECT DECRYPT must return the plaintext: {:?}",
        replies[2]
    );
}

#[test]
fn pgwire_insert_encrypted_fails_without_encrypt_at_rest() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    clear_key();
    let dir = TempDir::new().unwrap();
    let replies = pgwire_replies(
        &multi_args(&dir, false),
        &["INSERT ENCRYPTED INTO emp salary KEY 'hk' VALUE 'hello-secret' PK 1"],
    );
    assert!(
        replies[0].contains("column encryption key not configured"),
        "unexpected reply: {:?}",
        replies[0]
    );
}
