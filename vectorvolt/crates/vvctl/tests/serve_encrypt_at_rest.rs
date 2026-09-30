/// Tests for `vvctl serve --encrypt-at-rest` (VV-SEC-006).
///
/// Scope of this fix: `schema.txt` (the single-table metadata file) is
/// transparently AES-256-GCM encrypted when `--encrypt-at-rest` is set, using
/// `vectorvolt_crypto::{EncryptedWriter, EncryptedReader}` and a master key
/// resolved from `VECTORVOLT_MASTER_KEY`. Row data in `.vvsst` partitions/WAL
/// segments is NOT covered yet (see the doc comment on `ServeArgs::encrypt_at_rest`).
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
        cert: None,
        key: None,
        multi: false,
        config: None,
        master_config: None,
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
        tpm_unseal: false,
        encrypt_at_rest,
        metrics_port: 0,
        pgwire_port: None,
        cluster_bind: None,
        cluster_seeds: vec![],
        cluster_node_id: None,
        cluster_status_port: 7900,
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

#[test]
fn encrypt_at_rest_conflicts_with_multi_mode() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    set_test_key();
    let dir = TempDir::new().unwrap();
    let mut args = base_args(&dir, true);
    args.multi = true;
    let err = match bind(&args) {
        Ok(_) => panic!("must reject --encrypt-at-rest with --multi for now"),
        Err(e) => e,
    };
    assert!(
        err.to_string().contains("--multi"),
        "unexpected error: {err}"
    );
    clear_key();
}
