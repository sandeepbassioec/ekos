use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Permission {
    TableRead,
    TableWrite,
    TableAdmin,
    ClusterRead,
    ClusterAdmin,
    TenantAdmin,
    MetricsRead,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub name: String,
    pub permissions: HashSet<Permission>,
}

#[derive(Debug, Error)]
pub enum RbacError {
    #[error("role not found: {0}")]
    RoleNotFound(String),
}

pub struct RbacStore {
    roles: DashMap<String, Role>,
    user_roles: DashMap<String, Vec<String>>,
}

impl RbacStore {
    pub fn new() -> Self {
        Self {
            roles: DashMap::new(),
            user_roles: DashMap::new(),
        }
    }

    pub fn predefined_roles() -> Vec<Role> {
        use Permission::*;
        vec![
            Role {
                name: "Admin".into(),
                permissions: [
                    TableRead,
                    TableWrite,
                    TableAdmin,
                    ClusterRead,
                    ClusterAdmin,
                    TenantAdmin,
                    MetricsRead,
                ]
                .into(),
            },
            Role {
                name: "Operator".into(),
                permissions: [TableRead, TableWrite, TableAdmin, ClusterRead, MetricsRead].into(),
            },
            Role {
                name: "ReadOnly".into(),
                permissions: [TableRead, MetricsRead].into(),
            },
        ]
    }

    pub fn define_role(&self, role: Role) {
        self.roles.insert(role.name.clone(), role);
    }

    pub fn load_predefined(&self) {
        for r in Self::predefined_roles() {
            self.define_role(r);
        }
    }

    pub fn assign_role(&self, user_id: &str, role_name: &str) -> Result<(), RbacError> {
        if !self.roles.contains_key(role_name) {
            return Err(RbacError::RoleNotFound(role_name.into()));
        }
        let mut entry = self.user_roles.entry(user_id.to_string()).or_default();
        if !entry.contains(&role_name.to_string()) {
            entry.push(role_name.to_string());
        }
        Ok(())
    }

    pub fn revoke_role(&self, user_id: &str, role_name: &str) {
        if let Some(mut roles) = self.user_roles.get_mut(user_id) {
            roles.retain(|r| r != role_name);
        }
    }

    pub fn has_permission(&self, user_id: &str, perm: &Permission) -> bool {
        let role_names = match self.user_roles.get(user_id) {
            Some(r) => r.clone(),
            None => return false,
        };
        for rname in &role_names {
            if let Some(role) = self.roles.get(rname) && role.permissions.contains(perm) {
                return true;
            }
        }
        false
    }

    pub fn get_role(&self, name: &str) -> Result<Role, RbacError> {
        self.roles
            .get(name)
            .map(|r| r.clone())
            .ok_or_else(|| RbacError::RoleNotFound(name.into()))
    }

    pub fn user_permissions(&self, user_id: &str) -> HashSet<Permission> {
        let role_names = match self.user_roles.get(user_id) {
            Some(r) => r.clone(),
            None => return HashSet::new(),
        };
        let mut perms = HashSet::new();
        for rname in &role_names {
            if let Some(role) = self.roles.get(rname) {
                perms.extend(role.permissions.iter().cloned());
            }
        }
        perms
    }

    pub fn list_roles(&self) -> Vec<Role> {
        self.roles.iter().map(|e| e.value().clone()).collect()
    }

    pub fn user_roles(&self, user_id: &str) -> Vec<String> {
        self.user_roles
            .get(user_id)
            .map(|v| v.clone())
            .unwrap_or_default()
    }
}

impl Default for RbacStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store_with_predefined() -> RbacStore {
        let s = RbacStore::new();
        s.load_predefined();
        s
    }

    #[test]
    fn test_predefined_roles_count() {
        let roles = RbacStore::predefined_roles();
        assert_eq!(roles.len(), 3);
    }

    #[test]
    fn test_admin_has_all_perms() {
        let roles = RbacStore::predefined_roles();
        let admin = roles.iter().find(|r| r.name == "Admin").unwrap();
        assert!(admin.permissions.contains(&Permission::ClusterAdmin));
        assert!(admin.permissions.contains(&Permission::TenantAdmin));
    }

    #[test]
    fn test_operator_no_cluster_admin() {
        let roles = RbacStore::predefined_roles();
        let op = roles.iter().find(|r| r.name == "Operator").unwrap();
        assert!(!op.permissions.contains(&Permission::ClusterAdmin));
    }

    #[test]
    fn test_readonly_only_read() {
        let roles = RbacStore::predefined_roles();
        let ro = roles.iter().find(|r| r.name == "ReadOnly").unwrap();
        assert!(ro.permissions.contains(&Permission::TableRead));
        assert!(!ro.permissions.contains(&Permission::TableWrite));
    }

    #[test]
    fn test_assign_and_check_permission() {
        let s = store_with_predefined();
        s.assign_role("user1", "Admin").unwrap();
        assert!(s.has_permission("user1", &Permission::ClusterAdmin));
    }

    #[test]
    fn test_no_roles_no_permission() {
        let s = store_with_predefined();
        assert!(!s.has_permission("user2", &Permission::TableRead));
    }

    #[test]
    fn test_assign_unknown_role_err() {
        let s = RbacStore::new();
        let err = s.assign_role("u1", "Ghost").unwrap_err();
        assert!(matches!(err, RbacError::RoleNotFound(_)));
    }

    #[test]
    fn test_revoke_role() {
        let s = store_with_predefined();
        s.assign_role("u1", "Admin").unwrap();
        s.revoke_role("u1", "Admin");
        assert!(!s.has_permission("u1", &Permission::ClusterAdmin));
    }

    #[test]
    fn test_user_permissions_aggregate() {
        let s = store_with_predefined();
        s.assign_role("u1", "ReadOnly").unwrap();
        s.assign_role("u1", "Operator").unwrap();
        let perms = s.user_permissions("u1");
        assert!(perms.contains(&Permission::TableWrite));
        assert!(perms.contains(&Permission::TableRead));
    }

    #[test]
    fn test_get_role_ok() {
        let s = store_with_predefined();
        let r = s.get_role("Admin").unwrap();
        assert_eq!(r.name, "Admin");
    }

    #[test]
    fn test_get_role_missing() {
        let s = RbacStore::new();
        let err = s.get_role("Nope").unwrap_err();
        assert!(matches!(err, RbacError::RoleNotFound(_)));
    }

    #[test]
    fn test_list_roles() {
        let s = store_with_predefined();
        assert_eq!(s.list_roles().len(), 3);
    }

    #[test]
    fn test_user_roles_list() {
        let s = store_with_predefined();
        s.assign_role("u1", "Admin").unwrap();
        s.assign_role("u1", "ReadOnly").unwrap();
        let roles = s.user_roles("u1");
        assert_eq!(roles.len(), 2);
    }
}
