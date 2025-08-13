use crate::ai::AIOrchestrator;
use crate::functions::FunctionRegistry;
use oxide_core::config::CopilotConfig;
use oxide_core::types::{Context, Interaction};
// use serde_json::Value; // Reserved for future use

use crate::errors::CopilotError;
use crate::gemini_api::{FunctionCall, FunctionResponse, Part};
// use image::{ImageBuffer, Rgba}; // Reserved for future use
use log::{error, info};
use oxide_rpa::rpa::ScreenCapture;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct CopilotAgent {
    config: Arc<Mutex<CopilotConfig>>,
    ai_orchestrator: Arc<AIOrchestrator>,
    conversation_history: Mutex<Vec<Interaction>>,
    screen_capture: ScreenCapture,
    function_registry: Arc<FunctionRegistry>,
}

impl CopilotAgent {
    pub fn new(
        config: CopilotConfig,
        ai_orchestrator: Arc<AIOrchestrator>,
        function_registry: Arc<FunctionRegistry>,
    ) -> Self {
        Self {
            config: Arc::new(Mutex::new(config)),
            ai_orchestrator,
            conversation_history: Mutex::new(Vec::new()),
            screen_capture: ScreenCapture::new(),
            function_registry,
        }
    }

    pub async fn analyze_screen(&self) -> Result<String, CopilotError> {
        info!("CopilotAgent: Performing screen analysis.");
        let _screenshot = self
            .screen_capture
            .capture_screen()
            .await
            .map_err(CopilotError::ScreenCapture)?;
        // In a real scenario, this image would be sent to a vision-capable LLM
        // For now, we return a simple analysis result
        Ok("Screen analysis completed successfully".to_string())
    }

    pub async fn update_config(&self, new_config: CopilotConfig) {
        let mut config = self.config.lock().await;
        *config = new_config;
        info!("Copilot config updated.");
    }

    pub async fn handle_user_input(
        &self,
        user_input: String,
        context: Context,
    ) -> Result<String, CopilotError> {
        info!("Handling user input: {user_input}");

        // Get current history without holding the lock
        let mut current_history: Vec<Interaction> = {
            let history_lock = self.conversation_history.lock().await;
            history_lock.clone()
        };

        // Add initial user input to current history
        let initial_interaction = Interaction {
            id: uuid::Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
            user_input: user_input.clone(),
            agent_response: String::new(), // Will be filled later
            context: context.clone(),
        };
        current_history.push(initial_interaction.clone());

        #[allow(unused_assignments)]
        let mut final_agent_response = String::new();
        let mut turn_count = 0;
        const MAX_TURNS: usize = 10; // Safeguard against infinite loops

        loop {
            turn_count += 1;
            if turn_count > MAX_TURNS {
                return Err(CopilotError::MaxTurnsExceeded);
            }

            let agent_response_str = self
                .ai_orchestrator
                .generate_response(
                    &user_input, // The original prompt, or a follow-up if needed
                    &current_history,
                    Some(&self.function_registry),
                )
                .await?;

            if agent_response_str.starts_with("FUNCTION_CALL:") {
                let function_call_str = agent_response_str
                    .trim_start_matches("FUNCTION_CALL:")
                    .trim();
                match serde_json::from_str::<FunctionCall>(function_call_str) {
                    Ok(function_call) => {
                        info!(
                            "Executing function: {} with args: {}",
                            function_call.name, function_call.args
                        );
                        let function_result = self
                            .function_registry
                            .execute_function(&function_call.name, function_call.args.clone())
                            .await
                            .map_err(CopilotError::FunctionExecution)?;

                        // Check if this is an analyze_screen function that returned image data
                        let mut parts = vec![];
                        if function_call.name == "analyze_screen"
                            && function_result.get("image_data").is_some()
                        {
                            // Add the image as inline data for vision analysis
                            if let (Some(image_data), Some(mime_type)) = (
                                function_result.get("image_data").and_then(|v| v.as_str()),
                                function_result.get("mime_type").and_then(|v| v.as_str()),
                            ) {
                                parts.push(Part {
                                    text: None,
                                    function_call: None,
                                    function_response: None,
                                    inline_data: Some(crate::gemini_api::InlineData {
                                        mime_type: mime_type.to_string(),
                                        data: image_data.to_string(),
                                    }),
                                });
                            }
                        }

                        // Add the function response as text
                        parts.push(Part {
                            text: None,
                            function_call: None,
                            function_response: Some(FunctionResponse {
                                name: function_call.name.clone(),
                                response: function_result.clone(),
                            }),
                            inline_data: None,
                        });
                        info!(
                            "Function {} executed with result: {}",
                            function_call.name, function_result
                        );

                        // Add function call and response to history for the next turn
                        current_history.push(Interaction {
                            id: uuid::Uuid::new_v4(),
                            timestamp: chrono::Utc::now(),
                            user_input: String::new(), // No user input for model's function call
                            agent_response: format!(
                                "FUNCTION_CALL: {}",
                                serde_json::to_string(&function_call)
                                    .map_err(CopilotError::Serialization)?
                            ),
                            context: context.clone(),
                        });
                        current_history.push(Interaction {
                            id: uuid::Uuid::new_v4(),
                            timestamp: chrono::Utc::now(),
                            user_input: String::new(), // No user input for function response
                            agent_response: format!(
                                "FUNCTION_RESPONSE: {}",
                                serde_json::to_string(&FunctionResponse {
                                    name: function_call.name.clone(),
                                    response: function_result
                                })
                                .map_err(CopilotError::Serialization)?
                            ),
                            context: context.clone(),
                        });
                    }
                    Err(e) => {
                        error!("Failed to parse function call JSON: {e}");
                        return Err(CopilotError::InvalidFunctionCallJson(e.to_string()));
                    }
                }
            } else {
                // AI returned a text response, so we're done
                final_agent_response = agent_response_str;
                break;
            }
        }

        // Update the stored conversation history with the final interaction
        {
            let mut history_lock = self.conversation_history.lock().await;
            if let Some(last_interaction) = history_lock.last_mut() {
                if last_interaction.id == initial_interaction.id {
                    last_interaction.agent_response = final_agent_response.clone();
                }
            }
        }

        Ok(final_agent_response)
    }

    pub async fn get_conversation_history(&self) -> Vec<Interaction> {
        self.conversation_history.lock().await.clone()
    }
}
