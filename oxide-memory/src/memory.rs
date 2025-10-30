use crate::backend::MemoryBackend;
use chrono::{DateTime, Utc};
use log::{info, warn};
use oxide_core::types::{Interaction, SystemEvent};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub entry_type: MemoryEntryType,
    pub content: String,
    pub metadata: HashMap<String, String>,
    pub relevance_score: f32,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryEntryType {
    SystemEvent,
    UserInteraction,
    ThreatDetection,
    SystemOptimization,
    UserPattern,
    KnowledgeBase,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPattern {
    pub pattern_id: String,
    pub pattern_type: PatternType,
    pub frequency: u32,
    pub last_occurrence: DateTime<Utc>,
    pub confidence: f32,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternType {
    ApplicationUsage,
    TimeBasedActivity,
    CommandSequence,
    ErrorResolution,
    SystemConfiguration,
}

#[derive(Debug, Clone)]
pub struct ContextQuery {
    pub query: String,
    pub context_type: Option<MemoryEntryType>,
    pub time_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    pub max_results: usize,
    pub min_relevance: f32,
}

pub struct MemoryManager {
    memory_store: Arc<Mutex<HashMap<String, MemoryEntry>>>,
    user_patterns: Arc<Mutex<HashMap<String, UserPattern>>>,
    storage_path: String,
    max_entries: usize,
    backend: Option<Arc<dyn MemoryBackend>>,
}

impl MemoryManager {
    pub fn new(storage_path: Option<String>) -> Self {
        let path = storage_path.unwrap_or_else(|| "oxide_memory".to_string());

        Self {
            memory_store: Arc::new(Mutex::new(HashMap::new())),
            user_patterns: Arc::new(Mutex::new(HashMap::new())),
            storage_path: path,
            max_entries: 10000, // Configurable limit
            backend: None,
        }
    }

    /// Construct a memory manager backed by an external [`MemoryBackend`].
    ///
    /// This keeps the same in-memory caching logic but mirrors all writes to the
    /// provided backend (SurrealDB, etc.) for persistence and semantic search.
    pub fn with_backend(storage_path: Option<String>, backend: Arc<dyn MemoryBackend>) -> Self {
        let mut manager = Self::new(storage_path);
        manager.backend = Some(backend);
        manager
    }

    /// Attach or replace the external backend after construction.
    pub fn set_backend(&mut self, backend: Arc<dyn MemoryBackend>) {
        self.backend = Some(backend);
    }

    pub async fn initialize(&self) -> Result<(), String> {
        // Create storage directory if it doesn't exist
        if let Err(e) = fs::create_dir_all(&self.storage_path).await {
            return Err(format!("Failed to create storage directory: {e}"));
        }

        // Load existing memory from disk
        self.load_from_disk().await?;

        info!(
            "Memory manager initialized with storage path: {}",
            self.storage_path
        );
        Ok(())
    }

    pub async fn store_system_event(&self, event: SystemEvent) -> Result<(), String> {
        let memory_entry = MemoryEntry {
            id: event.id.to_string(),
            timestamp: event.timestamp,
            entry_type: MemoryEntryType::SystemEvent,
            content: serde_json::to_string(&event).map_err(|e| e.to_string())?,
            metadata: HashMap::from([
                ("event_type".to_string(), event.event_type.clone()),
                ("timestamp".to_string(), event.timestamp.to_rfc3339()),
            ]),
            relevance_score: self.calculate_relevance_score(&event),
            tags: self.extract_tags_from_event(&event),
        };

        self.store_memory_entry(memory_entry).await?;
        info!("Stored system event: {}", event.id);
        Ok(())
    }

    pub async fn store_interaction(&self, interaction: Interaction) -> Result<(), String> {
        let memory_entry = MemoryEntry {
            id: interaction.id.to_string(),
            timestamp: interaction.timestamp,
            entry_type: MemoryEntryType::UserInteraction,
            content: serde_json::to_string(&interaction).map_err(|e| e.to_string())?,
            metadata: HashMap::from([
                (
                    "user_input_length".to_string(),
                    interaction.user_input.len().to_string(),
                ),
                (
                    "response_length".to_string(),
                    interaction.agent_response.len().to_string(),
                ),
            ]),
            relevance_score: 0.8, // User interactions are generally highly relevant
            tags: self.extract_tags_from_interaction(&interaction),
        };

        self.store_memory_entry(memory_entry).await?;

        // Analyze for patterns
        self.analyze_user_patterns(&interaction).await?;

        info!("Stored interaction: {}", interaction.id);
        Ok(())
    }

    async fn store_memory_entry(&self, entry: MemoryEntry) -> Result<(), String> {
        let (content_for_backend, entry_type_for_backend) =
            (entry.content.clone(), format!("{:?}", entry.entry_type));
        {
            let mut store = self.memory_store.lock().await;

            // Check if we need to evict old entries
            if store.len() >= self.max_entries {
                self.evict_old_entries(&mut store);
            }

            store.insert(entry.id.clone(), entry);
        }

        // Persist to disk periodically
        self.save_to_disk().await?;

        // Mirror to backend (best-effort)
        if let Some(backend) = &self.backend {
            let metadata = serde_json::json!({ "source": "oxide-memory", "entry_type": entry_type_for_backend });
            if let Err(e) = backend
                .add_texts(vec![(content_for_backend, vec![])], metadata)
                .await
            {
                warn!("Backend add failed: {e}");
            }
        }

        Ok(())
    }

    fn evict_old_entries(&self, store: &mut HashMap<String, MemoryEntry>) {
        // Remove oldest 10% of entries
        let mut entries: Vec<_> = store.values().cloned().collect();
        entries.sort_by_key(|e| e.timestamp);

        let evict_count = self.max_entries / 10;
        let ids_to_remove: Vec<_> = entries
            .iter()
            .take(evict_count)
            .map(|e| e.id.clone())
            .collect();
        for id in ids_to_remove {
            store.remove(&id);
        }

        info!("Evicted {evict_count} old memory entries");
    }

    pub async fn retrieve_context(&self, query: &ContextQuery) -> Result<Vec<MemoryEntry>, String> {
        // If a backend is available, try it first
        if let Some(backend) = &self.backend {
            match backend.search(query.query.clone(), query.max_results).await {
                Ok(results) => {
                    let now = Utc::now();
                    let mut mapped: Vec<MemoryEntry> = Vec::new();
                    for r in results {
                        let mut metadata: HashMap<String, String> = HashMap::new();
                        if let Some(src) = r.source.clone() {
                            metadata.insert("source".to_string(), src);
                        }
                        // we don't attempt to stringify full meta here
                        mapped.push(MemoryEntry {
                            id: Uuid::new_v4().to_string(),
                            timestamp: now,
                            entry_type: MemoryEntryType::KnowledgeBase,
                            content: r.text,
                            metadata,
                            relevance_score: r.score,
                            tags: vec!["external".to_string()],
                        });
                    }
                    if !mapped.is_empty() {
                        info!(
                            "Retrieved {} backend results for query: {}",
                            mapped.len(),
                            query.query
                        );
                        return Ok(mapped);
                    }
                }
                Err(e) => warn!("Backend search failed: {e}"),
            }
        }
        let store = self.memory_store.lock().await;
        let mut relevant_entries = Vec::new();

        for entry in store.values() {
            // Filter by type if specified
            if let Some(ref context_type) = query.context_type {
                if std::mem::discriminant(&entry.entry_type) != std::mem::discriminant(context_type)
                {
                    continue;
                }
            }

            // Filter by time range if specified
            if let Some((start, end)) = query.time_range {
                if entry.timestamp < start || entry.timestamp > end {
                    continue;
                }
            }

            // Calculate relevance score for the query
            let relevance = self.calculate_query_relevance(&entry.content, &query.query);
            if relevance >= query.min_relevance {
                let mut entry_clone = entry.clone();
                entry_clone.relevance_score = relevance;
                relevant_entries.push(entry_clone);
            }
        }

        // Sort by relevance score (descending)
        relevant_entries.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());

        // Limit results
        relevant_entries.truncate(query.max_results);

        info!(
            "Retrieved {} relevant entries for query: {}",
            relevant_entries.len(),
            query.query
        );
        Ok(relevant_entries)
    }

    fn calculate_relevance_score(&self, event: &SystemEvent) -> f32 {
        // Since SystemEvent doesn't have severity, use event_type for scoring
        match event.event_type.as_str() {
            "threat_detected" => 1.0,
            "process_info" => 0.8,
            "file_access" => 0.6,
            "network_activity" => 0.4,
            _ => 0.3,
        }
    }

    fn extract_tags_from_event(&self, event: &SystemEvent) -> Vec<String> {
        let mut tags = vec![event.event_type.clone()];

        // Extract additional tags from event details
        if let Some(details_obj) = event.details.as_object() {
            for (key, value) in details_obj {
                if key == "process_name" || key == "file_path" || key == "network_address" {
                    if let Some(value_str) = value.as_str() {
                        tags.push(format!("{key}:{value_str}"));
                    }
                }
            }
        }

        tags
    }

    fn extract_tags_from_interaction(&self, interaction: &Interaction) -> Vec<String> {
        let mut tags = vec!["user_interaction".to_string()];

        // Extract keywords from user input
        let keywords = self.extract_keywords(&interaction.user_input);
        tags.extend(keywords);

        tags
    }

    fn extract_keywords(&self, text: &str) -> Vec<String> {
        // Simple keyword extraction (in a real implementation, use NLP libraries)
        let common_words = [
            "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with", "by",
        ];

        text.split_whitespace()
            .map(|word| {
                word.to_lowercase()
                    .trim_matches(|c: char| !c.is_alphanumeric())
                    .to_string()
            })
            .filter(|word| word.len() > 2 && !common_words.contains(&word.as_str()))
            .take(10) // Limit to 10 keywords
            .collect()
    }

    fn calculate_query_relevance(&self, content: &str, query: &str) -> f32 {
        // Simple relevance calculation (in a real implementation, use semantic similarity)
        let query_words: Vec<String> = query.split_whitespace().map(|w| w.to_lowercase()).collect();

        let content_lower = content.to_lowercase();
        let matches = query_words
            .iter()
            .filter(|word| content_lower.contains(*word))
            .count();

        if query_words.is_empty() {
            0.0
        } else {
            matches as f32 / query_words.len() as f32
        }
    }

    async fn analyze_user_patterns(&self, interaction: &Interaction) -> Result<(), String> {
        // Analyze user patterns (simplified implementation)
        let pattern_id = format!("user_pattern_{}", chrono::Utc::now().timestamp());

        let pattern = UserPattern {
            pattern_id: pattern_id.clone(),
            pattern_type: PatternType::ApplicationUsage,
            frequency: 1,
            last_occurrence: interaction.timestamp,
            confidence: 0.5,
            description: format!(
                "User interaction pattern: {}",
                interaction.user_input.chars().take(50).collect::<String>()
            ),
        };

        let mut patterns = self.user_patterns.lock().await;
        patterns.insert(pattern_id, pattern);

        Ok(())
    }

    pub async fn get_user_patterns(&self) -> Vec<UserPattern> {
        let patterns = self.user_patterns.lock().await;
        patterns.values().cloned().collect()
    }

    async fn save_to_disk(&self) -> Result<(), String> {
        let store = self.memory_store.lock().await;
        let patterns = self.user_patterns.lock().await;

        let memory_file = Path::new(&self.storage_path).join("memory.json");
        let patterns_file = Path::new(&self.storage_path).join("patterns.json");

        // Save memory entries
        let memory_json = serde_json::to_string_pretty(&*store)
            .map_err(|e| format!("Failed to serialize memory: {e}"))?;
        fs::write(&memory_file, memory_json)
            .await
            .map_err(|e| format!("Failed to write memory file: {e}"))?;

        // Save user patterns
        let patterns_json = serde_json::to_string_pretty(&*patterns)
            .map_err(|e| format!("Failed to serialize patterns: {e}"))?;
        fs::write(&patterns_file, patterns_json)
            .await
            .map_err(|e| format!("Failed to write patterns file: {e}"))?;

        Ok(())
    }

    async fn load_from_disk(&self) -> Result<(), String> {
        let memory_file = Path::new(&self.storage_path).join("memory.json");
        let patterns_file = Path::new(&self.storage_path).join("patterns.json");

        // Load memory entries if file exists
        if memory_file.exists() {
            match fs::read_to_string(&memory_file).await {
                Ok(content) => {
                    match serde_json::from_str::<HashMap<String, MemoryEntry>>(&content) {
                        Ok(loaded_memory) => {
                            let mut store = self.memory_store.lock().await;
                            *store = loaded_memory;
                            info!("Loaded {} memory entries from disk", store.len());
                        }
                        Err(e) => warn!("Failed to parse memory file: {e}"),
                    }
                }
                Err(e) => warn!("Failed to read memory file: {e}"),
            }
        }

        // Load user patterns if file exists
        if patterns_file.exists() {
            match fs::read_to_string(&patterns_file).await {
                Ok(content) => {
                    match serde_json::from_str::<HashMap<String, UserPattern>>(&content) {
                        Ok(loaded_patterns) => {
                            let mut patterns = self.user_patterns.lock().await;
                            *patterns = loaded_patterns;
                            info!("Loaded {} user patterns from disk", patterns.len());
                        }
                        Err(e) => warn!("Failed to parse patterns file: {e}"),
                    }
                }
                Err(e) => warn!("Failed to read patterns file: {e}"),
            }
        }

        Ok(())
    }

    pub async fn get_memory_stats(&self) -> MemoryStats {
        let store = self.memory_store.lock().await;
        let patterns = self.user_patterns.lock().await;

        MemoryStats {
            total_entries: store.len(),
            total_patterns: patterns.len(),
            storage_path: self.storage_path.clone(),
            max_entries: self.max_entries,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct MemoryStats {
    pub total_entries: usize,
    pub total_patterns: usize,
    pub storage_path: String,
    pub max_entries: usize,
}
