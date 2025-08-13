use log::{info, warn};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PermissionLevel {
    Allow,
    Ask,
    Deny,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, Hash, PartialEq)]
pub enum RPAActionType {
    MouseClick,
    MouseMove,
    KeyboardInput,
    ScreenCapture,
    FileOperation,
    NetworkAccess,
    SystemCommand,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionRequest {
    pub action_type: RPAActionType,
    pub description: String,
    pub target: Option<String>,
    pub severity: u8, // 1-5
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionDecision {
    pub request: PermissionRequest,
    pub granted: bool,
    pub user_confirmed: bool,
    pub timestamp: u64,
}

pub struct PermissionManager {
    permissions: HashMap<RPAActionType, PermissionLevel>,
    audit_log: Vec<PermissionDecision>,
    max_log_size: usize,
}

impl Default for PermissionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PermissionManager {
    pub fn new() -> Self {
        let mut permissions = HashMap::new();

        // Default permissions - more restrictive
        permissions.insert(RPAActionType::MouseClick, PermissionLevel::Ask);
        permissions.insert(RPAActionType::MouseMove, PermissionLevel::Allow);
        permissions.insert(RPAActionType::KeyboardInput, PermissionLevel::Ask);
        permissions.insert(RPAActionType::ScreenCapture, PermissionLevel::Ask);
        permissions.insert(RPAActionType::FileOperation, PermissionLevel::Ask);
        permissions.insert(RPAActionType::NetworkAccess, PermissionLevel::Deny);
        permissions.insert(RPAActionType::SystemCommand, PermissionLevel::Ask);

        Self {
            permissions,
            audit_log: Vec::new(),
            max_log_size: 1000,
        }
    }

    pub async fn request_permission(&mut self, request: PermissionRequest) -> bool {
        let level = self.permissions.get(&request.action_type).unwrap_or(&PermissionLevel::Ask);

        match level {
            PermissionLevel::Allow => {
                self.log_decision(request.clone(), true, false);
                true
            }
            PermissionLevel::Deny => {
                self.log_decision(request.clone(), false, false);
                false
            }
            PermissionLevel::Ask => {
                // In a real implementation, this would show a dialog
                // For now, we'll log and allow with user confirmation
                self.log_decision(request.clone(), true, true);
                true
            }
        }
    }

    pub fn log_decision(&mut self, request: PermissionRequest, granted: bool, user_confirmed: bool) {
        let decision = PermissionDecision {
            request,
            granted,
            user_confirmed,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        self.audit_log.push(decision);

        // Keep log size manageable
        if self.audit_log.len() > self.max_log_size {
            self.audit_log.remove(0);
        }
    }

    pub fn get_audit_log(&self) -> &[PermissionDecision] {
        &self.audit_log
    }

    pub fn set_permission(&mut self, action_type: RPAActionType, level: PermissionLevel) {
        self.permissions.insert(action_type, level);
    }

    pub fn get_permissions(&self) -> &HashMap<RPAActionType, PermissionLevel> {
        &self.permissions
    }

    pub fn check_permission(&self, action: &str) -> bool {
        info!("Checking permission for action: {action}");
        // Placeholder for actual permission logic
        // For now, all actions are allowed
        true
    }

    pub async fn request_confirmation(&self, message: &str) -> bool {
        warn!("User confirmation requested: {message}");
        // In a real system, this would display a dialog to the user
        // For now, always confirm
        true
    }

    pub fn log_action(&self, action: &str, status: &str) {
        info!("Action logged: {action} - Status: {status}");
        // In a real system, this would write to an audit log
    }

    pub fn rollback_action(&self, action: &str) {
        warn!("Attempting to rollback action: {action}");
        // Placeholder for actual rollback logic
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_manager_creation() {
        let manager = PermissionManager::new();
        // Test that the manager can be created successfully
        assert!(!manager.get_permissions().is_empty());
    }

    #[test]
    fn test_permission_levels() {
        let mut manager = PermissionManager::new();
        manager.set_permission(RPAActionType::MouseClick, PermissionLevel::Allow);

        let permissions = manager.get_permissions();
        assert_eq!(permissions.get(&RPAActionType::MouseClick), Some(&PermissionLevel::Allow));
    }

    #[tokio::test]
    async fn test_permission_request() {
        let mut manager = PermissionManager::new();
        let request = PermissionRequest {
            action_type: RPAActionType::MouseMove,
            description: "Test mouse move".to_string(),
            target: None,
            severity: 1,
            timestamp: 0,
        };

        let result = manager.request_permission(request).await;
        assert!(result); // MouseMove should be allowed by default
    }
}
