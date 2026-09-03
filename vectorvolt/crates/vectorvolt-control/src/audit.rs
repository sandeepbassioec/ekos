use serde::{Deserialize, Serialize};
use std::sync::{
    Mutex,
    atomic::{AtomicU64, Ordering},
};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: u64,
    pub tenant_id: String,
    pub user_id: String,
    pub action: String,
    pub resource: String,
    pub timestamp_ns: u64,
    pub success: bool,
    pub detail: Option<String>,
}

pub struct AuditLog {
    events: Mutex<Vec<AuditEvent>>,
    next_id: AtomicU64,
}

impl AuditLog {
    pub fn new() -> Self {
        Self {
            events: Mutex::new(Vec::new()),
            next_id: AtomicU64::new(1),
        }
    }

    pub fn record(
        &self,
        tenant_id: String,
        user_id: String,
        action: String,
        resource: String,
        success: bool,
        detail: Option<String>,
    ) -> AuditEvent {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let ev = AuditEvent {
            id,
            tenant_id,
            user_id,
            action,
            resource,
            timestamp_ns: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64,
            success,
            detail,
        };
        self.events.lock().unwrap().push(ev.clone());
        ev
    }

    pub fn query(&self, tenant_id: &str) -> Vec<AuditEvent> {
        self.events
            .lock()
            .unwrap()
            .iter()
            .filter(|e| e.tenant_id == tenant_id)
            .cloned()
            .collect()
    }

    pub fn query_all(&self) -> Vec<AuditEvent> {
        self.events.lock().unwrap().clone()
    }

    pub fn since(&self, timestamp_ns: u64) -> Vec<AuditEvent> {
        self.events
            .lock()
            .unwrap()
            .iter()
            .filter(|e| e.timestamp_ns >= timestamp_ns)
            .cloned()
            .collect()
    }

    pub fn count(&self) -> usize {
        self.events.lock().unwrap().len()
    }

    pub fn export_json(&self) -> String {
        let events = self.events.lock().unwrap().clone();
        serde_json::to_string(&events).unwrap_or_else(|_| "[]".into())
    }
}

impl Default for AuditLog {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn log() -> AuditLog {
        AuditLog::new()
    }

    fn record_event(al: &AuditLog, tenant: &str, user: &str, action: &str) -> AuditEvent {
        al.record(
            tenant.into(),
            user.into(),
            action.into(),
            "resource".into(),
            true,
            None,
        )
    }

    #[test]
    fn test_record_increments_id() {
        let al = log();
        let e1 = record_event(&al, "t1", "u1", "INSERT");
        let e2 = record_event(&al, "t1", "u1", "DELETE");
        assert!(e2.id > e1.id);
    }

    #[test]
    fn test_count() {
        let al = log();
        record_event(&al, "t1", "u1", "INSERT");
        record_event(&al, "t1", "u1", "INSERT");
        assert_eq!(al.count(), 2);
    }

    #[test]
    fn test_query_by_tenant() {
        let al = log();
        record_event(&al, "t1", "u1", "INSERT");
        record_event(&al, "t2", "u1", "INSERT");
        let evs = al.query("t1");
        assert_eq!(evs.len(), 1);
        assert_eq!(evs[0].tenant_id, "t1");
    }

    #[test]
    fn test_query_all() {
        let al = log();
        record_event(&al, "t1", "u1", "A");
        record_event(&al, "t2", "u2", "B");
        assert_eq!(al.query_all().len(), 2);
    }

    #[test]
    fn test_since_filter() {
        let al = log();
        let e = record_event(&al, "t1", "u1", "INSERT");
        let future_ts = e.timestamp_ns + 1_000_000_000_000;
        let before = al.since(0);
        let after = al.since(future_ts);
        assert!(!before.is_empty());
        assert!(after.is_empty());
    }

    #[test]
    fn test_export_json_valid() {
        let al = log();
        record_event(&al, "t1", "u1", "INSERT");
        let json = al.export_json();
        let parsed: Vec<AuditEvent> = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.len(), 1);
    }

    #[test]
    fn test_export_json_empty() {
        let al = log();
        let json = al.export_json();
        assert_eq!(json, "[]");
    }

    #[test]
    fn test_success_field_false() {
        let al = log();
        let ev = al.record(
            "t1".into(),
            "u1".into(),
            "DELETE".into(),
            "tbl".into(),
            false,
            Some("denied".into()),
        );
        assert!(!ev.success);
        assert_eq!(ev.detail.as_deref(), Some("denied"));
    }

    #[test]
    fn test_timestamp_nonzero() {
        let al = log();
        let ev = record_event(&al, "t1", "u1", "INSERT");
        assert!(ev.timestamp_ns > 0);
    }

    #[test]
    fn test_query_empty_tenant() {
        let al = log();
        record_event(&al, "t1", "u1", "INSERT");
        let evs = al.query("t99");
        assert!(evs.is_empty());
    }

    #[test]
    fn test_multiple_tenants_isolated() {
        let al = log();
        for _ in 0..5 {
            record_event(&al, "t1", "u1", "INSERT");
        }
        for _ in 0..3 {
            record_event(&al, "t2", "u2", "SELECT");
        }
        assert_eq!(al.query("t1").len(), 5);
        assert_eq!(al.query("t2").len(), 3);
    }

    #[test]
    fn test_ids_unique() {
        let al = log();
        let e1 = record_event(&al, "t1", "u1", "A");
        let e2 = record_event(&al, "t1", "u1", "B");
        let e3 = record_event(&al, "t1", "u1", "C");
        assert_ne!(e1.id, e2.id);
        assert_ne!(e2.id, e3.id);
    }

    #[test]
    fn test_detail_some() {
        let al = log();
        let ev = al.record(
            "t".into(),
            "u".into(),
            "act".into(),
            "res".into(),
            true,
            Some("extra".into()),
        );
        assert_eq!(ev.detail.as_deref(), Some("extra"));
    }
}
