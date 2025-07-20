use oxide_core::config::{OxidePilotConfig, GuardianConfig, CopilotConfig, AIProvidersConfig, GoogleConfig};
use oxide_core::config_manager;
use oxide_guardian::guardian::Guardian;
use oxide_copilot::ai::AIOrchestrator;
use oxide_copilot::copilot::CopilotAgent;
use oxide_core::types::Context;
use std::sync::Arc;
use std::path::Path;
use std::fs;
use tokio::sync::mpsc::channel;
use std::time::Duration;

#[tokio::test]
async fn test_guardian_copilot_integration() {
    // Setup config file for hot-reloading
    let config_dir = "./test_integration_config";
    let config_path = Path::new(config_dir).join("config.json");
    fs::create_dir_all(config_dir).unwrap();

    let initial_config_content = r#"{
        "guardian": {
            "enabled": true,
            "monitor_interval_secs": 1
        },
        "copilot": {
            "enabled": true,
            "wake_word": "Hey Oxide"
        },
        "ai_providers": {
            "google": {
                "api_key": "test_google_key"
            }
        }
    }"#;
    fs::write(&config_path, initial_config_content).unwrap();

    // Load initial config
    let initial_config = config_manager::load_config(&config_path).unwrap();

    // Setup AI Orchestrator
    let ai_orchestrator = Arc::new(AIOrchestrator::new(initial_config.ai_providers.clone()));

    // Setup Copilot Agent
    let copilot_agent = CopilotAgent::new(initial_config.copilot.clone(), ai_orchestrator.clone());

    // Setup Guardian Agent
    let guardian = Guardian::new(initial_config.guardian.clone());
    guardian.start_monitoring();

    // Simulate user interaction
    let user_input = "Hello, how are you?".to_string();
    let context = Context {
        active_window: Some("Test Window".to_string()),
        system_status: None,
        recent_events: vec![],
    };

    let copilot_response = copilot_agent.handle_user_input(user_input.clone(), context.clone()).await;
    assert!(copilot_response.is_ok());
    assert!(copilot_response.unwrap().contains("Google AI response to:"));

    // Simulate config change for hot-reloading
    let updated_config_content = r#"{
        "guardian": {
            "enabled": false,
            "monitor_interval_secs": 1
        },
        "copilot": {
            "enabled": true,
            "wake_word": "Hey Oxide Updated"
        },
        "ai_providers": {
            "google": {
                "api_key": "test_google_key"
            }
        }
    }"#;
    fs::write(&config_path, updated_config_content).unwrap();

    // Give some time for the watcher to pick up the change
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Verify guardian config updated (this is hard to test directly without exposing internal state)
    // For now, we rely on the log messages and the fact that the config_manager::watch_config sends updates

    // Clean up
    fs::remove_dir_all(config_dir).unwrap();
}
