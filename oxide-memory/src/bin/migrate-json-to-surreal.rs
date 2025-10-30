use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use oxide_memory::memory::{MemoryEntry, MemoryEntryType, UserPattern};
use oxide_memory::{AgentMemory, AgentType, MemorySource, SurrealBackend};
use serde::de::DeserializeOwned;
use serde_json::{json, Map, Number, Value};

#[derive(Parser, Debug)]
#[command(
    name = "migrate-json-to-surreal",
    about = "Migrate legacy JSON memory files into the SurrealDB backend used by Oxide Pilot."
)]
struct Args {
    /// Directory containing memory.json and patterns.json
    #[arg(long, default_value = "oxide_memory")]
    json_dir: PathBuf,

    /// Path to the SurrealDB RocksDB file (will be created if missing)
    #[arg(long, default_value = "./data/oxide.db")]
    surreal_db: PathBuf,

    /// Only print what would happen without writing to SurrealDB
    #[arg(long, default_value_t = false)]
    dry_run: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    println!(
        "Starting migration from '{}' to '{}'",
        args.json_dir.display(),
        args.surreal_db.display()
    );

    let backend = SurrealBackend::new(&args.surreal_db)
        .await
        .context("Failed to initialize SurrealDB backend")?;

    let memory_entries: HashMap<String, MemoryEntry> =
        load_json(&args.json_dir.join("memory.json"))
            .await?
            .unwrap_or_default();
    let patterns: HashMap<String, UserPattern> = load_json(&args.json_dir.join("patterns.json"))
        .await?
        .unwrap_or_default();

    println!(
        "Found {} memory entries and {} user patterns",
        memory_entries.len(),
        patterns.len()
    );

    let mut migrated_entries = 0usize;
    for (entry_id, entry) in memory_entries {
        let (agent_type, source) = map_entry_type(&entry.entry_type);
        let metadata = build_memory_metadata(&entry_id, &entry);
        let embedding = generate_embedding(&backend, &entry.content).await?;

        if args.dry_run {
            println!("[dry-run] Would migrate entry '{entry_id}' as {agent_type:?} ({source:?})");
            migrated_entries += 1;
            continue;
        }

        let record = AgentMemory {
            agent_type,
            content: entry.content.clone(),
            embedding,
            timestamp: entry.timestamp,
            source,
            metadata: Some(metadata),
        };

        backend
            .insert_agent_memory(record)
            .await
            .map_err(|e| anyhow!("Failed to insert memory entry {entry_id}: {e}"))?;

        migrated_entries += 1;
    }

    let mut migrated_patterns = 0usize;
    for (pattern_id, pattern) in patterns {
        let metadata = build_pattern_metadata(&pattern_id, &pattern);
        let content = format!(
            "{:?} pattern (frequency {}, confidence {:.2}) - {}",
            pattern.pattern_type, pattern.frequency, pattern.confidence, pattern.description
        );
        let embedding = generate_embedding(&backend, &content).await?;

        if args.dry_run {
            println!(
                "[dry-run] Would migrate pattern '{}' last seen {}",
                pattern_id, pattern.last_occurrence
            );
            migrated_patterns += 1;
            continue;
        }

        let record = AgentMemory {
            agent_type: AgentType::Copilot,
            content,
            embedding,
            timestamp: pattern.last_occurrence,
            source: MemorySource::UserQuery,
            metadata: Some(metadata),
        };

        backend
            .insert_agent_memory(record)
            .await
            .map_err(|e| anyhow!("Failed to insert pattern {pattern_id}: {e}"))?;

        migrated_patterns += 1;
    }

    println!(
        "Migration complete: {} memories, {} patterns{}.",
        migrated_entries,
        migrated_patterns,
        if args.dry_run {
            " (dry-run, no changes written)"
        } else {
            ""
        }
    );

    Ok(())
}

async fn load_json<T>(path: &PathBuf) -> Result<Option<T>>
where
    T: DeserializeOwned,
{
    if !path.exists() {
        return Ok(None);
    }

    let contents = tokio::fs::read_to_string(path)
        .await
        .with_context(|| format!("Failed to read {}", path.display()))?;
    let parsed = serde_json::from_str::<T>(&contents)
        .with_context(|| format!("Failed to parse {}", path.display()))?;
    Ok(Some(parsed))
}

fn map_entry_type(entry_type: &MemoryEntryType) -> (AgentType, MemorySource) {
    match entry_type {
        MemoryEntryType::SystemEvent | MemoryEntryType::ThreatDetection => {
            (AgentType::Guardian, MemorySource::SystemLog)
        }
        MemoryEntryType::SystemOptimization => {
            (AgentType::Guardian, MemorySource::PerformanceAnalysis)
        }
        MemoryEntryType::UserInteraction | MemoryEntryType::KnowledgeBase => {
            (AgentType::Copilot, MemorySource::UserQuery)
        }
        MemoryEntryType::UserPattern => (AgentType::Copilot, MemorySource::UserQuery),
    }
}

fn build_memory_metadata(entry_id: &str, entry: &MemoryEntry) -> Value {
    let mut map = Map::new();
    map.insert("legacy_id".to_string(), Value::String(entry_id.to_string()));
    map.insert(
        "entry_type".to_string(),
        Value::String(format!("{:?}", entry.entry_type)),
    );
    map.insert(
        "relevance_score".to_string(),
        number_from_f64(entry.relevance_score as f64),
    );

    if !entry.tags.is_empty() {
        map.insert(
            "tags".to_string(),
            Value::Array(
                entry
                    .tags
                    .iter()
                    .map(|t| Value::String(t.clone()))
                    .collect(),
            ),
        );
    }

    if !entry.metadata.is_empty() {
        map.insert(
            "metadata".to_string(),
            Value::Object(
                entry
                    .metadata
                    .iter()
                    .map(|(k, v)| (k.clone(), Value::String(v.clone())))
                    .collect(),
            ),
        );
    }

    Value::Object(map)
}

fn build_pattern_metadata(pattern_id: &str, pattern: &UserPattern) -> Value {
    json!({
        "pattern_id": pattern_id,
        "pattern_type": format!("{:?}", pattern.pattern_type),
        "frequency": pattern.frequency,
        "confidence": pattern.confidence,
        "last_occurrence": pattern.last_occurrence,
    })
}

async fn generate_embedding(backend: &SurrealBackend, text: &str) -> Result<Vec<f64>> {
    match backend.embed_text(text).await {
        Ok(vector) => Ok(vector),
        Err(err) => {
            eprintln!("Warning: embedding generation failed ({err}). Falling back to zero vector.");
            Ok(vec![0.0; backend.embedding_dimension()])
        }
    }
}

fn number_from_f64(value: f64) -> Value {
    Number::from_f64(value)
        .map(Value::Number)
        .unwrap_or_else(|| Value::Number(Number::from(0)))
}
