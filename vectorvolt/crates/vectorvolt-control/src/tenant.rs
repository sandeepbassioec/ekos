use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tenant {
    pub id: String,
    pub name: String,
    pub token: String,
    pub quota: TenantQuota,
    pub tables: Vec<String>,
    pub created_at_ns: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantQuota {
    pub max_tables: usize,
    pub max_rows_per_table: u64,
    pub max_bytes_total: u64,
}

impl Default for TenantQuota {
    fn default() -> Self {
        Self {
            max_tables: 100,
            max_rows_per_table: 10_000_000,
            max_bytes_total: 10 * 1024 * 1024 * 1024,
        }
    }
}

#[derive(Debug, Error)]
pub enum TenantError {
    #[error("duplicate id")]
    DuplicateId,
    #[error("not found")]
    NotFound,
    #[error("quota exceeded: {0}")]
    QuotaExceeded(String),
}

pub struct TenantRegistry {
    tenants: DashMap<String, Tenant>,
}

impl TenantRegistry {
    pub fn new() -> Self {
        Self {
            tenants: DashMap::new(),
        }
    }

    pub fn create(
        &self,
        id: String,
        name: String,
        token: String,
        quota: TenantQuota,
    ) -> Result<Tenant, TenantError> {
        if self.tenants.contains_key(&id) {
            return Err(TenantError::DuplicateId);
        }
        let t = Tenant {
            id: id.clone(),
            name,
            token,
            quota,
            tables: Vec::new(),
            created_at_ns: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64,
        };
        self.tenants.insert(id, t.clone());
        Ok(t)
    }

    pub fn authenticate(&self, id: &str, token: &str) -> Result<Tenant, TenantError> {
        let t = self.tenants.get(id).ok_or(TenantError::NotFound)?;
        if t.token == token {
            Ok(t.clone())
        } else {
            Err(TenantError::NotFound)
        }
    }

    pub fn get(&self, id: &str) -> Result<Tenant, TenantError> {
        self.tenants
            .get(id)
            .map(|t| t.clone())
            .ok_or(TenantError::NotFound)
    }

    pub fn assign_table(&self, id: &str, table: String) -> Result<(), TenantError> {
        let mut t = self.tenants.get_mut(id).ok_or(TenantError::NotFound)?;
        self.check_quota_inner(&t, "tables")?;
        if !t.tables.contains(&table) {
            t.tables.push(table);
        }
        Ok(())
    }

    fn check_quota_inner(&self, t: &Tenant, kind: &str) -> Result<(), TenantError> {
        if kind == "tables" && t.tables.len() >= t.quota.max_tables {
            return Err(TenantError::QuotaExceeded(format!(
                "max_tables ({}) reached",
                t.quota.max_tables
            )));
        }
        Ok(())
    }

    pub fn check_quota(&self, id: &str, kind: &str) -> Result<(), TenantError> {
        let t = self.tenants.get(id).ok_or(TenantError::NotFound)?;
        self.check_quota_inner(&t, kind)
    }

    pub fn list(&self) -> Vec<Tenant> {
        self.tenants.iter().map(|e| e.value().clone()).collect()
    }

    pub fn remove(&self, id: &str) -> Result<Tenant, TenantError> {
        self.tenants
            .remove(id)
            .map(|(_, v)| v)
            .ok_or(TenantError::NotFound)
    }
}

impl Default for TenantRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reg() -> TenantRegistry {
        TenantRegistry::new()
    }

    fn quota() -> TenantQuota {
        TenantQuota::default()
    }

    #[test]
    fn test_create_tenant() {
        let r = reg();
        let t = r
            .create("t1".into(), "Tenant 1".into(), "tok".into(), quota())
            .unwrap();
        assert_eq!(t.id, "t1");
    }

    #[test]
    fn test_duplicate_id() {
        let r = reg();
        r.create("t1".into(), "A".into(), "tok".into(), quota())
            .unwrap();
        let err = r
            .create("t1".into(), "B".into(), "tok2".into(), quota())
            .unwrap_err();
        assert!(matches!(err, TenantError::DuplicateId));
    }

    #[test]
    fn test_get_existing() {
        let r = reg();
        r.create("t1".into(), "A".into(), "tok".into(), quota())
            .unwrap();
        let t = r.get("t1").unwrap();
        assert_eq!(t.name, "A");
    }

    #[test]
    fn test_get_missing() {
        let r = reg();
        let err = r.get("nope").unwrap_err();
        assert!(matches!(err, TenantError::NotFound));
    }

    #[test]
    fn test_authenticate_ok() {
        let r = reg();
        r.create("t1".into(), "A".into(), "secret".into(), quota())
            .unwrap();
        let t = r.authenticate("t1", "secret").unwrap();
        assert_eq!(t.id, "t1");
    }

    #[test]
    fn test_authenticate_wrong_token() {
        let r = reg();
        r.create("t1".into(), "A".into(), "secret".into(), quota())
            .unwrap();
        let err = r.authenticate("t1", "wrong").unwrap_err();
        assert!(matches!(err, TenantError::NotFound));
    }

    #[test]
    fn test_assign_table() {
        let r = reg();
        r.create("t1".into(), "A".into(), "tok".into(), quota())
            .unwrap();
        r.assign_table("t1", "orders".into()).unwrap();
        let t = r.get("t1").unwrap();
        assert!(t.tables.contains(&"orders".to_string()));
    }

    #[test]
    fn test_assign_table_no_duplicate() {
        let r = reg();
        r.create("t1".into(), "A".into(), "tok".into(), quota())
            .unwrap();
        r.assign_table("t1", "orders".into()).unwrap();
        r.assign_table("t1", "orders".into()).unwrap();
        let t = r.get("t1").unwrap();
        assert_eq!(t.tables.len(), 1);
    }

    #[test]
    fn test_quota_exceeded() {
        let r = reg();
        let q = TenantQuota {
            max_tables: 1,
            max_rows_per_table: 100,
            max_bytes_total: 100,
        };
        r.create("t1".into(), "A".into(), "tok".into(), q).unwrap();
        r.assign_table("t1", "tbl1".into()).unwrap();
        let err = r.assign_table("t1", "tbl2".into()).unwrap_err();
        assert!(matches!(err, TenantError::QuotaExceeded(_)));
    }

    #[test]
    fn test_list() {
        let r = reg();
        r.create("t1".into(), "A".into(), "tok1".into(), quota())
            .unwrap();
        r.create("t2".into(), "B".into(), "tok2".into(), quota())
            .unwrap();
        let list = r.list();
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn test_remove() {
        let r = reg();
        r.create("t1".into(), "A".into(), "tok".into(), quota())
            .unwrap();
        r.remove("t1").unwrap();
        assert!(r.get("t1").is_err());
    }

    #[test]
    fn test_remove_missing() {
        let r = reg();
        let err = r.remove("nope").unwrap_err();
        assert!(matches!(err, TenantError::NotFound));
    }

    #[test]
    fn test_created_at_set() {
        let r = reg();
        let t = r
            .create("t1".into(), "A".into(), "tok".into(), quota())
            .unwrap();
        assert!(t.created_at_ns > 0);
    }
}
