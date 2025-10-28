use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use thiserror::Error;

use crate::permissions::Permission;

#[derive(Error, Debug)]
pub enum AuditError {
    #[error("Failed to write audit log: {0}")]
    WriteError(String),
    #[error("Failed to read audit log: {0}")]
    ReadError(String),
}

/// Audit log entry for RPA actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub action: String,
    pub permission: Permission,
    pub user_confirmed: bool,
    pub success: bool,
    pub error: Option<String>,
    pub metadata: serde_json::Value,
}

impl AuditEntry {
    pub fn new(action: String, permission: Permission, user_confirmed: bool) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            action,
            permission,
            user_confirmed,
            success: false,
            error: None,
            metadata: serde_json::json!({}),
        }
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn mark_success(mut self) -> Self {
        self.success = true;
        self
    }

    pub fn mark_error(mut self, error: String) -> Self {
        self.success = false;
        self.error = Some(error);
        self
    }
}

/// Audit logger for RPA actions
#[derive(Clone)]
pub struct AuditLogger {
    entries: Arc<Mutex<VecDeque<AuditEntry>>>,
    max_entries: usize,
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::new(1000)
    }
}

impl AuditLogger {
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Arc::new(Mutex::new(VecDeque::with_capacity(max_entries))),
            max_entries,
        }
    }

    /// Log an audit entry
    pub fn log(&self, entry: AuditEntry) -> Result<(), AuditError> {
        let mut entries = self
            .entries
            .lock()
            .map_err(|e| AuditError::WriteError(e.to_string()))?;

        if entries.len() >= self.max_entries {
            entries.pop_front();
        }

        entries.push_back(entry);
        Ok(())
    }

    /// Get all audit entries
    pub fn get_entries(&self) -> Result<Vec<AuditEntry>, AuditError> {
        let entries = self
            .entries
            .lock()
            .map_err(|e| AuditError::ReadError(e.to_string()))?;
        Ok(entries.iter().cloned().collect())
    }

    /// Get entries filtered by permission
    pub fn get_by_permission(&self, permission: Permission) -> Result<Vec<AuditEntry>, AuditError> {
        let entries = self
            .entries
            .lock()
            .map_err(|e| AuditError::ReadError(e.to_string()))?;
        Ok(entries
            .iter()
            .filter(|e| e.permission == permission)
            .cloned()
            .collect())
    }

    /// Get failed entries
    pub fn get_failed(&self) -> Result<Vec<AuditEntry>, AuditError> {
        let entries = self
            .entries
            .lock()
            .map_err(|e| AuditError::ReadError(e.to_string()))?;
        Ok(entries.iter().filter(|e| !e.success).cloned().collect())
    }

    /// Get entries within time range
    pub fn get_by_time_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<AuditEntry>, AuditError> {
        let entries = self
            .entries
            .lock()
            .map_err(|e| AuditError::ReadError(e.to_string()))?;
        Ok(entries
            .iter()
            .filter(|e| e.timestamp >= start && e.timestamp <= end)
            .cloned()
            .collect())
    }

    /// Clear all entries
    pub fn clear(&self) -> Result<(), AuditError> {
        let mut entries = self
            .entries
            .lock()
            .map_err(|e| AuditError::WriteError(e.to_string()))?;
        entries.clear();
        Ok(())
    }

    /// Get statistics
    pub fn get_stats(&self) -> Result<AuditStats, AuditError> {
        let entries = self
            .entries
            .lock()
            .map_err(|e| AuditError::ReadError(e.to_string()))?;

        let total = entries.len();
        let successful = entries.iter().filter(|e| e.success).count();
        let failed = total - successful;
        let confirmed = entries.iter().filter(|e| e.user_confirmed).count();

        Ok(AuditStats {
            total,
            successful,
            failed,
            confirmed,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditStats {
    pub total: usize,
    pub successful: usize,
    pub failed: usize,
    pub confirmed: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_entry_creation() {
        let entry = AuditEntry::new("mouse_click".to_string(), Permission::MouseClick, true);
        assert_eq!(entry.action, "mouse_click");
        assert_eq!(entry.permission, Permission::MouseClick);
        assert!(entry.user_confirmed);
        assert!(!entry.success);
    }

    #[test]
    fn test_audit_logger() {
        let logger = AuditLogger::new(10);

        let entry =
            AuditEntry::new("test_action".to_string(), Permission::MouseMove, false).mark_success();

        logger.log(entry.clone()).unwrap();

        let entries = logger.get_entries().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].action, "test_action");
    }

    #[test]
    fn test_max_entries() {
        let logger = AuditLogger::new(3);

        for i in 0..5 {
            let entry = AuditEntry::new(format!("action_{i}"), Permission::MouseMove, false);
            logger.log(entry).unwrap();
        }

        let entries = logger.get_entries().unwrap();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].action, "action_2");
    }

    #[test]
    fn test_filter_by_permission() {
        let logger = AuditLogger::new(10);

        logger
            .log(AuditEntry::new(
                "action1".to_string(),
                Permission::MouseMove,
                false,
            ))
            .unwrap();
        logger
            .log(AuditEntry::new(
                "action2".to_string(),
                Permission::MouseClick,
                false,
            ))
            .unwrap();
        logger
            .log(AuditEntry::new(
                "action3".to_string(),
                Permission::MouseMove,
                false,
            ))
            .unwrap();

        let filtered = logger.get_by_permission(Permission::MouseMove).unwrap();
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_stats() {
        let logger = AuditLogger::new(10);

        logger
            .log(AuditEntry::new("a1".to_string(), Permission::MouseMove, true).mark_success())
            .unwrap();
        logger
            .log(
                AuditEntry::new("a2".to_string(), Permission::MouseClick, false)
                    .mark_error("test".to_string()),
            )
            .unwrap();
        logger
            .log(AuditEntry::new("a3".to_string(), Permission::MouseMove, true).mark_success())
            .unwrap();

        let stats = logger.get_stats().unwrap();
        assert_eq!(stats.total, 3);
        assert_eq!(stats.successful, 2);
        assert_eq!(stats.failed, 1);
        assert_eq!(stats.confirmed, 2);
    }
}
