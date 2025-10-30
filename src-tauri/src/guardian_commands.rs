//! Tauri Commands for Guardian Agent
//!
//! This module exposes Guardian functionality to the Tauri frontend,
//! including metrics queries, process analysis, and threat detection.

use serde::{Deserialize, Serialize};

#[cfg(feature = "surrealdb-metrics")]
use oxide_memory::{
    BackendSearchItem, SurrealBackend, SystemMetric, ThreatTrainingSample,
};
#[cfg(feature = "surrealdb-metrics")]
use std::sync::Arc;
#[cfg(feature = "surrealdb-metrics")]
use chrono::{DateTime, Duration, Utc};
#[cfg(feature = "surrealdb-metrics")]
use log::{debug, warn};
#[cfg(feature = "surrealdb-metrics")]
use serde_json::{from_value, Value};
#[cfg(feature = "surrealdb-metrics")]
use tauri::{async_runtime, State, Window};

/// Shared state for Guardian commands
#[allow(dead_code)]
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
#[cfg(feature = "surrealdb-metrics")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsResponse {
    pub metrics: Vec<SystemMetric>,
    pub count: usize,
}

/// Aggregated hourly metrics row
#[cfg(feature = "surrealdb-metrics")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HourlyMetricsRow {
    pub avg_cpu: f64,
    pub peak_cpu: f64,
    pub avg_mem_percent: f64,
    pub hour_bucket: DateTime<Utc>,
    pub samples: usize,
}

/// Process hotspot analytics
#[cfg(feature = "surrealdb-metrics")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessHotspot {
    pub name: String,
    pub avg_cpu: f64,
    pub peak_cpu: f64,
    pub avg_memory_mb: f64,
    pub samples: usize,
}

/// Response for high CPU processes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighCpuProcessesResponse {
    pub processes: Vec<serde_json::Value>,
    pub count: usize,
}

/// Response for memory search
#[cfg(feature = "surrealdb-metrics")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySearchResponse {
    pub results: Vec<BackendSearchItem>,
    pub count: usize,
}

/// Aggregated metrics summary for dashboard widgets
#[cfg(feature = "surrealdb-metrics")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSummaryResponse {
    pub avg_cpu: f64,
    pub max_cpu: f64,
    pub avg_memory_percent: f64,
    pub max_memory_percent: f64,
    pub sample_count: usize,
    pub window_start: Option<DateTime<Utc>>,
    pub window_end: Option<DateTime<Utc>>,
}

/// Get system metrics for a time range
#[cfg(feature = "surrealdb-metrics")]
#[tauri::command]
pub async fn get_system_metrics(
    state: State<'_, GuardianState>,
    time_range: TimeRange,
) -> Result<MetricsResponse, String> {
    debug!("Fetching system metrics: {time_range:?}");

    let start = DateTime::parse_from_rfc3339(&time_range.start)
        .map_err(|e| format!("Invalid start timestamp: {e}"))?
        .with_timezone(&Utc);

    let end = DateTime::parse_from_rfc3339(&time_range.end)
        .map_err(|e| format!("Invalid end timestamp: {e}"))?
        .with_timezone(&Utc);

    let metrics = state
        .backend
        .query_metrics_by_time(start, end)
        .await
        .map_err(|e| format!("Failed to query metrics: {e}"))?;

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
    debug!("Fetching metrics for last {hours} hours");

    let end = Utc::now();
    let start = end - Duration::hours(hours);

    let metrics = state
        .backend
        .query_metrics_by_time(start, end)
        .await
        .map_err(|e| format!("Failed to query metrics: {e}"))?;

    let count = metrics.len();
    Ok(MetricsResponse { metrics, count })
}

/// Get aggregated metrics summary for the last N hours (default 6)
#[cfg(feature = "surrealdb-metrics")]
#[tauri::command]
pub async fn get_metrics_summary(
    state: State<'_, GuardianState>,
    hours: Option<i64>,
) -> Result<MetricsSummaryResponse, String> {
    let window_hours = hours.unwrap_or(6).max(1);
    debug!("Building metrics summary for last {window_hours} hours");

    let end = Utc::now();
    let start = end - Duration::hours(window_hours);

    let metrics = state
        .backend
        .query_metrics_by_time(start, end)
        .await
        .map_err(|e| format!("Failed to query metrics: {e}"))?;

    if metrics.is_empty() {
        return Ok(MetricsSummaryResponse {
            avg_cpu: 0.0,
            max_cpu: 0.0,
            avg_memory_percent: 0.0,
            max_memory_percent: 0.0,
            sample_count: 0,
            window_start: None,
            window_end: Some(end),
        });
    }

    let sample_count = metrics.len();
    let cpu_sum: f64 = metrics.iter().map(|m| m.cpu_usage).sum();
    let mem_sum: f64 = metrics.iter().map(|m| m.memory_usage.percent).sum();
    let cpu_max = metrics
        .iter()
        .map(|m| m.cpu_usage)
        .fold(0.0, |acc, value| acc.max(value));
    let mem_max = metrics
        .iter()
        .map(|m| m.memory_usage.percent)
        .fold(0.0, |acc, value| acc.max(value));

    let window_start = metrics.last().map(|m| m.timestamp);
    let window_end = metrics.first().map(|m| m.timestamp).or(Some(end));

    Ok(MetricsSummaryResponse {
        avg_cpu: cpu_sum / sample_count as f64,
        max_cpu,
        avg_memory_percent: mem_sum / sample_count as f64,
        max_memory_percent: mem_max,
        sample_count,
        window_start,
        window_end,
    })
}

/// Fetch hourly aggregated metrics for dashboard charts.
#[cfg(feature = "surrealdb-metrics")]
#[tauri::command]
pub async fn get_hourly_metrics(
    state: State<'_, GuardianState>,
    hours: Option<i64>,
) -> Result<Vec<HourlyMetricsRow>, String> {
    let lookback = hours.unwrap_or(24).max(1);
    let raw = state
        .backend
        .query_hourly_metrics(lookback)
        .await
        .map_err(|e| format!("Failed to query hourly metrics: {e}"))?;

    let mut rows = Vec::new();
    for value in raw {
        match from_value::<HourlyMetricsRow>(value) {
            Ok(row) => rows.push(row),
            Err(err) => warn!("Failed to deserialize hourly metrics row: {err}"),
        }
    }
    Ok(rows)
}

/// Graph analytics helper: identify top process hotspots.
#[cfg(feature = "surrealdb-metrics")]
#[tauri::command]
pub async fn get_process_hotspots(
    state: State<'_, GuardianState>,
    hours: Option<i64>,
) -> Result<Vec<ProcessHotspot>, String> {
    let lookback = hours.unwrap_or(8).max(1);
    let raw = state
        .backend
        .query_process_hotspots(lookback)
        .await
        .map_err(|e| format!("Failed to query process hotspots: {e}"))?;

    let mut hotspots = Vec::new();
    for value in raw {
        match from_value::<ProcessHotspot>(value) {
            Ok(entry) => hotspots.push(entry),
            Err(err) => warn!("Failed to deserialize process hotspot row: {err}"),
        }
    }
    Ok(hotspots)
}

/// Get processes with high CPU usage
#[cfg(feature = "surrealdb-metrics")]
#[tauri::command]
pub async fn get_high_cpu_processes(
    state: State<'_, GuardianState>,
    threshold: f64,
    hours: i64,
) -> Result<HighCpuProcessesResponse, String> {
    debug!("Fetching high CPU processes: threshold={threshold:.2}%, hours={hours}");

    let processes = state
        .backend
        .query_high_cpu_processes(threshold, hours)
        .await
        .map_err(|e| format!("Failed to query high CPU processes: {e}"))?;

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
    debug!("Searching agent memory: query='{query}', limit={limit}");

    // Use the MemoryBackend trait method
    use oxide_memory::MemoryBackend;
    let results = state
        .backend
        .search(query, limit)
        .await
        .map_err(|e| format!("Failed to search memory: {e}"))?;

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
        .map_err(|e| format!("Failed to query metrics: {e}"))?;

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

/// Predict threat risk score using SurrealML (with backend heuristic fallback).
#[cfg(feature = "surrealdb-metrics")]
#[tauri::command]
pub async fn predict_threat_risk(
    state: State<'_, GuardianState>,
    feature_vector: serde_json::Value,
) -> Result<serde_json::Value, String> {
    state
        .backend
        .ml_predict_threat(feature_vector)
        .await
        .map_err(|e| format!("Failed to run threat risk prediction: {e}"))
}

/// Submit a labeled training sample to enhance threat predictions.
#[cfg(feature = "surrealdb-metrics")]
#[tauri::command]
pub async fn submit_threat_training_sample(
    state: State<'_, GuardianState>,
    sample: ThreatTrainingSample,
) -> Result<(), String> {
    state
        .backend
        .upsert_threat_training_sample(sample)
        .await
        .map_err(|e| format!("Failed to store training sample: {e}"))
}

/// Subscribe frontend listeners to realtime metric updates.
#[cfg(feature = "surrealdb-metrics")]
#[tauri::command]
pub async fn subscribe_guardian_metrics(
    state: State<'_, GuardianState>,
    window: Window,
) -> Result<(), String> {
    let mut receiver = state.backend.subscribe_metrics();
    async_runtime::spawn(async move {
        loop {
            match receiver.recv().await {
                Ok(metric) => {
                    if window.emit("guardian://metrics", &metric).is_err() {
                        break;
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                    warn!("Guardian metrics subscriber lagged by {skipped} events");
                }
            }
        }
    });
    Ok(())
}

// Stub implementations when surrealdb feature is disabled
#[cfg(not(feature = "surrealdb-metrics"))]
#[tauri::command]
pub async fn get_system_metrics(_time_range: TimeRange) -> Result<String, String> {
    Err("SurrealDB metrics feature not enabled".to_string())
}

#[cfg(not(feature = "surrealdb-metrics"))]
#[tauri::command]
pub async fn get_recent_metrics(_hours: i64) -> Result<String, String> {
    Err("SurrealDB metrics feature not enabled".to_string())
}

#[cfg(not(feature = "surrealdb-metrics"))]
#[tauri::command]
pub async fn get_metrics_summary(_hours: Option<i64>) -> Result<String, String> {
    Err("SurrealDB metrics feature not enabled".to_string())
}

#[cfg(not(feature = "surrealdb-metrics"))]
#[tauri::command]
pub async fn get_hourly_metrics(_hours: Option<i64>) -> Result<String, String> {
    Err("SurrealDB metrics feature not enabled".to_string())
}

#[cfg(not(feature = "surrealdb-metrics"))]
#[tauri::command]
pub async fn get_process_hotspots(_hours: Option<i64>) -> Result<String, String> {
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
) -> Result<String, String> {
    Err("SurrealDB metrics feature not enabled".to_string())
}

#[cfg(not(feature = "surrealdb-metrics"))]
#[tauri::command]
pub async fn get_guardian_status() -> Result<serde_json::Value, String> {
    Err("SurrealDB metrics feature not enabled".to_string())
}

#[cfg(not(feature = "surrealdb-metrics"))]
#[tauri::command]
pub async fn predict_threat_risk(
    _feature_vector: serde_json::Value,
) -> Result<serde_json::Value, String> {
    Err("SurrealDB metrics feature not enabled".to_string())
}

#[cfg(not(feature = "surrealdb-metrics"))]
#[tauri::command]
pub async fn submit_threat_training_sample(
    _sample: serde_json::Value,
) -> Result<(), String> {
    Err("SurrealDB metrics feature not enabled".to_string())
}

#[cfg(not(feature = "surrealdb-metrics"))]
#[tauri::command]
pub async fn subscribe_guardian_metrics(_window: tauri::Window) -> Result<(), String> {
    Err("SurrealDB metrics feature not enabled".to_string())
}
