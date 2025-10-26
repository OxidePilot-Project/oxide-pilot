use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PermissionError {
    #[error("Action not permitted: {0}")]
    NotPermitted(String),
    #[error("Permission denied: {action} requires {permission:?}")]
    Denied { action: String, permission: Permission },
    #[error("Invalid permission configuration: {0}")]
    InvalidConfig(String),
}

/// Granular permissions for RPA actions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Permission {
    // Mouse permissions
    MouseMove,
    MouseClick,
    MouseScroll,
    MouseDrag,

    // Keyboard permissions
    KeyboardType,
    KeyboardPress,
    KeyboardHotkey,

    // Screen permissions
    ScreenCapture,
    ScreenCaptureArea,
    ScreenAnalyze,

    // File system permissions
    FileRead,
    FileWrite,
    FileDelete,

    // System permissions
    SystemCommand,
    ProcessControl,
    NetworkAccess,
}

/// Risk level for RPA actions
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl Permission {
    /// Get the risk level associated with this permission
    pub fn risk_level(&self) -> RiskLevel {
        match self {
            Permission::MouseMove | Permission::ScreenCapture | Permission::ScreenCaptureArea => {
                RiskLevel::Low
            }
            Permission::MouseClick
            | Permission::MouseScroll
            | Permission::KeyboardType
            | Permission::ScreenAnalyze => RiskLevel::Medium,
            Permission::MouseDrag
            | Permission::KeyboardPress
            | Permission::KeyboardHotkey
            | Permission::FileRead => RiskLevel::High,
            Permission::FileWrite
            | Permission::FileDelete
            | Permission::SystemCommand
            | Permission::ProcessControl
            | Permission::NetworkAccess => RiskLevel::Critical,
        }
    }

    /// Check if this permission requires user confirmation
    pub fn requires_confirmation(&self) -> bool {
        self.risk_level() >= RiskLevel::High
    }
}

/// Permission policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionPolicy {
    /// Allowed permissions
    pub allowed: HashSet<Permission>,
    /// Denied permissions (takes precedence over allowed)
    pub denied: HashSet<Permission>,
    /// Require confirmation for high-risk actions
    pub require_confirmation: bool,
    /// Maximum risk level allowed without confirmation
    pub max_auto_risk: RiskLevel,
}

impl Default for PermissionPolicy {
    fn default() -> Self {
        Self {
            allowed: HashSet::from([
                Permission::MouseMove,
                Permission::ScreenCapture,
                Permission::ScreenCaptureArea,
            ]),
            denied: HashSet::new(),
            require_confirmation: true,
            max_auto_risk: RiskLevel::Medium,
        }
    }
}

impl PermissionPolicy {
    /// Create a permissive policy (for testing/development)
    pub fn permissive() -> Self {
        Self {
            allowed: HashSet::from([
                Permission::MouseMove,
                Permission::MouseClick,
                Permission::MouseScroll,
                Permission::MouseDrag,
                Permission::KeyboardType,
                Permission::KeyboardPress,
                Permission::ScreenCapture,
                Permission::ScreenCaptureArea,
                Permission::ScreenAnalyze,
                Permission::FileRead,
            ]),
            denied: HashSet::new(),
            require_confirmation: true,
            max_auto_risk: RiskLevel::High,
        }
    }

    /// Create a restrictive policy (for production)
    pub fn restrictive() -> Self {
        Self {
            allowed: HashSet::from([Permission::ScreenCapture, Permission::ScreenCaptureArea]),
            denied: HashSet::from([
                Permission::FileWrite,
                Permission::FileDelete,
                Permission::SystemCommand,
                Permission::ProcessControl,
                Permission::NetworkAccess,
            ]),
            require_confirmation: true,
            max_auto_risk: RiskLevel::Low,
        }
    }

    /// Check if a permission is allowed
    pub fn is_allowed(&self, permission: Permission) -> bool {
        !self.denied.contains(&permission) && self.allowed.contains(&permission)
    }

    /// Check if an action needs user confirmation
    pub fn needs_confirmation(&self, permission: Permission) -> bool {
        if !self.require_confirmation {
            return false;
        }
        permission.risk_level() > self.max_auto_risk || permission.requires_confirmation()
    }

    /// Grant a permission
    pub fn grant(&mut self, permission: Permission) {
        self.allowed.insert(permission);
        self.denied.remove(&permission);
    }

    /// Revoke a permission
    pub fn revoke(&mut self, permission: Permission) {
        self.allowed.remove(&permission);
        self.denied.insert(permission);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_risk_levels() {
        assert_eq!(Permission::MouseMove.risk_level(), RiskLevel::Low);
        assert_eq!(Permission::MouseClick.risk_level(), RiskLevel::Medium);
        assert_eq!(Permission::FileRead.risk_level(), RiskLevel::High);
        assert_eq!(Permission::FileDelete.risk_level(), RiskLevel::Critical);
    }

    #[test]
    fn test_default_policy() {
        let policy = PermissionPolicy::default();
        assert!(policy.is_allowed(Permission::MouseMove));
        assert!(!policy.is_allowed(Permission::FileWrite));
    }

    #[test]
    fn test_permissive_policy() {
        let policy = PermissionPolicy::permissive();
        assert!(policy.is_allowed(Permission::MouseClick));
        assert!(policy.is_allowed(Permission::KeyboardType));
        assert!(!policy.is_allowed(Permission::FileWrite));
    }

    #[test]
    fn test_restrictive_policy() {
        let policy = PermissionPolicy::restrictive();
        assert!(policy.is_allowed(Permission::ScreenCapture));
        assert!(!policy.is_allowed(Permission::MouseClick));
        assert!(!policy.is_allowed(Permission::FileWrite));
    }

    #[test]
    fn test_grant_revoke() {
        let mut policy = PermissionPolicy::default();
        policy.grant(Permission::FileRead);
        assert!(policy.is_allowed(Permission::FileRead));

        policy.revoke(Permission::FileRead);
        assert!(!policy.is_allowed(Permission::FileRead));
    }

    #[test]
    fn test_confirmation_requirements() {
        let policy = PermissionPolicy::default();
        assert!(!policy.needs_confirmation(Permission::MouseMove));
        assert!(policy.needs_confirmation(Permission::FileRead));
        assert!(policy.needs_confirmation(Permission::FileDelete));
    }
}
