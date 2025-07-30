use oxide_guardian::guardian::Guardian;
use oxide_memory::memory::MemoryManager;
use oxide_copilot::functions::FunctionRegistry;
use oxide_core::config::GuardianConfig;
use oxide_core::types::{Interaction, Context};
use std::time::{Instant, Duration};
use tokio;
use chrono::Utc;

#[tokio::test]
async fn test_guardian_performance() {
    println!("🔍 Testing Guardian Agent performance...");

    let config = GuardianConfig {
        enabled: true,
        monitor_interval_secs: 1,
    };

    let guardian = Guardian::new(config);

    // Measure system status retrieval time
    let start = Instant::now();
    let status = guardian.get_system_status();
    let duration = start.elapsed();

    assert!(duration < Duration::from_millis(100), "System status should be retrieved quickly");
    println!("✓ System status retrieved in {:?}", duration);
    println!("  CPU: {:.2}%, Memory: {}/{} bytes, Processes: {}",
        status.cpu_usage, status.memory_usage.0, status.memory_usage.1, status.process_count);

    // Measure threat history retrieval
    let start = Instant::now();
    let threats = guardian.get_threat_history();
    let duration = start.elapsed();

    assert!(duration < Duration::from_millis(50), "Threat history should be retrieved quickly");
    println!("✓ Threat history retrieved in {:?} ({} threats)", duration, threats.len());
}

#[tokio::test]
async fn test_memory_performance() {
    println!("🧠 Testing Memory System performance...");

    let memory_manager = MemoryManager::new(Some("perf_test".to_string()));
    memory_manager.initialize().await.expect("Memory should initialize");

    // Test storing multiple interactions
    let start = Instant::now();
    let num_interactions = 100;

    for i in 0..num_interactions {
        let interaction = Interaction {
            id: uuid::Uuid::new_v4(),
            timestamp: Utc::now(),
            user_input: format!("Test query number {}", i),
            agent_response: format!("Response to query {}", i),
            context: Context {
                system_state: Default::default(),
                user_history: Vec::new(),
                relevant_events: Vec::new(),
                knowledge_entries: Vec::new(),
            },
        };

        memory_manager.store_interaction(interaction).await.expect("Should store interaction");
    }

    let store_duration = start.elapsed();
    let avg_store_time = store_duration / num_interactions;

    println!("✓ Stored {} interactions in {:?} (avg: {:?} per interaction)",
        num_interactions, store_duration, avg_store_time);

    assert!(avg_store_time < Duration::from_millis(10), "Each interaction should store quickly");

    // Test retrieval performance
    let start = Instant::now();
    let query = oxide_memory::memory::ContextQuery {
        query: "test query".to_string(),
        context_type: None,
        time_range: None,
        max_results: 50,
        min_relevance: 0.1,
    };

    let results = memory_manager.retrieve_context(&query).await.expect("Should retrieve context");
    let retrieve_duration = start.elapsed();

    println!("✓ Retrieved {} results in {:?}", results.len(), retrieve_duration);
    assert!(retrieve_duration < Duration::from_millis(100), "Context retrieval should be fast");

    // Test memory stats performance
    let start = Instant::now();
    let stats = memory_manager.get_memory_stats();
    let stats_duration = start.elapsed();

    println!("✓ Memory stats retrieved in {:?}", stats_duration);
    println!("  Total entries: {}, Patterns: {}", stats.total_entries, stats.total_patterns);
    assert!(stats_duration < Duration::from_millis(10), "Stats should be retrieved instantly");
}

#[tokio::test]
async fn test_function_execution_performance() {
    println!("⚙️ Testing Function Registry performance...");

    let function_registry = FunctionRegistry::new();

    // Test function schema retrieval
    let start = Instant::now();
    let schemas = function_registry.get_all_function_schemas();
    let schema_duration = start.elapsed();

    println!("✓ Retrieved {} function schemas in {:?}", schemas.len(), schema_duration);
    assert!(schema_duration < Duration::from_millis(10), "Schema retrieval should be instant");

    // Test function execution performance
    let functions_to_test = vec![
        ("get_current_time", serde_json::json!({})),
        ("read_file", serde_json::json!({"path": "Cargo.toml"})),
    ];

    for (function_name, args) in functions_to_test {
        let start = Instant::now();
        let result = function_registry.execute_function(function_name, args).await;
        let execution_duration = start.elapsed();

        match result {
            Ok(_) => {
                println!("✓ Function '{}' executed in {:?}", function_name, execution_duration);
                assert!(execution_duration < Duration::from_secs(1), "Function should execute within reasonable time");
            },
            Err(e) => {
                println!("⚠ Function '{}' failed: {} (may be expected)", function_name, e);
            }
        }
    }
}

#[tokio::test]
async fn test_concurrent_operations() {
    println!("🔄 Testing concurrent operations performance...");

    let memory_manager = std::sync::Arc::new(MemoryManager::new(Some("concurrent_test".to_string())));
    memory_manager.initialize().await.expect("Memory should initialize");

    let function_registry = std::sync::Arc::new(FunctionRegistry::new());

    // Test concurrent memory operations
    let start = Instant::now();
    let num_concurrent = 50;
    let mut handles = Vec::new();

    for i in 0..num_concurrent {
        let memory_clone = std::sync::Arc::clone(&memory_manager);
        let handle = tokio::spawn(async move {
            let interaction = Interaction {
                id: uuid::Uuid::new_v4(),
                timestamp: Utc::now(),
                user_input: format!("Concurrent query {}", i),
                agent_response: format!("Concurrent response {}", i),
                context: Context {
                    system_state: Default::default(),
                    user_history: Vec::new(),
                    relevant_events: Vec::new(),
                    knowledge_entries: Vec::new(),
                },
            };

            memory_clone.store_interaction(interaction).await
        });
        handles.push(handle);
    }

    // Wait for all operations to complete
    for handle in handles {
        handle.await.expect("Task should complete").expect("Store should succeed");
    }

    let concurrent_duration = start.elapsed();
    println!("✓ {} concurrent memory operations completed in {:?}", num_concurrent, concurrent_duration);

    // Test concurrent function executions
    let start = Instant::now();
    let mut handles = Vec::new();

    for _i in 0..20 {
        let registry_clone = std::sync::Arc::clone(&function_registry);
        let handle = tokio::spawn(async move {
            registry_clone.execute_function("get_current_time", serde_json::json!({})).await
        });
        handles.push(handle);
    }

    let mut successful_executions = 0;
    for handle in handles {
        if handle.await.expect("Task should complete").is_ok() {
            successful_executions += 1;
        }
    }

    let function_duration = start.elapsed();
    println!("✓ {}/20 concurrent function executions completed in {:?}",
        successful_executions, function_duration);

    assert!(successful_executions > 0, "At least some concurrent executions should succeed");
}

#[tokio::test]
async fn test_memory_usage() {
    println!("💾 Testing memory usage patterns...");

    // Get initial memory usage
    let initial_memory = get_process_memory_usage();
    println!("Initial memory usage: {} MB", initial_memory / 1024 / 1024);

    // Create components and measure memory growth
    let memory_manager = MemoryManager::new(Some("memory_usage_test".to_string()));
    memory_manager.initialize().await.expect("Memory should initialize");

    let after_memory_manager = get_process_memory_usage();
    println!("After MemoryManager: {} MB (+{} MB)",
        after_memory_manager / 1024 / 1024,
        (after_memory_manager - initial_memory) / 1024 / 1024);

    // Store many interactions and measure memory growth
    for i in 0..1000 {
        let interaction = Interaction {
            id: uuid::Uuid::new_v4(),
            timestamp: Utc::now(),
            user_input: format!("Memory test query {} with some additional text to increase size", i),
            agent_response: format!("Memory test response {} with detailed information about the query", i),
            context: Context {
                system_state: Default::default(),
                user_history: Vec::new(),
                relevant_events: Vec::new(),
                knowledge_entries: Vec::new(),
            },
        };

        memory_manager.store_interaction(interaction).await.expect("Should store");
    }

    let after_interactions = get_process_memory_usage();
    println!("After 1000 interactions: {} MB (+{} MB)",
        after_interactions / 1024 / 1024,
        (after_interactions - after_memory_manager) / 1024 / 1024);

    // Memory growth should be reasonable
    let memory_growth = after_interactions - initial_memory;
    assert!(memory_growth < 100 * 1024 * 1024, "Memory growth should be less than 100MB for 1000 interactions");

    println!("✓ Memory usage test completed - total growth: {} MB", memory_growth / 1024 / 1024);
}

fn get_process_memory_usage() -> u64 {
    use sysinfo::{System, SystemExt, ProcessExt, PidExt};

    let mut system = System::new();
    system.refresh_processes();

    let current_pid = sysinfo::get_current_pid().expect("Should get current PID");

    if let Some(process) = system.process(current_pid) {
        process.memory() * 1024 // Convert from KB to bytes
    } else {
        0
    }
}

#[tokio::test]
async fn test_system_resource_impact() {
    println!("📊 Testing system resource impact...");

    use sysinfo::{System, SystemExt, CpuExt};

    let mut system = System::new_all();
    system.refresh_all();

    let initial_cpu = system.global_cpu_info().cpu_usage();
    println!("Initial system CPU usage: {:.2}%", initial_cpu);

    // Create Guardian and let it run briefly
    let config = GuardianConfig {
        enabled: true,
        monitor_interval_secs: 1,
    };

    let guardian = Guardian::new(config);

    // Simulate some monitoring activity
    for _i in 0..10 {
        let _status = guardian.get_system_status();
        let _threats = guardian.get_threat_history();
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    system.refresh_all();
    let final_cpu = system.global_cpu_info().cpu_usage();
    println!("Final system CPU usage: {:.2}%", final_cpu);

    let cpu_impact = final_cpu - initial_cpu;
    println!("CPU impact: {:.2}%", cpu_impact);

    // The impact should be minimal
    assert!(cpu_impact < 10.0, "CPU impact should be less than 10%");

    println!("✓ System resource impact test completed");
}