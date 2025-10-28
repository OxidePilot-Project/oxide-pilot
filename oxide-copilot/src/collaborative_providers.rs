use crate::errors::CopilotError;
use crate::llm_orchestrator::{CollaborativeContext, CollaborativeLLM, LLMRole};
use async_trait::async_trait;
use log::info;
use oxide_core::gemini_auth::GeminiAuth;
use oxide_core::openai_client::{self, ChatMessage};
use oxide_core::qwen_auth::QwenAuth;

/// Gemini implementation for collaborative tasks
pub struct CollaborativeGemini {
    auth: GeminiAuth,
    role: LLMRole,
    model: Option<String>,
}

impl CollaborativeGemini {
    pub fn new(role: LLMRole, model: Option<String>) -> Self {
        Self {
            auth: GeminiAuth::new(),
            role,
            model,
        }
    }

    async fn ensure_authenticated(&self) -> Result<(), CopilotError> {
        if !self.auth.is_authenticated().await {
            // Try to initialize from environment
            if let Err(e) = self.auth.init_from_env().await {
                return Err(CopilotError::Authentication(format!(
                    "Gemini authentication failed: {e}"
                )));
            }
        }
        Ok(())
    }
}

#[async_trait]
impl CollaborativeLLM for CollaborativeGemini {
    fn name(&self) -> &str {
        "Gemini"
    }

    fn role(&self) -> LLMRole {
        self.role.clone()
    }

    async fn generate_response(
        &self,
        prompt: &str,
        context: &CollaborativeContext,
    ) -> Result<String, CopilotError> {
        self.ensure_authenticated().await?;

        // Enhance prompt with context
        let enhanced_prompt = format!(
            "{}\n\nContext:\n- Task Type: {}\n- System State: {}\n- Available Functions: {}\n- Constraints: {}",
            prompt,
            context.task_type,
            context.system_state,
            context.available_functions.join(", "),
            serde_json::to_string_pretty(&context.constraints).unwrap_or_default()
        );

        info!("Gemini ({}): Generating response", self.role());

        self.auth
            .send_message(&enhanced_prompt, self.model.as_deref())
            .await
            .map_err(|e| CopilotError::AIProvider(format!("Gemini error: {e}")))
    }

    async fn analyze_with_role(
        &self,
        task: &str,
        context: &CollaborativeContext,
        role_specific_prompt: &str,
    ) -> Result<String, CopilotError> {
        self.ensure_authenticated().await?;

        // Create role-specific system prompt
        let system_prompt = match self.role() {
            LLMRole::Coordinator => "You are the primary coordinator for system tasks. You analyze requests, create execution plans, and coordinate with other AI agents.",
            LLMRole::Analyst => "You are a technical analyst specializing in deep system analysis, performance optimization, and security assessment.",
            LLMRole::Executor => "You are a system executor responsible for carrying out system operations, commands, and automated tasks.",
            LLMRole::Innovator => "You are an innovator who provides creative solutions, alternative approaches, and optimization strategies.",
            LLMRole::Validator => "You are a validator who reviews and validates AI responses, ensuring quality, consistency, and safety.",
        };

        let full_prompt = format!(
            "{}\n\n{}\n\nTask: {}\nUser Input: {}",
            system_prompt, role_specific_prompt, task, context.user_input
        );

        info!(
            "Gemini ({}): Analyzing with role-specific prompt",
            self.role()
        );

        self.auth
            .send_message(&full_prompt, self.model.as_deref())
            .await
            .map_err(|e| CopilotError::AIProvider(format!("Gemini analysis error: {e}")))
    }
}

/// Qwen implementation for collaborative tasks
pub struct CollaborativeQwen {
    auth: QwenAuth,
    role: LLMRole,
    #[allow(dead_code)]
    model: Option<String>,
}

impl CollaborativeQwen {
    pub fn new(role: LLMRole, model: Option<String>) -> Self {
        Self {
            auth: QwenAuth::new(),
            role,
            model,
        }
    }

    async fn ensure_authenticated(&self) -> Result<(), CopilotError> {
        match self.auth.get_auth_status().await {
            Ok(status) => {
                if !status.contains("authenticated") {
                    return Err(CopilotError::Authentication(
                        "Qwen not authenticated. Please complete OAuth2 device flow.".to_string(),
                    ));
                }
            }
            Err(e) => {
                return Err(CopilotError::Authentication(format!(
                    "Qwen authentication check failed: {e}"
                )));
            }
        }
        Ok(())
    }

    async fn send_message(&self, prompt: &str) -> Result<String, CopilotError> {
        // TODO: Implement Qwen message sending
        // For now, we'll return a placeholder
        let _ = prompt;
        Err(CopilotError::AIProvider(
            "Qwen send_message not yet implemented".to_string(),
        ))
    }
}

#[async_trait]
impl CollaborativeLLM for CollaborativeQwen {
    fn name(&self) -> &str {
        "Qwen"
    }

    fn role(&self) -> LLMRole {
        self.role.clone()
    }

    async fn generate_response(
        &self,
        prompt: &str,
        context: &CollaborativeContext,
    ) -> Result<String, CopilotError> {
        self.ensure_authenticated().await?;

        // Enhance prompt with context
        let enhanced_prompt = format!(
            "{}\n\nContext:\n- Task Type: {}\n- System State: {}\n- Available Functions: {}\n- Constraints: {}",
            prompt,
            context.task_type,
            context.system_state,
            context.available_functions.join(", "),
            serde_json::to_string_pretty(&context.constraints).unwrap_or_default()
        );

        info!("Qwen ({}): Generating response", self.role());

        self.send_message(&enhanced_prompt).await
    }

    async fn analyze_with_role(
        &self,
        task: &str,
        context: &CollaborativeContext,
        role_specific_prompt: &str,
    ) -> Result<String, CopilotError> {
        self.ensure_authenticated().await?;

        // Create role-specific system prompt
        let system_prompt = match self.role() {
            LLMRole::Coordinator => "You are the primary coordinator for system tasks. You analyze requests, create execution plans, and coordinate with other AI agents.",
            LLMRole::Analyst => "You are a technical analyst specializing in deep system analysis, performance optimization, and security assessment.",
            LLMRole::Executor => "You are a system executor responsible for carrying out system operations, commands, and automated tasks.",
            LLMRole::Innovator => "You are an innovator who provides creative solutions, alternative approaches, and optimization strategies.",
            LLMRole::Validator => "You are a validator who reviews and validates AI responses, ensuring quality, consistency, and safety.",
        };

        let full_prompt = format!(
            "{}\n\n{}\n\nTask: {}\nUser Input: {}",
            system_prompt, role_specific_prompt, task, context.user_input
        );

        info!(
            "Qwen ({}): Analyzing with role-specific prompt",
            self.role()
        );

        self.send_message(&full_prompt).await
    }
}

/// OpenAI implementation for collaborative tasks (API Key based)
pub struct CollaborativeOpenAI {
    role: LLMRole,
    model: String,
}

impl CollaborativeOpenAI {
    pub fn new(role: LLMRole, model: Option<String>) -> Self {
        Self {
            role,
            model: model.unwrap_or_else(|| "gpt-4o".to_string()),
        }
    }

    async fn ensure_authenticated(&self) -> Result<(), CopilotError> {
        // Check if API key is available
        match oxide_core::openai_key::get_api_key().await {
            Ok(Some(_)) => Ok(()),
            Ok(None) => Err(CopilotError::Authentication(
                "OpenAI API key not configured. Please set OPENAI_API_KEY or configure via UI."
                    .to_string(),
            )),
            Err(e) => Err(CopilotError::Authentication(format!(
                "Failed to check OpenAI API key: {e}"
            ))),
        }
    }
}

#[async_trait]
impl CollaborativeLLM for CollaborativeOpenAI {
    fn name(&self) -> &str {
        "OpenAI"
    }

    fn role(&self) -> LLMRole {
        self.role.clone()
    }

    async fn generate_response(
        &self,
        prompt: &str,
        context: &CollaborativeContext,
    ) -> Result<String, CopilotError> {
        self.ensure_authenticated().await?;

        // Enhance prompt with context
        let enhanced_prompt = format!(
            "{}\n\nContext:\n- Task Type: {}\n- System State: {}\n- Available Functions: {}\n- Constraints: {}",
            prompt,
            context.task_type,
            context.system_state,
            context.available_functions.join(", "),
            serde_json::to_string_pretty(&context.constraints).unwrap_or_default()
        );

        info!(
            "OpenAI ({}): Generating response with model {}",
            self.role(),
            self.model
        );

        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: "You are an intelligent system assistant helping with system analysis and automation.".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: enhanced_prompt,
            },
        ];

        openai_client::chat_completion(&self.model, messages, Some(0.7), Some(2000))
            .await
            .map_err(|e| CopilotError::AIProvider(format!("OpenAI error: {e}")))
    }

    async fn analyze_with_role(
        &self,
        task: &str,
        context: &CollaborativeContext,
        role_specific_prompt: &str,
    ) -> Result<String, CopilotError> {
        self.ensure_authenticated().await?;

        // Create role-specific system prompt
        let system_prompt = match self.role() {
            LLMRole::Coordinator => "You are the primary coordinator for system tasks. You analyze requests, create execution plans, and coordinate with other AI agents.",
            LLMRole::Analyst => "You are a technical analyst specializing in deep system analysis, performance optimization, and security assessment.",
            LLMRole::Executor => "You are a system executor responsible for carrying out system operations, commands, and automated tasks.",
            LLMRole::Innovator => "You are an innovator who provides creative solutions, alternative approaches, and optimization strategies.",
            LLMRole::Validator => "You are a validator who reviews and validates AI responses, ensuring quality, consistency, and safety.",
        };

        let full_prompt = format!(
            "{}\n\nTask: {}\nUser Input: {}",
            role_specific_prompt, task, context.user_input
        );

        info!(
            "OpenAI ({}): Analyzing with role-specific prompt",
            self.role()
        );

        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: system_prompt.to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: full_prompt,
            },
        ];

        openai_client::chat_completion(&self.model, messages, Some(0.7), Some(2000))
            .await
            .map_err(|e| CopilotError::AIProvider(format!("OpenAI analysis error: {e}")))
    }
}

/// Factory for creating collaborative LLM providers
pub struct CollaborativeProviderFactory;

impl CollaborativeProviderFactory {
    /// Create a Gemini provider with specified role
    pub fn create_gemini(role: LLMRole, model: Option<String>) -> Box<dyn CollaborativeLLM> {
        Box::new(CollaborativeGemini::new(role, model))
    }

    /// Create an OpenAI provider with specified role (uses API Key)
    pub fn create_openai(role: LLMRole, model: Option<String>) -> Box<dyn CollaborativeLLM> {
        Box::new(CollaborativeOpenAI::new(role, model))
    }

    /// Create a Qwen provider with specified role
    pub fn create_qwen(role: LLMRole, model: Option<String>) -> Box<dyn CollaborativeLLM> {
        Box::new(CollaborativeQwen::new(role, model))
    }

    /// Create a default collaborative setup with Gemini as coordinator and Qwen as analyst
    pub fn create_default_setup() -> Vec<(String, Box<dyn CollaborativeLLM>, LLMRole)> {
        vec![
            (
                "gemini_coordinator".to_string(),
                Self::create_gemini(LLMRole::Coordinator, Some("gemini-1.5-pro".to_string())),
                LLMRole::Coordinator,
            ),
            (
                "qwen_analyst".to_string(),
                Self::create_qwen(LLMRole::Analyst, None),
                LLMRole::Analyst,
            ),
            (
                "gemini_executor".to_string(),
                Self::create_gemini(LLMRole::Executor, Some("gemini-1.5-flash".to_string())),
                LLMRole::Executor,
            ),
            (
                "qwen_validator".to_string(),
                Self::create_qwen(LLMRole::Validator, None),
                LLMRole::Validator,
            ),
        ]
    }
}
