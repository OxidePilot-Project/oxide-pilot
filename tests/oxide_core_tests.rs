use oxide_core::config::{OxidePilotConfig, GuardianConfig, CopilotConfig, AIProvidersConfig};
use oxide_core::config_manager;
use std::path::Path;
use std::fs;

#[test]
fn test_config_validation() {
    let valid_config = OxidePilotConfig {
        guardian: GuardianConfig {
            enabled: true,
            monitor_interval_secs: 10,
        },
        copilot: CopilotConfig {
            enabled: true,
            wake_word: "Hey Oxide".to_string(),
        },
        ai_providers: AIProvidersConfig {
            google: Some(oxide_core::config::GoogleConfig { api_key: "test_key".to_string() }),
            openai: None,
            anthropic: None,
            azure_openai: None,
            ollama: None,
        },
    };
    assert!(valid_config.validate().is_ok());

    let invalid_guardian_config = OxidePilotConfig {
        guardian: GuardianConfig {
            enabled: true,
            monitor_interval_secs: 0,
        },
        copilot: CopilotConfig {
            enabled: false,
            wake_word: "".to_string(),
        },
        ai_providers: AIProvidersConfig {
            google: None,
            openai: None,
            anthropic: None,
            azure_openai: None,
            ollama: None,
        },
    };
    assert!(invalid_guardian_config.validate().is_err());

    let invalid_copilot_config = OxidePilotConfig {
        guardian: GuardianConfig {
            enabled: false,
            monitor_interval_secs: 0,
        },
        copilot: CopilotConfig {
            enabled: true,
            wake_word: "".to_string(),
        },
        ai_providers: AIProvidersConfig {
            google: Some(oxide_core::config::GoogleConfig { api_key: "test_key".to_string() }),
            openai: None,
            anthropic: None,
            azure_openai: None,
            ollama: None,
        },
    };
    assert!(invalid_copilot_config.validate().is_err());

    let no_ai_provider_config = OxidePilotConfig {
        guardian: GuardianConfig {
            enabled: false,
            monitor_interval_secs: 0,
        },
        copilot: CopilotConfig {
            enabled: true,
            wake_word: "Hey Oxide".to_string(),
        },
        ai_providers: AIProvidersConfig {
            google: None,
            openai: None,
            anthropic: None,
            azure_openai: None,
            ollama: None,
        },
    };
    assert!(no_ai_provider_config.validate().is_err());
}

#[test]
fn test_load_config() {
    let config_content = r#"{
        "guardian": {
            "enabled": true,
            "monitor_interval_secs": 5
        },
        "copilot": {
            "enabled": true,
            "wake_word": "Hello"
        },
        "ai_providers": {
            "google": {
                "api_key": "some_google_key"
            }
        }
    }"#;

    let test_dir = "./test_config_load";
    let config_path = Path::new(test_dir).join("config.json");
    fs::create_dir_all(test_dir).unwrap();
    fs::write(&config_path, config_content).unwrap();

    let loaded_config = config_manager::load_config(&config_path).unwrap();
    assert_eq!(loaded_config.guardian.monitor_interval_secs, 5);
    assert_eq!(loaded_config.copilot.wake_word, "Hello");
    assert!(loaded_config.ai_providers.google.is_some());

    fs::remove_dir_all(test_dir).unwrap();
}
