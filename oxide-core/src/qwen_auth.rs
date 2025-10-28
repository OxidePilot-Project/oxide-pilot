use keyring::Entry;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::env;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum QwenAuthError {
    #[error("Keyring error: {0}")]
    Keyring(#[from] keyring::Error),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Environment error: {0}")]
    Env(String),
    #[error("Auth error: {0}")]
    Auth(String),
}

const QWEN_AUTH_SERVICE: &str = "oxide_pilot_qwen";
const AUTH_CONFIG_ENTRY: &str = "auth_config";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QwenAuthConfig {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceAuthStart {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    pub interval: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollResult {
    pub status: String,          // "pending" | "slow_down" | "success" | "error"
    pub message: Option<String>, // error or info
}

pub struct QwenAuth {
    keyring_service: String,
}

impl Default for QwenAuth {
    fn default() -> Self {
        Self {
            keyring_service: QWEN_AUTH_SERVICE.to_string(),
        }
    }
}

impl QwenAuth {
    pub fn new() -> Self {
        Self::default()
    }

    fn get_env(name: &str) -> Result<String, QwenAuthError> {
        env::var(name).map_err(|_| QwenAuthError::Env(format!("Missing env var: {name}")))
    }

    async fn store_auth_config(&self, cfg: &QwenAuthConfig) -> Result<(), QwenAuthError> {
        let entry = Entry::new(&self.keyring_service, AUTH_CONFIG_ENTRY)?;
        let s = serde_json::to_string(cfg)?;
        entry.set_password(&s)?;
        Ok(())
    }

    pub async fn get_auth_status(&self) -> Result<String, QwenAuthError> {
        let entry = Entry::new(&self.keyring_service, AUTH_CONFIG_ENTRY)?;
        match entry.get_password() {
            Ok(json) => {
                let cfg: QwenAuthConfig = serde_json::from_str(&json)?;
                if let Some(exp) = cfg.expires_at {
                    if chrono::Utc::now() < exp {
                        return Ok("OAuth Token".into());
                    } else {
                        return Ok("OAuth Token Expired".into());
                    }
                }
                Ok("OAuth Token".into())
            }
            Err(keyring::Error::NoEntry) => Ok("Not authenticated".into()),
            Err(e) => Err(e.into()),
        }
    }

    pub async fn clear_auth(&self) -> Result<(), QwenAuthError> {
        if let Ok(entry) = Entry::new(&self.keyring_service, AUTH_CONFIG_ENTRY) {
            let _ = entry.delete_password();
        }
        Ok(())
    }

    // Start OAuth2 Device Authorization flow
    pub async fn start_device_auth(&self) -> Result<DeviceAuthStart, QwenAuthError> {
        // Check for required environment variables with helpful error messages
        let device_url = Self::get_env("QWEN_DEVICE_AUTH_URL")
            .map_err(|_| QwenAuthError::Auth(
                "QWEN_DEVICE_AUTH_URL environment variable is required. Please configure it in src-tauri/.env file. See docs/ENVIRONMENT_SETUP.md for setup instructions.".to_string()
            ))?;

        let client_id = Self::get_env("QWEN_CLIENT_ID")
            .map_err(|_| QwenAuthError::Auth(
                "QWEN_CLIENT_ID environment variable is required. Please configure it in src-tauri/.env file. See docs/ENVIRONMENT_SETUP.md for setup instructions.".to_string()
            ))?;

        let scope = env::var("QWEN_SCOPE").unwrap_or_else(|_| "openid,profile,email".to_string());

        #[derive(Serialize)]
        struct Req<'a> {
            client_id: &'a str,
            scope: &'a str,
        }
        #[derive(Deserialize)]
        struct Resp {
            device_code: String,
            user_code: String,
            verification_uri: String,
            #[serde(default)]
            verification_uri_complete: Option<String>,
            expires_in: u64,
            #[serde(default)]
            interval: Option<u64>,
        }

        let client = reqwest::Client::new();
        let res = client
            .post(&device_url)
            .form(&Req {
                client_id: &client_id,
                scope: &scope,
            })
            .send()
            .await?;

        if !res.status().is_success() {
            let t = res.text().await.unwrap_or_default();
            error!("Device auth start failed: {t}");
            return Err(QwenAuthError::Auth(format!("device start failed: {t}")));
        }

        let resp: Resp = res.json().await?;
        let verification_uri = resp
            .verification_uri_complete
            .unwrap_or(resp.verification_uri);

        Ok(DeviceAuthStart {
            device_code: resp.device_code,
            user_code: resp.user_code,
            verification_uri,
            expires_in: resp.expires_in,
            interval: resp.interval,
        })
    }

    // Poll the token endpoint once; do not loop to keep UI responsive.
    pub async fn poll_device_once(&self, device_code: &str) -> Result<PollResult, QwenAuthError> {
        let token_url = Self::get_env("QWEN_DEVICE_TOKEN_URL")
            .map_err(|_| QwenAuthError::Auth(
                "QWEN_DEVICE_TOKEN_URL environment variable is required. Please configure it in src-tauri/.env file. See docs/ENVIRONMENT_SETUP.md for setup instructions.".to_string()
            ))?;

        let client_id = Self::get_env("QWEN_CLIENT_ID")
            .map_err(|_| QwenAuthError::Auth(
                "QWEN_CLIENT_ID environment variable is required. Please configure it in src-tauri/.env file. See docs/ENVIRONMENT_SETUP.md for setup instructions.".to_string()
            ))?;

        let client_secret = env::var("QWEN_CLIENT_SECRET").ok();

        #[derive(Serialize)]
        struct TokenReq<'a> {
            grant_type: &'a str,
            device_code: &'a str,
            client_id: &'a str,
            #[serde(skip_serializing_if = "Option::is_none")]
            client_secret: Option<&'a str>,
        }

        #[derive(Deserialize)]
        struct TokenOk {
            access_token: String,
            #[serde(default)]
            refresh_token: Option<String>,
            #[allow(dead_code)]
            token_type: String,
            #[serde(default)]
            expires_in: Option<u64>,
        }

        #[derive(Deserialize)]
        struct TokenErr {
            error: String,
            #[serde(default)]
            error_description: Option<String>,
        }

        let client = reqwest::Client::new();
        let res = client
            .post(&token_url)
            .form(&TokenReq {
                grant_type: "urn:ietf:params:oauth:grant-type:device_code",
                device_code,
                client_id: &client_id,
                client_secret: client_secret.as_deref(),
            })
            .send()
            .await?;

        if res.status().is_success() {
            let ok: TokenOk = res.json().await?;
            let expires_at = ok
                .expires_in
                .map(|sec| chrono::Utc::now() + chrono::Duration::seconds(sec as i64));
            let cfg = QwenAuthConfig {
                access_token: ok.access_token.clone(),
                refresh_token: ok.refresh_token.clone(),
                expires_at,
            };
            self.store_auth_config(&cfg).await?;
            info!("Qwen OAuth token stored");
            return Ok(PollResult {
                status: "success".into(),
                message: None,
            });
        }

        // Try to parse OAuth device errors
        let text = res.text().await.unwrap_or_default();
        if let Ok(e) = serde_json::from_str::<TokenErr>(&text) {
            match e.error.as_str() {
                "authorization_pending" => {
                    return Ok(PollResult {
                        status: "pending".into(),
                        message: e.error_description,
                    })
                }
                "slow_down" => {
                    return Ok(PollResult {
                        status: "slow_down".into(),
                        message: e.error_description,
                    })
                }
                "expired_token" | "expired_token_code" => {
                    return Ok(PollResult {
                        status: "error".into(),
                        message: Some("expired".into()),
                    })
                }
                _ => {
                    return Ok(PollResult {
                        status: "error".into(),
                        message: Some(e.error_description.unwrap_or(e.error)),
                    })
                }
            }
        }
        warn!("Unexpected token error: {text}");
        Ok(PollResult {
            status: "error".into(),
            message: Some(text),
        })
    }
}

impl QwenAuth {
    /// Retrieve the stored access token, ensuring it is not expired
    pub async fn get_access_token(&self) -> Result<String, QwenAuthError> {
        let entry = Entry::new(&self.keyring_service, AUTH_CONFIG_ENTRY)?;
        match entry.get_password() {
            Ok(json) => {
                let cfg: QwenAuthConfig = serde_json::from_str(&json)?;
                if let Some(exp) = cfg.expires_at {
                    if chrono::Utc::now() >= exp {
                        return Err(QwenAuthError::Auth("Token expired".into()));
                    }
                }
                Ok(cfg.access_token)
            }
            Err(keyring::Error::NoEntry) => Err(QwenAuthError::Auth("Not authenticated".into())),
            Err(e) => Err(e.into()),
        }
    }

    /// Build the Authorization header value using the stored access token
    pub async fn get_auth_header(&self) -> Result<String, QwenAuthError> {
        let token = self.get_access_token().await?;
        Ok(format!("Bearer {token}"))
    }
}
