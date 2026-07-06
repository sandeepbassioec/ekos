//! Integration tests for all 6 Phase 1 KoStore syscalls.

use ekos_kernel::{KnowledgeObject, KoKind, KoStatus, KoStore};
use tempfile::TempDir;
use vectorvolt_store::StoreConfig;

fn open(dir: &std::path::Path) -> KoStore {
    KoStore::open(
        StoreConfig::new(dir)
            .with_fsync(false)
            .with_flush_threshold(256),
    )
    .unwrap()
}

fn req(ns: &str, kind: &str, payload: &str) -> KnowledgeObject {
    KnowledgeObject::new(ns, KoKind::new(kind), payload.as_bytes().to_vec())
}

// ── Syscall 1: ko_put ─────────────────────────────────────────────────────

#[test]
fn ko_put_creates_version_1() {
    let dir = TempDir::new().unwrap();
    let store = open(dir.path());

    let ko = req("ekos", "Requirement", r#"{"title":"REQ-001"}"#);
    let saved = store.ko_put(ko).unwrap();

    assert_eq!(saved.version, 1);
    assert_eq!(saved.kind.as_str(), "Requirement");
    assert_eq!(saved.namespace, "ekos");
}

#[test]
fn ko_put_increments_version_on_update() {
    let dir = TempDir::new().unwrap();
    let store = open(dir.path());

    let ko = req("ekos", "Decision", r#"{"title":"ADR-001"}"#);
    let v1 = store.ko_put(ko).unwrap();

    let updated = v1.next_version(r#"{"title":"ADR-001","status":"approved"}"#.as_bytes());
    let v2 = store.ko_put(updated).unwrap();

    assert_eq!(v2.version, 2);
    assert_eq!(v2.id, v1.id);
}

#[test]
fn ko_put_assigns_tx_time() {
    let dir = TempDir::new().unwrap();
    let store = open(dir.path());
    let ko = req("ekos", "Component", "{}");
    let saved = store.ko_put(ko).unwrap();
    // tx_time must be a real timestamp (not epoch zero)
    assert!(saved.tx_time.timestamp() > 0);
}

// ── Syscall 2: ko_get ─────────────────────────────────────────────────────

#[test]
fn ko_get_returns_latest_version() {
    let dir = TempDir::new().unwrap();
    let store = open(dir.path());

    let ko = req("ekos", "Requirement", r#"{"title":"v1"}"#);
    let v1 = store.ko_put(ko).unwrap();
    let id = v1.id.as_str();

    let v2 = store.ko_put(v1.next_version(r#"{"title":"v2"}"#.as_bytes())).unwrap();
    let _v3 = store.ko_put(v2.next_version(r#"{"title":"v3"}"#.as_bytes())).unwrap();

    let fetched = store.ko_get("ekos", &id).unwrap();
    assert_eq!(fetched.version, 3);
    assert_eq!(fetched.payload, r#"{"title":"v3"}"#.as_bytes());
}

#[test]
fn ko_get_not_found_returns_error() {
    let dir = TempDir::new().unwrap();
    let store = open(dir.path());
    let result = store.ko_get("ekos", "00000000-0000-0000-0000-000000000000");
    assert!(matches!(result, Err(ekos_kernel::KernelError::NotFound(_))));
}

// ── Syscall 3: ko_delete ──────────────────────────────────────────────────

#[test]
fn ko_delete_marks_as_deleted() {
    let dir = TempDir::new().unwrap();
    let store = open(dir.path());

    let ko = req("ekos", "Risk", r#"{"title":"RISK-001"}"#);
    let saved = store.ko_put(ko).unwrap();
    let id = saved.id.as_str();

    store.ko_delete("ekos", &id).unwrap();

    let result = store.ko_get("ekos", &id);
    assert!(matches!(result, Err(ekos_kernel::KernelError::Deleted(_))));
}

#[test]
fn ko_delete_twice_returns_error() {
    let dir = TempDir::new().unwrap();
    let store = open(dir.path());

    let ko = req("ekos", "Risk", "{}");
    let saved = store.ko_put(ko).unwrap();
    let id = saved.id.as_str();

    store.ko_delete("ekos", &id).unwrap();
    let result = store.ko_delete("ekos", &id);
    assert!(matches!(result, Err(ekos_kernel::KernelError::Deleted(_))));
}

#[test]
fn ko_delete_excluded_from_scan() {
    let dir = TempDir::new().unwrap();
    let store = open(dir.path());

    let alive = store.ko_put(req("ekos", "Requirement", r#"{"title":"alive"}"#)).unwrap();
    let dead  = store.ko_put(req("ekos", "Requirement", r#"{"title":"dead"}"#)).unwrap();
    store.ko_delete("ekos", &dead.id.as_str()).unwrap();

    let results = store.ko_scan("ekos", None, None).unwrap();
    let ids: Vec<_> = results.iter().map(|k| k.id.as_str()).collect();
    assert!(ids.contains(&alive.id.as_str()));
    assert!(!ids.contains(&dead.id.as_str()));
}

// ── Syscall 4: ko_scan ────────────────────────────────────────────────────

#[test]
fn ko_scan_returns_all_active_in_namespace() {
    let dir = TempDir::new().unwrap();
    let store = open(dir.path());

    store.ko_put(req("ekos", "Requirement", "{}")).unwrap();
    store.ko_put(req("ekos", "Decision",    "{}")).unwrap();
    store.ko_put(req("ekos", "Component",   "{}")).unwrap();

    let results = store.ko_scan("ekos", None, None).unwrap();
    assert_eq!(results.len(), 3);
}

#[test]
fn ko_scan_filtered_by_kind() {
    let dir = TempDir::new().unwrap();
    let store = open(dir.path());

    store.ko_put(req("ekos", "Requirement", "{}")).unwrap();
    store.ko_put(req("ekos", "Requirement", "{}")).unwrap();
    store.ko_put(req("ekos", "Decision",    "{}")).unwrap();

    let reqs = store.ko_scan("ekos", Some("Requirement"), None).unwrap();
    assert_eq!(reqs.len(), 2);
    assert!(reqs.iter().all(|ko| ko.kind.as_str() == "Requirement"));
}

#[test]
fn ko_scan_namespace_isolation() {
    let dir = TempDir::new().unwrap();
    let store = open(dir.path());

    store.ko_put(req("ekos",  "Requirement", "{}")).unwrap();
    store.ko_put(req("esip",  "Requirement", "{}")).unwrap();

    let ekos_results = store.ko_scan("ekos", None, None).unwrap();
    let esip_results = store.ko_scan("esip", None, None).unwrap();

    assert_eq!(ekos_results.len(), 1);
    assert_eq!(esip_results.len(), 1);
    assert_ne!(ekos_results[0].id, esip_results[0].id);
}

#[test]
fn ko_scan_with_limit() {
    let dir = TempDir::new().unwrap();
    let store = open(dir.path());

    for _ in 0..10 {
        store.ko_put(req("ekos", "Requirement", "{}")).unwrap();
    }

    let results = store.ko_scan("ekos", None, Some(3)).unwrap();
    assert_eq!(results.len(), 3);
}

// ── Syscall 5: ko_history ─────────────────────────────────────────────────

#[test]
fn ko_history_returns_all_versions() {
    let dir = TempDir::new().unwrap();
    let store = open(dir.path());

    let ko = req("ekos", "Decision", r#"{"status":"draft"}"#);
    let v1 = store.ko_put(ko).unwrap();
    let v2 = store.ko_put(v1.next_version(r#"{"status":"proposed"}"#.as_bytes())).unwrap();
    let _v3 = store.ko_put(v2.next_version(r#"{"status":"approved"}"#.as_bytes())).unwrap();

    let id = v1.id.as_str();
    let history = store.ko_history("ekos", &id).unwrap();

    assert_eq!(history.len(), 3);
    assert_eq!(history[0].version, 1);
    assert_eq!(history[1].version, 2);
    assert_eq!(history[2].version, 3);
}

#[test]
fn ko_history_includes_deleted_tombstone() {
    let dir = TempDir::new().unwrap();
    let store = open(dir.path());

    let ko = req("ekos", "Risk", "{}");
    let saved = store.ko_put(ko).unwrap();
    let id = saved.id.as_str();
    store.ko_delete("ekos", &id).unwrap();

    let history = store.ko_history("ekos", &id).unwrap();
    assert_eq!(history.len(), 2);
    assert_eq!(history.last().unwrap().status, KoStatus::Deleted);
}

#[test]
fn ko_history_not_found_returns_error() {
    let dir = TempDir::new().unwrap();
    let store = open(dir.path());
    let result = store.ko_history("ekos", "00000000-0000-0000-0000-000000000000");
    assert!(matches!(result, Err(ekos_kernel::KernelError::NotFound(_))));
}

// ── Syscall 6: ko_query_tag ───────────────────────────────────────────────

#[test]
fn ko_query_tag_finds_tagged_kos() {
    let dir = TempDir::new().unwrap();
    let store = open(dir.path());

    let ko1 = req("ekos", "Requirement", "{}").with_tags(["phase-1", "critical"]);
    let ko2 = req("ekos", "Requirement", "{}").with_tags(["phase-2"]);
    let ko3 = req("ekos", "Decision",    "{}").with_tags(["phase-1"]);

    store.ko_put(ko1).unwrap();
    store.ko_put(ko2).unwrap();
    store.ko_put(ko3).unwrap();

    let phase1 = store.ko_query_tag("ekos", "phase-1", None).unwrap();
    assert_eq!(phase1.len(), 2);

    let critical = store.ko_query_tag("ekos", "critical", None).unwrap();
    assert_eq!(critical.len(), 1);

    let phase2 = store.ko_query_tag("ekos", "phase-2", None).unwrap();
    assert_eq!(phase2.len(), 1);
}

#[test]
fn ko_query_tag_deleted_kos_excluded() {
    let dir = TempDir::new().unwrap();
    let store = open(dir.path());

    let ko = req("ekos", "Risk", "{}").with_tags(["active"]);
    let saved = store.ko_put(ko).unwrap();
    store.ko_delete("ekos", &saved.id.as_str()).unwrap();

    let results = store.ko_query_tag("ekos", "active", None).unwrap();
    assert_eq!(results.len(), 0);
}

#[test]
fn ko_query_tag_updates_when_tags_change() {
    let dir = TempDir::new().unwrap();
    let store = open(dir.path());

    let ko = req("ekos", "Requirement", "{}").with_tags(["old-tag"]);
    let v1 = store.ko_put(ko).unwrap();

    // Update with different tags
    let mut v2 = v1.next_version(b"updated");
    v2.tags = vec!["new-tag".to_string()];
    store.ko_put(v2).unwrap();

    assert_eq!(store.ko_query_tag("ekos", "old-tag", None).unwrap().len(), 0);
    assert_eq!(store.ko_query_tag("ekos", "new-tag", None).unwrap().len(), 1);
}
