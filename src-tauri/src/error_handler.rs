use log::{error, info, warn};
use oxide_copilot::errors::CopilotError;
use oxide_core::google_auth::AuthError;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fmt;
use thiserror::Error;

/// Centralized error type for the Oxide Pilot application
#[derive(Error, Debug)]
#[allow(dead_code)] // Some variants reserved for future use
pub enum OxideError {
    #[error("System initialization failed: {0}")]
    SystemInit(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Authentication error: {0}")]
    Auth(#[from] AuthError),

    #[error("Copilot error: {0}")]
    Copilot(#[from] CopilotError),

    #[error("Memory operation failed: {0}")]
    Memory(String),

    #[error("Voice processing error: {0}")]
    Voice(String),

    #[error("Audio system error: {0}")]
    Audio(String),

    #[error("Guardian monitoring error: {0}")]
    Guardian(String),

    #[error("Performance monitoring error: {0}")]
    Performance(String),

    #[error("File system error: {0}")]
    FileSystem(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Timeout error: operation timed out after {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },

    #[error("Resource unavailable: {resource}")]
    ResourceUnavailable { resource: String },

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<String> for OxideError {
    fn from(error: String) -> Self {
        OxideError::Internal(error)
    }
}

impl From<&str> for OxideError {
    fn from(error: &str) -> Self {
        OxideError::Internal(error.to_string())
    }
}

/// Error severity levels for monitoring and alerting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Structured error response for Tauri commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error_type: String,
    pub message: String,
    pub severity: ErrorSeverity,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub context: Option<serde_json::Value>,
    pub recovery_suggestions: Vec<String>,
}

impl From<OxideError> for ErrorResponse {
    fn from(error: OxideError) -> Self {
        let (severity, recovery_suggestions) = match &error {
            OxideError::SystemInit(_) => (
                ErrorSeverity::Critical,
                vec![
                    "Check system requirements".to_string(),
                    "Verify configuration files".to_string(),
                    "Restart the application".to_string(),
                ],
            ),
            OxideError::Auth(_) => (
                ErrorSeverity::High,
                vec![
                    "Check internet connection".to_string(),
                    "Verify API credentials".to_string(),
                    "Re-authenticate if needed".to_string(),
                ],
            ),
            OxideError::Copilot(_) => (
                ErrorSeverity::Medium,
                vec![
                    "Try the request again".to_string(),
                    "Check AI provider status".to_string(),
                    "Verify input format".to_string(),
                ],
            ),
            OxideError::Audio(_) | OxideError::Voice(_) => (
                ErrorSeverity::Medium,
                vec![
                    "Check audio device connections".to_string(),
                    "Verify microphone permissions".to_string(),
                    "Restart audio services".to_string(),
                ],
            ),
            OxideError::Network(_) => (
                ErrorSeverity::Medium,
                vec![
                    "Check internet connection".to_string(),
                    "Verify firewall settings".to_string(),
                    "Try again in a few moments".to_string(),
                ],
            ),
            OxideError::Timeout { .. } => (
                ErrorSeverity::Medium,
                vec![
                    "Try the operation again".to_string(),
                    "Check system performance".to_string(),
                    "Reduce operation complexity".to_string(),
                ],
            ),
            OxideError::InvalidInput(_) => (
                ErrorSeverity::Low,
                vec![
                    "Check input format".to_string(),
                    "Refer to documentation".to_string(),
                ],
            ),
            _ => (
                ErrorSeverity::Medium,
                vec!["Contact support if the issue persists".to_string()],
            ),
        };

        ErrorResponse {
            error_type: error.error_type(),
            message: error.to_string(),
            severity,
            timestamp: chrono::Utc::now(),
            context: None,
            recovery_suggestions,
        }
    }
}

impl OxideError {
    /// Get the error type as a string for categorization
    pub fn error_type(&self) -> String {
        match self {
            OxideError::SystemInit(_) => "SystemInit".to_string(),
            OxideError::Config(_) => "Config".to_string(),
            OxideError::Auth(_) => "Auth".to_string(),
            OxideError::Copilot(_) => "Copilot".to_string(),
            OxideError::Memory(_) => "Memory".to_string(),
            OxideError::Voice(_) => "Voice".to_string(),
            OxideError::Audio(_) => "Audio".to_string(),
            OxideError::Guardian(_) => "Guardian".to_string(),
            OxideError::Performance(_) => "Performance".to_string(),
            OxideError::FileSystem(_) => "FileSystem".to_string(),
            OxideError::Serialization(_) => "Serialization".to_string(),
            OxideError::Network(_) => "Network".to_string(),
            OxideError::Timeout { .. } => "Timeout".to_string(),
            OxideError::ResourceUnavailable { .. } => "ResourceUnavailable".to_string(),
            OxideError::InvalidInput(_) => "InvalidInput".to_string(),
            OxideError::Internal(_) => "Internal".to_string(),
        }
    }

    /// Check if the error is recoverable
    pub fn is_recoverable(&self) -> bool {
        match self {
            OxideError::SystemInit(_) => false,
            OxideError::Config(_) => false,
            OxideError::Auth(_) => true,
            OxideError::Copilot(_) => true,
            OxideError::Memory(_) => true,
            OxideError::Voice(_) => true,
            OxideError::Audio(_) => true,
            OxideError::Guardian(_) => true,
            OxideError::Performance(_) => true,
            OxideError::FileSystem(_) => true,
            OxideError::Serialization(_) => false,
            OxideError::Network(_) => true,
            OxideError::Timeout { .. } => true,
            OxideError::ResourceUnavailable { .. } => true,
            OxideError::InvalidInput(_) => false,
            OxideError::Internal(_) => false,
        }
    }
}

/// Error handler for Tauri commands
pub struct ErrorHandler;

impl ErrorHandler {
    /// Handle and log an error, returning a structured response
    pub fn handle_error(error: OxideError, context: Option<serde_json::Value>) -> ErrorResponse {
        let mut response = ErrorResponse::from(error);
        response.context = context;

        // Log the error based on severity
        match response.severity {
            ErrorSeverity::Critical => {
                error!(
                    "CRITICAL ERROR: {} - Context: {:?}",
                    response.message, response.context
                );
            }
            ErrorSeverity::High => {
                error!(
                    "HIGH SEVERITY ERROR: {} - Context: {:?}",
                    response.message, response.context
                );
            }
            ErrorSeverity::Medium => {
                warn!(
                    "MEDIUM SEVERITY ERROR: {} - Context: {:?}",
                    response.message, response.context
                );
            }
            ErrorSeverity::Low => {
                info!(
                    "LOW SEVERITY ERROR: {} - Context: {:?}",
                    response.message, response.context
                );
            }
        }

        response
    }

    /// Attempt to recover from an error
    pub async fn attempt_recovery(error: &OxideError) -> Result<(), OxideError> {
        if !error.is_recoverable() {
            return Err(OxideError::Internal("Error is not recoverable".to_string()));
        }

        match error {
            OxideError::Network(_) => {
                info!("Attempting network recovery...");
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                Ok(())
            }
            OxideError::Timeout { .. } => {
                info!("Attempting timeout recovery...");
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                Ok(())
            }
            OxideError::ResourceUnavailable { resource } => {
                info!("Attempting to recover resource: {resource}");
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                Ok(())
            }
            _ => Ok(()),
        }
    }
}

/// Macro for handling errors in Tauri commands
#[macro_export]
macro_rules! handle_tauri_error {
    ($result:expr) => {
        match $result {
            Ok(value) => Ok(value),
            Err(error) => {
                let oxide_error = OxideError::from(error);
                let response = ErrorHandler::handle_error(oxide_error, None);
                Err(serde_json::to_string(&response)
                    .unwrap_or_else(|_| "Serialization error".to_string()))
            }
        }
    };
    ($result:expr, $context:expr) => {
        match $result {
            Ok(value) => Ok(value),
            Err(error) => {
                let oxide_error = OxideError::from(error);
                let response = ErrorHandler::handle_error(oxide_error, Some($context));
                Err(serde_json::to_string(&response)
                    .unwrap_or_else(|_| "Serialization error".to_string()))
            }
        }
    };
}

/// Retry mechanism with exponential backoff
pub struct RetryConfig {
    pub max_attempts: u32,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay_ms: 100,
            max_delay_ms: 5000,
            backoff_multiplier: 2.0,
        }
    }
}

pub async fn retry_with_backoff<F, T, E>(operation: F, config: RetryConfig) -> Result<T, OxideError>
where
    F: Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>> + Send>>,
    E: Into<OxideError> + fmt::Debug,
{
    let mut delay = config.base_delay_ms;

    for attempt in 1..=config.max_attempts {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(error) => {
                let oxide_error = error.into();

                if attempt == config.max_attempts || !oxide_error.is_recoverable() {
                    return Err(oxide_error);
                }

                warn!(
                    "Attempt {attempt} failed: {oxide_error:?}. Retrying in {delay}ms..."
                );

                // Attempt recovery
                if let Err(recovery_error) = ErrorHandler::attempt_recovery(&oxide_error).await {
                    warn!("Recovery attempt failed: {recovery_error:?}");
                }

                tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
                delay = std::cmp::min(
                    (delay as f64 * config.backoff_multiplier) as u64,
                    config.max_delay_ms,
                );
            }
        }
    }

    Err(OxideError::Internal(
        "Retry loop completed without result".to_string(),
    ))
}

/// Error monitoring and metrics collection
pub struct ErrorMonitor {
    error_counts: std::sync::Arc<std::sync::Mutex<std::collections::HashMap<String, u64>>>,
    last_errors: std::sync::Arc<std::sync::Mutex<std::collections::VecDeque<ErrorResponse>>>,
}

impl ErrorMonitor {
    pub fn new() -> Self {
        Self {
            error_counts: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::HashMap::new(),
            )),
            last_errors: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::VecDeque::new(),
            )),
        }
    }

    pub fn record_error(&self, error_response: &ErrorResponse) {
        // Update error counts
        if let Ok(mut counts) = self.error_counts.lock() {
            *counts.entry(error_response.error_type.clone()).or_insert(0) += 1;
        }

        // Store recent errors (keep last 100)
        if let Ok(mut last_errors) = self.last_errors.lock() {
            last_errors.push_back(error_response.clone());
            if last_errors.len() > 100 {
                last_errors.pop_front();
            }
        }
    }

    pub fn get_error_stats(&self) -> Result<serde_json::Value, OxideError> {
        let counts = self
            .error_counts
            .lock()
            .map_err(|_| OxideError::Internal("Failed to lock error counts".to_string()))?;

        let last_errors = self
            .last_errors
            .lock()
            .map_err(|_| OxideError::Internal("Failed to lock last errors".to_string()))?;

        Ok(json!({
            "error_counts": *counts,
            "total_errors": counts.values().sum::<u64>(),
            "recent_errors_count": last_errors.len(),
            "last_error": last_errors.back()
        }))
    }

    pub fn get_recent_errors(&self, limit: usize) -> Result<Vec<ErrorResponse>, OxideError> {
        let last_errors = self
            .last_errors
            .lock()
            .map_err(|_| OxideError::Internal("Failed to lock last errors".to_string()))?;

        Ok(last_errors.iter().rev().take(limit).cloned().collect())
    }
}

impl Default for ErrorMonitor {
    fn default() -> Self {
        Self::new()
    }
}

// Global error monitor instance
lazy_static::lazy_static! {
    pub static ref GLOBAL_ERROR_MONITOR: ErrorMonitor = ErrorMonitor::new();
}

/// Enhanced error handler with monitoring
impl ErrorHandler {
    /// Handle and log an error with monitoring
    pub fn handle_error_with_monitoring(
        error: OxideError,
        context: Option<serde_json::Value>,
    ) -> ErrorResponse {
        let response = Self::handle_error(error, context);

        // Record the error for monitoring
        GLOBAL_ERROR_MONITOR.record_error(&response);

        response
    }
}
