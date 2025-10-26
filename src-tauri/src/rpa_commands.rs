use oxide_rpa::permissions::{Permission, PermissionPolicy, RiskLevel};
use oxide_rpa::audit::{AuditEntry, AuditStats};
use oxide_rpa::rollback::ReversibleAction;
use oxide_rpa::confirmation::{ConfirmationRequest, ConfirmationResponse};
use oxide_rpa::secure_rpa::SecureRPAController;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;

pub struct RPAState {
    pub controller: Arc<RwLock<Option<SecureRPAController>>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RPAInitConfig {
    pub policy_type: String, // "default", "permissive", "restrictive"
    pub max_audit_entries: Option<usize>,
    pub max_rollback_history: Option<usize>,
}

// ==============================
// RPA Initialization Commands
// ==============================

#[tauri::command]
pub async fn rpa_initialize(
    config: RPAInitConfig,
    state: State<'_, RPAState>,
) -> Result<String, String> {
    let policy = match config.policy_type.as_str() {
        "permissive" => PermissionPolicy::permissive(),
        "restrictive" => PermissionPolicy::restrictive(),
        _ => PermissionPolicy::default(),
    };

    let mut controller = SecureRPAController::new(policy);

    if let Some(audit_size) = config.max_audit_entries {
        controller = controller.with_audit_size(audit_size);
    }

    if let Some(rollback_size) = config.max_rollback_history {
        controller = controller.with_rollback_size(rollback_size);
    }

    let mut state_lock = state.controller.write().await;
    *state_lock = Some(controller);

    Ok("RPA system initialized successfully".to_string())
}

#[tauri::command]
pub async fn rpa_shutdown(state: State<'_, RPAState>) -> Result<String, String> {
    let mut state_lock = state.controller.write().await;
    *state_lock = None;
    Ok("RPA system shutdown successfully".to_string())
}

// ==============================
// Permission Management Commands
// ==============================

#[tauri::command]
pub async fn rpa_grant_permission(
    permission: String,
    state: State<'_, RPAState>,
) -> Result<(), String> {
    let perm = parse_permission(&permission)?;

    let state_lock = state.controller.read().await;
    let controller = state_lock.as_ref().ok_or("RPA not initialized")?;

    // Note: This requires making policy mutable - for now return info
    Ok(())
}

#[tauri::command]
pub async fn rpa_check_permission(
    permission: String,
    state: State<'_, RPAState>,
) -> Result<bool, String> {
    let perm = parse_permission(&permission)?;

    let state_lock = state.controller.read().await;
    let controller = state_lock.as_ref().ok_or("RPA not initialized")?;

    // Access policy through controller (would need getter method)
    Ok(true) // Placeholder
}

// ==============================
// Mouse Control Commands
// ==============================

#[tauri::command]
pub async fn rpa_move_mouse(
    x: i32,
    y: i32,
    state: State<'_, RPAState>,
) -> Result<(), String> {
    let state_lock = state.controller.read().await;
    let controller = state_lock.as_ref().ok_or("RPA not initialized")?;

    controller.move_mouse(x, y).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn rpa_click_mouse(
    button: String,
    state: State<'_, RPAState>,
) -> Result<(), String> {
    let state_lock = state.controller.read().await;
    let controller = state_lock.as_ref().ok_or("RPA not initialized")?;

    let btn = match button.as_str() {
        "left" => rdev::Button::Left,
        "right" => rdev::Button::Right,
        "middle" => rdev::Button::Middle,
        _ => return Err("Invalid button".to_string()),
    };

    controller.click_mouse(btn).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn rpa_scroll_mouse(
    delta_x: i32,
    delta_y: i32,
    state: State<'_, RPAState>,
) -> Result<(), String> {
    let state_lock = state.controller.read().await;
    let controller = state_lock.as_ref().ok_or("RPA not initialized")?;

    controller.scroll_mouse(delta_x, delta_y).await.map_err(|e| e.to_string())
}

// ==============================
// Keyboard Control Commands
// ==============================

#[tauri::command]
pub async fn rpa_type_text(
    text: String,
    state: State<'_, RPAState>,
) -> Result<(), String> {
    let state_lock = state.controller.read().await;
    let controller = state_lock.as_ref().ok_or("RPA not initialized")?;

    controller.type_text(&text).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn rpa_press_key(
    key: String,
    state: State<'_, RPAState>,
) -> Result<(), String> {
    let state_lock = state.controller.read().await;
    let controller = state_lock.as_ref().ok_or("RPA not initialized")?;

    let k = parse_key(&key)?;
    controller.press_key(k).await.map_err(|e| e.to_string())
}

// ==============================
// Screen Capture Commands
// ==============================

#[tauri::command]
pub async fn rpa_capture_screen(
    state: State<'_, RPAState>,
) -> Result<Vec<u8>, String> {
    let state_lock = state.controller.read().await;
    let controller = state_lock.as_ref().ok_or("RPA not initialized")?;

    let image = controller.capture_screen().await.map_err(|e| e.to_string())?;

    // Convert image to PNG bytes
    let mut bytes: Vec<u8> = Vec::new();
    image.write_to(&mut std::io::Cursor::new(&mut bytes), image::ImageOutputFormat::Png)
        .map_err(|e| e.to_string())?;

    Ok(bytes)
}

// ==============================
// Audit Commands
// ==============================

#[tauri::command]
pub async fn rpa_get_audit_entries(
    state: State<'_, RPAState>,
) -> Result<Vec<AuditEntry>, String> {
    let state_lock = state.controller.read().await;
    let controller = state_lock.as_ref().ok_or("RPA not initialized")?;

    controller.audit().get_entries().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn rpa_get_audit_stats(
    state: State<'_, RPAState>,
) -> Result<AuditStats, String> {
    let state_lock = state.controller.read().await;
    let controller = state_lock.as_ref().ok_or("RPA not initialized")?;

    controller.audit().get_stats().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn rpa_get_failed_actions(
    state: State<'_, RPAState>,
) -> Result<Vec<AuditEntry>, String> {
    let state_lock = state.controller.read().await;
    let controller = state_lock.as_ref().ok_or("RPA not initialized")?;

    controller.audit().get_failed().map_err(|e| e.to_string())
}

// ==============================
// Rollback Commands
// ==============================

#[tauri::command]
pub async fn rpa_get_rollback_history(
    state: State<'_, RPAState>,
) -> Result<Vec<ReversibleAction>, String> {
    let state_lock = state.controller.read().await;
    let controller = state_lock.as_ref().ok_or("RPA not initialized")?;

    controller.rollback().get_history().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn rpa_rollback_last(
    state: State<'_, RPAState>,
) -> Result<(), String> {
    let state_lock = state.controller.read().await;
    let controller = state_lock.as_ref().ok_or("RPA not initialized")?;

    controller.rollback_last().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn rpa_get_reversible_count(
    state: State<'_, RPAState>,
) -> Result<usize, String> {
    let state_lock = state.controller.read().await;
    let controller = state_lock.as_ref().ok_or("RPA not initialized")?;

    controller.rollback().reversible_count().map_err(|e| e.to_string())
}

// ==============================
// Confirmation Commands
// ==============================

#[tauri::command]
pub async fn rpa_get_pending_confirmations(
    state: State<'_, RPAState>,
) -> Result<Vec<ConfirmationRequest>, String> {
    let state_lock = state.controller.read().await;
    let controller = state_lock.as_ref().ok_or("RPA not initialized")?;

    controller.confirmation().get_pending().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn rpa_respond_confirmation(
    request_id: String,
    approved: bool,
    reason: Option<String>,
    state: State<'_, RPAState>,
) -> Result<(), String> {
    let state_lock = state.controller.read().await;
    let controller = state_lock.as_ref().ok_or("RPA not initialized")?;

    controller.confirmation()
        .respond(&request_id, approved, reason)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn rpa_add_auto_approve(
    permission: String,
    state: State<'_, RPAState>,
) -> Result<(), String> {
    let perm = parse_permission(&permission)?;

    let state_lock = state.controller.read().await;
    let controller = state_lock.as_ref().ok_or("RPA not initialized")?;

    controller.confirmation()
        .add_auto_approve(perm)
        .map_err(|e| e.to_string())
}

// ==============================
// Helper Functions
// ==============================

fn parse_permission(s: &str) -> Result<Permission, String> {
    match s.to_lowercase().as_str() {
        "mouse_move" => Ok(Permission::MouseMove),
        "mouse_click" => Ok(Permission::MouseClick),
        "mouse_scroll" => Ok(Permission::MouseScroll),
        "mouse_drag" => Ok(Permission::MouseDrag),
        "keyboard_type" => Ok(Permission::KeyboardType),
        "keyboard_press" => Ok(Permission::KeyboardPress),
        "keyboard_hotkey" => Ok(Permission::KeyboardHotkey),
        "screen_capture" => Ok(Permission::ScreenCapture),
        "screen_capture_area" => Ok(Permission::ScreenCaptureArea),
        "screen_analyze" => Ok(Permission::ScreenAnalyze),
        "file_read" => Ok(Permission::FileRead),
        "file_write" => Ok(Permission::FileWrite),
        "file_delete" => Ok(Permission::FileDelete),
        "system_command" => Ok(Permission::SystemCommand),
        "process_control" => Ok(Permission::ProcessControl),
        "network_access" => Ok(Permission::NetworkAccess),
        _ => Err(format!("Unknown permission: {}", s)),
    }
}

fn parse_key(s: &str) -> Result<rdev::Key, String> {
    match s.to_lowercase().as_str() {
        "enter" | "return" => Ok(rdev::Key::Return),
        "escape" | "esc" => Ok(rdev::Key::Escape),
        "space" => Ok(rdev::Key::Space),
        "tab" => Ok(rdev::Key::Tab),
        "backspace" => Ok(rdev::Key::Backspace),
        "delete" => Ok(rdev::Key::Delete),
        "up" => Ok(rdev::Key::UpArrow),
        "down" => Ok(rdev::Key::DownArrow),
        "left" => Ok(rdev::Key::LeftArrow),
        "right" => Ok(rdev::Key::RightArrow),
        _ => Err(format!("Unknown key: {}", s)),
    }
}
