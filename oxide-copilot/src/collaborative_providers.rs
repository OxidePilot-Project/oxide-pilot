use crate::errors::CopilotError;
use crate::llm_orchestrator::{CollaborativeLLM, CollaborativeContext, LLMRole};
use crate::functions::FunctionRegistry;
use async_trait::async_trait;
use log::{info, warn};
use oxide_core::gemini_auth::GeminiAuth;
use oxide_core::qwen_auth::QwenAuth;
use serde_json::json;

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
                    "Gemini authentication failed: {}", e
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
            .map_err(|e| CopilotError::AIProvider(format!("Gemini error: {}", e)))
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
            system_prompt,
            role_specific_prompt,
            task,
            context.user_input
        );

        info!("Gemini ({}): Analyzing with role-specific prompt", self.role());

        self.auth
            .send_message(&full_prompt, self.model.as_deref())
            .await
            .map_err(|e| CopilotError::AIProvider(format!("Gemini analysis error: {}", e)))
    }
}

/// Qwen implementation for collaborative tasks
pub struct CollaborativeQwen {
    auth: QwenAuth,
    role: LLMRole,
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
                        "Qwen not authenticated. Please complete OAuth2 device flow.".to_string()
                    ));
                }
            }
            Err(e) => {
                return Err(CopilotError::Authentication(format!(
                    "Qwen authentication check failed: {}", e
                )));
            }
        }
        Ok(())
    }

    async fn send_message(&self, prompt: &str) -> Result<String, CopilotError> {
        // TODO: Implement Qwen message sending
        // For now, we'll return a placeholder
        let _ = prompt;
        Err(CopilotError::AIProvider("Qwen send_message not yet implemented".to_string()))
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
            system_prompt,
            role_specific_prompt,
            task,
            context.user_input
        );

        info!("Qwen ({}): Analyzing with role-specific prompt", self.role());

        self.send_message(&full_prompt).await
    }
}

/// Factory for creating collaborative LLM providers
pub struct CollaborativeProviderFactory;

impl CollaborativeProviderFactory {
    /// Create a Gemini provider with specified role
    pub fn create_gemini(role: LLMRole, model: Option<String>) -> Box<dyn CollaborativeLLM> {
        Box::new(CollaborativeGemini::new(role, model))
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
