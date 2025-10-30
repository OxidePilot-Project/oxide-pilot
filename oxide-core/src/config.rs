use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OxidePilotConfig {
    pub guardian: GuardianConfig,
    pub copilot: CopilotConfig,
    pub ai_providers: AIProvidersConfig,
    // Optional memory backend configuration; if present and enabled, Cognee may be used
    pub cognee: Option<CogneeConfig>,
    // Optional SurrealDB embedded database configuration
    pub surreal: Option<SurrealDbConfig>,
    // Optional embedded MCP server configuration
    pub mcp: Option<McpConfig>,
}

impl OxidePilotConfig {
    pub fn validate(&self) -> Result<(), String> {
        self.guardian.validate()?;
        self.copilot.validate()?;
        self.ai_providers.validate(self.copilot.enabled)?;
        if let Some(cognee) = &self.cognee {
            cognee.validate()?;
        }
        if let Some(surreal) = &self.surreal {
            surreal.validate()?;
        }
        if let Some(mcp) = &self.mcp {
            mcp.validate()?;
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GuardianConfig {
    pub enabled: bool,
    pub monitor_interval_secs: u64,
    // Antivirus feature toggles and settings
    pub antivirus_enabled: Option<bool>,
    pub signatures_path: Option<String>,
    pub quarantine_dir: Option<String>,
    pub max_file_size_mb: Option<u64>,
    // External malware scan providers
    pub virustotal_api_key: Option<crate::encryption::EncryptedData>,
    pub hybrid_analysis_api_key: Option<crate::encryption::EncryptedData>,
    // Game booster toggle
    pub game_booster_enabled: Option<bool>,
    // VT cache tuning
    pub vt_cache_ttl_secs: Option<u64>,
    pub vt_cache_max_entries: Option<usize>,
    // Folder scan tuning
    pub folder_scan_max_workers: Option<usize>,
    pub folder_scan_max_depth: Option<usize>,
    // Optional YARA feature toggles/paths (feature-gated in guardian)
    pub yara_enabled: Option<bool>,
    pub yara_rules_paths: Option<Vec<String>>,
}

impl GuardianConfig {
    fn validate(&self) -> Result<(), String> {
        if self.enabled && self.monitor_interval_secs == 0 {
            return Err("monitor_interval_secs must be greater than 0".to_string());
        }
        if let Some(mb) = self.max_file_size_mb {
            if mb == 0 {
                return Err("max_file_size_mb must be greater than 0".to_string());
            }
        }
        if let Some(ttl) = self.vt_cache_ttl_secs {
            if ttl == 0 {
                return Err("vt_cache_ttl_secs must be greater than 0".to_string());
            }
        }
        if let Some(max) = self.vt_cache_max_entries {
            if max == 0 {
                return Err("vt_cache_max_entries must be greater than 0".to_string());
            }
        }
        if let Some(w) = self.folder_scan_max_workers {
            if w == 0 {
                return Err("folder_scan_max_workers must be greater than 0".to_string());
            }
        }
        if let Some(d) = self.folder_scan_max_depth {
            if d == 0 {
                return Err("folder_scan_max_depth must be greater than 0".to_string());
            }
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CopilotConfig {
    pub enabled: bool,
    pub wake_word: String,
}

impl CopilotConfig {
    fn validate(&self) -> Result<(), String> {
        if self.enabled && self.wake_word.is_empty() {
            return Err("wake_word must not be empty".to_string());
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CogneeConfig {
    // Whether Cognee backend should be attempted at runtime
    pub enabled: bool,
    // Base URL of the Cognee sidecar HTTP service
    pub url: String,
    // Optional encrypted bearer token for secure local storage
    // Use oxide_core::encryption::EncryptionManager to encrypt/decrypt
    pub token: Option<crate::encryption::EncryptedData>,
}

impl CogneeConfig {
    fn validate(&self) -> Result<(), String> {
        if self.enabled && self.url.is_empty() {
            return Err("Cognee URL must not be empty when enabled".to_string());
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SurrealDbConfig {
    #[serde(default)]
    pub enabled: bool,
    pub db_path: Option<String>,
    #[serde(default)]
    pub collect_metrics: bool,
    #[serde(default)]
    pub metrics_interval_secs: Option<u64>,
    #[serde(default)]
    pub distributed: bool,
    #[serde(default)]
    pub tikv_endpoints: Option<Vec<String>>,
    #[serde(default)]
    pub enable_js_functions: bool,
    #[serde(default)]
    pub enable_computed_views: bool,
}

impl SurrealDbConfig {
    fn validate(&self) -> Result<(), String> {
        if self.enabled {
            if let Some(path) = &self.db_path {
                if path.trim().is_empty() {
                    return Err("SurrealDB path must not be empty when enabled".to_string());
                }
            }
            if let Some(interval) = self.metrics_interval_secs {
                if interval == 0 {
                    return Err(
                        "SurrealDB metrics interval must be greater than 0 seconds".to_string()
                    );
                }
            }
            if self.distributed {
                let endpoint_count = self
                    .tikv_endpoints
                    .as_ref()
                    .map(|v| v.iter().filter(|item| !item.trim().is_empty()).count())
                    .unwrap_or(0);
                if endpoint_count == 0 {
                    return Err(
                        "SurrealDB distributed mode requires at least one TiKV endpoint"
                            .to_string(),
                    );
                }
            }
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct McpConfig {
    // Whether the embedded MCP server should run
    pub enabled: bool,
    // Localhost port for the MCP HTTP server (streamable)
    pub port: u16,
    // Optional encrypted password for simple bearer auth
    pub password: Option<crate::encryption::EncryptedData>,
}

impl McpConfig {
    fn validate(&self) -> Result<(), String> {
        if self.enabled {
            if self.port == 0 {
                return Err("MCP port must be greater than 0".to_string());
            }
            // Disallow privileged or invalid ranges conservatively
            if self.port < 1024 {
                return Err("MCP port must be between 1024 and 65535".to_string());
            }
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AIProvidersConfig {
    pub google: Option<GoogleConfig>,
    pub openai: Option<OpenAIConfig>,
    pub anthropic: Option<AnthropicConfig>,
    pub azure_openai: Option<AzureOpenAIConfig>,
    pub ollama: Option<OllamaConfig>,
}

impl AIProvidersConfig {
    fn validate(&self, copilot_enabled: bool) -> Result<(), String> {
        if copilot_enabled {
            let mut at_least_one_provider = false;
            if let Some(google) = &self.google {
                google.validate()?;
                at_least_one_provider = true;
            }
            if let Some(openai) = &self.openai {
                openai.validate()?;
                at_least_one_provider = true;
            }
            if let Some(anthropic) = &self.anthropic {
                anthropic.validate()?;
                at_least_one_provider = true;
            }
            if let Some(azure_openai) = &self.azure_openai {
                azure_openai.validate()?;
                at_least_one_provider = true;
            }
            if let Some(ollama) = &self.ollama {
                ollama.validate()?;
                at_least_one_provider = true;
            }
            if !at_least_one_provider {
                return Err(
                    "At least one AI provider must be configured when copilot is enabled"
                        .to_string(),
                );
            }
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GoogleConfig {
    pub api_key: String,
}

impl GoogleConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.api_key.is_empty() {
            return Err("Google API key cannot be empty".to_string());
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OpenAIConfig {
    pub api_key: String,
}

impl OpenAIConfig {
    fn validate(&self) -> Result<(), String> {
        if self.api_key.is_empty() {
            return Err("OpenAI API key must not be empty".to_string());
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnthropicConfig {
    pub api_key: String,
}

impl AnthropicConfig {
    fn validate(&self) -> Result<(), String> {
        if self.api_key.is_empty() {
            return Err("Anthropic API key must not be empty".to_string());
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AzureOpenAIConfig {
    pub api_key: String,
    pub endpoint: String,
}

impl AzureOpenAIConfig {
    fn validate(&self) -> Result<(), String> {
        if self.api_key.is_empty() {
            return Err("Azure OpenAI API key must not be empty".to_string());
        }
        if self.endpoint.is_empty() {
            return Err("Azure OpenAI endpoint must not be empty".to_string());
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OllamaConfig {
    pub url: String,
}

impl OllamaConfig {
    fn validate(&self) -> Result<(), String> {
        if self.url.is_empty() {
            return Err("Ollama URL must not be empty".to_string());
        }
        Ok(())
    }
}
