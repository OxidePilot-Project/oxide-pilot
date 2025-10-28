use crate::oauth::{GoogleOAuthConfig, GoogleOAuthManager, OAuthToken};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("OAuth error: {0}")]
    OAuth(String),
    #[error("Environment error: {0}")]
    Environment(String),
    #[error("Authentication error: {0}")]
    Auth(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthMethod {
    ApiKey(String),
    OAuth(OAuthToken),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub method: AuthMethod,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_used: chrono::DateTime<chrono::Utc>,
}

pub struct AuthManager {
    config_path: PathBuf,
    oauth_manager: GoogleOAuthManager,
}

impl AuthManager {
    pub fn new() -> Result<Self, AuthError> {
        let config_dir = Self::get_config_dir()?;
        fs::create_dir_all(&config_dir)?;

        let config_path = config_dir.join("auth.json");

        // Initialize OAuth manager with default config
        let oauth_config = GoogleOAuthConfig::default();
        let oauth_manager =
            GoogleOAuthManager::new(oauth_config).map_err(|e| AuthError::OAuth(e.to_string()))?;

        Ok(Self {
            config_path,
            oauth_manager,
        })
    }

    fn get_config_dir() -> Result<PathBuf, AuthError> {
        let home_dir = env::var("USERPROFILE")
            .or_else(|_| env::var("HOME"))
            .map_err(|_| {
                AuthError::Environment("Could not determine home directory".to_string())
            })?;

        Ok(PathBuf::from(home_dir).join(".oxidepilot"))
    }

    pub async fn get_auth_token(&mut self) -> Result<String, AuthError> {
        // First, check for environment variable API key
        if let Ok(api_key) = env::var("GEMINI_API_KEY") {
            if !api_key.is_empty() {
                println!("🔑 Using Gemini API key from environment variable");
                return Ok(api_key);
            }
        }

        // Check for saved auth config
        if let Ok(config) = self.load_auth_config() {
            match &config.method {
                AuthMethod::ApiKey(key) => {
                    println!("🔑 Using saved API key");
                    return Ok(key.clone());
                }
                AuthMethod::OAuth(token) => {
                    // Check if token is still valid (simple check)
                    if let Some(expires_in) = token.expires_in {
                        let expires_at =
                            config.created_at + chrono::Duration::seconds(expires_in as i64);
                        if chrono::Utc::now() < expires_at {
                            println!("🔐 Using saved OAuth token");
                            return Ok(token.access_token.clone());
                        } else if let Some(refresh_token) = &token.refresh_token {
                            println!("🔄 Refreshing OAuth token...");
                            match self.oauth_manager.refresh_token(refresh_token).await {
                                Ok(new_token) => {
                                    let new_config = AuthConfig {
                                        method: AuthMethod::OAuth(new_token.clone()),
                                        created_at: chrono::Utc::now(),
                                        last_used: chrono::Utc::now(),
                                    };
                                    self.save_auth_config(&new_config)?;
                                    return Ok(new_token.access_token);
                                }
                                Err(e) => {
                                    eprintln!("⚠️  Failed to refresh token: {e}");
                                    // Fall through to new authentication
                                }
                            }
                        }
                    }
                }
            }
        }

        // No valid auth found, prompt user for choice
        self.prompt_for_auth().await
    }

    async fn prompt_for_auth(&mut self) -> Result<String, AuthError> {
        println!("\n🚀 Welcome to Oxide Pilot!");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("To get started, you need to authenticate with Google Gemini.");
        println!("Choose your preferred authentication method:");
        println!();
        println!("1. 🔐 OAuth Login (Recommended)");
        println!("   - Secure browser-based authentication");
        println!("   - Automatic token refresh");
        println!("   - No need to manage API keys");
        println!();
        println!("2. 🔑 API Key");
        println!("   - Manual API key from Google AI Studio");
        println!("   - Simple setup");
        println!("   - You manage the key");
        println!();
        println!("3. ❌ Exit");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        loop {
            print!("Enter your choice (1-3): ");
            std::io::Write::flush(&mut std::io::stdout()).map_err(AuthError::Io)?;

            let mut input = String::new();
            std::io::stdin()
                .read_line(&mut input)
                .map_err(AuthError::Io)?;
            let choice = input.trim();

            match choice {
                "1" => {
                    println!("\n🔐 Starting OAuth authentication...");
                    return self.authenticate_with_oauth().await;
                }
                "2" => {
                    println!("\n🔑 API Key Setup");
                    return self.authenticate_with_api_key().await;
                }
                "3" => {
                    println!("👋 Goodbye!");
                    std::process::exit(0);
                }
                _ => {
                    println!("❌ Invalid choice. Please enter 1, 2, or 3.");
                    continue;
                }
            }
        }
    }

    async fn authenticate_with_oauth(&mut self) -> Result<String, AuthError> {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("📋 OAuth Authentication Instructions:");
        println!("1. A browser window will open automatically");
        println!("2. Sign in with your Google account");
        println!("3. Grant permissions to Oxide Pilot");
        println!("4. Return to this terminal");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!();

        let token = self
            .oauth_manager
            .authenticate()
            .await
            .map_err(|e| AuthError::OAuth(e.to_string()))?;

        let config = AuthConfig {
            method: AuthMethod::OAuth(token.clone()),
            created_at: chrono::Utc::now(),
            last_used: chrono::Utc::now(),
        };

        self.save_auth_config(&config)?;
        println!("💾 Authentication saved successfully!");

        Ok(token.access_token)
    }

    async fn authenticate_with_api_key(&mut self) -> Result<String, AuthError> {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("📋 API Key Setup Instructions:");
        println!("1. Visit: https://aistudio.google.com/apikey");
        println!("2. Sign in with your Google account");
        println!("3. Click 'Create API Key'");
        println!("4. Copy the generated API key");
        println!("5. Paste it below");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!();

        loop {
            print!("Enter your Gemini API key: ");
            std::io::Write::flush(&mut std::io::stdout()).map_err(AuthError::Io)?;

            let mut api_key = String::new();
            std::io::stdin()
                .read_line(&mut api_key)
                .map_err(AuthError::Io)?;
            let api_key = api_key.trim().to_string();

            if api_key.is_empty() {
                println!("❌ API key cannot be empty. Please try again.");
                continue;
            }

            if !api_key.starts_with("AIza") {
                println!("⚠️  Warning: API key doesn't look like a valid Gemini API key.");
                print!("Continue anyway? (y/N): ");
                std::io::Write::flush(&mut std::io::stdout()).map_err(AuthError::Io)?;

                let mut confirm = String::new();
                std::io::stdin()
                    .read_line(&mut confirm)
                    .map_err(AuthError::Io)?;
                if confirm.trim().to_lowercase() != "y" {
                    continue;
                }
            }

            let config = AuthConfig {
                method: AuthMethod::ApiKey(api_key.clone()),
                created_at: chrono::Utc::now(),
                last_used: chrono::Utc::now(),
            };

            self.save_auth_config(&config)?;
            println!("💾 API key saved successfully!");

            return Ok(api_key);
        }
    }

    fn load_auth_config(&self) -> Result<AuthConfig, AuthError> {
        let content = fs::read_to_string(&self.config_path)?;
        let config: AuthConfig = serde_json::from_str(&content)?;
        Ok(config)
    }

    fn save_auth_config(&self, config: &AuthConfig) -> Result<(), AuthError> {
        let content = serde_json::to_string_pretty(config)?;
        fs::write(&self.config_path, content)?;
        Ok(())
    }

    pub fn clear_auth(&self) -> Result<(), AuthError> {
        if self.config_path.exists() {
            fs::remove_file(&self.config_path)?;
            println!("🗑️  Authentication cleared successfully!");
        }
        Ok(())
    }

    pub fn get_auth_status(&self) -> Result<String, AuthError> {
        if let Ok(config) = self.load_auth_config() {
            match &config.method {
                AuthMethod::ApiKey(_) => Ok("API Key".to_string()),
                AuthMethod::OAuth(_) => Ok("OAuth Token".to_string()),
            }
        } else {
            Ok("Not authenticated".to_string())
        }
    }
}
