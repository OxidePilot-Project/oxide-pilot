//! Tauri Commands for Guardian Agent
//!
//! This module exposes Guardian functionality to the Tauri frontend,
//! including metrics queries, process analysis, and threat detection.

use chrono::{DateTime, Duration, Utc};
use log::debug;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;

#[cfg(feature = "surrealdb-metrics")]
use oxide_memory::{BackendSearchItem, SurrealBackend, SystemMetric};

/// Shared state for Guardian commands
pub struct GuardianState {
    #[cfg(feature = "surrealdb-metrics")]
    pub backend: Arc<SurrealBackend>,
}

/// Time range for metrics queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    /// Start timestamp (ISO 8601)
    pub start: String,
    /// End timestamp (ISO 8601)
    pub end: String,
}

/// Response for metrics query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsResponse {
    pub metrics: Vec<SystemMetric>,
    pub count: usize,
}

/// Response for high CPU processes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighCpuProcessesResponse {
    pub processes: Vec<serde_json::Value>,
    pub count: usize,
}

/// Response for memory search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySearchResponse {
    pub results: Vec<BackendSearchItem>,
    pub count: usize,
}

/// Get system metrics for a time range
#[cfg(feature = "surrealdb-metrics")]
#[tauri::command]
pub async fn get_system_metrics(
    state: State<'_, GuardianState>,
    time_range: TimeRange,
) -> Result<MetricsResponse, String> {
    debug!("Fetching system metrics: {:?}", time_range);

    let start = DateTime::parse_from_rfc3339(&time_range.start)
        .map_err(|e| format!("Invalid start timestamp: {}", e))?
        .with_timezone(&Utc);

    let end = DateTime::parse_from_rfc3339(&time_range.end)
        .map_err(|e| format!("Invalid end timestamp: {}", e))?
        .with_timezone(&Utc);

    let metrics = state
        .backend
        .query_metrics_by_time(start, end)
        .await
        .map_err(|e| format!("Failed to query metrics: {}", e))?;

    let count = metrics.len();
    Ok(MetricsResponse { metrics, count })
}

/// Get system metrics for the last N hours
#[cfg(feature = "surrealdb-metrics")]
#[tauri::command]
pub async fn get_recent_metrics(
    state: State<'_, GuardianState>,
    hours: i64,
) -> Result<MetricsResponse, String> {
    debug!("Fetching metrics for last {} hours", hours);

    let end = Utc::now();
    let start = end - Duration::hours(hours);

    let metrics = state
        .backend
        .query_metrics_by_time(start, end)
        .await
        .map_err(|e| format!("Failed to query metrics: {}", e))?;

    let count = metrics.len();
    Ok(MetricsResponse { metrics, count })
}

/// Get processes with high CPU usage
#[cfg(feature = "surrealdb-metrics")]
#[tauri::command]
pub async fn get_high_cpu_processes(
    state: State<'_, GuardianState>,
    threshold: f64,
    hours: i64,
) -> Result<HighCpuProcessesResponse, String> {
    debug!(
        "Fetching high CPU processes: threshold={:.2}%, hours={}",
        threshold, hours
    );

    let processes = state
        .backend
        .query_high_cpu_processes(threshold, hours)
        .await
        .map_err(|e| format!("Failed to query high CPU processes: {}", e))?;

    let count = processes.len();
    Ok(HighCpuProcessesResponse { processes, count })
}

/// Search agent memory with semantic similarity
#[cfg(feature = "surrealdb-metrics")]
#[tauri::command]
pub async fn search_agent_memory(
    state: State<'_, GuardianState>,
    query: String,
    limit: usize,
) -> Result<MemorySearchResponse, String> {
    debug!("Searching agent memory: query='{}', limit={}", query, limit);

    // Use the MemoryBackend trait method
    use oxide_memory::MemoryBackend;
    let results = state
        .backend
        .search(query, limit)
        .await
        .map_err(|e| format!("Failed to search memory: {}", e))?;

    let count = results.len();
    Ok(MemorySearchResponse { results, count })
}

/// Get current system status summary
#[cfg(feature = "surrealdb-metrics")]
#[tauri::command]
pub async fn get_guardian_status(
    state: State<'_, GuardianState>,
) -> Result<serde_json::Value, String> {
    debug!("Fetching system status");

    let end = Utc::now();
    let start = end - Duration::minutes(5);

    let metrics = state
        .backend
        .query_metrics_by_time(start, end)
        .await
        .map_err(|e| format!("Failed to query metrics: {}", e))?;

    if let Some(latest) = metrics.first() {
        Ok(serde_json::json!({
            "timestamp": latest.timestamp,
            "cpu_usage": latest.cpu_usage,
            "memory_usage": latest.memory_usage,
            "disk_io": latest.disk_io,
            "network_stats": latest.network_stats,
            "status": if latest.cpu_usage > 90.0 || latest.memory_usage.percent > 90.0 {
                "warning"
            } else if latest.cpu_usage > 70.0 || latest.memory_usage.percent > 70.0 {
                "caution"
            } else {
                "healthy"
            },
        }))
    } else {
        Ok(serde_json::json!({
            "status": "no_data",
            "message": "No recent metrics available"
        }))
    }
}

// Stub implementations when surrealdb feature is disabled
#[cfg(not(feature = "surrealdb-metrics"))]
#[tauri::command]
pub async fn get_system_metrics(_time_range: TimeRange) -> Result<MetricsResponse, String> {
    Err("SurrealDB metrics feature not enabled".to_string())
}

#[cfg(not(feature = "surrealdb-metrics"))]
#[tauri::command]
pub async fn get_recent_metrics(_hours: i64) -> Result<MetricsResponse, String> {
    Err("SurrealDB metrics feature not enabled".to_string())
}

#[cfg(not(feature = "surrealdb-metrics"))]
#[tauri::command]
pub async fn get_high_cpu_processes(
    _threshold: f64,
    _hours: i64,
) -> Result<HighCpuProcessesResponse, String> {
    Err("SurrealDB metrics feature not enabled".to_string())
}

#[cfg(not(feature = "surrealdb-metrics"))]
#[tauri::command]
pub async fn search_agent_memory(
    _query: String,
    _limit: usize,
) -> Result<MemorySearchResponse, String> {
    Err("SurrealDB metrics feature not enabled".to_string())
}

#[cfg(not(feature = "surrealdb-metrics"))]
#[tauri::command]
pub async fn get_guardian_status() -> Result<serde_json::Value, String> {
    Err("SurrealDB metrics feature not enabled".to_string())
}
