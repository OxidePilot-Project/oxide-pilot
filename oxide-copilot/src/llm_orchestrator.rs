use crate::errors::CopilotError;
use async_trait::async_trait;
use log::{info, warn};
use serde_json::{json, Value};
use std::collections::HashMap;

/// Represents the role of an LLM in collaborative tasks
#[derive(Debug, Clone, PartialEq)]
pub enum LLMRole {
    /// Primary coordinator and decision maker
    Coordinator,
    /// Specialized in deep technical analysis
    Analyst,
    /// Handles system operations and commands
    Executor,
    /// Provides creative solutions and alternatives
    Innovator,
    /// Validates and reviews outputs
    Validator,
}

impl std::fmt::Display for LLMRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LLMRole::Coordinator => write!(f, "Coordinator"),
            LLMRole::Analyst => write!(f, "Analyst"),
            LLMRole::Executor => write!(f, "Executor"),
            LLMRole::Innovator => write!(f, "Innovator"),
            LLMRole::Validator => write!(f, "Validator"),
        }
    }
}

/// Configuration for LLM collaboration
#[derive(Debug, Clone)]
pub struct LLMConfig {
    pub provider: String,
    pub model: Option<String>,
    pub role: LLMRole,
    pub temperature: f32,
    pub max_tokens: Option<u32>,
    pub system_prompt: String,
}

/// Result of a collaborative LLM task
#[derive(Debug, Clone)]
pub struct CollaborativeResult {
    pub primary_response: String,
    pub secondary_responses: HashMap<String, String>,
    pub consensus_score: f32,
    pub confidence: f32,
    pub execution_plan: Option<Value>,
}

/// Trait for LLM providers that can participate in collaborative tasks
#[async_trait]
pub trait CollaborativeLLM: Send + Sync {
    fn name(&self) -> &str;
    fn role(&self) -> LLMRole;

    async fn generate_response(
        &self,
        prompt: &str,
        context: &CollaborativeContext,
    ) -> Result<String, CopilotError>;

    async fn analyze_with_role(
        &self,
        task: &str,
        context: &CollaborativeContext,
        role_specific_prompt: &str,
    ) -> Result<String, CopilotError>;
}

/// Context shared between collaborating LLMs
#[derive(Debug, Clone)]
pub struct CollaborativeContext {
    pub task_type: String,
    pub system_state: Value,
    pub user_input: String,
    pub conversation_history: Vec<Value>,
    pub available_functions: Vec<String>,
    pub constraints: HashMap<String, Value>,
}

/// Orchestrator for collaborative LLM tasks
pub struct LLMOrchestrator {
    providers: HashMap<String, Box<dyn CollaborativeLLM>>,
    configs: HashMap<String, LLMConfig>,
    // TODO: Re-enable when FunctionRegistry is available
    // function_registry: Option<FunctionRegistry>,
}

impl LLMOrchestrator {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
            configs: HashMap::new(),
            // function_registry: None,
        }
    }

    /// Add an LLM provider to the orchestrator
    pub fn add_provider(&mut self, name: String, provider: Box<dyn CollaborativeLLM>, config: LLMConfig) {
        self.providers.insert(name.clone(), provider);
        self.configs.insert(name, config);
    }

    /// Set the function registry for tool calling
    // TODO: Re-enable when FunctionRegistry is available
    // pub fn set_function_registry(&mut self, registry: FunctionRegistry) {
    //     self.function_registry = Some(registry);
    // }

    /// Execute a collaborative task using multiple LLMs
    pub async fn execute_collaborative_task(
        &self,
        task: &str,
        context: CollaborativeContext,
    ) -> Result<CollaborativeResult, CopilotError> {
        info!("Starting collaborative task: {}", task);

        // Step 1: Coordinator analyzes the task and creates execution plan
        let coordinator_response = self.execute_coordinator_phase(&task, &context).await?;

        // Step 2: Specialized LLMs provide their analysis
        let specialized_responses = self.execute_specialized_phase(&task, &context, &coordinator_response).await?;

        // Step 3: Validator reviews all responses
        let validation_result = self.execute_validation_phase(&task, &context, &coordinator_response, &specialized_responses).await?;

        // Step 4: Generate final consensus
        let consensus = self.generate_consensus(&coordinator_response, &specialized_responses, &validation_result).await?;

        Ok(consensus)
    }

    /// Coordinator phase: Primary LLM creates execution plan
    async fn execute_coordinator_phase(
        &self,
        task: &str,
        context: &CollaborativeContext,
    ) -> Result<String, CopilotError> {
        let coordinator = self.find_provider_by_role(LLMRole::Coordinator)?;

        let prompt = format!(
            "You are the primary coordinator for this system task. Analyze the following request and create a detailed execution plan:\n\n\
            Task: {}\n\
            System State: {}\n\
            User Input: {}\n\n\
            Provide:\n\
            1. Task breakdown into subtasks\n\
            2. Required system operations\n\
            3. Risk assessment\n\
            4. Recommended approach\n\
            5. Success criteria\n\n\
            Format your response as a structured plan.",
            task,
            context.system_state,
            context.user_input
        );

        coordinator.generate_response(&prompt, context).await
    }

    /// Specialized phase: Different LLMs provide role-specific analysis
    async fn execute_specialized_phase(
        &self,
        task: &str,
        context: &CollaborativeContext,
        coordinator_response: &str,
    ) -> Result<HashMap<String, String>, CopilotError> {
        let mut responses = HashMap::new();

        // Get all specialized providers (excluding coordinator)
        let specialized_providers: Vec<_> = self.providers
            .iter()
            .filter(|(_, provider)| provider.role() != LLMRole::Coordinator)
            .collect();

        // Execute specialized analysis concurrently
        let mut tasks = Vec::new();
        for (name, provider) in specialized_providers {
            let task_clone = task.to_string();
            let context_clone = context.clone();
            let coordinator_response_clone = coordinator_response.to_string();
            let provider_name = name.clone();

            let task = async move {
                let role_prompt = self.generate_role_specific_prompt(
                    &task_clone,
                    &context_clone,
                    provider.role(),
                    &coordinator_response_clone,
                );

                match provider.analyze_with_role(&task_clone, &context_clone, &role_prompt).await {
                    Ok(response) => Ok((provider_name, response)),
                    Err(e) => {
                        warn!("Specialized provider {} failed: {}", name, e);
                        Err(e)
                    }
                }
            };

            tasks.push(task);
        }

        // Wait for all specialized responses
        for task in tasks {
            match task.await {
                Ok((name, response)) => {
                    responses.insert(name, response);
                }
                Err(e) => {
                    warn!("Specialized analysis failed: {}", e);
                }
            }
        }

        Ok(responses)
    }

    /// Validation phase: Validator reviews all responses
    async fn execute_validation_phase(
        &self,
        task: &str,
        context: &CollaborativeContext,
        coordinator_response: &str,
        specialized_responses: &HashMap<String, String>,
    ) -> Result<String, CopilotError> {
        let validator = self.find_provider_by_role(LLMRole::Validator)?;

        let prompt = format!(
            "You are the validator for this collaborative task. Review all responses and provide validation:\n\n\
            Original Task: {}\n\
            Coordinator Plan: {}\n\
            Specialized Responses: {}\n\n\
            Provide:\n\
            1. Quality assessment of each response\n\
            2. Consistency check across responses\n\
            3. Risk identification\n\
            4. Recommendations for improvement\n\
            5. Final validation score (0-100)\n\n\
            Format as a structured validation report.",
            task,
            coordinator_response,
            serde_json::to_string_pretty(specialized_responses).unwrap_or_default()
        );

        validator.generate_response(&prompt, context).await
    }

    /// Generate final consensus from all responses
    async fn generate_consensus(
        &self,
        coordinator_response: &str,
        specialized_responses: &HashMap<String, String>,
        validation_result: &str,
    ) -> Result<CollaborativeResult, CopilotError> {
        // Calculate consensus score based on response consistency
        let consensus_score = self.calculate_consensus_score(specialized_responses);

        // Calculate confidence based on validation score
        let confidence = self.extract_confidence_from_validation(validation_result);

        // Create execution plan from coordinator response
        let execution_plan = self.extract_execution_plan(coordinator_response);

        Ok(CollaborativeResult {
            primary_response: coordinator_response.to_string(),
            secondary_responses: specialized_responses.clone(),
            consensus_score,
            confidence,
            execution_plan,
        })
    }

    /// Find provider by role
    fn find_provider_by_role(&self, role: LLMRole) -> Result<&Box<dyn CollaborativeLLM>, CopilotError> {
        self.providers
            .values()
            .find(|provider| provider.role() == role)
            .ok_or_else(|| CopilotError::AIProvider(format!("No provider found for role: {:?}", role)))
    }

    /// Generate role-specific prompt
    fn generate_role_specific_prompt(
        &self,
        task: &str,
        context: &CollaborativeContext,
        role: LLMRole,
        coordinator_response: &str,
    ) -> String {
        match role {
            LLMRole::Analyst => format!(
                "As a technical analyst, provide deep analysis of this task:\n\n\
                Task: {}\n\
                Coordinator Plan: {}\n\
                System State: {}\n\n\
                Focus on:\n\
                - Technical feasibility\n\
                - Performance implications\n\
                - Security considerations\n\
                - Detailed implementation steps",
                task, coordinator_response, context.system_state
            ),

            LLMRole::Executor => format!(
                "As a system executor, provide execution details for this task:\n\n\
                Task: {}\n\
                Coordinator Plan: {}\n\
                Available Functions: {}\n\n\
                Focus on:\n\
                - Specific commands to execute\n\
                - System operations required\n\
                - Error handling strategies\n\
                - Monitoring and logging",
                task,
                coordinator_response,
                context.available_functions.join(", ")
            ),

            LLMRole::Innovator => format!(
                "As an innovator, provide creative solutions for this task:\n\n\
                Task: {}\n\
                Coordinator Plan: {}\n\
                Constraints: {}\n\n\
                Focus on:\n\
                - Alternative approaches\n\
                - Creative solutions\n\
                - Optimization opportunities\n\
                - Future improvements",
                task,
                coordinator_response,
                serde_json::to_string_pretty(&context.constraints).unwrap_or_default()
            ),

            _ => format!(
                "Provide your specialized analysis for this task:\n\n\
                Task: {}\n\
                Coordinator Plan: {}\n\
                Context: {}\n\n\
                Provide detailed analysis from your perspective.",
                task, coordinator_response, context.user_input
            ),
        }
    }

    /// Calculate consensus score based on response consistency
    fn calculate_consensus_score(&self, responses: &HashMap<String, String>) -> f32 {
        if responses.is_empty() {
            return 0.0;
        }

        // Simple consensus calculation based on response length similarity
        let lengths: Vec<usize> = responses.values().map(|r| r.len()).collect();
        let avg_length = lengths.iter().sum::<usize>() as f32 / lengths.len() as f32;
        let variance = lengths.iter()
            .map(|&len| (len as f32 - avg_length).powi(2))
            .sum::<f32>() / lengths.len() as f32;

        // Convert variance to consensus score (lower variance = higher consensus)
        (1.0 - (variance / avg_length.powi(2))).max(0.0).min(1.0)
    }

    /// Extract confidence score from validation result
    fn extract_confidence_from_validation(&self, validation: &str) -> f32 {
        // Look for confidence score in validation text
        if let Some(score_match) = regex::Regex::new(r"confidence[:\s]+(\d+(?:\.\d+)?)")
            .unwrap()
            .captures(validation)
        {
            if let Ok(score) = score_match[1].parse::<f32>() {
                return (score / 100.0).max(0.0).min(1.0);
            }
        }

        // Default confidence based on validation length and content
        if validation.len() > 500 && validation.contains("validation score") {
            0.8
        } else if validation.len() > 200 {
            0.6
        } else {
            0.4
        }
    }

    /// Extract execution plan from coordinator response
    fn extract_execution_plan(&self, coordinator_response: &str) -> Option<Value> {
        // Try to parse structured plan from coordinator response
        if let Ok(plan) = serde_json::from_str::<Value>(coordinator_response) {
            return Some(plan);
        }

        // Create a simple plan structure from text response
        Some(json!({
            "plan": coordinator_response,
            "steps": coordinator_response.split('\n').collect::<Vec<_>>(),
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    }
}

impl Default for LLMOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}
