#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod error_handler;
mod guardian_commands;
mod local_llm;
mod mcp_server;
mod oxide_system;
mod rpa_commands;
mod threat_consensus;

#[cfg(test)]
mod rpa_integration_test;

use crate::mcp_server::McpServerHandle;
use error_handler::{
    retry_with_backoff, ErrorHandler, OxideError, RetryConfig, GLOBAL_ERROR_MONITOR,
};
use log::{error, info, warn};
use oxide_copilot::auth_manager::AuthManager;
use oxide_core::config::OxidePilotConfig;
use oxide_core::google_auth;
use oxide_core::openai_auth;
use oxide_core::openai_key;
use oxide_core::qwen_auth::{DeviceAuthStart, PollResult, QwenAuth};
use oxide_guardian::guardian::{SystemStatus, ThreatEvent};
use oxide_guardian::scanner::FileScanReport;
use oxide_memory::memory::MemoryStats;
use oxide_system::OxideSystem;
use serde_json::json;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Instant;
use tauri::{Manager, State};
use tokio::sync::{mpsc, Mutex, RwLock};

// Define a struct to hold the application state with async-safe mutexes
pub struct AppState {
    oxide_system: Arc<RwLock<Option<OxideSystem>>>,
    auth_manager: Arc<RwLock<Option<AuthManager>>>,
    mcp_server: Arc<RwLock<Option<McpServerHandle>>>,
    // Track folder scan cancellation flags by scan_id
    folder_scan_cancels: Arc<RwLock<HashMap<String, Arc<AtomicBool>>>>,
    // RPA controller state
    rpa_state: Arc<RwLock<Option<oxide_rpa::secure_rpa::SecureRPAController>>>,
    // Guardian state
    #[cfg(feature = "surrealdb-metrics")]
    guardian_state: Arc<guardian_commands::GuardianState>,
}

// ==============================
// Local LLM (LM Studio) Commands
// ==============================
#[tauri::command]
async fn local_llm_server_start(port: Option<u16>, cors: Option<bool>) -> Result<String, String> {
    local_llm::server_start(port, cors.unwrap_or(true)).await
}

#[tauri::command]
async fn local_llm_server_stop() -> Result<String, String> {
    local_llm::server_stop().await
}

#[tauri::command]
async fn local_llm_server_status() -> Result<serde_json::Value, String> {
    let status = local_llm::server_status().await?;
    serde_json::to_value(status).map_err(|e| e.to_string())
}

#[tauri::command]
async fn local_llm_ls() -> Result<String, String> {
    local_llm::ls_json().await
}

#[tauri::command]
async fn local_llm_get(
    model_spec: String,
    gguf: Option<bool>,
    yes: Option<bool>,
) -> Result<String, String> {
    local_llm::get_model(&model_spec, gguf.unwrap_or(true), yes.unwrap_or(true)).await
}

#[tauri::command]
async fn local_llm_load(
    model_key: String,
    identifier: Option<String>,
    context_len: Option<u32>,
    gpu: Option<String>,
    ttl_secs: Option<u32>,
) -> Result<String, String> {
    local_llm::load_model(
        &model_key,
        identifier.as_deref(),
        context_len,
        gpu.as_deref(),
        ttl_secs,
    )
    .await
}

#[tauri::command]
async fn local_llm_chat(
    base_url: Option<String>,
    api_key: Option<String>,
    model: Option<String>,
    system_prompt: Option<String>,
    user_prompt: String,
) -> Result<String, String> {
    let resolved_base = base_url.or_else(|| std::env::var("LOCAL_LLM_BASE_URL").ok());
    let resolved_key = api_key.or_else(|| std::env::var("LOCAL_LLM_API_KEY").ok());
    let resolved_model = model
        .or_else(|| std::env::var("LOCAL_LLM_MODEL").ok())
        .unwrap_or_else(|| "ui-tars-local".to_string());
    local_llm::chat_completion(
        resolved_base,
        resolved_key,
        resolved_model,
        system_prompt,
        user_prompt,
    )
    .await
}

// Call Qwen Chat Completions API using stored OAuth token
async fn qwen_chat_completion(prompt: &str, model: Option<String>) -> Result<String, String> {
    // Resolve config
    let base =
        std::env::var("QWEN_API_BASE").map_err(|_| "Missing env QWEN_API_BASE".to_string())?;
    let path = std::env::var("QWEN_CHAT_COMPLETIONS_PATH")
        .unwrap_or_else(|_| "/v1/chat/completions".to_string());
    let url = format!("{base}{path}");
    let model_name = model
        .or_else(|| std::env::var("QWEN_MODEL").ok())
        .unwrap_or_else(|| "qwen-plus".to_string());

    // Auth header from stored OAuth token
    let qauth = QwenAuth::new();
    let auth_header = qauth.get_auth_header().await.map_err(|e| e.to_string())?;

    let body = serde_json::json!({
        "model": model_name,
        "messages": [
            {"role": "system", "content": "You are an expert OS internals, performance, and security analyst. Respond concisely and technically."},
            {"role": "user", "content": prompt}
        ],
        "temperature": 0.2
    });

    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .header("Authorization", auth_header)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Qwen API error: {status} - {text}"));
    }

    let v: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    // Prefer OpenAI-compatible shape: choices[0].message.content
    if let Some(content) = v
        .get("choices")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|first| first.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
    {
        return Ok(content.to_string());
    }
    // Fallbacks
    if let Some(text) = v.get("text").and_then(|t| t.as_str()) {
        return Ok(text.to_string());
    }
    Err("Unexpected Qwen response format".to_string())
}

// Enhanced collaborative LLM analysis using the new orchestrator
#[tauri::command]
async fn run_collaborative_analysis(
    state: State<'_, AppState>,
    user_input: String,
    task_type: Option<String>,
) -> Result<String, String> {
    let snapshot_val = get_system_snapshot(state).await?;

    // Create collaborative context
    let context = oxide_copilot::llm_orchestrator::CollaborativeContext {
        task_type: task_type.unwrap_or_else(|| "system_analysis".to_string()),
        system_state: snapshot_val,
        user_input,
        conversation_history: vec![],
        available_functions: vec![
            "scan_file".to_string(),
            "get_system_status".to_string(),
            "run_system_analysis".to_string(),
            "get_threat_history".to_string(),
        ],
        constraints: {
            let mut map = std::collections::HashMap::new();
            map.insert("max_execution_time".to_string(), serde_json::json!(300));
            map.insert("security_level".to_string(), serde_json::json!("high"));
            map.insert(
                "performance_impact".to_string(),
                serde_json::json!("minimal"),
            );
            map
        },
    };

    // Create and configure the orchestrator
    let mut orchestrator = oxide_copilot::llm_orchestrator::LLMOrchestrator::new();

    // Add collaborative providers
    use oxide_copilot::collaborative_providers::CollaborativeProviderFactory;
    use oxide_copilot::llm_orchestrator::{LLMConfig, LLMRole};

    let providers = CollaborativeProviderFactory::create_default_setup();
    for (name, provider, role) in providers {
        let config = LLMConfig {
            provider: name.clone(),
            model: Some("default".to_string()),
            role: role.clone(),
            temperature: match role {
                LLMRole::Coordinator => 0.3,
                LLMRole::Analyst => 0.1,
                LLMRole::Executor => 0.2,
                LLMRole::Innovator => 0.7,
                LLMRole::Validator => 0.1,
            },
            max_tokens: Some(2048),
            system_prompt: format!("You are a {role} for the Oxide Pilot system."),
        };
        orchestrator.add_provider(name, provider, config);
    }

    // Execute collaborative task
    let task = "Analyze system performance and security, provide recommendations, and create an execution plan";

    match orchestrator.execute_collaborative_task(task, context).await {
        Ok(result) => {
            let response = serde_json::json!({
                "success": true,
                "primary_response": result.primary_response,
                "secondary_responses": result.secondary_responses,
                "consensus_score": result.consensus_score,
                "confidence": result.confidence,
                "execution_plan": result.execution_plan,
                "timestamp": chrono::Utc::now().to_rfc3339()
            });
            Ok(serde_json::to_string_pretty(&response)
                .unwrap_or_else(|_| "Serialization error".to_string()))
        }
        Err(e) => {
            error!("Collaborative analysis failed: {e}");
            Err(format!("Collaborative analysis failed: {e}"))
        }
    }
}

// Legacy multi-agent orchestration (kept for backward compatibility)
#[tauri::command]
async fn run_multi_agent_analysis(
    state: State<'_, AppState>,
    gemini_model: Option<String>,
    qwen_model: Option<String>,
) -> Result<String, String> {
    let snapshot_val = get_system_snapshot(state).await?;
    let snapshot_str =
        serde_json::to_string_pretty(&snapshot_val).unwrap_or_else(|_| snapshot_val.to_string());

    // Prompts for each agent
    let gemini_prompt = format!(
        "You are an expert OS performance and security analyst. Given this JSON snapshot, produce a concise analysis with:\n\
        - Key performance issues and likely root causes\n\
        - Suspicious processes or threats (if any)\n\
        - Immediate remediation steps (bulleted)\n\
        - Risk score (0-100) and confidence.\n\nSnapshot:\n{snapshot_str}"
    );

    let qwen_prompt = format!(
        "Perform a deep technical analysis of this system snapshot focusing on:\n\
        - Hot threads and blocking syscalls\n\
        - Memory pressure, leaks, fragmentation indicators\n\
        - Process anomalies (handles, CPU spikes, I/O)\n\
        - Concrete remediation with commands and config changes.\n\nSnapshot:\n{snapshot_str}"
    );

    use oxide_core::gemini_auth::GeminiAuth;
    let gauth = GeminiAuth::new();
    let _ = gauth.init_from_env().await; // best-effort API key init

    // Run both analyses concurrently
    let (g_res, q_res) = tokio::join!(
        async {
            gauth
                .send_message(&gemini_prompt, gemini_model.as_deref())
                .await
                .map_err(|e| e.to_string())
        },
        async { qwen_chat_completion(&qwen_prompt, qwen_model).await }
    );

    let result = serde_json::json!({
        "gemini_summary": g_res.as_deref().unwrap_or("Gemini analysis failed"),
        "qwen_deep_analysis": q_res.as_deref().unwrap_or("Qwen analysis failed"),
        "snapshot": snapshot_val,
    });
    Ok(result.to_string())
}

#[tauri::command]
async fn set_google_api_key(_api_key: String) -> Result<(), String> {
    // API key-based authentication is disabled. Use OAuth 2.0 instead.
    let msg =
        "Gemini API key authentication is disabled. Please use OAuth 2.0 via Google credentials.";
    error!("{msg}");
    Err(msg.to_string())
}

#[tauri::command]
async fn set_google_client_credentials(
    client_id: String,
    client_secret: String,
) -> Result<(), String> {
    google_auth::store_client_credentials(&client_id, &client_secret)
        .await
        .map_err(|e| {
            error!("Failed to store Google client credentials: {e}");
            e.to_string()
        })
}

#[tauri::command]
async fn authenticate_google_command(app: tauri::AppHandle) -> Result<String, String> {
    match google_auth::authenticate_google().await {
        Ok(token) => {
            let _ = app.emit_all(
                "google_auth_complete",
                json!({
                    "status": "success",
                    "provider": "google",
                    "timestamp": std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_secs())
                        .unwrap_or(0),
                }),
            );
            Ok(token)
        }
        Err(e) => {
            error!("Google authentication failed: {e}");
            let _ = app.emit_all(
                "google_auth_complete",
                json!({
                    "status": "error",
                    "provider": "google",
                    "message": e.to_string(),
                }),
            );
            Err(e.to_string())
        }
    }
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
                    .map_err(OxideError::SystemInit)?;
                system
                    .start()
                    .await
                    .map_err(OxideError::SystemInit)?;
                Ok::<OxideSystem, OxideError>(system)
            })
        },
        retry_config,
    )
    .await;

    match result {
        Ok(system) => {
            let mut system_lock = state.oxide_system.write().await;
            *system_lock = Some(system);
            info!("Oxide System initialized and started");
            Ok(())
        }
        Err(error) => {
            let context = json!({
                "config": config,
                "operation": "initialize_system"
            });
            let response = ErrorHandler::handle_error_with_monitoring(error, Some(context));
            Err(serde_json::to_string(&response)
                .unwrap_or_else(|_| "Serialization error".to_string()))
        }
    }
}

#[tauri::command]
async fn handle_user_input_command(
    user_input: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    // First, try to use the collaborative LLM system if available
    if let Ok(collaborative_result) = run_collaborative_analysis(
        state.clone(),
        user_input.clone(),
        Some("user_query".to_string()),
    )
    .await
    {
        // Parse the collaborative result and extract the primary response
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&collaborative_result) {
            if let Some(primary_response) = parsed.get("primary_response").and_then(|v| v.as_str())
            {
                info!("Using collaborative LLM response for user input");
                return Ok(primary_response.to_string());
            }
        }
    }

    // Fallback to the original system if collaborative analysis fails
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
                    system_ref
                        .handle_text_input(input_clone)
                        .await
                        .map_err(OxideError::Internal)
                })
            },
            retry_config,
        )
        .await;

        match result {
            Ok(response) => Ok(response),
            Err(error) => {
                let context = json!({
                    "user_input": user_input,
                    "operation": "handle_user_input"
                });
                let response = ErrorHandler::handle_error_with_monitoring(error, Some(context));
                Err(serde_json::to_string(&response)
                    .unwrap_or_else(|_| "Serialization error".to_string()))
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
    let system = state.oxide_system.read().await;
    let system = system.as_ref().ok_or("System not initialized")?;
    Ok(system.get_system_status())
}

#[tauri::command]
async fn scan_file_command(
    path: String,
    use_cloud: bool,
    quarantine: bool,
    state: State<'_, AppState>,
) -> Result<FileScanReport, String> {
    let system = state.oxide_system.read().await;
    let system = system.as_ref().ok_or("System not initialized")?;
    system.scan_file(path, use_cloud, quarantine).await
}

#[tauri::command]
async fn start_folder_scan(
    root: String,
    use_cloud: bool,
    quarantine: bool,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<String, String> {
    // Ensure system exists
    let system_guard = state.oxide_system.read().await;
    let Some(system) = system_guard.as_ref() else {
        return Err("System not initialized".to_string());
    };
    let system_clone = system.clone();
    drop(system_guard);

    // Resolve config for limits
    let cfg = system_clone.get_config().await;
    let max_workers = cfg.guardian.folder_scan_max_workers.unwrap_or(8).max(1);
    let max_depth = cfg.guardian.folder_scan_max_depth.unwrap_or(usize::MAX);
    let max_file_size_bytes: Option<u64> = cfg
        .guardian
        .max_file_size_mb
        .map(|mb| mb * 1024 * 1024);

    // Create cancel flag and scan id
    let scan_id = uuid::Uuid::new_v4().to_string();
    let cancel_flag = Arc::new(AtomicBool::new(false));
    {
        let mut cancels = state.folder_scan_cancels.write().await;
        cancels.insert(scan_id.clone(), cancel_flag.clone());
    }

    let root_path = PathBuf::from(root.clone());
    let app_clone = app.clone();
    let state_clone = AppState {
        oxide_system: state.oxide_system.clone(),
        auth_manager: state.auth_manager.clone(),
        mcp_server: state.mcp_server.clone(),
        folder_scan_cancels: state.folder_scan_cancels.clone(),
        rpa_state: state.rpa_state.clone(),
        #[cfg(feature = "surrealdb-metrics")]
        guardian_state: state.guardian_state.clone(),
    };

    // Clone scan_id for the async task
    let scan_id_for_task = scan_id.clone();
    let root_for_task = root.clone();

    // Spawn background task
    tokio::spawn(async move {
        let start = Instant::now();
        let _ = app_clone.emit_all(
            "folder_scan_started",
            serde_json::json!({
                "scan_id": scan_id_for_task,
                "root": root_for_task,
            }),
        );

        // Discover files breadth-first up to max_depth, respecting cancellation
        let mut files: Vec<PathBuf> = Vec::new();
        let mut q: VecDeque<(PathBuf, usize)> = VecDeque::new();
        q.push_back((root_path.clone(), 0));

        while let Some((dir, depth)) = q.pop_front() {
            if cancel_flag.load(Ordering::SeqCst) {
                break;
            }
            match std::fs::read_dir(&dir) {
                Ok(read_dir) => {
                    for entry in read_dir.flatten() {
                        if cancel_flag.load(Ordering::SeqCst) {
                            break;
                        }
                        let path = entry.path();
                        match entry.file_type() {
                            Ok(ft) if ft.is_dir() => {
                                if depth < max_depth {
                                    q.push_back((path, depth + 1));
                                }
                            }
                            Ok(ft) if ft.is_file() => {
                                // size filter
                                if let Some(limit) = max_file_size_bytes {
                                    if let Ok(meta) = entry.metadata() {
                                        if meta.len() > limit {
                                            continue;
                                        }
                                    }
                                }
                                files.push(path);
                            }
                            _ => {}
                        }
                    }
                }
                Err(e) => {
                    let _ = app_clone.emit_all(
                        "folder_scan_progress",
                        serde_json::json!({
                            "scan_id": scan_id_for_task,
                            "error": format!("read_dir error at {}: {}", dir.display(), e),
                        }),
                    );
                }
            }
        }

        let total = files.len();
        let _ = app_clone.emit_all(
            "folder_scan_progress",
            serde_json::json!({
                "scan_id": scan_id_for_task,
                "discovered": total,
            }),
        );

        if cancel_flag.load(Ordering::SeqCst) {
            let _ = app_clone.emit_all(
                "folder_scan_cancelled",
                serde_json::json!({
                    "scan_id": scan_id_for_task,
                    "scanned": 0,
                    "total": total,
                    "malicious": 0,
                    "errors": 0,
                    "duration_ms": start.elapsed().as_millis(),
                }),
            );
            let mut cancels = state_clone.folder_scan_cancels.write().await;
            cancels.remove(&scan_id_for_task);
            return;
        }

        // Scan concurrently with a worker pool using mpsc
        let (tx, rx) = mpsc::channel::<String>(std::cmp::max(1, total));
        for path in files {
            if cancel_flag.load(Ordering::SeqCst) {
                break;
            }
            let _ = tx.send(path.to_string_lossy().to_string()).await;
        }
        drop(tx);

        let rx = Arc::new(Mutex::new(rx));
        let scanned_c = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let malicious_c = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let errors_c = Arc::new(std::sync::atomic::AtomicUsize::new(0));

        let mut handles = Vec::new();
        for _ in 0..max_workers {
            let rx = rx.clone();
            let cancel_chk = cancel_flag.clone();
            let app_emit = app_clone.clone();
            let sys = system_clone.clone();
            let scanned_c = scanned_c.clone();
            let malicious_c = malicious_c.clone();
            let errors_c = errors_c.clone();
            let scan_id_cl = scan_id_for_task.clone();
            handles.push(tokio::spawn(async move {
                loop {
                    if cancel_chk.load(Ordering::SeqCst) {
                        break;
                    }
                    let next = {
                        let mut guard = rx.lock().await;
                        guard.recv().await
                    };
                    let Some(path_str) = next else {
                        break;
                    };
                    if cancel_chk.load(Ordering::SeqCst) {
                        break;
                    }

                    let res = sys.scan_file(path_str.clone(), use_cloud, quarantine).await;
                    match res {
                        Ok(report) => {
                            let s = scanned_c.fetch_add(1, Ordering::SeqCst) + 1;
                            if report.malicious {
                                malicious_c.fetch_add(1, Ordering::SeqCst);
                            }
                            let m = malicious_c.load(Ordering::SeqCst);
                            let e = errors_c.load(Ordering::SeqCst);
                            let _ = app_emit.emit_all(
                                "folder_scan_progress",
                                serde_json::json!({
                                    "scan_id": scan_id_cl,
                                    "scanned": s,
                                    "total": total,
                                    "malicious": m,
                                    "errors": e,
                                    "current_file": path_str,
                                    "local_match": report.local_match,
                                    "external_verdict": report.external_verdict,
                                }),
                            );
                        }
                        Err(err) => {
                            let s = scanned_c.fetch_add(1, Ordering::SeqCst) + 1;
                            let e = errors_c.fetch_add(1, Ordering::SeqCst) + 1;
                            let m = malicious_c.load(Ordering::SeqCst);
                            let _ = app_emit.emit_all(
                                "folder_scan_progress",
                                serde_json::json!({
                                    "scan_id": scan_id_cl,
                                    "scanned": s,
                                    "total": total,
                                    "malicious": m,
                                    "errors": e,
                                    "current_file": path_str,
                                    "error": err,
                                }),
                            );
                        }
                    }
                }
            }));
        }

        for h in handles {
            let _ = h.await;
        }

        let scanned = scanned_c.load(Ordering::SeqCst);
        let malicious = malicious_c.load(Ordering::SeqCst);
        let errors = errors_c.load(Ordering::SeqCst);

        // Emit final event
        if cancel_flag.load(Ordering::SeqCst) {
            let _ = app_clone.emit_all(
                "folder_scan_cancelled",
                serde_json::json!({
                    "scan_id": scan_id_for_task,
                    "scanned": scanned,
                    "total": total,
                    "malicious": malicious,
                    "errors": errors,
                    "duration_ms": start.elapsed().as_millis(),
                }),
            );
        } else {
            let _ = app_clone.emit_all(
                "folder_scan_completed",
                serde_json::json!({
                    "scan_id": scan_id_for_task,
                    "scanned": scanned,
                    "total": total,
                    "malicious": malicious,
                    "errors": errors,
                    "duration_ms": start.elapsed().as_millis(),
                }),
            );
        }

        // Cleanup cancel flag
        let mut cancels = state_clone.folder_scan_cancels.write().await;
        cancels.remove(&scan_id_for_task);
    });

    Ok(scan_id)
}

#[tauri::command]
async fn cancel_folder_scan(scan_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let cancels = state.folder_scan_cancels.write().await;
    if let Some(flag) = cancels.get(&scan_id) {
        flag.store(true, Ordering::SeqCst);
        Ok(())
    } else {
        Err("Unknown scan_id".to_string())
    }
}

#[tauri::command]
async fn is_virustotal_configured(state: State<'_, AppState>) -> Result<bool, String> {
    let system_guard = state.oxide_system.read().await;
    if let Some(system) = system_guard.as_ref() {
        // Clone ref to avoid holding lock across await
        let system_clone = system.clone();
        drop(system_guard);
        Ok(system_clone.has_virustotal_key().await)
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
    GLOBAL_ERROR_MONITOR
        .get_error_stats()
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_recent_errors(
    limit: Option<usize>,
) -> Result<Vec<error_handler::ErrorResponse>, String> {
    let limit = limit.unwrap_or(10);
    GLOBAL_ERROR_MONITOR
        .get_recent_errors(limit)
        .map_err(|e| e.to_string())
}

// TODO: Implement PerformanceAlert type and get_performance_alerts method
// #[tauri::command]
// async fn get_performance_alerts(state: State<'_, AppState>) -> Result<Vec<oxide_core::performance::PerformanceAlert>, String> {
//     let system_guard = state.oxide_system.read().await;
//     if let Some(system) = system_guard.as_ref() {
//         Ok(system.get_performance_alerts().await)
//     } else {
//         Err("System not initialized".to_string())
//     }
// }

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

// TODO: Implement get_operation_profiles method
// #[tauri::command]
// async fn get_operation_profiles(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
//     let system_guard = state.oxide_system.read().await;
//     if let Some(system) = system_guard.as_ref() {
//         let profiles = system.get_operation_profiles().await;
//         serde_json::to_value(profiles).map_err(|e| e.to_string())
//     } else {
//         Err("System not initialized".to_string())
//     }
// }

#[tauri::command]
async fn set_performance_monitoring(
    state: State<'_, AppState>,
    enabled: bool,
) -> Result<(), String> {
    let system_guard = state.oxide_system.read().await;
    if let Some(system) = system_guard.as_ref() {
        system.set_performance_monitoring(enabled).await;
        Ok(())
    } else {
        Err("System not initialized".to_string())
    }
}

#[tauri::command]
async fn validate_input(
    state: State<'_, AppState>,
    field_name: String,
    value: String,
) -> Result<String, String> {
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
        system
            .create_security_session(user_id, permissions, ip_address, user_agent)
            .await
    } else {
        Err("System not initialized".to_string())
    }
}

#[tauri::command]
async fn validate_security_session(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<bool, String> {
    let system_guard = state.oxide_system.read().await;
    if let Some(system) = system_guard.as_ref() {
        system.validate_security_session(&session_id).await
    } else {
        Err("System not initialized".to_string())
    }
}

#[tauri::command]
async fn check_security_permission(
    state: State<'_, AppState>,
    session_id: String,
    permission: String,
) -> Result<bool, String> {
    let system_guard = state.oxide_system.read().await;
    if let Some(system) = system_guard.as_ref() {
        system
            .check_security_permission(&session_id, &permission)
            .await
    } else {
        Err("System not initialized".to_string())
    }
}

#[tauri::command]
async fn get_security_events(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<Vec<oxide_core::security_manager::SecurityEvent>, String> {
    let system_guard = state.oxide_system.read().await;
    if let Some(system) = system_guard.as_ref() {
        Ok(system.get_security_events(limit).await)
    } else {
        Err("System not initialized".to_string())
    }
}

#[tauri::command]
async fn get_security_policy(
    state: State<'_, AppState>,
) -> Result<oxide_core::security_manager::SecurityPolicy, String> {
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
        auth_manager
            .get_auth_token()
            .await
            .map_err(|e| e.to_string())
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
        error!("Failed to get available models: {e}");
        e.to_string()
    })
}

#[tauri::command]
async fn clear_google_auth() -> Result<(), String> {
    google_auth::clear_auth().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn send_message_to_gemini(message: String, model: Option<String>) -> Result<String, String> {
    use oxide_core::gemini_auth::GeminiAuth;
    let auth = GeminiAuth::new();

    // Try to initialize from environment first
    let _ = auth.init_from_env().await;

    auth.send_message(&message, model.as_deref())
        .await
        .map_err(|e| {
            error!("Failed to send message to Gemini: {e}");
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

#[tauri::command]
async fn qwen_start_device_auth() -> Result<DeviceAuthStart, String> {
    let auth = QwenAuth::new();
    auth.start_device_auth().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn qwen_poll_device_auth(device_code: String) -> Result<PollResult, String> {
    let auth = QwenAuth::new();
    auth.poll_device_once(&device_code)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn qwen_get_auth_status() -> Result<String, String> {
    let auth = QwenAuth::new();
    auth.get_auth_status().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn qwen_clear_auth() -> Result<(), String> {
    let auth = QwenAuth::new();
    auth.clear_auth().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn openai_start_oauth(client_id: String, client_secret: String) -> Result<String, String> {
    // Store credentials first
    openai_auth::store_client_credentials(&client_id, &client_secret)
        .await
        .map_err(|e| e.to_string())?;

    // Start OAuth flow
    match openai_auth::authenticate_openai().await {
        Ok(token) => Ok(token),
        Err(e) => {
            error!("OpenAI OAuth failed: {e}");
            Err(e.to_string())
        }
    }
}

#[tauri::command]
async fn openai_set_api_key(api_key: String) -> Result<(), String> {
    match openai_key::store_api_key(&api_key).await {
        Ok(()) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
async fn openai_get_auth_status() -> Result<String, String> {
    // Prefer API key if present (env or keyring)
    match openai_key::get_api_key().await {
        Ok(Some(key)) if !key.trim().is_empty() => return Ok("API Key".to_string()),
        Ok(_) => { /* fall through */ }
        Err(e) => warn!("Failed to read OpenAI API key: {e}"),
    }

    // Fallback to OAuth status
    openai_auth::get_auth_status().await.map_err(|e| {
        error!("OpenAI auth status check failed: {e}");
        e.to_string()
    })
}

#[tauri::command]
async fn openai_clear_auth() -> Result<(), String> {
    let mut errors: Vec<String> = Vec::new();
    if let Err(e) = openai_key::clear_api_key().await {
        errors.push(format!("key: {e}"));
    }
    if let Err(e) = openai_auth::clear_auth().await {
        errors.push(format!("oauth: {e}"));
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "Failed to clear some OpenAI auth items: {}",
            errors.join(", ")
        ))
    }
}

// Collect a comprehensive snapshot of the current system state for analysis
#[tauri::command]
async fn get_system_snapshot(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let system_guard = state.oxide_system.read().await;
    if let Some(system) = system_guard.as_ref() {
        let system_clone = system.clone();
        drop(system_guard);

        // Gather pieces in parallel where possible
        let status = system_clone.get_system_status();
        let threats = system_clone.get_threat_history();
        let memory_stats = system_clone.get_memory_stats().await;
        let perf_metrics = system_clone.get_performance_metrics().await;

        let perf_metrics_val = serde_json::to_value(perf_metrics).map_err(|e| e.to_string())?;

        let snapshot = json!({
            "status": status,
            "threats": threats,
            "memory": memory_stats,
            "performance": perf_metrics_val,
            "collected_at_unix": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        });
        Ok(snapshot)
    } else {
        Err("System not initialized".to_string())
    }
}

// Orchestrate system analysis: collect snapshot and summarize with Gemini
#[tauri::command]
async fn run_system_analysis(
    state: State<'_, AppState>,
    model: Option<String>,
) -> Result<String, String> {
    let snapshot = get_system_snapshot(state).await?;

    // Build an analyst-style prompt for Gemini
    let prompt = format!(
        "You are an expert OS performance and security analyst. Given this JSON snapshot, produce a concise analysis with:\n\
        - Key performance issues and likely root causes\n\
        - Suspicious processes or threats (if any)\n\
        - Immediate remediation steps (bulleted)\n\
        - Risk score (0-100) and confidence.\n\nSnapshot:\n{snapshot}"
    );

    use oxide_core::gemini_auth::GeminiAuth;
    let auth = GeminiAuth::new();
    auth.send_message(&prompt, model.as_deref())
        .await
        .map_err(|e| {
            error!("System analysis via Gemini failed: {e}");
            e.to_string()
        })
}

// Run autonomous threat consensus without external VT. Uses both LLMs if available; if only one is available, uses that one.
// Gemini search will be always enabled when Gemini is used (no env toggles).
#[tauri::command]
async fn run_threat_consensus(state: State<'_, AppState>) -> Result<String, String> {
    let snapshot = get_system_snapshot(state).await?;
    let report = threat_consensus::run_consensus(snapshot, true)
        .await
        .map_err(|e| {
            error!("Threat consensus failed: {e}");
            e
        })?;
    serde_json::to_string(&report).map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_threat_recommendations(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let snapshot = get_system_snapshot(state).await?;
    let report = threat_consensus::run_consensus(snapshot, true)
        .await
        .map_err(|e| {
            error!("Threat consensus (recommendations) failed: {e}");
            e
        })?;
    Ok(threat_consensus::recommendations_from_report(&report))
}

#[tauri::command]
async fn mcp_start(
    state: State<'_, AppState>,
    port_override: Option<u16>,
    password_override: Option<String>,
) -> Result<String, String> {
    // If already running, stop and restart with new params
    {
        let mut mcp = state.mcp_server.write().await;
        if let Some(mut handle) = mcp.take() {
            handle.stop().await;
        }
    }

    // Resolve port/password from override or config
    let (port, password): (u16, Option<String>) = {
        // Try to read from current system config if available
        let system_guard = state.oxide_system.read().await;
        if let Some(system) = system_guard.as_ref() {
            let cfg = system.get_config().await;
            let from_cfg = cfg.mcp;
            let resolved_port = port_override
                .or_else(|| from_cfg.as_ref().map(|m| m.port))
                .unwrap_or(7999);
            let resolved_pwd = if let Some(p) = password_override {
                Some(p)
            } else if let Some(enc) = from_cfg.and_then(|m| m.password) {
                let bytes = system
                    .decrypt_data(&enc)
                    .map_err(|e| format!("Failed to decrypt MCP password: {e}"))?;
                let s = String::from_utf8(bytes)
                    .map_err(|_| "Decrypted MCP password is not valid UTF-8".to_string())?;
                Some(s)
            } else {
                None
            };
            (resolved_port, resolved_pwd)
        } else {
            (port_override.unwrap_or(7999), password_override)
        }
    };

    let handle = McpServerHandle::start(port, password)
        .await
        .map_err(|e| e.to_string())?;
    let addr = handle.addr();

    let mut mcp = state.mcp_server.write().await;
    *mcp = Some(handle);

    Ok(format!("mcp_started: http://{addr}"))
}

#[tauri::command]
async fn mcp_stop(state: State<'_, AppState>) -> Result<String, String> {
    let mut mcp = state.mcp_server.write().await;
    if let Some(mut handle) = mcp.take() {
        handle.stop().await;
        Ok("mcp_stopped".to_string())
    } else {
        Ok("mcp_not_running".to_string())
    }
}

#[tauri::command]
async fn mcp_status(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let mcp = state.mcp_server.read().await;
    if let Some(handle) = mcp.as_ref() {
        Ok(serde_json::json!({
            "running": true,
            "addr": handle.addr().to_string(),
            "password_enabled": handle.password_enabled(),
        }))
    } else {
        Ok(serde_json::json!({"running": false}))
    }
}

fn main() {
    // Load environment variables from .env file
    dotenv::dotenv().ok();

    // Initialize logging
    env_logger::init();

    info!("Starting Oxide Pilot Application");

    // Initialize Guardian backend if feature is enabled
    #[cfg(feature = "surrealdb-metrics")]
    let guardian_state = {
        use oxide_memory::SurrealBackend;
        let db_path =
            std::env::var("OXIDE_DB_PATH").unwrap_or_else(|_| "./data/oxide.db".to_string());

        let backend = tokio::runtime::Runtime::new()
            .expect("Failed to create runtime")
            .block_on(async {
                SurrealBackend::new(&db_path)
                    .await
                    .expect("Failed to initialize SurrealDB backend")
            });

        Arc::new(guardian_commands::GuardianState {
            backend: Arc::new(backend),
        })
    };

    tauri::Builder::default()
        .manage(AppState {
            oxide_system: Arc::new(RwLock::new(None)),
            auth_manager: Arc::new(RwLock::new(None)),
            mcp_server: Arc::new(RwLock::new(None)),
            folder_scan_cancels: Arc::new(RwLock::new(HashMap::new())),
            rpa_state: Arc::new(RwLock::new(None)),
            #[cfg(feature = "surrealdb-metrics")]
            guardian_state,
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
            run_collaborative_analysis,
            run_multi_agent_analysis,
            run_threat_consensus,
            get_threat_recommendations,
            get_system_status,
            scan_file_command,
            start_folder_scan,
            cancel_folder_scan,
            is_virustotal_configured,
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
            // get_performance_alerts, // TODO: Implement missing methods
            clear_performance_alerts,
            // get_operation_profiles, // TODO: Implement missing methods
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
            clear_google_auth,
            startup_check,
            get_system_snapshot,
            run_system_analysis,
            run_multi_agent_analysis,
            // Local LLM (LM Studio) controls
            local_llm_server_start,
            local_llm_server_stop,
            local_llm_server_status,
            local_llm_ls,
            local_llm_get,
            local_llm_load,
            local_llm_chat,
            qwen_start_device_auth,
            qwen_poll_device_auth,
            qwen_get_auth_status,
            qwen_clear_auth,
            openai_set_api_key,
            openai_start_oauth,
            openai_get_auth_status,
            openai_clear_auth,
            open_url,
            mcp_start,
            mcp_stop,
            mcp_status,
            // RPA Commands
            rpa_commands::rpa_initialize,
            rpa_commands::rpa_shutdown,
            rpa_commands::rpa_grant_permission,
            rpa_commands::rpa_check_permission,
            rpa_commands::rpa_move_mouse,
            rpa_commands::rpa_click_mouse,
            rpa_commands::rpa_scroll_mouse,
            rpa_commands::rpa_type_text,
            rpa_commands::rpa_press_key,
            rpa_commands::rpa_capture_screen,
            rpa_commands::rpa_get_audit_entries,
            rpa_commands::rpa_get_audit_stats,
            rpa_commands::rpa_get_failed_actions,
            rpa_commands::rpa_get_rollback_history,
            rpa_commands::rpa_rollback_last,
            rpa_commands::rpa_get_reversible_count,
            rpa_commands::rpa_get_pending_confirmations,
            rpa_commands::rpa_respond_confirmation,
            rpa_commands::rpa_add_auto_approve,
            // Guardian Commands
            guardian_commands::get_system_metrics,
            guardian_commands::get_recent_metrics,
            guardian_commands::get_high_cpu_processes,
            guardian_commands::search_agent_memory,
            guardian_commands::get_guardian_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn send_notification(title: String, body: String) {
    // For Tauri 2.x, notifications are handled differently
    // This is a placeholder implementation
    log::info!("Notification: {title} - {body}");
}

#[tauri::command]
async fn open_url(url: String, app_handle: tauri::AppHandle) -> Result<(), String> {
    use tauri::api::shell;
    match shell::open(&app_handle.shell_scope(), &url, None) {
        Ok(_) => {
            log::info!("Opened URL: {url}");
            Ok(())
        }
        Err(e) => {
            log::error!("Failed to open URL {url}: {e}");
            Err(format!("Failed to open URL: {e}"))
        }
    }
}
