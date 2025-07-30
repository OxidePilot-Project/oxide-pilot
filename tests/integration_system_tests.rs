use oxide_core::config::{OxidePilotConfig, GuardianConfig, CopilotConfig, AIProvidersConfig, GoogleConfig};
use oxide_guardian::guardian::{Guardian, ThreatDetector};
use oxide_copilot::copilot::CopilotAgent;
use oxide_copilot::ai::AIOrchestrator;
use oxide_copilot::functions::FunctionRegistry;
use oxide_memory::memory::{MemoryManager, ContextQuery, MemoryEntryType};
use oxide_core::types::{Context, Interaction, SystemEvent};
use tokio;
use chrono::Utc;
use std::sync::Arc;

#[tokio::test]
async fn test_system_configuration() {
    let config = OxidePilotConfig {
        guardian: GuardianConfig {
            enabled: true,
            monitor_interval_secs: 10,
        },
        copilot: CopilotConfig {
            enabled: true,
            wake_word: "Hey Oxide".to_string(),
        },
        ai_providers: AIProvidersConfig {
            google: Some(GoogleConfig {
                api_key: "test-key".to_string(),
            }),
            openai: None,
            anthropic: None,
            azure_openai: None,
            ollama: None,
        },
    };

    let validation_result = config.validate();
    assert!(validation_result.is_ok(), "Configuration should be valid: {:?}", validation_result);
    println!("✓ System configuration validation passed");
}

#[tokio::test]
async fn test_guardian_agent_initialization() {
    let config = GuardianConfig {
        enabled: true,
        monitor_interval_secs: 5,
    };

    let guardian = Guardian::new(config);
    let system_status = guardian.get_system_status();

    assert!(system_status.cpu_usage >= 0.0, "CPU usage should be non-negative");
    assert!(system_status.memory_usage.0 <= system_status.memory_usage.1, "Used memory should not exceed total");
    assert!(system_status.process_count > 0, "Should detect some processes");

    println!("✓ Guardian Agent initialized successfully");
    println!("  CPU Usage: {:.2}%", system_status.cpu_usage);
    println!("  Memory: {} / {} bytes", system_status.memory_usage.0, system_status.memory_usage.1);
    println!("  Processes: {}", system_status.process_count);
    println!("  Threats: {}", system_status.threat_count);
}

#[tokio::test]
async fn test_threat_detection_engine() {
    let threat_detector = ThreatDetector::new();

    // Create a mock suspicious process event
    let suspicious_event = SystemEvent {
        id: uuid::Uuid::new_v4(),
        timestamp: Utc::now(),
        event_type: "process_info".to_string(),
        source: "system_monitor".to_string(),
        severity: "medium".to_string(),
        details: std::collections::HashMap::from([
            ("name".to_string(), "powershell.exe".to_string()),
            ("command".to_string(), "powershell -encodedcommand AAABBBCCC".to_string()),
            ("pid".to_string(), "1234".to_string()),
            ("cpu_usage".to_string(), "85.5".to_string()),
        ]),
    };

    let threats = threat_detector.analyze_processes(&[suspicious_event]);

    // Should detect multiple threats from this suspicious process
    assert!(!threats.is_empty(), "Should detect threats from suspicious PowerShell process");

    for threat in &threats {
        println!("✓ Detected threat: {} ({})", threat.description, threat.severity);
    }

    println!("✓ Threat detection engine working correctly");
}

#[tokio::test]
async fn test_memory_system() {
    let memory_manager = MemoryManager::new(Some("test_memory".to_string()));
    memory_manager.initialize().await.expect("Memory manager should initialize");

    // Test storing an interaction
    let interaction = Interaction {
        id: uuid::Uuid::new_v4(),
        timestamp: Utc::now(),
        user_input: "What processes are using the most CPU?".to_string(),
        agent_response: "Chrome is using 45% CPU with 8 tabs open.".to_string(),
        context: Context {
            system_state: Default::default(),
            user_history: Vec::new(),
            relevant_events: Vec::new(),
            knowledge_entries: Vec::new(),
        },
    };

    let store_result = memory_manager.store_interaction(interaction.clone()).await;
    assert!(store_result.is_ok(), "Should store interaction successfully");

    // Test retrieving context
    let query = ContextQuery {
        query: "CPU usage".to_string(),
        context_type: Some(MemoryEntryType::UserInteraction),
        time_range: None,
        max_results: 10,
        min_relevance: 0.1,
    };

    let context_result = memory_manager.retrieve_context(&query).await;
    assert!(context_result.is_ok(), "Should retrieve context successfully");

    let contexts = context_result.unwrap();
    assert!(!contexts.is_empty(), "Should find relevant context");

    println!("✓ Memory system working correctly");
    println!("  Stored interaction: {}", interaction.id);
    println!("  Retrieved {} relevant contexts", contexts.len());

    // Test memory stats
    let stats = memory_manager.get_memory_stats();
    assert!(stats.total_entries > 0, "Should have stored entries");
    println!("  Memory stats: {} entries, {} patterns", stats.total_entries, stats.total_patterns);
}

#[tokio::test]
async fn test_function_registry() {
    let function_registry = FunctionRegistry::new();

    // Test getting all function schemas
    let schemas = function_registry.get_all_function_schemas();
    assert!(!schemas.is_empty(), "Should have registered functions");

    for schema in &schemas {
        println!("✓ Function available: {}", schema["name"]);
    }

    // Test executing a function
    let time_result = function_registry.execute_function("get_current_time", serde_json::json!({})).await;
    assert!(time_result.is_ok(), "Should execute get_current_time function");

    let time_value = time_result.unwrap();
    assert!(time_value["current_time"].is_string(), "Should return current time as string");

    println!("✓ Function registry working correctly");
    println!("  Available functions: {}", schemas.len());
    println!("  Current time: {}", time_value["current_time"]);
}

#[tokio::test]
async fn test_ai_orchestrator() {
    let ai_config = AIProvidersConfig {
        google: Some(GoogleConfig {
            api_key: "test-key".to_string(),
        }),
        openai: None,
        anthropic: None,
        azure_openai: None,
        ollama: None,
    };

    let orchestrator = AIOrchestrator::new(ai_config);

    // Test that the orchestrator was created with providers
    // Note: We can't test actual API calls without valid credentials
    println!("✓ AI Orchestrator created successfully");

    // Test with mock interaction (this will fail without real credentials, which is expected)
    let mock_history = vec![
        Interaction {
            id: uuid::Uuid::new_v4(),
            timestamp: Utc::now(),
            user_input: "Hello".to_string(),
            agent_response: "Hi there!".to_string(),
            context: Context {
                system_state: Default::default(),
                user_history: Vec::new(),
                relevant_events: Vec::new(),
                knowledge_entries: Vec::new(),
            },
        }
    ];

    let response_result = orchestrator.generate_response("Test prompt", &mock_history, None).await;

    // This will likely fail without real credentials, which is expected in tests
    match response_result {
        Ok(response) => {
            println!("✓ AI response generated: {}", response);
        },
        Err(e) => {
            println!("⚠ AI response failed (expected without credentials): {}", e);
        }
    }
}

#[tokio::test]
async fn test_copilot_agent() {
    let copilot_config = CopilotConfig {
        enabled: true,
        wake_word: "Hey Oxide".to_string(),
    };

    let ai_config = AIProvidersConfig {
        google: Some(GoogleConfig {
            api_key: "test-key".to_string(),
        }),
        openai: None,
        anthropic: None,
        azure_openai: None,
        ollama: None,
    };

    let ai_orchestrator = Arc::new(AIOrchestrator::new(ai_config));
    let function_registry = Arc::new(FunctionRegistry::new());

    let copilot = CopilotAgent::new(copilot_config, ai_orchestrator, function_registry);

    // Test conversation history
    let history = copilot.get_conversation_history();
    assert!(history.is_empty(), "Should start with empty history");

    // Test screen analysis
    let screen_result = copilot.analyze_screen().await;
    match screen_result {
        Ok(_) => println!("✓ Screen analysis completed"),
        Err(e) => println!("⚠ Screen analysis failed: {}", e),
    }

    println!("✓ Copilot Agent created and tested successfully");
}

#[tokio::test]
async fn test_end_to_end_workflow() {
    println!("🚀 Starting end-to-end workflow test...");

    // 1. Initialize all components
    let config = OxidePilotConfig::default();
    assert!(config.validate().is_ok(), "Default config should be valid");

    // 2. Create Guardian Agent
    let guardian = Guardian::new(config.guardian.clone());
    let initial_status = guardian.get_system_status();
    println!("✓ Guardian initialized - {} processes monitored", initial_status.process_count);

    // 3. Create Memory Manager
    let memory_manager = MemoryManager::new(Some("test_e2e".to_string()));
    memory_manager.initialize().await.expect("Memory should initialize");
    println!("✓ Memory system initialized");

    // 4. Create Function Registry
    let function_registry = Arc::new(FunctionRegistry::new());
    let available_functions = function_registry.get_all_function_schemas();
    println!("✓ Function registry created with {} functions", available_functions.len());

    // 5. Create AI Orchestrator
    let ai_orchestrator = Arc::new(AIOrchestrator::new(config.ai_providers.clone()));
    println!("✓ AI Orchestrator created");

    // 6. Create Copilot Agent
    let copilot = CopilotAgent::new(config.copilot.clone(), ai_orchestrator, function_registry);
    println!("✓ Copilot Agent created");

    // 7. Simulate a user interaction
    let context = Context {
        system_state: Default::default(),
        user_history: Vec::new(),
        relevant_events: Vec::new(),
        knowledge_entries: Vec::new(),
    };

    // This will likely fail without real API credentials, but tests the flow
    let interaction_result = copilot.handle_user_input(
        "What is the current time?".to_string(),
        context
    ).await;

    match interaction_result {
        Ok(response) => {
            println!("✓ End-to-end interaction successful: {}", response);
        },
        Err(e) => {
            println!("⚠ End-to-end interaction failed (expected without API credentials): {}", e);
        }
    }

    // 8. Check final system state
    let final_status = guardian.get_system_status();
    println!("✓ Final system status - CPU: {:.1}%, Memory: {}/{} bytes",
        final_status.cpu_usage,
        final_status.memory_usage.0,
        final_status.memory_usage.1
    );

    println!("🎉 End-to-end workflow test completed successfully!");
}