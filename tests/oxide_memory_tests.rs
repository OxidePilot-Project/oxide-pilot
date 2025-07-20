use oxide_memory::memory::MemoryManager;
use oxide_core::types::{SystemEvent, Interaction, Context};
use uuid::Uuid;
use chrono::Utc;

#[tokio::test]
async fn test_memory_manager_store_and_retrieve_system_event() {
    let mut manager = MemoryManager::new();
    let event = SystemEvent {
        id: Uuid::new_v4(),
        timestamp: Utc::now(),
        event_type: "test_event".to_string(),
        details: serde_json::json!({ "key": "value" }),
    };

    let store_result = manager.store_system_event(event.clone()).await;
    assert!(store_result.is_ok());

    let retrieved_data = manager.retrieve_context(&event.id.to_string()).await;
    assert!(retrieved_data.is_ok());
    assert!(retrieved_data.unwrap().contains("test_event"));
}

#[tokio::test]
async fn test_memory_manager_store_and_retrieve_interaction() {
    let mut manager = MemoryManager::new();
    let interaction = Interaction {
        id: Uuid::new_v4(),
        timestamp: Utc::now(),
        user_input: "Hello".to_string(),
        agent_response: "Hi there!".to_string(),
        context: Context {
            active_window: None,
            system_status: None,
            recent_events: vec![],
        },
    };

    let store_result = manager.store_interaction(interaction.clone()).await;
    assert!(store_result.is_ok());

    let retrieved_data = manager.retrieve_context(&interaction.id.to_string()).await;
    assert!(retrieved_data.is_ok());
    assert!(retrieved_data.unwrap().contains("Hi there!"));
}
