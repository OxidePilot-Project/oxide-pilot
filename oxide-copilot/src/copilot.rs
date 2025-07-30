use crate::ai::AIOrchestrator;
use crate::functions::FunctionRegistry;
use oxide_core::config::CopilotConfig;
use oxide_core::types::{Context, Interaction};
use serde_json::Value;

use crate::errors::CopilotError;
use crate::gemini_api::{Content, FunctionCall, FunctionResponse, InlineData, Part};
use image::{ImageBuffer, Rgba};
use log::{error, info};
use oxide_rpa::rpa::ScreenCapture;
use std::sync::{Arc, Mutex};

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

    pub async fn analyze_screen(&self) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, CopilotError> {
        info!("CopilotAgent: Performing screen analysis.");
        let screenshot = self
            .screen_capture
            .capture_screen()
            .await
            .map_err(|e| CopilotError::ScreenCapture(e))?;
        // In a real scenario, this image would be sent to a vision-capable LLM
        // For now, we just return the image.
        Ok(screenshot)
    }

    pub fn update_config(&self, new_config: CopilotConfig) {
        let mut config = self.config.lock().unwrap();
        *config = new_config;
        info!("Copilot config updated.");
    }

    pub async fn handle_user_input(
        &self,
        user_input: String,
        context: Context,
    ) -> Result<String, CopilotError> {
        info!("Handling user input: {}", user_input);

        let mut history_lock = self.conversation_history.lock().unwrap();
        let mut current_history: Vec<Interaction> = history_lock.clone();

        // Add initial user input to current history
        let initial_interaction = Interaction {
            id: uuid::Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
            user_input: user_input.clone(),
            agent_response: String::new(), // Will be filled later
            context: context.clone(),
        };
        current_history.push(initial_interaction.clone());

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
                            .map_err(|e| CopilotError::FunctionExecution(e))?;

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
                                    .map_err(|e| CopilotError::Serialization(e))?
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
                                .map_err(|e| CopilotError::Serialization(e))?
                            ),
                            context: context.clone(),
                        });
                    }
                    Err(e) => {
                        error!("Failed to parse function call JSON: {}", e);
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
        if let Some(last_interaction) = history_lock.last_mut() {
            if last_interaction.id == initial_interaction.id {
                last_interaction.agent_response = final_agent_response.clone();
            }
        }

        Ok(final_agent_response)
    }

    pub fn get_conversation_history(&self) -> Vec<Interaction> {
        self.conversation_history.lock().unwrap().clone()
    }
}
