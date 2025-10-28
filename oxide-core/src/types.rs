use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SystemEvent {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub event_type: String,
    pub details: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Interaction {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub user_input: String,
    pub agent_response: String,
    pub context: Context,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AgentAction {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub action_type: String,
    pub parameters: serde_json::Value,
    pub result: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Context {
    pub active_window: Option<String>,
    pub system_status: Option<serde_json::Value>,
    pub recent_events: Vec<SystemEvent>,
}
