//! PP-54 / PP-56 (C4 / C6) and the A12 follow-up: encryption at rest for the
//! engine's side files, authenticated snapshot and PITR markers, and a
//! deleted `wal/tail.seal` on a sealed table.
use std::path::{Path, PathBuf};
use std::sync::Arc;

use tempfile::TempDir;
use vectorvolt_crypto::KeyStore;
use vectorvolt_engine::snapshot::{SnapshotMeta, write_marker};
use vectorvolt_engine::{
    CdcLog, Durability, EngineOptions, IntentState, TableEngine, TableRegistry, TableTxn,
    TableWriteSet, TxnIntentLog, read_catalog_wal,
};
use vectorvolt_txn::ops::Mutation;
use vectorvolt_wal::{SnapshotMarker, WalConfig, WalWriter};

/// Distinctive row value. CDC and intent records are JSON, so it would
/// appear on disk as its decimal digits.
const MARKER: u64 = 0xDEAD_BEEF_CAFE_F00D;

fn marker_text() -> Vec<u8> {
    MARKER.to_string().into_bytes()
}

fn key() -> Arc<KeyStore> {
    Arc::new(
        KeyStore::from_hex("891cbeb0ca01a43cadf845ce73c7756c5e4ce5e63e831132fe44c1803c473c35")
            .unwrap(),
    )
}

fn other_key() -> Arc<KeyStore> {
    Arc::new(
        KeyStore::from_hex("ac8795ba9b94370dbdd53f308f7c22099ae3af377a0f6433bb50855aa1f70dec")
            .unwrap(),
    )
}

fn schema() -> Vec<String> {
    vec!["pk".into(), "v".into()]
}

fn opts() -> EngineOptions {
    EngineOptions::new(1_000_000).with_durability(Durability::WriteOnly)
}

fn open_keyed(dir: &Path) -> Result<TableEngine, vectorvolt_engine::EngineError> {
    TableEngine::open_with_encryption(schema(), dir, opts(), key())
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len()).any(|w| w == needle)
}

fn err_text<T>(r: Result<T, vectorvolt_engine::EngineError>) -> String {
    match r {
        Ok(_) => panic!("expected an error"),
        Err(e) => e.to_string(),
    }
}

// ── cdc.log ──────────────────────────────────────────────────────────────────

#[test]
fn cdc_log_is_encrypted_and_reads_back_after_reopen() {
    let tmp = TempDir::new().unwrap();
    {
        let e = open_keyed(tmp.path()).unwrap();
        e.insert(1, vec![1, MARKER]).unwrap();
    }
    let bytes = std::fs::read(tmp.path().join("cdc.log")).unwrap();
    assert!(!bytes.is_empty());
    assert!(!contains(&bytes, &marker_text()), "row value in plaintext");
    assert!(!contains(&bytes, b"\"values\""), "JSON in plaintext");

    let e = open_keyed(tmp.path()).unwrap();
    let evs = e.read_log_from(0).unwrap();
    assert_eq!(evs.len(), 1);
    assert_eq!(evs[0].values, Some(vec![1, MARKER]));
    // Header-only scans still work on the encrypted framing.
    assert_eq!(e.cdc_log_stats().unwrap().event_count, 1);
}

#[test]
fn encrypted_cdc_log_without_key_names_the_file() {
    let tmp = TempDir::new().unwrap();
    {
        let e = open_keyed(tmp.path()).unwrap();
        e.insert(1, vec![1, MARKER]).unwrap();
    }
    let path = tmp.path().join("cdc.log");
    let err = CdcLog::open(path).unwrap().read_from(0).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("cdc.log") && msg.contains("no key"), "{msg}");
}

#[test]
fn cdc_log_copied_from_another_table_fails() {
    let tmp = TempDir::new().unwrap();
    let (a, b) = (tmp.path().join("a"), tmp.path().join("b"));
    {
        let ea = open_keyed(&a).unwrap();
        ea.insert(1, vec![1, MARKER]).unwrap();
        let eb = open_keyed(&b).unwrap();
        eb.insert(1, vec![1, 2]).unwrap();
    }
    std::fs::copy(a.join("cdc.log"), b.join("cdc.log")).unwrap();
    let eb = open_keyed(&b).unwrap();
    let msg = err_text(eb.read_log_from(0));
    assert!(
        msg.contains("cdc.log") && msg.contains("authentication failed"),
        "{msg}"
    );
}

#[test]
fn legacy_plaintext_cdc_log_still_reads_once_keyed() {
    let tmp = TempDir::new().unwrap();
    {
        let e = TableEngine::open_with(schema(), tmp.path(), opts()).unwrap();
        e.insert(1, vec![1, 10]).unwrap();
    }
    let e = open_keyed(tmp.path()).unwrap();
    e.insert(2, vec![2, MARKER]).unwrap();
    let evs = e.read_log_from(0).unwrap();
    assert_eq!(evs.len(), 2);
    assert_eq!(evs[0].values, Some(vec![1, 10]));
    assert_eq!(evs[1].values, Some(vec![2, MARKER]));
    let bytes = std::fs::read(tmp.path().join("cdc.log")).unwrap();
    assert!(!contains(&bytes, &marker_text()));
}

#[test]
fn failed_cdc_append_still_repairs_on_encrypted_log() {
    // B17: the record is sealed before it is written, so the cut-back of a
    // failed append is unchanged; compaction keeps sealed records readable.
    let tmp = TempDir::new().unwrap();
    let e = open_keyed(tmp.path()).unwrap();
    for pk in 1..=3 {
        e.insert(pk, vec![pk, MARKER]).unwrap();
    }
    assert_eq!(e.compact_cdc_log(2).unwrap(), 1);
    let evs = e.read_log_from(0).unwrap();
    assert_eq!(evs.iter().map(|e| e.pk).collect::<Vec<_>>(), vec![2, 3]);
}

// ── snapshot.vvss ────────────────────────────────────────────────────────────

fn snapshot_path(dir: &Path) -> PathBuf {
    dir.join("data").join("snapshot.vvss")
}

#[test]
fn snapshot_marker_is_encrypted_and_authenticated() {
    let tmp = TempDir::new().unwrap();
    {
        let e = open_keyed(tmp.path()).unwrap();
        e.insert(1, vec![1, MARKER]).unwrap();
        e.take_snapshot().unwrap();
        e.insert(2, vec![2, 3]).unwrap();
    }
    let bytes = std::fs::read(snapshot_path(tmp.path())).unwrap();
    assert!(!bytes.starts_with(b"VVSS"), "plaintext marker");
    let e = open_keyed(tmp.path()).unwrap();
    let meta = e.snapshot_info().unwrap().unwrap();
    assert_eq!(meta.row_count, 1);
    assert!(e.get(1).unwrap().is_some());
    assert!(e.get(2).unwrap().is_some(), "frame after snapshot replayed");
}

#[test]
fn forged_plaintext_snapshot_marker_on_keyed_table_is_rejected() {
    let tmp = TempDir::new().unwrap();
    {
        let e = open_keyed(tmp.path()).unwrap();
        e.insert(1, vec![1, MARKER]).unwrap();
    }
    // A forged marker claiming every frame is already in partition files
    // would make replay skip the unflushed insert.
    write_marker(
        &snapshot_path(tmp.path()),
        &SnapshotMeta {
            row_count: 0,
            byte_size: 0,
            wal_seq: u64::MAX,
        },
    )
    .unwrap();
    let msg = err_text(open_keyed(tmp.path()));
    assert!(
        msg.contains("snapshot.vvss") && msg.contains("not authenticated"),
        "{msg}"
    );
}

#[test]
fn snapshot_marker_from_another_table_or_key_is_rejected() {
    let tmp = TempDir::new().unwrap();
    let (a, b) = (tmp.path().join("a"), tmp.path().join("b"));
    {
        let ea = open_keyed(&a).unwrap();
        ea.insert(1, vec![1, 1]).unwrap();
        ea.take_snapshot().unwrap();
        let eb = open_keyed(&b).unwrap();
        eb.insert(1, vec![1, 1]).unwrap();
    }
    std::fs::copy(snapshot_path(&a), snapshot_path(&b)).unwrap();
    let msg = err_text(open_keyed(&b));
    assert!(msg.contains("snapshot.vvss"), "{msg}");

    // Read without the key: the error names the file and the key id.
    let msg = vectorvolt_engine::snapshot::read_marker(&snapshot_path(&a))
        .unwrap_err()
        .to_string();
    assert!(
        msg.contains("snapshot.vvss") && msg.contains("no key"),
        "{msg}"
    );
}

#[test]
fn legacy_plaintext_snapshot_marker_is_ignored_on_first_keyed_open() {
    let tmp = TempDir::new().unwrap();
    {
        let e = TableEngine::open_with(schema(), tmp.path(), opts()).unwrap();
        e.insert(1, vec![1, 1]).unwrap();
        e.take_snapshot().unwrap();
        e.insert(2, vec![2, 2]).unwrap();
    }
    let e = open_keyed(tmp.path()).unwrap();
    assert!(e.get(1).unwrap().is_some());
    assert!(e.get(2).unwrap().is_some());
    e.take_snapshot().unwrap();
    drop(e);
    let e = open_keyed(tmp.path()).unwrap();
    assert_eq!(e.snapshot_info().unwrap().unwrap().row_count, 2);
}

// ── secondary_index.catalog ─────────────────────────────────────────────────

#[test]
fn secondary_index_catalog_is_encrypted_and_bound_to_the_table() {
    let tmp = TempDir::new().unwrap();
    let (a, b) = (tmp.path().join("a"), tmp.path().join("b"));
    {
        let ea = open_keyed(&a).unwrap();
        ea.insert(1, vec![1, 7]).unwrap();
        ea.build_secondary_index(1).unwrap();
        let eb = open_keyed(&b).unwrap();
        eb.build_secondary_index(1).unwrap();
    }
    let path = a.join("secondary_index.catalog");
    let bytes = std::fs::read(&path).unwrap();
    // Plaintext would be `[1u64][1u64]`.
    assert_ne!(bytes.len(), 16);
    assert!(!bytes.starts_with(&1u64.to_le_bytes()));
    let ea = open_keyed(&a).unwrap();
    assert!(ea.has_secondary_index(1));
    drop(ea);

    std::fs::copy(&path, b.join("secondary_index.catalog")).unwrap();
    let msg = err_text(open_keyed(&b));
    assert!(msg.contains("secondary_index.catalog"), "{msg}");
}

#[test]
fn legacy_plaintext_index_catalog_still_reads() {
    let tmp = TempDir::new().unwrap();
    {
        let e = TableEngine::open_with(schema(), tmp.path(), opts()).unwrap();
        e.insert(1, vec![1, 7]).unwrap();
        e.build_secondary_index(1).unwrap();
    }
    let e = open_keyed(tmp.path()).unwrap();
    assert!(e.has_secondary_index(1));
}

// ── registry schema.txt + catalog WAL ────────────────────────────────────────

#[test]
fn registry_schema_and_catalog_wal_are_encrypted() {
    let tmp = TempDir::new().unwrap();
    let cat = tmp.path().join("_catalog_wal");
    {
        let mut reg =
            TableRegistry::open_with_encryption(tmp.path(), EngineOptions::new(1_000_000), key())
                .unwrap();
        reg.enable_catalog_wal(&cat).unwrap();
        reg.create_table(
            "secrettablexyz",
            vec!["pk".into(), "secretcolumnxyz".into()],
        )
        .unwrap();
    }
    let schema_bytes = std::fs::read(tmp.path().join("secrettablexyz/schema.txt")).unwrap();
    assert!(!contains(&schema_bytes, b"secretcolumnxyz"));
    let mut wal_bytes = Vec::new();
    for e in std::fs::read_dir(&cat).unwrap() {
        wal_bytes.extend(std::fs::read(e.unwrap().path()).unwrap());
    }
    assert!(!contains(&wal_bytes, b"secrettablexyz"));

    let reg = TableRegistry::open_with_encryption(tmp.path(), EngineOptions::new(1_000_000), key())
        .unwrap();
    assert_eq!(
        reg.read_schema("secrettablexyz").unwrap().names(),
        vec!["pk".to_string(), "secretcolumnxyz".to_string()]
    );
    let ddl = read_catalog_wal(&cat, Some(&key())).unwrap();
    assert!(matches!(
        &ddl[..],
        [Mutation::DdlCreate { table_name }] if table_name == "secrettablexyz"
    ));
    let msg = err_text(read_catalog_wal(&cat, None));
    assert!(msg.contains("no key was provided"), "{msg}");
    assert!(read_catalog_wal(&cat, Some(&other_key())).is_err());
}

#[test]
fn registry_schema_copied_from_another_table_fails() {
    let tmp = TempDir::new().unwrap();
    {
        let reg =
            TableRegistry::open_with_encryption(tmp.path(), EngineOptions::new(1_000_000), key())
                .unwrap();
        reg.create_table("a", vec!["pk".into(), "x".into()])
            .unwrap();
        reg.create_table("b", vec!["pk".into(), "y".into()])
            .unwrap();
    }
    std::fs::copy(
        tmp.path().join("a/schema.txt"),
        tmp.path().join("b/schema.txt"),
    )
    .unwrap();
    let msg = err_text(TableRegistry::open_with_encryption(
        tmp.path(),
        EngineOptions::new(1_000_000),
        key(),
    ));
    assert!(msg.contains("schema.txt"), "{msg}");
}

#[test]
fn legacy_plaintext_registry_schema_is_read_and_migrated() {
    let tmp = TempDir::new().unwrap();
    {
        let reg = TableRegistry::open(tmp.path(), EngineOptions::new(1_000_000)).unwrap();
        reg.create_table("t", vec!["pk".into(), "secretcolumnxyz".into()])
            .unwrap();
    }
    {
        let reg =
            TableRegistry::open_with_encryption(tmp.path(), EngineOptions::new(1_000_000), key())
                .unwrap();
        assert!(reg.contains("t"));
    }
    let bytes = std::fs::read(tmp.path().join("t/schema.txt")).unwrap();
    assert!(!contains(&bytes, b"secretcolumnxyz"), "not migrated");
    let reg = TableRegistry::open_with_encryption(tmp.path(), EngineOptions::new(1_000_000), key())
        .unwrap();
    assert_eq!(reg.read_schema("t").unwrap().names()[1], "secretcolumnxyz");
}

// ── cross-table intent log ───────────────────────────────────────────────────

fn tables() -> Vec<TableTxn> {
    vec![TableTxn {
        table: "a".into(),
        txn_id: 0,
    }]
}

fn write_set() -> Vec<TableWriteSet> {
    vec![TableWriteSet {
        table: "a".into(),
        ops: vec![(1, Some(vec![1, MARKER]))],
    }]
}

#[test]
fn intent_log_is_encrypted_per_record() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("_txn_intent.log");
    {
        let log = TxnIntentLog::open_with_encryption(&path, key()).unwrap();
        log.prepare_with_write_set("t1", tables(), write_set())
            .unwrap();
        log.mark_committed("t1", tables()).unwrap();
    }
    let bytes = std::fs::read(&path).unwrap();
    assert!(!contains(&bytes, &marker_text()));
    assert!(!contains(&bytes, b"t1"));

    let log = TxnIntentLog::open_with_encryption(&path, key()).unwrap();
    let recs = log.read_all().unwrap();
    assert_eq!(recs.len(), 2);
    assert_eq!(recs[0].write_set, write_set());
    assert_eq!(recs[1].state, IntentState::Committed);

    let msg = err_text(TxnIntentLog::open(&path).unwrap().read_all());
    assert!(
        msg.contains("_txn_intent.log") && msg.contains("no key"),
        "{msg}"
    );
    let msg = err_text(
        TxnIntentLog::open_with_encryption(&path, other_key())
            .unwrap()
            .read_all(),
    );
    assert!(
        msg.contains("_txn_intent.log") && msg.contains("key id"),
        "{msg}"
    );
}

#[test]
fn intent_log_reads_legacy_prefix_and_rejects_injected_plaintext() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("intent.log");
    TxnIntentLog::open(&path)
        .unwrap()
        .prepare("legacy", tables())
        .unwrap();
    TxnIntentLog::open_with_encryption(&path, key())
        .unwrap()
        .prepare_with_write_set("t2", tables(), write_set())
        .unwrap();
    let log = TxnIntentLog::open_with_encryption(&path, key()).unwrap();
    let recs = log.read_all().unwrap();
    assert_eq!(
        recs.iter().map(|r| r.txn_uuid.as_str()).collect::<Vec<_>>(),
        ["legacy", "t2"]
    );
    // A plaintext PREPARE planted after the encrypted records would make
    // recovery apply its write set.
    TxnIntentLog::open(&path)
        .unwrap()
        .prepare_with_write_set("planted", tables(), write_set())
        .unwrap();
    let msg = err_text(log.read_all());
    assert!(msg.contains("not authenticated"), "{msg}");
}

#[test]
fn intent_log_torn_tail_is_cut_and_records_cannot_be_reordered() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("intent.log");
    {
        let log = TxnIntentLog::open_with_encryption(&path, key()).unwrap();
        log.prepare("a", tables()).unwrap();
        log.prepare("b", tables()).unwrap();
    }
    // Torn append: half a record, no newline.
    let mut bytes = std::fs::read(&path).unwrap();
    bytes.extend_from_slice(b"vvsf1:5656534601");
    std::fs::write(&path, &bytes).unwrap();
    {
        let log = TxnIntentLog::open_with_encryption(&path, key()).unwrap();
        log.prepare("c", tables()).unwrap();
        let ids: Vec<String> = log
            .read_all()
            .unwrap()
            .into_iter()
            .map(|r| r.txn_uuid)
            .collect();
        assert_eq!(ids, ["a", "b", "c"]);
    }
    // Swapping two lines breaks their ordinal binding.
    let text = std::fs::read_to_string(&path).unwrap();
    let mut lines: Vec<&str> = text.lines().collect();
    lines.swap(0, 1);
    std::fs::write(&path, lines.join("\n") + "\n").unwrap();
    let log = TxnIntentLog::open_with_encryption(&path, key()).unwrap();
    assert!(log.read_all().is_err());
}

// ── PITR markers ─────────────────────────────────────────────────────────────

fn append_raw_frame(dir: &Path, payload: Vec<u8>) {
    let w = WalWriter::open(
        WalConfig::default()
            .with_dir(dir.join("wal"))
            .with_fsync(false)
            .with_encryption_key_id(Some(key().key_id())),
    )
    .unwrap();
    w.append(payload).unwrap();
    w.flush().unwrap();
}

fn fake_marker(id: u64, lsn: u64) -> SnapshotMarker {
    SnapshotMarker {
        snapshot_id: id,
        lsn,
        timestamp_ns: id,
        table_name: "default".into(),
        tier_files: vec![],
    }
}

#[test]
fn pitr_markers_are_macd_and_verified_on_use() {
    let tmp = TempDir::new().unwrap();
    let real = {
        let e = open_keyed(tmp.path()).unwrap();
        e.insert(1, vec![1, 1]).unwrap();
        let m = e.create_snapshot().unwrap();
        e.insert(2, vec![2, 2]).unwrap();
        m
    };
    // A marker without a MAC (as an attacker, or an old version, writes it).
    append_raw_frame(tmp.path(), fake_marker(7, 1).encode());

    let e = open_keyed(tmp.path()).unwrap();
    let listed = e.list_snapshots().unwrap();
    assert_eq!(
        listed.iter().map(|m| m.snapshot_id).collect::<Vec<_>>(),
        [real.snapshot_id],
        "unauthenticated marker listed"
    );
    drop(e);

    let dst = tmp.path().join("restore-bad");
    let msg = err_text(TableEngine::restore_to_marker_with(
        schema(),
        tmp.path(),
        &dst,
        7,
        key(),
    ));
    assert!(msg.contains("has no MAC"), "{msg}");

    let dst = tmp.path().join("restore-ok");
    let restored =
        TableEngine::restore_to_marker_with(schema(), tmp.path(), &dst, real.snapshot_id, key())
            .unwrap();
    assert!(restored.get(1).unwrap().is_some());
    assert!(restored.get(2).unwrap().is_none());
}

#[test]
fn pitr_marker_with_forged_mac_is_rejected() {
    let tmp = TempDir::new().unwrap();
    {
        let e = open_keyed(tmp.path()).unwrap();
        e.insert(1, vec![1, 1]).unwrap();
    }
    let mut v = serde_json::to_value(fake_marker(9, 1)).unwrap();
    v["mac"] = serde_json::Value::String("00".repeat(32));
    let mut payload = vec![vectorvolt_wal::snapshot::OP_SNAPSHOT_MARKER];
    payload.extend(serde_json::to_vec(&v).unwrap());
    append_raw_frame(tmp.path(), payload);
    let e = open_keyed(tmp.path()).unwrap();
    let msg = err_text(e.list_snapshots());
    assert!(msg.contains("forged"), "{msg}");
}

// ── A12 follow-up: deleted tail.seal ─────────────────────────────────────────

fn seal_path(dir: &Path) -> PathBuf {
    dir.join("wal").join("tail.seal")
}

#[test]
fn deleted_tail_seal_on_sealed_table_is_rejected() {
    let tmp = TempDir::new().unwrap();
    {
        let e = open_keyed(tmp.path()).unwrap();
        e.insert(1, vec![1, 1]).unwrap();
    }
    assert!(seal_path(tmp.path()).exists());
    std::fs::remove_file(seal_path(tmp.path())).unwrap();
    let msg = err_text(open_keyed(tmp.path()));
    assert!(
        msg.contains("tail.seal") && msg.contains("seal was deleted"),
        "{msg}"
    );
}

#[test]
fn brand_new_keyed_table_without_seal_opens_and_gets_sealed() {
    let tmp = TempDir::new().unwrap();
    assert!(!seal_path(tmp.path()).exists());
    let e = open_keyed(tmp.path()).unwrap();
    assert!(e.table_id().is_some());
    assert!(seal_path(tmp.path()).exists());
    drop(e);
    // Crash window: the table marker was synced but its seal never landed.
    std::fs::remove_file(seal_path(tmp.path())).unwrap();
    let e = open_keyed(tmp.path()).unwrap();
    assert!(seal_path(tmp.path()).exists(), "not resealed");
    e.insert(1, vec![1, 1]).unwrap();
}

#[test]
fn legacy_unsealed_keyed_table_still_opens() {
    // A keyed table written with `Buffered` durability never sealed a
    // commit; nothing records that it was sealed, so it must open.
    let tmp = TempDir::new().unwrap();
    {
        let e = TableEngine::open_with(schema(), tmp.path(), opts()).unwrap();
        e.insert(1, vec![1, 1]).unwrap();
    }
    assert!(!seal_path(tmp.path()).exists());
    let e = open_keyed(tmp.path()).unwrap();
    assert!(e.get(1).unwrap().is_some());
}

// ── cold restart of an encrypted table (C3 wiring) ───────────────────────────

fn fresh_key() -> Arc<KeyStore> {
    // A new Arc, as a new process would build: nothing registered earlier
    // can still be alive once the previous engine is dropped.
    Arc::new(KeyStore::from_hex(&key().to_hex()).unwrap())
}

#[test]
fn encrypted_partitions_reopen_cold_without_quarantine() {
    let tmp = TempDir::new().unwrap();
    {
        let k = fresh_key();
        let e = TableEngine::open_with_encryption(schema(), tmp.path(), opts(), k).unwrap();
        for pk in 1..=50 {
            e.insert(pk, vec![pk, MARKER]).unwrap();
        }
        e.flush().unwrap();
    }
    {
        let e =
            TableEngine::open_with_encryption(schema(), tmp.path(), opts(), fresh_key()).unwrap();
        assert!(e.quarantined_partitions().is_empty());
        assert_eq!(e.get(7).unwrap().unwrap().values, vec![7, MARKER]);
        e.take_snapshot().unwrap();
    }
    let e = TableEngine::open_with_encryption(schema(), tmp.path(), opts(), fresh_key()).unwrap();
    assert!(e.quarantined_partitions().is_empty());
    assert_eq!(e.snapshot_info().unwrap().unwrap().row_count, 50);
    for pk in [1, 25, 50] {
        assert_eq!(e.get(pk).unwrap().unwrap().values, vec![pk, MARKER]);
    }
    assert_eq!(e.scan(0, 0, u64::MAX).unwrap().len(), 50);
}
