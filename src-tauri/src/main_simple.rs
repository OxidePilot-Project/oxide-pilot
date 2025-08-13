#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use oxide_core::gemini_auth::GeminiAuth;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {name}! You've been greeted from Rust!")
}

#[tauri::command]
async fn authenticate_google_command() -> Result<String, String> {
    let auth = GeminiAuth::new();
    match auth.start_oauth_flow().await {
        Ok(message) => Ok(message),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
async fn set_google_api_key(api_key: String) -> Result<(), String> {
    let auth = GeminiAuth::new();
    match auth.store_api_key(&api_key).await {
        Ok(_) => {
            println!("API key stored successfully");
            Ok(())
        },
        Err(e) => {
            println!("Failed to store API key: {e}");
            Err(e.to_string())
        }
    }
}

#[tauri::command]
async fn set_google_client_credentials(
    client_id: String,
    client_secret: String,
) -> Result<(), String> {
    // For OAuth flow - this would be implemented later
    println!("OAuth credentials received: {client_id} / {client_secret}");
    Ok(())
}

#[tauri::command]
async fn startup_check() -> Result<String, String> {
    let auth = GeminiAuth::new();
    match auth.get_auth_status().await {
        Ok(status) => Ok(status),
        Err(e) => {
            println!("Error checking auth status: {e}");
            Ok("Not authenticated".to_string())
        }
    }
}

#[tauri::command]
async fn handle_user_input_command(user_input: String) -> Result<String, String> {
    let auth = GeminiAuth::new();

    // Check if authenticated
    if !auth.is_authenticated().await {
        return Err("Please authenticate with Google Gemini API first".to_string());
    }

    // For now, just echo back - later this would call Gemini API
    Ok(format!("Oxide Pilot: I received your message: '{user_input}'"))
}

#[tauri::command]
async fn get_available_models() -> Result<Vec<String>, String> {
    let auth = GeminiAuth::new();
    match auth.get_available_models().await {
        Ok(models) => Ok(models),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
async fn clear_auth() -> Result<(), String> {
    let auth = GeminiAuth::new();
    match auth.clear_auth().await {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            greet,
            authenticate_google_command,
            set_google_api_key,
            set_google_client_credentials,
            startup_check,
            handle_user_input_command,
            get_available_models,
            clear_auth
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}