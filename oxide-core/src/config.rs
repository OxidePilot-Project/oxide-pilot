use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OxidePilotConfig {
    pub guardian: GuardianConfig,
    pub copilot: CopilotConfig,
    pub ai_providers: AIProvidersConfig,
    // Optional memory backend configuration; if present and enabled, Cognee may be used
    pub cognee: Option<CogneeConfig>,
}

impl OxidePilotConfig {
    pub fn validate(&self) -> Result<(), String> {
        self.guardian.validate()?;
        self.copilot.validate()?;
        self.ai_providers.validate(self.copilot.enabled)?;
        if let Some(cognee) = &self.cognee {
            cognee.validate()?;
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GuardianConfig {
    pub enabled: bool,
    pub monitor_interval_secs: u64,
}

impl GuardianConfig {
    fn validate(&self) -> Result<(), String> {
        if self.enabled && self.monitor_interval_secs == 0 {
            return Err("monitor_interval_secs must be greater than 0".to_string());
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
                return Err("At least one AI provider must be configured when copilot is enabled".to_string());
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
