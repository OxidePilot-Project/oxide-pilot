use oxide_copilot::ai::{AIOrchestrator, GoogleAIProvider, OpenAIProvider};
use oxide_copilot::copilot::CopilotAgent;
use oxide_copilot::functions::FunctionRegistry;
use oxide_core::config::{AIProvidersConfig, GoogleConfig, OpenAIConfig, CopilotConfig};
use oxide_core::types::Context;
use std::sync::Arc;

#[tokio::test]
async fn test_ai_orchestrator_generate_response() {
    let config = AIProvidersConfig {
        google: Some(GoogleConfig { api_key: "test_google_key".to_string() }),
        openai: Some(OpenAIConfig { api_key: "test_openai_key".to_string() }),
        anthropic: None,
        azure_openai: None,
        ollama: None,
    };
    let orchestrator = AIOrchestrator::new(config);

    let prompt = "Hello, AI!";
    let history = vec![];

    let response = orchestrator.generate_response(prompt, &history).await;
    assert!(response.is_ok());
    let response_text = response.unwrap();
    assert!(response_text.contains("Google AI response to:") || response_text.contains("OpenAI response to:"));
}

#[tokio::test]
async fn test_copilot_agent_handle_user_input() {
    let ai_config = AIProvidersConfig {
        google: Some(GoogleConfig { api_key: "test_google_key".to_string() }),
        openai: None,
        anthropic: None,
        azure_openai: None,
        ollama: None,
    };
    let ai_orchestrator = Arc::new(AIOrchestrator::new(ai_config));

    let copilot_config = CopilotConfig {
        enabled: true,
        wake_word: "Hey Copilot".to_string(),
    };
    let copilot_agent = CopilotAgent::new(copilot_config, ai_orchestrator.clone());

    let user_input = "What is the weather like?".to_string();
    let context = Context {
        active_window: None,
        system_status: None,
        recent_events: vec![],
    };

    let response = copilot_agent.handle_user_input(user_input.clone(), context.clone()).await;
    assert!(response.is_ok());
    let response_text = response.unwrap();
    assert!(response_text.contains("Google AI response to:"));

    let history = copilot_agent.get_conversation_history();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].user_input, user_input);
    assert_eq!(history[0].agent_response, response_text);
}

#[tokio::test]
async fn test_function_registry() {
    let registry = FunctionRegistry::new();
    let function_name = "get_current_time";
    let args = serde_json::json!({});

    let result = registry.execute_function(function_name, args).await;
    assert!(result.is_ok());
    let result_value = result.unwrap();
    assert!(result_value["current_time"].is_string());
}
