#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod error_handler;
mod oxide_system;
mod cognee_supervisor;

use error_handler::{ErrorHandler, OxideError, RetryConfig, retry_with_backoff, GLOBAL_ERROR_MONITOR};
use log::{error, info};
use serde_json::json;
use oxide_core::config::OxidePilotConfig;
use oxide_core::google_auth;
use oxide_guardian::guardian::{SystemStatus, ThreatEvent};
use oxide_memory::memory::MemoryStats;
use oxide_system::OxideSystem;
use oxide_copilot::auth_manager::AuthManager;
use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;

// Define a struct to hold the application state with async-safe mutexes
pub struct AppState {
    oxide_system: Arc<RwLock<Option<OxideSystem>>>,
    auth_manager: Arc<RwLock<Option<AuthManager>>>,
}

#[tauri::command]
async fn set_google_api_key(api_key: String) -> Result<(), String> {
    use oxide_core::gemini_auth::GeminiAuth;
    let auth = GeminiAuth::new();
    auth.store_api_key(&api_key).await.map_err(|e| {
        error!("Failed to store Google API key: {}", e);
        e.to_string()
    })
}

#[tauri::command]
async fn set_google_client_credentials(
    client_id: String,
    client_secret: String,
) -> Result<(), String> {
    google_auth::store_client_credentials(&client_id, &client_secret)
        .await
        .map_err(|e| {
            error!("Failed to store Google client credentials: {}", e);
            e.to_string()
        })
}

#[tauri::command]
async fn authenticate_google_command() -> Result<String, String> {
    google_auth::authenticate_google().await.map_err(|e| {
        error!("Google authentication failed: {}", e);
        e.to_string()
    })
}

#[tauri::command]
async fn initialize_system(
    config: OxidePilotConfig,
    state: State<'_, AppState>,
) -> Result<(), String> {
    info!("Initializing Oxide System...");

    // Use retry mechanism for system initialization
    let retry_config = RetryConfig {
        max_attempts: 2,
        base_delay_ms: 1000,
        max_delay_ms: 3000,
        backoff_multiplier: 2.0,
    };

    let result = retry_with_backoff(
        || {
            let config_clone = config.clone();
            Box::pin(async move {
                let system = OxideSystem::new(config_clone)
                    .await
                    .map_err(|e| OxideError::SystemInit(e))?;
                system.start()
                    .await
                    .map_err(|e| OxideError::SystemInit(e))?;
                Ok::<OxideSystem, OxideError>(system)
            })
        },
        retry_config,
    ).await;

    match result {
        Ok(system) => {
            let mut system_lock = state.oxide_system.write().await;
            *system_lock = Some(system);
            info!("Oxide System initialized and started");
            Ok(())
        },
        Err(error) => {
            let context = json!({
                "config": config,
                "operation": "initialize_system"
            });
            let response = ErrorHandler::handle_error_with_monitoring(error, Some(context));
            Err(serde_json::to_string(&response).unwrap_or_else(|_| "Serialization error".to_string()))
        }
    }
}

#[tauri::command]
async fn handle_user_input_command(
    user_input: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let system_guard = state.oxide_system.read().await;
    if let Some(system) = system_guard.as_ref() {
        // Clone the system reference to avoid holding the lock across await
        let system_clone = system.clone();
        drop(system_guard); // Explicitly drop the guard

        // Use retry mechanism for user input processing
        let retry_config = RetryConfig::default();

        let result = retry_with_backoff(
            || {
                let input_clone = user_input.clone();
                let system_ref = system_clone.clone();
                Box::pin(async move {
                    system_ref.handle_text_input(input_clone)
                        .await
                        .map_err(|e| OxideError::Internal(e))
                })
            },
            retry_config,
        ).await;

        match result {
            Ok(response) => Ok(response),
            Err(error) => {
                let context = json!({
                    "user_input": user_input,
                    "operation": "handle_user_input"
                });
                let response = ErrorHandler::handle_error_with_monitoring(error, Some(context));
                Err(serde_json::to_string(&response).unwrap_or_else(|_| "Serialization error".to_string()))
            }
        }
    } else {
        let error = OxideError::SystemInit("System not initialized".to_string());
        let response = ErrorHandler::handle_error_with_monitoring(error, None);
        Err(serde_json::to_string(&response).unwrap_or_else(|_| "Serialization error".to_string()))
    }
}

#[tauri::command]
async fn get_system_status(state: State<'_, AppState>) -> Result<SystemStatus, String> {
    let system_guard = state.oxide_system.read().await;
    if let Some(system) = system_guard.as_ref() {
        Ok(system.get_system_status())
    } else {
        Err("System not initialized".to_string())
    }
}

#[tauri::command]
async fn get_threat_history(state: State<'_, AppState>) -> Result<Vec<ThreatEvent>, String> {
    let system_guard = state.oxide_system.read().await;
    if let Some(system) = system_guard.as_ref() {
        Ok(system.get_threat_history())
    } else {
        Err("System not initialized".to_string())
    }
}

#[tauri::command]
async fn get_memory_stats(state: State<'_, AppState>) -> Result<MemoryStats, String> {
    let system_guard = state.oxide_system.read().await;
    if let Some(system) = system_guard.as_ref() {
        // Clone the system reference to avoid holding the lock across await
        let system_clone = system.clone();
        drop(system_guard); // Explicitly drop the guard
        Ok(system_clone.get_memory_stats().await)
    } else {
        Err("System not initialized".to_string())
    }
}

#[tauri::command]
async fn update_system_config(
    config: OxidePilotConfig,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let system_guard = state.oxide_system.read().await;
    if let Some(system) = system_guard.as_ref() {
        // Clone the system reference to avoid holding the lock across await
        let system_clone = system.clone();
        drop(system_guard); // Explicitly drop the guard
        system_clone.update_config(config).await
    } else {
        Err("System not initialized".to_string())
    }
}

#[tauri::command]
async fn get_system_config(state: State<'_, AppState>) -> Result<OxidePilotConfig, String> {
    let system_guard = state.oxide_system.read().await;
    if let Some(system) = system_guard.as_ref() {
        // Clone the system reference to avoid holding the lock across await
        let system_clone = system.clone();
        drop(system_guard); // Explicitly drop the guard
        Ok(system_clone.get_config().await)
    } else {
        Err("System not initialized".to_string())
    }
}

#[tauri::command]
async fn record_audio(duration_secs: f32, state: State<'_, AppState>) -> Result<Vec<u8>, String> {
    let system_guard = state.oxide_system.read().await;
    if let Some(system) = system_guard.as_ref() {
        // Clone the system reference to avoid holding the lock across await
        let system_clone = system.clone();
        drop(system_guard); // Explicitly drop the guard
        system_clone.record_audio(duration_secs).await
    } else {
        Err("System not initialized".to_string())
    }
}

#[tauri::command]
async fn play_audio(audio_data: Vec<u8>, state: State<'_, AppState>) -> Result<(), String> {
    let system_guard = state.oxide_system.read().await;
    if let Some(system) = system_guard.as_ref() {
        // Clone the system reference to avoid holding the lock across await
        let system_clone = system.clone();
        drop(system_guard); // Explicitly drop the guard
        system_clone.play_audio(&audio_data).await
    } else {
        Err("System not initialized".to_string())
    }
}

#[tauri::command]
async fn get_audio_devices(
    state: State<'_, AppState>,
) -> Result<(Vec<String>, Vec<String>), String> {
    let system_guard = state.oxide_system.read().await;
    if let Some(system) = system_guard.as_ref() {
        // Clone the system reference to avoid holding the lock across await
        let system_clone = system.clone();
        drop(system_guard); // Explicitly drop the guard
        Ok(system_clone.get_audio_devices().await)
    } else {
        Err("System not initialized".to_string())
    }
}

#[tauri::command]
async fn get_input_volume(state: State<'_, AppState>) -> Result<f32, String> {
    let system_guard = state.oxide_system.read().await;
    if let Some(system) = system_guard.as_ref() {
        // Clone the system reference to avoid holding the lock across await
        let system_clone = system.clone();
        drop(system_guard); // Explicitly drop the guard
        system_clone.get_input_volume().await
    } else {
        Err("System not initialized".to_string())
    }
}

#[tauri::command]
async fn get_performance_metrics(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let system_guard = state.oxide_system.read().await;
    if let Some(system) = system_guard.as_ref() {
        let metrics = system.get_performance_metrics().await;
        serde_json::to_value(metrics).map_err(|e| e.to_string())
    } else {
        Err("System not initialized".to_string())
    }
}

#[tauri::command]
async fn get_performance_score(state: State<'_, AppState>) -> Result<f32, String> {
    let system_guard = state.oxide_system.read().await;
    if let Some(system) = system_guard.as_ref() {
        Ok(system.get_performance_score().await)
    } else {
        Err("System not initialized".to_string())
    }
}

#[tauri::command]
async fn optimize_performance(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let system_guard = state.oxide_system.read().await;
    if let Some(system) = system_guard.as_ref() {
        // Clone the system reference to avoid holding the lock across await
        let system_clone = system.clone();
        drop(system_guard); // Explicitly drop the guard
        Ok(system_clone.optimize_performance().await)
    } else {
        Err("System not initialized".to_string())
    }
}

#[tauri::command]
async fn get_error_statistics() -> Result<serde_json::Value, String> {
    GLOBAL_ERROR_MONITOR.get_error_stats()
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_recent_errors(limit: Option<usize>) -> Result<Vec<error_handler::ErrorResponse>, String> {
    let limit = limit.unwrap_or(10);
    GLOBAL_ERROR_MONITOR.get_recent_errors(limit)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_performance_alerts(state: State<'_, AppState>) -> Result<Vec<oxide_core::performance::PerformanceAlert>, String> {
    let system_guard = state.oxide_system.read().await;
    if let Some(system) = system_guard.as_ref() {
        Ok(system.get_performance_alerts().await)
    } else {
        Err("System not initialized".to_string())
    }
}

#[tauri::command]
async fn clear_performance_alerts(state: State<'_, AppState>) -> Result<(), String> {
    let system_guard = state.oxide_system.read().await;
    if let Some(system) = system_guard.as_ref() {
        system.clear_performance_alerts().await;
        Ok(())
    } else {
        Err("System not initialized".to_string())
    }
}

#[tauri::command]
async fn get_operation_profiles(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let system_guard = state.oxide_system.read().await;
    if let Some(system) = system_guard.as_ref() {
        let profiles = system.get_operation_profiles().await;
        serde_json::to_value(profiles).map_err(|e| e.to_string())
    } else {
        Err("System not initialized".to_string())
    }
}

#[tauri::command]
async fn set_performance_monitoring(state: State<'_, AppState>, enabled: bool) -> Result<(), String> {
    let system_guard = state.oxide_system.read().await;
    if let Some(system) = system_guard.as_ref() {
        system.set_performance_monitoring(enabled).await;
        Ok(())
    } else {
        Err("System not initialized".to_string())
    }
}

#[tauri::command]
async fn validate_input(state: State<'_, AppState>, field_name: String, value: String) -> Result<String, String> {
    let system_guard = state.oxide_system.read().await;
    if let Some(system) = system_guard.as_ref() {
        system.validate_input(&field_name, &value).await
    } else {
        Err("System not initialized".to_string())
    }
}

#[tauri::command]
async fn create_security_session(
    state: State<'_, AppState>,
    user_id: String,
    permissions: Vec<String>,
    ip_address: Option<String>,
    user_agent: Option<String>,
) -> Result<String, String> {
    let system_guard = state.oxide_system.read().await;
    if let Some(system) = system_guard.as_ref() {
        system.create_security_session(user_id, permissions, ip_address, user_agent).await
    } else {
        Err("System not initialized".to_string())
    }
}

#[tauri::command]
async fn validate_security_session(state: State<'_, AppState>, session_id: String) -> Result<bool, String> {
    let system_guard = state.oxide_system.read().await;
    if let Some(system) = system_guard.as_ref() {
        system.validate_security_session(&session_id).await
    } else {
        Err("System not initialized".to_string())
    }
}

#[tauri::command]
async fn check_security_permission(state: State<'_, AppState>, session_id: String, permission: String) -> Result<bool, String> {
    let system_guard = state.oxide_system.read().await;
    if let Some(system) = system_guard.as_ref() {
        system.check_security_permission(&session_id, &permission).await
    } else {
        Err("System not initialized".to_string())
    }
}

#[tauri::command]
async fn get_security_events(state: State<'_, AppState>, limit: Option<usize>) -> Result<Vec<oxide_core::security_manager::SecurityEvent>, String> {
    let system_guard = state.oxide_system.read().await;
    if let Some(system) = system_guard.as_ref() {
        Ok(system.get_security_events(limit).await)
    } else {
        Err("System not initialized".to_string())
    }
}

#[tauri::command]
async fn get_security_policy(state: State<'_, AppState>) -> Result<oxide_core::security_manager::SecurityPolicy, String> {
    let system_guard = state.oxide_system.read().await;
    if let Some(system) = system_guard.as_ref() {
        Ok(system.get_security_policy().await)
    } else {
        Err("System not initialized".to_string())
    }
}

#[tauri::command]
async fn check_rate_limit(state: State<'_, AppState>, identifier: String) -> Result<(), String> {
    let system_guard = state.oxide_system.read().await;
    if let Some(system) = system_guard.as_ref() {
        system.check_rate_limit(&identifier).await
    } else {
        Err("System not initialized".to_string())
    }
}

#[tauri::command]
async fn initialize_auth_manager(state: State<'_, AppState>) -> Result<(), String> {
    let auth_manager = AuthManager::new().map_err(|e| e.to_string())?;
    let mut auth_guard = state.auth_manager.write().await;
    *auth_guard = Some(auth_manager);
    Ok(())
}

#[tauri::command]
async fn get_auth_token(state: State<'_, AppState>) -> Result<String, String> {
    let mut auth_guard = state.auth_manager.write().await;
    if let Some(auth_manager) = auth_guard.as_mut() {
        auth_manager.get_auth_token().await.map_err(|e| e.to_string())
    } else {
        Err("Auth manager not initialized".to_string())
    }
}

#[tauri::command]
async fn get_auth_status(state: State<'_, AppState>) -> Result<String, String> {
    let auth_guard = state.auth_manager.read().await;
    if let Some(auth_manager) = auth_guard.as_ref() {
        auth_manager.get_auth_status().map_err(|e| e.to_string())
    } else {
        Ok("Not initialized".to_string())
    }
}

#[tauri::command]
async fn clear_auth(state: State<'_, AppState>) -> Result<(), String> {
    let auth_guard = state.auth_manager.read().await;
    if let Some(auth_manager) = auth_guard.as_ref() {
        auth_manager.clear_auth().map_err(|e| e.to_string())
    } else {
        Err("Auth manager not initialized".to_string())
    }
}

#[tauri::command]
async fn get_available_models() -> Result<Vec<String>, String> {
    use oxide_core::gemini_auth::GeminiAuth;
    let auth = GeminiAuth::new();
    auth.get_available_models().await.map_err(|e| {
        error!("Failed to get available models: {}", e);
        e.to_string()
    })
}

#[tauri::command]
async fn send_message_to_gemini(message: String, model: Option<String>) -> Result<String, String> {
    use oxide_core::gemini_auth::GeminiAuth;
    let auth = GeminiAuth::new();

    // Try to initialize from environment first
    let _ = auth.init_from_env().await;

    auth.send_message(&message, model.as_deref()).await.map_err(|e| {
        error!("Failed to send message to Gemini: {}", e);
        e.to_string()
    })
}

#[tauri::command]
async fn check_auth_from_env() -> Result<String, String> {
    use oxide_core::gemini_auth::GeminiAuth;
    let auth = GeminiAuth::new();

    // Try to initialize from environment
    match auth.init_from_env().await {
        Ok(true) => Ok("Initialized from environment".to_string()),
        Ok(false) => Ok("No environment configuration found".to_string()),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
async fn startup_check(state: State<'_, AppState>) -> Result<String, String> {
    // Try to initialize from environment first
    let _ = check_auth_from_env().await;

    // Initialize auth manager if not already done
    {
        let auth_guard = state.auth_manager.read().await;
        if auth_guard.is_none() {
            drop(auth_guard);
            initialize_auth_manager(state.clone()).await?;
        }
    }

    // Check authentication status
    use oxide_core::gemini_auth::GeminiAuth;
    let auth = GeminiAuth::new();
    auth.get_auth_status().await.map_err(|e| e.to_string())
}



fn main() {
    // Load environment variables from .env file
    dotenv::dotenv().ok();

    // Initialize logging
    env_logger::init();

    info!("Starting Oxide Pilot Application");

    tauri::Builder::default()
        .manage(AppState {
            oxide_system: Arc::new(RwLock::new(None)),
            auth_manager: Arc::new(RwLock::new(None)),
        })
        .invoke_handler(tauri::generate_handler![
            send_notification,
            set_google_api_key,
            set_google_client_credentials,
            authenticate_google_command,
            get_available_models,
            send_message_to_gemini,
            check_auth_from_env,
            initialize_system,
            handle_user_input_command,
            get_system_status,
            get_threat_history,
            get_memory_stats,
            update_system_config,
            get_system_config,
            record_audio,
            play_audio,
            get_audio_devices,
            get_input_volume,
            get_performance_metrics,
            get_performance_score,
            optimize_performance,
            get_error_statistics,
            get_recent_errors,
            get_performance_alerts,
            clear_performance_alerts,
            get_operation_profiles,
            set_performance_monitoring,
            validate_input,
            create_security_session,
            validate_security_session,
            check_security_permission,
            get_security_events,
            get_security_policy,
            check_rate_limit,
            initialize_auth_manager,
            get_auth_token,
            get_auth_status,
            clear_auth,
            startup_check
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn send_notification(title: String, body: String) {
    // For Tauri 2.x, notifications are handled differently
    // This is a placeholder implementation
    log::info!("Notification: {} - {}", title, body);
}
