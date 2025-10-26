use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use thiserror::Error;
use tokio::sync::oneshot;

use crate::permissions::{Permission, RiskLevel};

#[derive(Error, Debug)]
pub enum ConfirmationError {
    #[error("User denied action: {0}")]
    Denied(String),
    #[error("Confirmation timeout")]
    Timeout,
    #[error("Confirmation system error: {0}")]
    SystemError(String),
}

/// Request for user confirmation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmationRequest {
    pub id: String,
    pub action: String,
    pub permission: Permission,
    pub risk_level: RiskLevel,
    pub description: String,
    pub metadata: serde_json::Value,
    pub timeout_seconds: u64,
}

impl ConfirmationRequest {
    pub fn new(action: String, permission: Permission, description: String) -> Self {
        let risk_level = permission.risk_level();
        let timeout_seconds = match risk_level {
            RiskLevel::Low => 30,
            RiskLevel::Medium => 60,
            RiskLevel::High => 120,
            RiskLevel::Critical => 300,
        };

        Self {
            id: uuid::Uuid::new_v4().to_string(),
            action,
            permission,
            risk_level,
            description,
            metadata: serde_json::json!({}),
            timeout_seconds,
        }
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn with_timeout(mut self, seconds: u64) -> Self {
        self.timeout_seconds = seconds;
        self
    }
}

/// Response from user confirmation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmationResponse {
    pub request_id: String,
    pub approved: bool,
    pub reason: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Manages user confirmations for RPA actions
#[derive(Clone)]
pub struct ConfirmationManager {
    pending: Arc<Mutex<Vec<PendingConfirmation>>>,
    auto_approve: Arc<Mutex<Vec<Permission>>>,
}

struct PendingConfirmation {
    request: ConfirmationRequest,
    sender: oneshot::Sender<ConfirmationResponse>,
}

impl Default for ConfirmationManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfirmationManager {
    pub fn new() -> Self {
        Self {
            pending: Arc::new(Mutex::new(Vec::new())),
            auto_approve: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Add a permission to auto-approve list
    pub fn add_auto_approve(&self, permission: Permission) -> Result<(), ConfirmationError> {
        let mut auto_approve = self.auto_approve.lock()
            .map_err(|e| ConfirmationError::SystemError(e.to_string()))?;
        if !auto_approve.contains(&permission) {
            auto_approve.push(permission);
        }
        Ok(())
    }

    /// Remove a permission from auto-approve list
    pub fn remove_auto_approve(&self, permission: Permission) -> Result<(), ConfirmationError> {
        let mut auto_approve = self.auto_approve.lock()
            .map_err(|e| ConfirmationError::SystemError(e.to_string()))?;
        auto_approve.retain(|p| *p != permission);
        Ok(())
    }

    /// Check if a permission is auto-approved
    pub fn is_auto_approved(&self, permission: Permission) -> Result<bool, ConfirmationError> {
        let auto_approve = self.auto_approve.lock()
            .map_err(|e| ConfirmationError::SystemError(e.to_string()))?;
        Ok(auto_approve.contains(&permission))
    }

    /// Request user confirmation
    pub async fn request_confirmation(
        &self,
        request: ConfirmationRequest,
    ) -> Result<ConfirmationResponse, ConfirmationError> {
        // Check auto-approve
        if self.is_auto_approved(request.permission)? {
            return Ok(ConfirmationResponse {
                request_id: request.id.clone(),
                approved: true,
                reason: Some("Auto-approved".to_string()),
                timestamp: chrono::Utc::now(),
            });
        }

        let (tx, rx) = oneshot::channel();
        let timeout = Duration::from_secs(request.timeout_seconds);

        // Add to pending
        {
            let mut pending = self.pending.lock()
                .map_err(|e| ConfirmationError::SystemError(e.to_string()))?;
            pending.push(PendingConfirmation {
                request: request.clone(),
                sender: tx,
            });
        }

        // Wait for response with timeout
        match tokio::time::timeout(timeout, rx).await {
            Ok(Ok(response)) => Ok(response),
            Ok(Err(_)) => Err(ConfirmationError::SystemError("Channel closed".to_string())),
            Err(_) => {
                // Remove from pending on timeout
                self.remove_pending(&request.id)?;
                Err(ConfirmationError::Timeout)
            }
        }
    }

    /// Respond to a confirmation request
    pub fn respond(
        &self,
        request_id: &str,
        approved: bool,
        reason: Option<String>,
    ) -> Result<(), ConfirmationError> {
        let mut pending = self.pending.lock()
            .map_err(|e| ConfirmationError::SystemError(e.to_string()))?;

        let pos = pending.iter().position(|p| p.request.id == request_id)
            .ok_or_else(|| ConfirmationError::SystemError("Request not found".to_string()))?;

        let confirmation = pending.remove(pos);
        let response = ConfirmationResponse {
            request_id: request_id.to_string(),
            approved,
            reason,
            timestamp: chrono::Utc::now(),
        };

        confirmation.sender.send(response)
            .map_err(|_| ConfirmationError::SystemError("Failed to send response".to_string()))?;

        Ok(())
    }

    /// Get all pending confirmation requests
    pub fn get_pending(&self) -> Result<Vec<ConfirmationRequest>, ConfirmationError> {
        let pending = self.pending.lock()
            .map_err(|e| ConfirmationError::SystemError(e.to_string()))?;
        Ok(pending.iter().map(|p| p.request.clone()).collect())
    }

    /// Remove a pending request
    fn remove_pending(&self, request_id: &str) -> Result<(), ConfirmationError> {
        let mut pending = self.pending.lock()
            .map_err(|e| ConfirmationError::SystemError(e.to_string()))?;
        pending.retain(|p| p.request.id != request_id);
        Ok(())
    }

    /// Clear all pending requests
    pub fn clear_pending(&self) -> Result<(), ConfirmationError> {
        let mut pending = self.pending.lock()
            .map_err(|e| ConfirmationError::SystemError(e.to_string()))?;
        pending.clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confirmation_request_creation() {
        let request = ConfirmationRequest::new(
            "mouse_click".to_string(),
            Permission::MouseClick,
            "Click at (100, 100)".to_string(),
        );

        assert_eq!(request.action, "mouse_click");
        assert_eq!(request.permission, Permission::MouseClick);
        assert_eq!(request.risk_level, RiskLevel::Medium);
    }

    #[test]
    fn test_auto_approve() {
        let manager = ConfirmationManager::new();

        manager.add_auto_approve(Permission::MouseMove).unwrap();
        assert!(manager.is_auto_approved(Permission::MouseMove).unwrap());

        manager.remove_auto_approve(Permission::MouseMove).unwrap();
        assert!(!manager.is_auto_approved(Permission::MouseMove).unwrap());
    }

    #[tokio::test]
    async fn test_auto_approved_request() {
        let manager = ConfirmationManager::new();
        manager.add_auto_approve(Permission::MouseMove).unwrap();

        let request = ConfirmationRequest::new(
            "mouse_move".to_string(),
            Permission::MouseMove,
            "Move to (100, 100)".to_string(),
        );

        let response = manager.request_confirmation(request).await.unwrap();
        assert!(response.approved);
    }

    #[tokio::test]
    async fn test_manual_confirmation() {
        let manager = ConfirmationManager::new();

        let request = ConfirmationRequest::new(
            "file_write".to_string(),
            Permission::FileWrite,
            "Write to file".to_string(),
        ).with_timeout(5);

        let request_id = request.id.clone();
        let manager_clone = manager.clone();

        // Spawn task to respond
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(100)).await;
            manager_clone.respond(&request_id, true, Some("Approved by user".to_string())).unwrap();
        });

        let response = manager.request_confirmation(request).await.unwrap();
        assert!(response.approved);
    }

    #[tokio::test]
    async fn test_confirmation_timeout() {
        let manager = ConfirmationManager::new();

        let request = ConfirmationRequest::new(
            "file_write".to_string(),
            Permission::FileWrite,
            "Write to file".to_string(),
        ).with_timeout(1);

        let result = manager.request_confirmation(request).await;
        assert!(matches!(result, Err(ConfirmationError::Timeout)));
    }

    #[test]
    fn test_get_pending() {
        let manager = ConfirmationManager::new();

        // Initially empty
        let pending = manager.get_pending().unwrap();
        assert_eq!(pending.len(), 0);
    }
}
