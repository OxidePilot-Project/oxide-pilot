use thiserror::Error;
use keyring::Entry;
use log::{info, error, warn};
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::env;

#[derive(Error, Debug)]
pub enum GeminiAuthError {
    #[error("Keyring error: {0}")]
    Keyring(#[from] keyring::Error),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid API key")]
    InvalidApiKey,
    #[error("Authentication failed: {0}")]
    AuthFailed(String),
    #[error("No authentication method configured")]
    NoAuthMethod,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthMethod {
    ApiKey(String),
    OAuth {
        access_token: String,
        refresh_token: Option<String>,
        expires_at: Option<chrono::DateTime<chrono::Utc>>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiAuthConfig {
    pub method: AuthMethod,
    pub project_id: Option<String>,
}

const GEMINI_AUTH_SERVICE: &str = "oxide_pilot_gemini";
const API_KEY_ENTRY: &str = "api_key";
const AUTH_CONFIG_ENTRY: &str = "auth_config";

pub struct GeminiAuth {
    keyring_service: String,
}

impl Default for GeminiAuth {
    fn default() -> Self {
        Self::new()
    }
}

impl GeminiAuth {
    pub fn new() -> Self {
        Self {
            keyring_service: GEMINI_AUTH_SERVICE.to_string(),
        }
    }

    /// Store API key for simple authentication
    pub async fn store_api_key(&self, api_key: &str) -> Result<(), GeminiAuthError> {
        // Validate API key format
        if !api_key.starts_with("AIza") || api_key.len() < 35 {
            return Err(GeminiAuthError::InvalidApiKey);
        }

        // Test the API key by making a simple request
        if let Err(e) = self.test_api_key(api_key).await {
            warn!("API key validation failed: {e}");
            return Err(GeminiAuthError::InvalidApiKey);
        }

        let entry = Entry::new(&self.keyring_service, API_KEY_ENTRY)?;
        entry.set_password(api_key)?;

        let config = GeminiAuthConfig {
            method: AuthMethod::ApiKey(api_key.to_string()),
            project_id: None,
        };

        self.store_auth_config(&config).await?;
        info!("API key stored successfully");
        Ok(())
    }

    /// Get API key from environment variable or keyring
    pub async fn get_api_key_from_env_or_store(&self) -> Result<Option<String>, GeminiAuthError> {
        // First try environment variables (support both names)
        if let Ok(api_key) = env::var("GEMINI_API_KEY").or_else(|_| env::var("GOOGLE_GEMINI_API_KEY")) {
            if !api_key.is_empty() && api_key.starts_with("AIza") {
                info!("Using API key from environment variable");
                return Ok(Some(api_key));
            }
        }

        // Fallback to keyring
        self.get_api_key().await
    }

    /// Initialize from environment variables if available
    pub async fn init_from_env(&self) -> Result<bool, GeminiAuthError> {
        if let Ok(api_key) = env::var("GEMINI_API_KEY").or_else(|_| env::var("GOOGLE_GEMINI_API_KEY")) {
            if !api_key.is_empty() && api_key.starts_with("AIza") {
                info!("Initializing from environment variable");
                self.store_api_key(&api_key).await?;
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Get stored API key
    pub async fn get_api_key(&self) -> Result<Option<String>, GeminiAuthError> {
        let entry = Entry::new(&self.keyring_service, API_KEY_ENTRY)?;
        match entry.get_password() {
            Ok(api_key) => Ok(Some(api_key)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Test if an API key is valid
    async fn test_api_key(&self, api_key: &str) -> Result<(), GeminiAuthError> {
        let client = reqwest::Client::new();
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models?key={api_key}"
        );

        let response = client.get(&url).send().await?;

        if response.status().is_success() {
            info!("API key validation successful");
            Ok(())
        } else {
            error!("API key validation failed: {}", response.status());
            Err(GeminiAuthError::InvalidApiKey)
        }
    }

    /// Store authentication configuration
    async fn store_auth_config(&self, config: &GeminiAuthConfig) -> Result<(), GeminiAuthError> {
        let entry = Entry::new(&self.keyring_service, AUTH_CONFIG_ENTRY)?;
        let config_json = serde_json::to_string(config)?;
        entry.set_password(&config_json)?;
        Ok(())
    }

    /// Get authentication configuration
    pub async fn get_auth_config(&self) -> Result<Option<GeminiAuthConfig>, GeminiAuthError> {
        let entry = Entry::new(&self.keyring_service, AUTH_CONFIG_ENTRY)?;
        match entry.get_password() {
            Ok(config_json) => {
                let config: GeminiAuthConfig = serde_json::from_str(&config_json)?;
                Ok(Some(config))
            },
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Get current authentication status
    pub async fn get_auth_status(&self) -> Result<String, GeminiAuthError> {
        if let Some(config) = self.get_auth_config().await? {
            match config.method {
                AuthMethod::ApiKey(_) => {
                    // Verify API key is still valid
                    if let Some(api_key) = self.get_api_key().await? {
                        match self.test_api_key(&api_key).await {
                            Ok(_) => Ok("API Key".to_string()),
                            Err(_) => Ok("API Key Invalid".to_string()),
                        }
                    } else {
                        Ok("Not authenticated".to_string())
                    }
                },
                AuthMethod::OAuth { expires_at, .. } => {
                    if let Some(expiry) = expires_at {
                        if chrono::Utc::now() < expiry {
                            Ok("OAuth Token".to_string())
                        } else {
                            Ok("OAuth Token Expired".to_string())
                        }
                    } else {
                        Ok("OAuth Token".to_string())
                    }
                }
            }
        } else {
            Ok("Not authenticated".to_string())
        }
    }

    /// Get authorization header for API requests
    pub async fn get_auth_header(&self) -> Result<String, GeminiAuthError> {
        if let Some(config) = self.get_auth_config().await? {
            match config.method {
                AuthMethod::ApiKey(api_key) => Ok(format!("Bearer {api_key}")),
                AuthMethod::OAuth { access_token, .. } => Ok(format!("Bearer {access_token}")),
            }
        } else {
            Err(GeminiAuthError::NoAuthMethod)
        }
    }

    /// Clear all authentication data
    pub async fn clear_auth(&self) -> Result<(), GeminiAuthError> {
        // Clear API key
        if let Ok(entry) = Entry::new(&self.keyring_service, API_KEY_ENTRY) {
            let _ = entry.delete_password();
        }

        // Clear auth config
        if let Ok(entry) = Entry::new(&self.keyring_service, AUTH_CONFIG_ENTRY) {
            let _ = entry.delete_password();
        }

        info!("Authentication data cleared");
        Ok(())
    }

    /// Open browser for OAuth authentication (simplified version)
    pub async fn start_oauth_flow(&self) -> Result<String, GeminiAuthError> {
        // For now, we'll provide instructions for manual OAuth setup
        // In a full implementation, this would start the OAuth flow
        let auth_url = "https://console.cloud.google.com/apis/credentials";

        info!("Please visit {auth_url} to set up OAuth credentials");

        // Try to open the URL in the default browser
        #[cfg(target_os = "windows")]
        {
            let _ = Command::new("cmd")
                .args(["/C", "start", auth_url])
                .spawn();
        }

        #[cfg(target_os = "macos")]
        {
            let _ = Command::new("open")
                .arg(auth_url)
                .spawn();
        }

        #[cfg(target_os = "linux")]
        {
            let _ = Command::new("xdg-open")
                .arg(auth_url)
                .spawn();
        }

        Ok(format!("Please visit: {auth_url}"))
    }

    /// Check if user has any authentication method configured
    pub async fn is_authenticated(&self) -> bool {
        self.get_auth_config().await.is_ok_and(|config| config.is_some())
    }

    /// Get available models (requires OAuth access token)
    pub async fn get_available_models(&self) -> Result<Vec<String>, GeminiAuthError> {
        // Prefer OAuth via google_auth
        let access_token = crate::google_auth::get_access_token().await
            .map_err(|e| GeminiAuthError::AuthFailed(format!("OAuth access token error: {e}")))?
            .ok_or(GeminiAuthError::NoAuthMethod)?;

        let client = reqwest::Client::new();
        let url = "https://generativelanguage.googleapis.com/v1beta/models";

        let response = client
            .get(url)
            .bearer_auth(&access_token)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(GeminiAuthError::AuthFailed(
                format!("Failed to fetch models: {}", response.status())
            ));
        }

        #[derive(Deserialize)]
        struct ModelsResponse {
            models: Vec<Model>,
        }

        #[derive(Deserialize)]
        struct Model {
            name: String,
            #[serde(rename = "displayName")]
            display_name: Option<String>,
        }

        let models_response: ModelsResponse = response.json().await?;
        let model_names = models_response.models
            .into_iter()
            .map(|m| {
                // Extract just the model name (e.g., "gemini-1.5-pro" from "models/gemini-1.5-pro")
                let name = m.name.split('/').next_back().unwrap_or(&m.name).to_string();
                m.display_name.unwrap_or(name)
            })
            .collect();

        Ok(model_names)
    }

    /// Send a message to Gemini API using OAuth (no API key)
    pub async fn send_message(&self, message: &str, model: Option<&str>) -> Result<String, GeminiAuthError> {
        // Prefer OAuth via google_auth
        let access_token = crate::google_auth::get_access_token().await
            .map_err(|e| GeminiAuthError::AuthFailed(format!("OAuth access token error: {e}")))?
            .ok_or(GeminiAuthError::NoAuthMethod)?;

        let model_name = model.unwrap_or("gemini-1.5-flash");
        let client = reqwest::Client::new();
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{model_name}:generateContent"
        );

        #[derive(Serialize)]
        struct GenerateRequest {
            contents: Vec<Content>,
        }

        #[derive(Serialize)]
        struct Content {
            parts: Vec<Part>,
        }

        #[derive(Serialize)]
        struct Part {
            text: String,
        }

        let request_body = GenerateRequest {
            contents: vec![Content {
                parts: vec![Part {
                    text: message.to_string(),
                }],
            }],
        };

        let response = client
            .post(&url)
            .bearer_auth(&access_token)
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(GeminiAuthError::AuthFailed(
                format!("API request failed: {error_text}")
            ));
        }

        #[derive(Deserialize)]
        struct GenerateResponse {
            candidates: Vec<Candidate>,
        }

        #[derive(Deserialize)]
        struct Candidate {
            content: ResponseContent,
        }

        #[derive(Deserialize)]
        struct ResponseContent {
            parts: Vec<ResponsePart>,
        }

        #[derive(Deserialize)]
        struct ResponsePart {
            text: String,
        }

        let generate_response: GenerateResponse = response.json().await?;

        if let Some(candidate) = generate_response.candidates.first() {
            if let Some(part) = candidate.content.parts.first() {
                return Ok(part.text.clone());
            }
        }

        Err(GeminiAuthError::AuthFailed("No response from API".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_api_key_validation() {
        let auth = GeminiAuth::new();

        // Test invalid API key format
        assert!(auth.test_api_key("invalid_key").await.is_err());

        // Test valid format but potentially invalid key
        // Note: This would fail in real testing without a valid key
        // assert!(auth.test_api_key("AIzaSyDummy_Key_For_Testing_12345678901234567890").await.is_err());
    }

    #[tokio::test]
    async fn test_auth_status() {
        let auth = GeminiAuth::new();
        let status = auth.get_auth_status().await.unwrap();
        assert_eq!(status, "Not authenticated");
    }
}