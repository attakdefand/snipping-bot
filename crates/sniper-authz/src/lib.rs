//! Role-Based Access Control (RBAC) for the snipping bot
//!
//! This crate provides role-based access control functionality
//! for securing the snipping bot services.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use tracing::debug;

/// User identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(pub String);

/// Role identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RoleId(pub String);

/// Permission identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PermissionId(pub String);

/// User information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub roles: HashSet<RoleId>,
}

/// Role information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub id: RoleId,
    pub permissions: HashSet<PermissionId>,
    pub name: String,
    pub description: Option<String>,
}

/// Permission information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    pub id: PermissionId,
    pub name: String,
    pub description: Option<String>,
}

/// Authorization manager
pub struct AuthzManager {
    users: HashMap<UserId, User>,
    roles: HashMap<RoleId, Role>,
    permissions: HashMap<PermissionId, Permission>,
}

impl AuthzManager {
    /// Create a new authorization manager
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
            roles: HashMap::new(),
            permissions: HashMap::new(),
        }
    }

    /// Add a permission
    pub fn add_permission(&mut self, permission: Permission) -> Result<()> {
        debug!("Adding permission: {}", permission.id.0);
        self.permissions.insert(permission.id.clone(), permission);
        Ok(())
    }

    /// Add a role
    pub fn add_role(&mut self, role: Role) -> Result<()> {
        debug!("Adding role: {}", role.id.0);
        self.roles.insert(role.id.clone(), role);
        Ok(())
    }

    /// Add a user
    pub fn add_user(&mut self, user: User) -> Result<()> {
        debug!("Adding user: {}", user.id.0);
        self.users.insert(user.id.clone(), user);
        Ok(())
    }

    /// Assign a role to a user
    pub fn assign_role_to_user(&mut self, user_id: &UserId, role_id: &RoleId) -> Result<()> {
        debug!("Assigning role {} to user {}", role_id.0, user_id.0);

        if !self.roles.contains_key(role_id) {
            return Err(anyhow::anyhow!("Role {} not found", role_id.0));
        }

        if let Some(user) = self.users.get_mut(user_id) {
            user.roles.insert(role_id.clone());
            Ok(())
        } else {
            Err(anyhow::anyhow!("User {} not found", user_id.0))
        }
    }

    /// Check if a user has a specific permission
    pub fn user_has_permission(&self, user_id: &UserId, permission_id: &PermissionId) -> bool {
        debug!(
            "Checking if user {} has permission {}",
            user_id.0, permission_id.0
        );

        if let Some(user) = self.users.get(user_id) {
            // Check if any of the user's roles has the permission
            for role_id in &user.roles {
                if let Some(role) = self.roles.get(role_id) {
                    if role.permissions.contains(permission_id) {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Get all permissions for a user
    pub fn get_user_permissions(&self, user_id: &UserId) -> Result<HashSet<PermissionId>> {
        debug!("Getting permissions for user {}", user_id.0);

        if let Some(user) = self.users.get(user_id) {
            let mut permissions = HashSet::new();

            // Collect permissions from all user's roles
            for role_id in &user.roles {
                if let Some(role) = self.roles.get(role_id) {
                    permissions.extend(role.permissions.iter().cloned());
                }
            }

            Ok(permissions)
        } else {
            Err(anyhow::anyhow!("User {} not found", user_id.0))
        }
    }

    /// Get all roles for a user
    pub fn get_user_roles(&self, user_id: &UserId) -> Result<HashSet<RoleId>> {
        debug!("Getting roles for user {}", user_id.0);

        if let Some(user) = self.users.get(user_id) {
            Ok(user.roles.clone())
        } else {
            Err(anyhow::anyhow!("User {} not found", user_id.0))
        }
    }
}

impl Default for AuthzManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_authz_manager_creation() {
        let manager = AuthzManager::new();
        assert_eq!(manager.users.len(), 0);
        assert_eq!(manager.roles.len(), 0);
        assert_eq!(manager.permissions.len(), 0);
    }

    #[test]
    fn test_permission_management() {
        let mut manager = AuthzManager::new();

        let permission = Permission {
            id: PermissionId("read".to_string()),
            name: "Read".to_string(),
            description: Some("Read permission".to_string()),
        };

        assert!(manager.add_permission(permission).is_ok());
        assert_eq!(manager.permissions.len(), 1);
    }

    #[test]
    fn test_role_management() {
        let mut manager = AuthzManager::new();

        let role = Role {
            id: RoleId("admin".to_string()),
            permissions: HashSet::new(),
            name: "Administrator".to_string(),
            description: Some("Administrator role".to_string()),
        };

        assert!(manager.add_role(role).is_ok());
        assert_eq!(manager.roles.len(), 1);
    }

    #[test]
    fn test_user_management() {
        let mut manager = AuthzManager::new();

        let user = User {
            id: UserId("user1".to_string()),
            roles: HashSet::new(),
        };

        assert!(manager.add_user(user).is_ok());
        assert_eq!(manager.users.len(), 1);
    }

    #[test]
    fn test_role_assignment() {
        let mut manager = AuthzManager::new();

        // Add role
        let role = Role {
            id: RoleId("admin".to_string()),
            permissions: HashSet::new(),
            name: "Administrator".to_string(),
            description: Some("Administrator role".to_string()),
        };
        manager.add_role(role).unwrap();

        // Add user
        let user = User {
            id: UserId("user1".to_string()),
            roles: HashSet::new(),
        };
        manager.add_user(user).unwrap();

        // Assign role to user
        assert!(manager
            .assign_role_to_user(&UserId("user1".to_string()), &RoleId("admin".to_string()))
            .is_ok());

        // Check user roles
        let roles = manager
            .get_user_roles(&UserId("user1".to_string()))
            .unwrap();
        assert!(roles.contains(&RoleId("admin".to_string())));
    }

    #[test]
    fn test_permission_check() {
        let mut manager = AuthzManager::new();

        // Add permission
        let permission = Permission {
            id: PermissionId("read".to_string()),
            name: "Read".to_string(),
            description: Some("Read permission".to_string()),
        };
        manager.add_permission(permission).unwrap();

        // Add role with permission
        let mut permissions = HashSet::new();
        permissions.insert(PermissionId("read".to_string()));
        let role = Role {
            id: RoleId("reader".to_string()),
            permissions,
            name: "Reader".to_string(),
            description: Some("Reader role".to_string()),
        };
        manager.add_role(role).unwrap();

        // Add user
        let user = User {
            id: UserId("user1".to_string()),
            roles: HashSet::new(),
        };
        manager.add_user(user).unwrap();

        // Assign role to user
        manager
            .assign_role_to_user(&UserId("user1".to_string()), &RoleId("reader".to_string()))
            .unwrap();

        // Check permission
        assert!(manager.user_has_permission(
            &UserId("user1".to_string()),
            &PermissionId("read".to_string())
        ));
    }
}
