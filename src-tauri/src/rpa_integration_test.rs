#[cfg(test)]
mod tests {
    use super::*;
    use crate::rpa_commands::*;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    fn create_test_app_state() -> AppState {
        AppState {
            oxide_system: Arc::new(RwLock::new(None)),
            auth_manager: Arc::new(RwLock::new(None)),
            mcp_server: Arc::new(RwLock::new(None)),
            folder_scan_cancels: Arc::new(RwLock::new(std::collections::HashMap::new())),
            rpa_state: Arc::new(RwLock::new(None)),
        }
    }

    #[tokio::test]
    async fn test_rpa_initialization() {
        let state = create_test_app_state();
        let config = RPAInitConfig {
            policy_type: "default".to_string(),
            max_audit_entries: Some(100),
            max_rollback_history: Some(50),
        };

        let result = rpa_initialize(config, tauri::State::from(&state)).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "RPA system initialized successfully");

        // Verify RPA state is set
        let rpa_lock = state.rpa_state.read().await;
        assert!(rpa_lock.is_some());
    }

    #[tokio::test]
    async fn test_rpa_shutdown() {
        let state = create_test_app_state();

        // Initialize first
        let config = RPAInitConfig {
            policy_type: "permissive".to_string(),
            max_audit_entries: None,
            max_rollback_history: None,
        };
        let _ = rpa_initialize(config, tauri::State::from(&state)).await;

        // Then shutdown
        let result = rpa_shutdown(tauri::State::from(&state)).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "RPA system shutdown successfully");

        // Verify RPA state is cleared
        let rpa_lock = state.rpa_state.read().await;
        assert!(rpa_lock.is_none());
    }

    #[tokio::test]
    async fn test_rpa_commands_without_initialization() {
        let state = create_test_app_state();

        // Try to use RPA commands without initialization
        let result = rpa_move_mouse(100, 100, tauri::State::from(&state)).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("RPA not initialized"));
    }

    #[test]
    fn test_permission_parsing() {
        assert!(parse_permission("mouse_move").is_ok());
        assert!(parse_permission("keyboard_type").is_ok());
        assert!(parse_permission("screen_capture").is_ok());
        assert!(parse_permission("invalid_permission").is_err());
    }

    #[test]
    fn test_key_parsing() {
        assert!(parse_key("enter").is_ok());
        assert!(parse_key("escape").is_ok());
        assert!(parse_key("space").is_ok());
        assert!(parse_key("invalid_key").is_err());
    }
}
