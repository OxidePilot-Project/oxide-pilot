use crate::errors::CopilotError;
use crate::functions::FunctionRegistry;
use crate::gemini_api::{
    Content, FunctionCall, FunctionDeclaration, FunctionResponse, GenerateContentRequest,
    GenerateContentResponse, InlineData, Part, Tool,
};
use async_trait::async_trait;
use log::{error, info};
use oxide_core::config::{
    AIProvidersConfig, AnthropicConfig, AzureOpenAIConfig, GoogleConfig, OllamaConfig, OpenAIConfig,
};
use oxide_core::google_auth::{authenticate_google, get_access_token};
use oxide_core::types::{AgentAction, Interaction};
use reqwest::Client;
use serde_json::json;
use std::sync::{Arc, Mutex};

#[async_trait]
pub trait AIProvider {
    fn name(&self) -> &str;
    async fn generate_response(
        &self,
        prompt: &str,
        history: &[Interaction],
        function_registry: Option<&FunctionRegistry>,
    ) -> Result<String, CopilotError>;
    async fn call_function(&self, action: &AgentAction) -> Result<serde_json::Value, CopilotError>;
}

pub struct GoogleAIProvider {
    config: GoogleConfig,
    http_client: Client,
}

impl GoogleAIProvider {
    pub fn new(config: GoogleConfig) -> Self {
        Self {
            config,
            http_client: Client::new(),
        }
    }

    async fn get_valid_access_token(&self) -> Result<String, CopilotError> {
        if let Some(token) = get_access_token()
            .await
            .map_err(|e| CopilotError::Authentication(e.to_string()))?
        {
            info!("Using existing Google access token.");
            Ok(token)
        } else {
            info!("No existing Google access token found. Initiating new authentication flow.");
            authenticate_google()
                .await
                .map_err(|e| CopilotError::Authentication(e.to_string()))
        }
    }
}

#[async_trait]
impl AIProvider for GoogleAIProvider {
    fn name(&self) -> &str {
        "Google AI"
    }

    async fn generate_response(
        &self,
        prompt: &str,
        history: &[Interaction],
        function_registry: Option<&FunctionRegistry>,
    ) -> Result<String, CopilotError> {
        info!("Google AI: Generating response for prompt: {}", prompt);
        let access_token = self.get_valid_access_token().await?;

        let mut contents: Vec<Content> = Vec::new();

        for interaction in history {
            // User input part
            contents.push(Content {
                role: "user".to_string(),
                parts: vec![Part {
                    text: Some(interaction.user_input.clone()),
                    function_call: None,
                    function_response: None,
                }],
            });

            // Agent response part (can be text, function call, or function response)
            if !interaction.agent_response.is_empty() {
                if interaction.agent_response.starts_with("FUNCTION_CALL:") {
                    let call_str = interaction
                        .agent_response
                        .trim_start_matches("FUNCTION_CALL:")
                        .trim();
                    match serde_json::from_str::<FunctionCall>(call_str) {
                        Ok(function_call) => {
                            contents.push(Content {
                                role: "model".to_string(),
                                parts: vec![Part {
                                    text: None,
                                    function_call: Some(function_call),
                                    function_response: None,
                                }],
                            });
                        }
                        Err(e) => {
                            error!("Failed to parse stored function call: {}", e);
                            return Err(CopilotError::Serialization(e));
                        }
                    }
                } else if interaction.agent_response.starts_with("FUNCTION_RESPONSE:") {
                    let response_str = interaction
                        .agent_response
                        .trim_start_matches("FUNCTION_RESPONSE:")
                        .trim();
                    match serde_json::from_str::<FunctionResponse>(response_str) {
                        Ok(function_response) => {
                            contents.push(Content {
                                role: "function".to_string(), // Role for function response is 'function'
                                parts: vec![Part {
                                    text: None,
                                    function_call: None,
                                    function_response: Some(function_response),
                                }],
                            });
                        }
                        Err(e) => {
                            error!("Failed to parse stored function response: {}", e);
                            return Err(CopilotError::Serialization(e));
                        }
                    }
                } else {
                    contents.push(Content {
                        role: "model".to_string(),
                        parts: vec![Part {
                            text: Some(interaction.agent_response.clone()),
                            function_call: None,
                            function_response: None,
                        }],
                    });
                }
            }
        }

        // Add the current prompt
        contents.push(Content {
            role: "user".to_string(),
            parts: vec![Part {
                text: Some(prompt.to_string()),
                function_call: None,
                function_response: None,
            }],
        });

        let mut request_body = GenerateContentRequest {
            contents,
            tools: None,
        };

        if let Some(registry) = function_registry {
            let function_declarations: Vec<FunctionDeclaration> = registry
                .get_all_function_schemas()
                .into_iter()
                .map(|schema| FunctionDeclaration {
                    name: schema["name"].as_str().unwrap_or("").to_string(),
                    description: schema["description"].as_str().map(|s| s.to_string()),
                    parameters: Some(schema["parameters"].clone()),
                })
                .collect();

            if !function_declarations.is_empty() {
                request_body.tools = Some(vec![Tool {
                    function_declarations,
                }]);
            }
        }

        // Use gemini-pro-vision if there are images in the request
        let has_images = request_body
            .contents
            .iter()
            .any(|content| content.parts.iter().any(|part| part.inline_data.is_some()));

        let model_endpoint = if has_images {
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-pro-vision:generateContent"
        } else {
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-pro:generateContent"
        };

        let response = self
            .http_client
            .post(model_endpoint)
            .bearer_auth(&access_token)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| CopilotError::APIRequest(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("Gemini API error: Status: {}, Body: {}", status, error_text);
            return Err(CopilotError::APIRequest(format!(
                "Gemini API returned non-success status: {} - {}",
                status, error_text
            )));
        }

        let api_response = response
            .json::<GenerateContentResponse>()
            .await
            .map_err(|e| CopilotError::APIResponseParse(e.to_string()))?;

        info!("Google AI API raw response: {:?}", api_response);

        if let Some(prompt_feedback) = api_response.prompt_feedback {
            if let Some(safety_ratings) = prompt_feedback.safety_ratings {
                for rating in safety_ratings {
                    warn!(
                        "Gemini API Safety Rating: Category: {}, Probability: {}",
                        rating.category, rating.probability
                    );
                }
            }
        }

        if let Some(candidate) = api_response.candidates.into_iter().next() {
            if let Some(part) = candidate.content.parts.into_iter().next() {
                if let Some(text) = part.text {
                    Ok(text)
                } else if let Some(function_call) = part.function_call {
                    Ok(format!(
                        "FUNCTION_CALL: {}",
                        serde_json::to_string(&function_call).unwrap_or_default()
                    ))
                } else {
                    Err(CopilotError::NoAIResponseContent)
                }
            } else {
                Err(CopilotError::NoAIResponseContent)
            }
        } else {
            Err(CopilotError::NoAICandidates)
        }
    }

    async fn call_function(&self, action: &AgentAction) -> Result<serde_json::Value, CopilotError> {
        info!("Google AI: Calling function: {}", action.action_type);
        let access_token = self.get_valid_access_token().await?;
        // This function is called by the AIOrchestrator *after* the AI has requested a function call.
        // The result of this function execution would then be sent back to the AI.
        // For now, we'll just return a dummy success.
        Ok(json!({ "status": "success", "action": action.action_type, "token_used": access_token }))
    }
}

pub struct OpenAIProvider {
    config: OpenAIConfig,
}

impl OpenAIProvider {
    pub fn new(config: OpenAIConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl AIProvider for OpenAIProvider {
    fn name(&self) -> &str {
        "OpenAI"
    }

    async fn generate_response(
        &self,
        prompt: &str,
        history: &[Interaction],
        function_registry: Option<&FunctionRegistry>,
    ) -> Result<String, CopilotError> {
        info!("OpenAI: Generating response for prompt: {}", prompt);
        // Placeholder for actual OpenAI API call
        Ok(format!("OpenAI response to: {}", prompt))
    }

    async fn call_function(&self, action: &AgentAction) -> Result<serde_json::Value, CopilotError> {
        info!("OpenAI: Calling function: {}", action.action_type);
        // Placeholder for actual OpenAI function call
        Ok(serde_json::json!({ "status": "success", "action": action.action_type }))
    }
}

pub struct AnthropicProvider {
    config: AnthropicConfig,
}

impl AnthropicProvider {
    pub fn new(config: AnthropicConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl AIProvider for AnthropicProvider {
    fn name(&self) -> &str {
        "Anthropic"
    }

    async fn generate_response(
        &self,
        prompt: &str,
        history: &[Interaction],
        function_registry: Option<&FunctionRegistry>,
    ) -> Result<String, CopilotError> {
        info!("Anthropic: Generating response for prompt: {}", prompt);
        // Placeholder for actual Anthropic API call
        Ok(format!("Anthropic response to: {}", prompt))
    }

    async fn call_function(&self, action: &AgentAction) -> Result<serde_json::Value, CopilotError> {
        info!("Anthropic: Calling function: {}", action.action_type);
        // Placeholder for actual Anthropic function call
        Ok(serde_json::json!({ "status": "success", "action": action.action_type }))
    }
}

pub struct AzureOpenAIProvider {
    config: AzureOpenAIConfig,
}

impl AzureOpenAIProvider {
    pub fn new(config: AzureOpenAIConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl AIProvider for AzureOpenAIProvider {
    fn name(&self) -> &str {
        "Azure OpenAI"
    }

    async fn generate_response(
        &self,
        prompt: &str,
        history: &[Interaction],
        function_registry: Option<&FunctionRegistry>,
    ) -> Result<String, CopilotError> {
        info!("Azure OpenAI: Generating response for prompt: {}", prompt);
        // Placeholder for actual Azure OpenAI API call
        Ok(format!("Azure OpenAI response to: {}", prompt))
    }

    async fn call_function(&self, action: &AgentAction) -> Result<serde_json::Value, CopilotError> {
        info!("Azure OpenAI: Calling function: {}", action.action_type);
        // Placeholder for actual Azure OpenAI function call
        Ok(serde_json::json!({ "status": "success", "action": action.action_type }))
    }
}

pub struct OllamaProvider {
    config: OllamaConfig,
}

impl OllamaProvider {
    pub fn new(config: OllamaConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl AIProvider for OllamaProvider {
    fn name(&self) -> &str {
        "Ollama"
    }

    async fn generate_response(
        &self,
        prompt: &str,
        history: &[Interaction],
        function_registry: Option<&FunctionRegistry>,
    ) -> Result<String, CopilotError> {
        info!("Ollama: Generating response for prompt: {}", prompt);
        // Placeholder for actual Ollama API call
        Ok(format!("Ollama response to: {}", prompt))
    }

    async fn call_function(&self, action: &AgentAction) -> Result<serde_json::Value, CopilotError> {
        info!("Ollama: Calling function: {}", action.action_type);
        // Placeholder for actual Ollama function call
        Ok(serde_json::json!({ "status": "success", "action": action.action_type }))
    }
}

pub struct AIOrchestrator {
    providers: Vec<Box<dyn AIProvider + Send + Sync>>,
    current_provider_index: Mutex<usize>,
}

impl AIOrchestrator {
    pub fn new(config: AIProvidersConfig) -> Self {
        let mut providers: Vec<Box<dyn AIProvider + Send + Sync>> = Vec::new();

        if let Some(google_config) = config.google {
            providers.push(Box::new(GoogleAIProvider::new(google_config)));
        }
        if let Some(openai_config) = config.openai {
            providers.push(Box::new(OpenAIProvider::new(openai_config)));
        }
        if let Some(anthropic_config) = config.anthropic {
            providers.push(Box::new(AnthropicProvider::new(anthropic_config)));
        }
        if let Some(azure_openai_config) = config.azure_openai {
            providers.push(Box::new(AzureOpenAIProvider::new(azure_openai_config)));
        }
        if let Some(ollama_config) = config.ollama {
            providers.push(Box::new(OllamaProvider::new(ollama_config)));
        }

        Self {
            providers,
            current_provider_index: Mutex::new(0),
        }
    }

    pub async fn generate_response(
        &self,
        prompt: &str,
        history: &[Interaction],
        function_registry: Option<&FunctionRegistry>,
    ) -> Result<String, CopilotError> {
        let mut current_index = self.current_provider_index.lock().unwrap();
        let initial_index = *current_index;

        loop {
            let provider = &self.providers[*current_index];
            info!(
                "Attempting to generate response with {} provider.",
                provider.name()
            );
            match provider
                .generate_response(prompt, history, function_registry)
                .await
            {
                Ok(response) => return Ok(response),
                Err(e) => {
                    error!("Provider {} failed: {}", provider.name(), e);
                    *current_index = (*current_index + 1) % self.providers.len();
                    if *current_index == initial_index {
                        return Err(CopilotError::AIProvider(format!(
                            "All AI providers failed to generate a response: {}",
                            e
                        )));
                    }
                }
            }
        }
    }

    pub async fn call_function(
        &self,
        action: &AgentAction,
    ) -> Result<serde_json::Value, CopilotError> {
        let mut current_index = self.current_provider_index.lock().unwrap();
        let initial_index = *current_index;

        loop {
            let provider = &self.providers[*current_index];
            info!(
                "Attempting to call function with {} provider.",
                provider.name()
            );
            match provider.call_function(action).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    error!(
                        "Provider {} failed to call function: {}",
                        provider.name(),
                        e
                    );
                    *current_index = (*current_index + 1) % self.providers.len();
                    if *current_index == initial_index {
                        return Err(CopilotError::AIProvider(format!(
                            "All AI providers failed to call the function: {}",
                            e
                        )));
                    }
                }
            }
        }
    }
}
