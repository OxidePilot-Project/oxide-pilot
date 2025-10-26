use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RollbackError {
    #[error("No actions to rollback")]
    NoActions,
    #[error("Rollback failed: {0}")]
    Failed(String),
    #[error("Action not reversible: {0}")]
    NotReversible(String),
}

/// Represents an action that can be rolled back
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReversibleAction {
    pub id: String,
    pub action_type: ActionType,
    pub state_before: serde_json::Value,
    pub state_after: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionType {
    MouseMove { from_x: i32, from_y: i32, to_x: i32, to_y: i32 },
    MouseClick { x: i32, y: i32, button: String },
    KeyboardType { text: String },
    FileWrite { path: String, content_hash: String },
    FileDelete { path: String, content: Vec<u8> },
    SystemCommand { command: String },
}

impl ActionType {
    /// Check if this action type is reversible
    pub fn is_reversible(&self) -> bool {
        match self {
            ActionType::MouseMove { .. } => true,
            ActionType::MouseClick { .. } => false, // Clicks can't be undone
            ActionType::KeyboardType { .. } => false, // Typing can't be undone
            ActionType::FileWrite { .. } => true,
            ActionType::FileDelete { .. } => true,
            ActionType::SystemCommand { .. } => false, // Commands can't be undone
        }
    }

    /// Get a description of how to reverse this action
    pub fn reverse_description(&self) -> Option<String> {
        match self {
            ActionType::MouseMove { from_x, from_y, .. } => {
                Some(format!("Move mouse back to ({from_x}, {from_y})"))
            }
            ActionType::FileWrite { path, .. } => {
                Some(format!("Restore previous content of {path}"))
            }
            ActionType::FileDelete { path, .. } => {
                Some(format!("Restore deleted file {path}"))
            }
            _ => None,
        }
    }
}

/// Manages rollback of RPA actions
#[derive(Clone)]
pub struct RollbackManager {
    actions: Arc<Mutex<VecDeque<ReversibleAction>>>,
    max_history: usize,
}

impl Default for RollbackManager {
    fn default() -> Self {
        Self::new(100)
    }
}

impl RollbackManager {
    pub fn new(max_history: usize) -> Self {
        Self {
            actions: Arc::new(Mutex::new(VecDeque::with_capacity(max_history))),
            max_history,
        }
    }

    /// Record an action for potential rollback
    pub fn record(&self, action: ReversibleAction) -> Result<(), RollbackError> {
        let mut actions = self.actions.lock()
            .map_err(|e| RollbackError::Failed(e.to_string()))?;

        if actions.len() >= self.max_history {
            actions.pop_front();
        }

        actions.push_back(action);
        Ok(())
    }

    /// Get the last action without removing it
    pub fn peek_last(&self) -> Result<Option<ReversibleAction>, RollbackError> {
        let actions = self.actions.lock()
            .map_err(|e| RollbackError::Failed(e.to_string()))?;
        Ok(actions.back().cloned())
    }

    /// Rollback the last action
    pub fn rollback_last(&self) -> Result<ReversibleAction, RollbackError> {
        let mut actions = self.actions.lock()
            .map_err(|e| RollbackError::Failed(e.to_string()))?;

        let action = actions.pop_back().ok_or(RollbackError::NoActions)?;

        if !action.action_type.is_reversible() {
            return Err(RollbackError::NotReversible(
                format!("Action {:?} cannot be reversed", action.action_type)
            ));
        }

        Ok(action)
    }

    /// Rollback multiple actions
    pub fn rollback_n(&self, n: usize) -> Result<Vec<ReversibleAction>, RollbackError> {
        let mut rolled_back = Vec::new();

        for _ in 0..n {
            match self.rollback_last() {
                Ok(action) => rolled_back.push(action),
                Err(RollbackError::NoActions) => break,
                Err(e) => return Err(e),
            }
        }

        if rolled_back.is_empty() {
            return Err(RollbackError::NoActions);
        }

        Ok(rolled_back)
    }

    /// Get all recorded actions
    pub fn get_history(&self) -> Result<Vec<ReversibleAction>, RollbackError> {
        let actions = self.actions.lock()
            .map_err(|e| RollbackError::Failed(e.to_string()))?;
        Ok(actions.iter().cloned().collect())
    }

    /// Get reversible actions only
    pub fn get_reversible_history(&self) -> Result<Vec<ReversibleAction>, RollbackError> {
        let actions = self.actions.lock()
            .map_err(|e| RollbackError::Failed(e.to_string()))?;
        Ok(actions.iter()
            .filter(|a| a.action_type.is_reversible())
            .cloned()
            .collect())
    }

    /// Clear all history
    pub fn clear(&self) -> Result<(), RollbackError> {
        let mut actions = self.actions.lock()
            .map_err(|e| RollbackError::Failed(e.to_string()))?;
        actions.clear();
        Ok(())
    }

    /// Get count of reversible actions
    pub fn reversible_count(&self) -> Result<usize, RollbackError> {
        let actions = self.actions.lock()
            .map_err(|e| RollbackError::Failed(e.to_string()))?;
        Ok(actions.iter().filter(|a| a.action_type.is_reversible()).count())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_action(action_type: ActionType) -> ReversibleAction {
        ReversibleAction {
            id: uuid::Uuid::new_v4().to_string(),
            action_type,
            state_before: serde_json::json!({}),
            state_after: serde_json::json!({}),
            timestamp: chrono::Utc::now(),
        }
    }

    #[test]
    fn test_action_reversibility() {
        let mouse_move = ActionType::MouseMove { from_x: 0, from_y: 0, to_x: 100, to_y: 100 };
        assert!(mouse_move.is_reversible());

        let mouse_click = ActionType::MouseClick { x: 50, y: 50, button: "left".to_string() };
        assert!(!mouse_click.is_reversible());

        let file_write = ActionType::FileWrite {
            path: "test.txt".to_string(),
            content_hash: "abc123".to_string()
        };
        assert!(file_write.is_reversible());
    }

    #[test]
    fn test_rollback_manager() {
        let manager = RollbackManager::new(10);

        let action = create_test_action(ActionType::MouseMove {
            from_x: 0,
            from_y: 0,
            to_x: 100,
            to_y: 100,
        });

        manager.record(action.clone()).unwrap();

        let history = manager.get_history().unwrap();
        assert_eq!(history.len(), 1);
    }

    #[test]
    fn test_rollback_last() {
        let manager = RollbackManager::new(10);

        let action = create_test_action(ActionType::MouseMove {
            from_x: 0,
            from_y: 0,
            to_x: 100,
            to_y: 100,
        });

        manager.record(action.clone()).unwrap();

        let rolled_back = manager.rollback_last().unwrap();
        assert_eq!(rolled_back.id, action.id);

        let history = manager.get_history().unwrap();
        assert_eq!(history.len(), 0);
    }

    #[test]
    fn test_rollback_non_reversible() {
        let manager = RollbackManager::new(10);

        let action = create_test_action(ActionType::MouseClick {
            x: 50,
            y: 50,
            button: "left".to_string(),
        });

        manager.record(action).unwrap();

        let result = manager.rollback_last();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RollbackError::NotReversible(_)));
    }

    #[test]
    fn test_max_history() {
        let manager = RollbackManager::new(3);

        for i in 0..5 {
            let action = create_test_action(ActionType::MouseMove {
                from_x: i,
                from_y: i,
                to_x: i + 10,
                to_y: i + 10,
            });
            manager.record(action).unwrap();
        }

        let history = manager.get_history().unwrap();
        assert_eq!(history.len(), 3);
    }

    #[test]
    fn test_reversible_count() {
        let manager = RollbackManager::new(10);

        manager.record(create_test_action(ActionType::MouseMove {
            from_x: 0, from_y: 0, to_x: 10, to_y: 10
        })).unwrap();

        manager.record(create_test_action(ActionType::MouseClick {
            x: 50, y: 50, button: "left".to_string()
        })).unwrap();

        manager.record(create_test_action(ActionType::FileWrite {
            path: "test.txt".to_string(),
            content_hash: "abc".to_string()
        })).unwrap();

        assert_eq!(manager.reversible_count().unwrap(), 2);
    }
}
