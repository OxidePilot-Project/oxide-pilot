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

#[cfg(feature = "cognee")]
pub struct CogneeBackend {
    client: oxide_cognee_bridge::CogneeClient,
}

#[cfg(feature = "cognee")]
impl CogneeBackend {
    pub fn new(base_url: String, token: Option<String>) -> Result<Self, String> {
        let client = oxide_cognee_bridge::CogneeClient::new(base_url, token)
            .map_err(|e| e.to_string())?;
        Ok(Self { client })
    }
}

#[cfg(feature = "cognee")]
#[async_trait]
impl MemoryBackend for CogneeBackend {
    async fn add_texts(
        &self,
        items: Vec<(String, Vec<String>)>,
        metadata: Value,
    ) -> Result<(), String> {
        use oxide_cognee_bridge::types::{AddItem, AddRequest};
        let items = items
            .into_iter()
            .map(|(text, tags)| AddItem::Text { text, tags })
            .collect::<Vec<_>>();
        let req = AddRequest { items, metadata };
        self.client.add(&req).await.map_err(|e| e.to_string())
    }

    async fn search(&self, query: String, top_k: usize) -> Result<Vec<BackendSearchItem>, String> {
        use oxide_cognee_bridge::types::SearchRequest;
        let req = SearchRequest {
            query,
            top_k: Some(top_k as u32),
        };
        let res = self.client.search(&req).await.map_err(|e| e.to_string())?;
        Ok(res
            .into_iter()
            .map(|r| BackendSearchItem {
                text: r.text,
                score: r.score,
                source: r.source,
                meta: r.meta,
            })
            .collect())
    }
}
