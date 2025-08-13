use crate::encryption::EncryptionManager;
use log::{info, warn, error};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, Duration};
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SecurityError {
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    #[error("Authorization denied: {0}")]
    AuthorizationDenied(String),
    #[error("Session expired")]
    SessionExpired,
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Encryption error: {0}")]
    EncryptionError(String),
    #[error("Security policy violation: {0}")]
    PolicyViolation(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySession {
    pub session_id: String,
    pub user_id: String,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub permissions: Vec<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityEvent {
    pub event_id: String,
    pub event_type: SecurityEventType,
    pub severity: SecuritySeverity,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub description: String,
    pub metadata: HashMap<String, String>,
    pub ip_address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityEventType {
    LoginAttempt,
    LoginSuccess,
    LoginFailure,
    Logout,
    SessionExpired,
    PermissionDenied,
    RateLimitExceeded,
    SuspiciousActivity,
    DataAccess,
    ConfigurationChange,
    EncryptionFailure,
    PolicyViolation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecuritySeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub max_requests: u32,
    pub window_duration: Duration,
    pub block_duration: Duration,
}

#[derive(Debug, Clone)]
struct RateLimitEntry {
    requests: Vec<SystemTime>,
    blocked_until: Option<SystemTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPolicy {
    pub session_timeout: Duration,
    pub max_failed_attempts: u32,
    pub lockout_duration: Duration,
    pub require_strong_passwords: bool,
    pub enable_two_factor: bool,
    pub allowed_ip_ranges: Vec<String>,
    pub blocked_ip_addresses: Vec<String>,
    pub enable_audit_logging: bool,
    pub data_retention_days: u32,
}

impl Default for SecurityPolicy {
    fn default() -> Self {
        Self {
            session_timeout: Duration::from_secs(3600), // 1 hour
            max_failed_attempts: 5,
            lockout_duration: Duration::from_secs(900), // 15 minutes
            require_strong_passwords: true,
            enable_two_factor: false,
            allowed_ip_ranges: vec!["127.0.0.1/32".to_string()],
            blocked_ip_addresses: Vec::new(),
            enable_audit_logging: true,
            data_retention_days: 90,
        }
    }
}

pub struct SecurityManager {
    sessions: RwLock<HashMap<String, SecuritySession>>,
    security_events: RwLock<Vec<SecurityEvent>>,
    rate_limits: RwLock<HashMap<String, RateLimitEntry>>,
    #[allow(dead_code)]
    failed_attempts: RwLock<HashMap<String, (u32, SystemTime)>>,
    encryption_manager: EncryptionManager,
    policy: RwLock<SecurityPolicy>,
    rate_limit_config: RateLimitConfig,
}

impl SecurityManager {
    pub fn new(encryption_key: &[u8]) -> Result<Self, SecurityError> {
        let encryption_manager = EncryptionManager::new(encryption_key)
            .map_err(|e| SecurityError::EncryptionError(e.to_string()))?;

        Ok(Self {
            sessions: RwLock::new(HashMap::new()),
            security_events: RwLock::new(Vec::new()),
            rate_limits: RwLock::new(HashMap::new()),
            failed_attempts: RwLock::new(HashMap::new()),
            encryption_manager,
            policy: RwLock::new(SecurityPolicy::default()),
            rate_limit_config: RateLimitConfig {
                max_requests: 100,
                window_duration: Duration::from_secs(60),
                block_duration: Duration::from_secs(300),
            },
        })
    }

    pub async fn create_session(
        &self,
        user_id: String,
        permissions: Vec<String>,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<SecuritySession, SecurityError> {
        let policy = self.policy.read().await;
        let session_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let expires_at = now + chrono::Duration::from_std(policy.session_timeout)
            .map_err(|_| SecurityError::AuthenticationFailed("Invalid session timeout".to_string()))?;

        let session = SecuritySession {
            session_id: session_id.clone(),
            user_id: user_id.clone(),
            created_at: now,
            last_activity: now,
            expires_at,
            permissions,
            ip_address: ip_address.clone(),
            user_agent,
            is_active: true,
        };

        let mut sessions = self.sessions.write().await;
        sessions.insert(session_id.clone(), session.clone());

        // Log session creation
        self.log_security_event(
            SecurityEventType::LoginSuccess,
            SecuritySeverity::Low,
            Some(user_id),
            Some(session_id),
            "Session created successfully".to_string(),
            HashMap::new(),
            ip_address,
        ).await;

        info!("Created new session for user: {}", session.user_id);
        Ok(session)
    }

    pub async fn validate_session(&self, session_id: &str) -> Result<SecuritySession, SecurityError> {
        let mut sessions = self.sessions.write().await;

        if let Some(session) = sessions.get_mut(session_id) {
            let now = Utc::now();

            if !session.is_active {
                return Err(SecurityError::SessionExpired);
            }

            if now > session.expires_at {
                session.is_active = false;
                self.log_security_event(
                    SecurityEventType::SessionExpired,
                    SecuritySeverity::Medium,
                    Some(session.user_id.clone()),
                    Some(session_id.to_string()),
                    "Session expired".to_string(),
                    HashMap::new(),
                    session.ip_address.clone(),
                ).await;
                return Err(SecurityError::SessionExpired);
            }

            // Update last activity
            session.last_activity = now;

            Ok(session.clone())
        } else {
            Err(SecurityError::AuthenticationFailed("Invalid session".to_string()))
        }
    }

    pub async fn check_permission(&self, session_id: &str, permission: &str) -> Result<bool, SecurityError> {
        let session = self.validate_session(session_id).await?;

        let has_permission = self.encryption_manager.has_permission(&session.user_id, permission);

        if !has_permission {
            self.log_security_event(
                SecurityEventType::PermissionDenied,
                SecuritySeverity::Medium,
                Some(session.user_id),
                Some(session_id.to_string()),
                format!("Permission denied for: {permission}"),
                HashMap::from([("permission".to_string(), permission.to_string())]),
                session.ip_address,
            ).await;
        }

        Ok(has_permission)
    }

    pub async fn check_rate_limit(&self, identifier: &str) -> Result<(), SecurityError> {
        let mut rate_limits = self.rate_limits.write().await;
        let now = SystemTime::now();

        let entry = rate_limits.entry(identifier.to_string()).or_insert_with(|| RateLimitEntry {
            requests: Vec::new(),
            blocked_until: None,
        });

        // Check if currently blocked
        if let Some(blocked_until) = entry.blocked_until {
            if now < blocked_until {
                return Err(SecurityError::RateLimitExceeded);
            } else {
                entry.blocked_until = None;
                entry.requests.clear();
            }
        }

        // Clean old requests outside the window
        let window_start = now - self.rate_limit_config.window_duration;
        entry.requests.retain(|&request_time| request_time > window_start);

        // Check if rate limit exceeded
        if entry.requests.len() >= self.rate_limit_config.max_requests as usize {
            entry.blocked_until = Some(now + self.rate_limit_config.block_duration);

            self.log_security_event(
                SecurityEventType::RateLimitExceeded,
                SecuritySeverity::High,
                None,
                None,
                format!("Rate limit exceeded for: {identifier}"),
                HashMap::from([("identifier".to_string(), identifier.to_string())]),
                None,
            ).await;

            return Err(SecurityError::RateLimitExceeded);
        }

        // Add current request
        entry.requests.push(now);
        Ok(())
    }

    pub async fn log_security_event(
        &self,
        event_type: SecurityEventType,
        severity: SecuritySeverity,
        user_id: Option<String>,
        session_id: Option<String>,
        description: String,
        metadata: HashMap<String, String>,
        ip_address: Option<String>,
    ) {
        let event = SecurityEvent {
            event_id: Uuid::new_v4().to_string(),
            event_type: event_type.clone(),
            severity: severity.clone(),
            user_id,
            session_id,
            timestamp: Utc::now(),
            description: description.clone(),
            metadata,
            ip_address,
        };

        let mut events = self.security_events.write().await;
        events.push(event);

        // Keep only recent events (based on policy)
        let policy = self.policy.read().await;
        let retention_duration = chrono::Duration::days(policy.data_retention_days as i64);
        let cutoff = Utc::now() - retention_duration;
        events.retain(|event| event.timestamp > cutoff);

        // Log based on severity
        match severity {
            SecuritySeverity::Critical => error!("SECURITY [CRITICAL] {event_type:?}: {description}"),
            SecuritySeverity::High => warn!("SECURITY [HIGH] {event_type:?}: {description}"),
            SecuritySeverity::Medium => warn!("SECURITY [MEDIUM] {event_type:?}: {description}"),
            SecuritySeverity::Low => info!("SECURITY [LOW] {event_type:?}: {description}"),
        }
    }

    pub async fn get_security_events(&self, limit: Option<usize>) -> Vec<SecurityEvent> {
        let events = self.security_events.read().await;
        let limit = limit.unwrap_or(100);

        events.iter()
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    pub async fn invalidate_session(&self, session_id: &str) -> Result<(), SecurityError> {
        let mut sessions = self.sessions.write().await;

        if let Some(session) = sessions.get_mut(session_id) {
            session.is_active = false;

            self.log_security_event(
                SecurityEventType::Logout,
                SecuritySeverity::Low,
                Some(session.user_id.clone()),
                Some(session_id.to_string()),
                "Session invalidated".to_string(),
                HashMap::new(),
                session.ip_address.clone(),
            ).await;

            info!("Invalidated session: {session_id}");
        }

        Ok(())
    }

    pub async fn cleanup_expired_sessions(&self) {
        let mut sessions = self.sessions.write().await;
        let now = Utc::now();

        let expired_sessions: Vec<String> = sessions
            .iter()
            .filter(|(_, session)| now > session.expires_at || !session.is_active)
            .map(|(id, _)| id.clone())
            .collect();

        for session_id in expired_sessions {
            sessions.remove(&session_id);
        }
    }

    pub async fn update_security_policy(&self, policy: SecurityPolicy) {
        let mut current_policy = self.policy.write().await;
        *current_policy = policy;

        self.log_security_event(
            SecurityEventType::ConfigurationChange,
            SecuritySeverity::Medium,
            None,
            None,
            "Security policy updated".to_string(),
            HashMap::new(),
            None,
        ).await;

        info!("Security policy updated");
    }

    pub async fn get_security_policy(&self) -> SecurityPolicy {
        self.policy.read().await.clone()
    }
}
