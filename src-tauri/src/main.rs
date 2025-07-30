#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod oxide_system;

use log::{error, info};
use oxide_core::config::OxidePilotConfig;
use oxide_core::google_auth;
use oxide_guardian::guardian::{SystemStatus, ThreatEvent};
use oxide_memory::memory::MemoryStats;
use oxide_system::OxideSystem;
use std::sync::{Arc, Mutex};
use tauri::State;

// Define a struct to hold the application state
pub struct AppState {
    oxide_system: Arc<Mutex<Option<OxideSystem>>>,
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

    let system = OxideSystem::new(config).await?;
    system.start().await?;

    let mut system_lock = state.oxide_system.lock().unwrap();
    *system_lock = Some(system);

    info!("Oxide System initialized and started");
    Ok(())
}

#[tauri::command]
async fn handle_user_input_command(
    user_input: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let system_lock = state.oxide_system.lock().unwrap();
    if let Some(system) = system_lock.as_ref() {
        system.handle_text_input(user_input).await
    } else {
        Err("System not initialized".to_string())
    }
}

#[tauri::command]
async fn get_system_status(state: State<'_, AppState>) -> Result<SystemStatus, String> {
    let system_lock = state.oxide_system.lock().unwrap();
    if let Some(system) = system_lock.as_ref() {
        Ok(system.get_system_status())
    } else {
        Err("System not initialized".to_string())
    }
}

#[tauri::command]
async fn get_threat_history(state: State<'_, AppState>) -> Result<Vec<ThreatEvent>, String> {
    let system_lock = state.oxide_system.lock().unwrap();
    if let Some(system) = system_lock.as_ref() {
        Ok(system.get_threat_history())
    } else {
        Err("System not initialized".to_string())
    }
}

#[tauri::command]
async fn get_memory_stats(state: State<'_, AppState>) -> Result<MemoryStats, String> {
    let system_lock = state.oxide_system.lock().unwrap();
    if let Some(system) = system_lock.as_ref() {
        Ok(system.get_memory_stats())
    } else {
        Err("System not initialized".to_string())
    }
}

#[tauri::command]
async fn update_system_config(
    config: OxidePilotConfig,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let system_lock = state.oxide_system.lock().unwrap();
    if let Some(system) = system_lock.as_ref() {
        system.update_config(config).await
    } else {
        Err("System not initialized".to_string())
    }
}

#[tauri::command]
async fn get_system_config(state: State<'_, AppState>) -> Result<OxidePilotConfig, String> {
    let system_lock = state.oxide_system.lock().unwrap();
    if let Some(system) = system_lock.as_ref() {
        Ok(system.get_config())
    } else {
        Err("System not initialized".to_string())
    }
}

#[tauri::command]
async fn record_audio(duration_secs: f32, state: State<'_, AppState>) -> Result<Vec<u8>, String> {
    let system_lock = state.oxide_system.lock().unwrap();
    if let Some(system) = system_lock.as_ref() {
        system.record_audio(duration_secs).await
    } else {
        Err("System not initialized".to_string())
    }
}

#[tauri::command]
async fn play_audio(audio_data: Vec<u8>, state: State<'_, AppState>) -> Result<(), String> {
    let system_lock = state.oxide_system.lock().unwrap();
    if let Some(system) = system_lock.as_ref() {
        system.play_audio(&audio_data).await
    } else {
        Err("System not initialized".to_string())
    }
}

#[tauri::command]
async fn get_audio_devices(
    state: State<'_, AppState>,
) -> Result<(Vec<String>, Vec<String>), String> {
    let system_lock = state.oxide_system.lock().unwrap();
    if let Some(system) = system_lock.as_ref() {
        Ok(system.get_audio_devices())
    } else {
        Err("System not initialized".to_string())
    }
}

#[tauri::command]
async fn get_input_volume(state: State<'_, AppState>) -> Result<f32, String> {
    let system_lock = state.oxide_system.lock().unwrap();
    if let Some(system) = system_lock.as_ref() {
        system.get_input_volume()
    } else {
        Err("System not initialized".to_string())
    }
}

#[tauri::command]
async fn get_performance_metrics(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let system_lock = state.oxide_system.lock().unwrap();
    if let Some(system) = system_lock.as_ref() {
        let metrics = system.get_performance_metrics();
        serde_json::to_value(metrics).map_err(|e| e.to_string())
    } else {
        Err("System not initialized".to_string())
    }
}

#[tauri::command]
async fn get_performance_score(state: State<'_, AppState>) -> Result<f32, String> {
    let system_lock = state.oxide_system.lock().unwrap();
    if let Some(system) = system_lock.as_ref() {
        Ok(system.get_performance_score())
    } else {
        Err("System not initialized".to_string())
    }
}

#[tauri::command]
async fn optimize_performance(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let system_lock = state.oxide_system.lock().unwrap();
    if let Some(system) = system_lock.as_ref() {
        Ok(system.optimize_performance().await)
    } else {
        Err("System not initialized".to_string())
    }
}

#[tauri::command]
async fn get_performance_metrics(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let system_lock = state.oxide_system.lock().unwrap();
    if let Some(system) = system_lock.as_ref() {
        let metrics = system.get_performance_metrics();
        serde_json::to_value(metrics).map_err(|e| e.to_string())
    } else {
        Err("System not initialized".to_string())
    }
}

#[tauri::command]
async fn get_performance_score(state: State<'_, AppState>) -> Result<f32, String> {
    let system_lock = state.oxide_system.lock().unwrap();
    if let Some(system) = system_lock.as_ref() {
        Ok(system.get_performance_score())
    } else {
        Err("System not initialized".to_string())
    }
}

#[tauri::command]
async fn optimize_performance(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let system_lock = state.oxide_system.lock().unwrap();
    if let Some(system) = system_lock.as_ref() {
        Ok(system.optimize_performance())
    } else {
        Err("System not initialized".to_string())
    }
}

fn main() {
    // Initialize logging
    env_logger::init();

    info!("Starting Oxide Pilot Application");

    tauri::Builder::default()
        .manage(AppState {
            oxide_system: Arc::new(Mutex::new(None)),
        })
        .invoke_handler(tauri::generate_handler![
            send_notification,
            set_google_client_credentials,
            authenticate_google_command,
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
            optimize_performance
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn send_notification(title: String, body: String) {
    tauri::api::notification::Notification::new("com.tauri.dev")
        .title(title)
        .body(body)
        .show()
        .unwrap();
}
