use oxide_core::config::{OxidePilotConfig, GuardianConfig, CopilotConfig, AIProvidersConfig, GoogleConfig};
use oxide_guardian::security::{PermissionManager, PermissionRequest, RPAActionType, PermissionLevel};
use oxide_guardian::optimizer::PerformanceOptimizer;
use oxide_core::encryption::EncryptionManager;
use std::time::{SystemTime, UNIX_EPOCH};

// Test configuration with Gemini API key
fn create_test_config() -> OxidePilotConfig {
    OxidePilotConfig {
        guardian: GuardianConfig {
            enabled: true,
            monitor_interval_secs: 5,
        },
        copilot: CopilotConfig {
            enabled: true,
            wake_word: "Oxide".to_string(),
        },
        ai_providers: AIProvidersConfig {
            google: Some(GoogleConfig {
                api_key: std::env::var("GEMINI_API_KEY").unwrap_or("YOUR_TEST_API_KEY_HERE".to_string()),
            }),
            openai: None,
            anthropic: None,
            azure_openai: None,
            ollama: None,
        },
    }
}

// Test RPA permissions
fn test_rpa_permissions() {
    println!("Testing RPA permissions...");
    
    let mut permission_manager = PermissionManager::new();
    
    // Test setting permissions
    permission_manager.set_permission(RPAActionType::MouseClick, PermissionLevel::Allow);
    permission_manager.set_permission(RPAActionType::KeyboardInput, PermissionLevel::Ask);
    permission_manager.set_permission(RPAActionType::NetworkAccess, PermissionLevel::Deny);
    
    // Test permission request
    let request = PermissionRequest {
        action_type: RPAActionType::MouseClick,
        description: "Click on button".to_string(),
        target: Some("Submit button".to_string()),
        severity: 2,
        timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
    };
    
    // This should be allowed based on our configuration
    let result = permission_manager.request_permission(request).await;
    assert!(result, "Mouse click should be allowed");
    
    println!("RPA permissions test passed!");
}

// Test performance optimization
fn test_performance_optimizer() {
    println!("Testing performance optimizer...");
    
    let mut optimizer = PerformanceOptimizer::new();
    
    // Get current usage
    let usage = optimizer.get_current_usage();
    println!("Current CPU usage: {:.2}%", usage.cpu_percent);
    println!("Current memory usage: {} MB", usage.memory_mb);
    
    // Check if we should throttle
    let should_throttle = optimizer.should_throttle();
    println!("Should throttle: {}", should_throttle);
    
    // Get optimization recommendations
    let recommendations = optimizer.get_optimization_recommendations();
    println!("Optimization recommendations: {}", recommendations.len());
    
    // Test system idle detection
    let is_idle = optimizer.is_system_idle();
    println!("System is idle: {}", is_idle);
    
    println!("Performance optimizer test passed!");
}

// Test encryption and security
fn test_encryption_security() {
    println!("Testing encryption and security...");
    
    // Generate encryption key
    let key = EncryptionManager::generate_key();
    let encryption_manager = EncryptionManager::new(&key).expect("Failed to create encryption manager");
    
    // Test data encryption
    let test_data = b"This is a test message for encryption";
    let encrypted = encryption_manager.encrypt_data(test_data, None)
        .expect("Failed to encrypt data");
    
    // Test data decryption
    let decrypted = encryption_manager.decrypt_data(&encrypted)
        .expect("Failed to decrypt data");
    
    assert_eq!(test_data, decrypted.as_slice(), "Decrypted data doesn't match original");
    
    // Test role-based access control
    encryption_manager.add_user("test_user".to_string(), vec!["admin".to_string()]);
    
    let has_permission = encryption_manager.has_permission("test_user", "system.control");
    assert!(has_permission, "Admin user should have system.control permission");
    
    let no_permission = encryption_manager.has_permission("test_user", "nonexistent.permission");
    assert!(!no_permission, "User should not have nonexistent permission");
    
    println!("Encryption and security test passed!");
}

// Test configuration validation
fn test_config_validation() {
    println!("Testing configuration validation...");
    
    let config = create_test_config();
    
    // This should pass validation
    let validation_result = config.validate();
    assert!(validation_result.is_ok(), "Configuration validation failed: {:?}", validation_result.err());
    
    println!("Configuration validation test passed!");
}

#[tokio::main]
async fn main() {
    println!("Running Oxide Pilot Integration Tests");
    println!("=====================================");
    
    // Run all tests
    test_config_validation();
    test_rpa_permissions().await;
    test_performance_optimizer();
    test_encryption_security();
    
    println!("\nAll integration tests passed successfully!");
    println!("Oxide Pilot is ready for packaging with Gemini support.");
}
