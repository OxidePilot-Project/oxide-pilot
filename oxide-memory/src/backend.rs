use async_trait::async_trait;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct BackendSearchItem {
    pub text: String,
    pub score: f32,
    pub source: Option<String>,
    pub meta: Option<Value>,
}

#[async_trait]
pub trait MemoryBackend: Send + Sync {
    async fn add_texts(
        &self,
        items: Vec<(String, Vec<String>)>,
        metadata: Value,
    ) -> Result<(), String>;

    async fn search(&self, query: String, top_k: usize) -> Result<Vec<BackendSearchItem>, String>;
}
