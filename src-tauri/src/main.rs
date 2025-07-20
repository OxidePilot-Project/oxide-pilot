
#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use oxide_core::google_auth;
use log::{error, info};
use tauri::State;
use std::sync::{Arc, Mutex};
use oxide_copilot::copilot::CopilotAgent;
use oxide_copilot::ai::AIOrchestrator;
use oxide_copilot::functions::FunctionRegistry;
use oxide_core::config::{OxidePilotConfig, CopilotConfig, AIProvidersConfig, GuardianConfig};
use oxide_core::types::Context;

// Define a struct to hold the application state
pub struct AppState {
    copilot_agent: Arc<CopilotAgent>,
}

#[tauri::command]
async fn set_google_client_credentials(client_id: String, client_secret: String) -> Result<(), String> {
    google_auth::store_client_credentials(&client_id, &client_secret)
        .await
        .map_err(|e| {
            error!("Failed to store Google client credentials: {}", e);
            e.to_string()
        })
}

#[tauri::command]
async fn authenticate_google_command() -> Result<String, String> {
    google_auth::authenticate_google()
        .await
        .map_err(|e| {
            error!("Google authentication failed: {}", e);
            e.to_string()
        })
}

#[tauri::command]
async fn handle_user_input_command(user_input: String, context: Context, state: State<'_, AppState>) -> Result<String, String> {
    let copilot_agent = &state.copilot_agent;
    copilot_agent.handle_user_input(user_input, context)
        .await
        .map_err(|e| {
            error!("Error handling user input: {}", e);
            e.to_string()
        })
}

fn main() {
    // Initialize logging
    env_logger::init();

    // Load configuration (placeholder for now, will be from file later)
    let config = OxidePilotConfig {
        guardian: GuardianConfig { enabled: false, monitor_interval_secs: 0 },
        copilot: CopilotConfig { enabled: true, wake_word: "Hey Oxide".to_string() },
        ai_providers: AIProvidersConfig {
            google: Some(oxide_core::config::GoogleConfig { api_key: "dummy_key".to_string() }),
            openai: None,
            anthropic: None,
            azure_openai: None,
            ollama: None,
        },
    };

    // Initialize AI Orchestrator and Function Registry
    let ai_orchestrator = Arc::new(AIOrchestrator::new(config.ai_providers));
    let function_registry = Arc::new(FunctionRegistry::new());

    // Initialize Copilot Agent
    let copilot_agent = Arc::new(CopilotAgent::new(
        config.copilot,
        ai_orchestrator,
        function_registry,
    ));

    tauri::Builder::default()
        .manage(AppState { copilot_agent })
        .invoke_handler(tauri::generate_handler![
            send_notification,
            set_google_client_credentials,
            authenticate_google_command,
            handle_user_input_command
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn send_notification(title: String, body: String) {
    tauri::api::notification::Notification::new("com.tauri.dev")
        .title(title)
        .body(body)
        .show().unwrap();
}
