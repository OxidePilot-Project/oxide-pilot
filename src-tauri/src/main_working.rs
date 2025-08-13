#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use log::{error, info};
use oxide_core::gemini_auth::GeminiAuth;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;

// Define a simplified struct to hold the application state
pub struct AppState {
    #[allow(dead_code)]
    auth: Arc<RwLock<Option<GeminiAuth>>>,
}

#[tauri::command]
async fn set_google_api_key(api_key: String) -> Result<(), String> {
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
    // For now, just log the credentials - OAuth implementation would go here
    info!("OAuth credentials received: {} / {}", client_id, client_secret);
    Ok(())
}

#[tauri::command]
async fn authenticate_google_command() -> Result<String, String> {
    let auth = GeminiAuth::new();
    auth.start_oauth_flow().await.map_err(|e| {
        error!("Google authentication failed: {}", e);
        e.to_string()
    })
}

#[tauri::command]
async fn get_available_models() -> Result<Vec<String>, String> {
    let auth = GeminiAuth::new();
    auth.get_available_models().await.map_err(|e| {
        error!("Failed to get available models: {}", e);
        e.to_string()
    })
}

#[tauri::command]
async fn send_message_to_gemini(message: String, model: Option<String>) -> Result<String, String> {
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
    let auth = GeminiAuth::new();

    // Try to initialize from environment
    match auth.init_from_env().await {
        Ok(true) => Ok("Initialized from environment".to_string()),
        Ok(false) => Ok("No environment configuration found".to_string()),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
async fn startup_check() -> Result<String, String> {
    // Try to initialize from environment first
    let _ = check_auth_from_env().await;

    // Check authentication status
    let auth = GeminiAuth::new();
    auth.get_auth_status().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_auth_status() -> Result<String, String> {
    let auth = GeminiAuth::new();
    auth.get_auth_status().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn clear_auth() -> Result<(), String> {
    let auth = GeminiAuth::new();
    auth.clear_auth().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn handle_user_input_command(user_input: String) -> Result<String, String> {
    let auth = GeminiAuth::new();

    // Check if authenticated
    if !auth.is_authenticated().await {
        return Err("Please authenticate with Google Gemini API first".to_string());
    }

    // Send message to Gemini
    auth.send_message(&user_input, None).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_system_config() -> Result<serde_json::Value, String> {
    // Return a basic config structure
    let config = json!({
        "ai_providers": {
            "google": {
                "api_key": null,
                "model": "gemini-1.5-flash"
            }
        },
        "system": {
            "debug": false,
            "log_level": "info"
        }
    });
    Ok(config)
}

#[tauri::command]
async fn update_system_config(config: serde_json::Value) -> Result<(), String> {
    info!("System config updated: {}", config);
    Ok(())
}

#[tauri::command]
fn send_notification(title: String, body: String) {
    log::info!("Notification: {} - {}", title, body);
}

fn main() {
    // Load environment variables from .env file
    dotenv::dotenv().ok();

    // Initialize logging
    env_logger::init();

    info!("Starting Oxide Pilot Application");

    tauri::Builder::default()
        .manage(AppState {
            auth: Arc::new(RwLock::new(None)),
        })
        .invoke_handler(tauri::generate_handler![
            send_notification,
            set_google_api_key,
            set_google_client_credentials,
            authenticate_google_command,
            get_available_models,
            send_message_to_gemini,
            check_auth_from_env,
            handle_user_input_command,
            get_system_config,
            update_system_config,
            get_auth_status,
            clear_auth,
            startup_check
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}