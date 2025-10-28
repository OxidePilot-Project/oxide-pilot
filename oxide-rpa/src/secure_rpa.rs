use log::{info, warn};
use rdev::{Button, Key};
use std::sync::Arc;
use thiserror::Error;

use crate::audit::{AuditEntry, AuditLogger};
use crate::confirmation::{ConfirmationManager, ConfirmationRequest};
use crate::permissions::{Permission, PermissionPolicy};
use crate::rollback::{ActionType, ReversibleAction, RollbackManager};
use crate::rpa::{KeyboardController, MouseController, ScreenCapture};

#[derive(Error, Debug)]
pub enum SecureRPAError {
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("User confirmation required but denied: {0}")]
    ConfirmationDenied(String),
    #[error("RPA operation failed: {0}")]
    OperationFailed(String),
    #[error("Audit error: {0}")]
    AuditError(#[from] crate::audit::AuditError),
    #[error("Rollback error: {0}")]
    RollbackError(#[from] crate::rollback::RollbackError),
    #[error("Confirmation error: {0}")]
    ConfirmationError(#[from] crate::confirmation::ConfirmationError),
}

/// Secure RPA controller with permissions, audit logging, and rollback
pub struct SecureRPAController {
    mouse: MouseController,
    keyboard: KeyboardController,
    screen: ScreenCapture,
    policy: Arc<PermissionPolicy>,
    audit: AuditLogger,
    rollback: RollbackManager,
    confirmation: ConfirmationManager,
}

impl SecureRPAController {
    pub fn new(policy: PermissionPolicy) -> Self {
        Self {
            mouse: MouseController::new(),
            keyboard: KeyboardController::new(),
            screen: ScreenCapture::new(),
            policy: Arc::new(policy),
            audit: AuditLogger::default(),
            rollback: RollbackManager::default(),
            confirmation: ConfirmationManager::new(),
        }
    }

    pub fn with_audit_size(mut self, max_entries: usize) -> Self {
        self.audit = AuditLogger::new(max_entries);
        self
    }

    pub fn with_rollback_size(mut self, max_history: usize) -> Self {
        self.rollback = RollbackManager::new(max_history);
        self
    }

    /// Get reference to audit logger
    pub fn audit(&self) -> &AuditLogger {
        &self.audit
    }

    /// Get reference to rollback manager
    pub fn rollback(&self) -> &RollbackManager {
        &self.rollback
    }

    /// Get reference to confirmation manager
    pub fn confirmation(&self) -> &ConfirmationManager {
        &self.confirmation
    }

    /// Update permission policy
    pub fn update_policy(&mut self, policy: PermissionPolicy) {
        self.policy = Arc::new(policy);
    }

    /// Check permission and request confirmation if needed
    async fn check_permission_and_confirm(
        &self,
        permission: Permission,
        action: &str,
        description: &str,
    ) -> Result<bool, SecureRPAError> {
        // Check if permission is allowed
        if !self.policy.is_allowed(permission) {
            warn!("Permission denied for action: {action}");
            return Err(SecureRPAError::PermissionDenied(format!(
                "Action '{action}' requires {permission:?} permission"
            )));
        }

        // Check if confirmation is needed
        if self.policy.needs_confirmation(permission) {
            let request =
                ConfirmationRequest::new(action.to_string(), permission, description.to_string());

            let response = self.confirmation.request_confirmation(request).await?;

            if !response.approved {
                warn!("User denied action: {action}");
                return Err(SecureRPAError::ConfirmationDenied(
                    response
                        .reason
                        .unwrap_or_else(|| "No reason provided".to_string()),
                ));
            }

            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Log action to audit
    fn log_audit(
        &self,
        action: &str,
        permission: Permission,
        user_confirmed: bool,
        result: Result<(), String>,
    ) {
        let entry = match result {
            Ok(_) => AuditEntry::new(action.to_string(), permission, user_confirmed).mark_success(),
            Err(e) => AuditEntry::new(action.to_string(), permission, user_confirmed).mark_error(e),
        };

        if let Err(e) = self.audit.log(entry) {
            warn!("Failed to log audit entry: {e}");
        }
    }

    /// Move mouse with security checks
    pub async fn move_mouse(&self, x: i32, y: i32) -> Result<(), SecureRPAError> {
        let action = format!("move_mouse({x}, {y})");
        let confirmed = self
            .check_permission_and_confirm(
                Permission::MouseMove,
                &action,
                &format!("Move mouse to ({x}, {y})"),
            )
            .await?;

        // Get current position for rollback (simplified - would need actual position)
        let from_x = 0; // Would get actual position
        let from_y = 0;

        // Execute action
        self.mouse.move_to(x, y);
        info!("Mouse moved to ({x}, {y})");

        // Record for rollback
        let reversible = ReversibleAction {
            id: uuid::Uuid::new_v4().to_string(),
            action_type: ActionType::MouseMove {
                from_x,
                from_y,
                to_x: x,
                to_y: y,
            },
            state_before: serde_json::json!({ "x": from_x, "y": from_y }),
            state_after: serde_json::json!({ "x": x, "y": y }),
            timestamp: chrono::Utc::now(),
        };
        let _ = self.rollback.record(reversible);

        self.log_audit(&action, Permission::MouseMove, confirmed, Ok(()));

        Ok(())
    }

    /// Click mouse with security checks
    pub async fn click_mouse(&self, button: Button) -> Result<(), SecureRPAError> {
        let action = format!("click_mouse({button:?})");
        let confirmed = self
            .check_permission_and_confirm(
                Permission::MouseClick,
                &action,
                &format!("Click mouse button: {button:?}"),
            )
            .await?;

        self.mouse.click(button);
        info!("Mouse clicked: {button:?}");

        self.log_audit(&action, Permission::MouseClick, confirmed, Ok(()));

        Ok(())
    }

    /// Scroll mouse with security checks
    pub async fn scroll_mouse(&self, delta_x: i32, delta_y: i32) -> Result<(), SecureRPAError> {
        let action = format!("scroll_mouse({delta_x}, {delta_y})");
        let confirmed = self
            .check_permission_and_confirm(
                Permission::MouseScroll,
                &action,
                &format!("Scroll mouse by ({delta_x}, {delta_y})"),
            )
            .await?;

        self.mouse.scroll(delta_x, delta_y);
        info!("Mouse scrolled by ({delta_x}, {delta_y})");

        self.log_audit(&action, Permission::MouseScroll, confirmed, Ok(()));

        Ok(())
    }

    /// Type text with security checks
    pub async fn type_text(&self, text: &str) -> Result<(), SecureRPAError> {
        let action = format!("type_text({text})");
        let confirmed = self
            .check_permission_and_confirm(
                Permission::KeyboardType,
                &action,
                &format!("Type text: {text}"),
            )
            .await?;

        self.keyboard.type_text(text);
        info!("Text typed: {text}");

        self.log_audit(&action, Permission::KeyboardType, confirmed, Ok(()));

        Ok(())
    }

    /// Press key with security checks
    pub async fn press_key(&self, key: Key) -> Result<(), SecureRPAError> {
        let action = format!("press_key({key:?})");
        let confirmed = self
            .check_permission_and_confirm(
                Permission::KeyboardPress,
                &action,
                &format!("Press key: {key:?}"),
            )
            .await?;

        self.keyboard.press_key(key);
        info!("Key pressed: {key:?}");

        self.log_audit(&action, Permission::KeyboardPress, confirmed, Ok(()));

        Ok(())
    }

    /// Capture screen with security checks
    pub async fn capture_screen(
        &self,
    ) -> Result<image::ImageBuffer<image::Rgba<u8>, Vec<u8>>, SecureRPAError> {
        let action = "capture_screen".to_string();
        let confirmed = self
            .check_permission_and_confirm(
                Permission::ScreenCapture,
                &action,
                "Capture entire screen",
            )
            .await?;

        let result = self.screen.capture_screen().await;

        self.log_audit(
            &action,
            Permission::ScreenCapture,
            confirmed,
            result.as_ref().map(|_| ()).map_err(|e| e.clone()),
        );

        result.map_err(SecureRPAError::OperationFailed)
    }

    /// Rollback last action
    pub async fn rollback_last(&self) -> Result<(), SecureRPAError> {
        let action = self.rollback.rollback_last()?;

        info!("Rolling back action: {:?}", action.action_type);

        // Execute rollback based on action type
        match action.action_type {
            ActionType::MouseMove { from_x, from_y, .. } => {
                self.mouse.move_to(from_x, from_y);
                info!("Mouse position restored to ({from_x}, {from_y})");
            }
            _ => {
                warn!(
                    "Rollback not implemented for action type: {:?}",
                    action.action_type
                );
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_permission_denied() {
        let policy = PermissionPolicy::restrictive();
        let controller = SecureRPAController::new(policy);

        let result = controller.click_mouse(Button::Left).await;
        assert!(matches!(result, Err(SecureRPAError::PermissionDenied(_))));
    }

    #[tokio::test]
    async fn test_audit_logging() {
        let policy = PermissionPolicy::permissive();
        let controller = SecureRPAController::new(policy);

        // This will fail in test environment but should log
        let _ = controller.move_mouse(100, 100).await;

        let entries = controller.audit().get_entries().unwrap();
        assert!(!entries.is_empty());
    }

    #[test]
    fn test_policy_update() {
        let policy = PermissionPolicy::restrictive();
        let mut controller = SecureRPAController::new(policy);

        let new_policy = PermissionPolicy::permissive();
        controller.update_policy(new_policy);
    }
}
