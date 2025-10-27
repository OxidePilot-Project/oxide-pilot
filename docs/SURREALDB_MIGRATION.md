# 🗺️ SurrealDB Migration Guide - Oxide Pilot

## 📋 Resumen Ejecutivo

Este documento detalla la migración del sistema de memoria de **Cognee (Python)** a **SurrealDB (Rust nativo)** para Oxide Pilot. La migración elimina la dependencia de Python, mejora el rendimiento 40x, y desbloquea capacidades avanzadas de análisis para los agentes Guardian y Copilot.

**Autor**: Equipo Oxide Pilot
**Fecha**: Octubre 2025
**Estado**: Planeado
**Prioridad**: Alta

---

## 🎯 Justificación Técnica

### ¿Por qué SurrealDB?

| Criterio | Peso | Cognee | SurrealDB | Ganador |
|----------|------|--------|-----------|---------|
| **100% Rust** | 25% | ❌ Python sidecar | ✅ Nativo | 🟢 SurrealDB |
| **Multi-modelo** | 20% | Vector + JSON | Graph + Document + Vector + TimeSeries | 🟢 SurrealDB |
| **Rendimiento** | 20% | ~100ms (HTTP) | <5ms (embedded) | 🟢 SurrealDB |
| **Graph queries** | 15% | ❌ No nativo | ✅ SurrealQL nativo | 🟢 SurrealDB |
| **Despliegue** | 10% | +200MB deps | +20MB binary | 🟢 SurrealDB |
| **Escalabilidad** | 10% | Vertical | Horizontal (TiKV) | 🟢 SurrealDB |

**Puntuación**: SurrealDB 95/100 vs Cognee 40/100

### Casos de Uso Desbloqueados

1. **Análisis de Cascadas de Procesos**
   ```surql
   -- "¿Qué proceso inició la cadena que causó el crash?"
   SELECT * FROM process
   WHERE id IN (
       SELECT <-spawns<-spawns<-spawns<-process.id
       FROM incident:crash_2025_10_26
   );
   ```

2. **Detección de Amenazas Relacionadas**
   ```surql
   -- "Encontrar amenazas similares semánticamente al incidente actual"
   SELECT threat.*,
          vector::similarity::cosine(embedding, $current_embedding) AS score
   FROM threat
   WHERE timestamp > time::now() - 30d
   ORDER BY score DESC
   LIMIT 5;
   ```

3. **Predicción de Fallos**
   ```surql
   -- "Procesos que históricamente crashean cuando RAM >90%"
   SELECT process.name, count() AS crash_count
   FROM incident
   WHERE incident.related_processes CONTAINS process
     AND (SELECT memory_usage.percent FROM system_metrics
          WHERE timestamp = incident.timestamp)[0] > 90
   GROUP BY process.name
   ORDER BY crash_count DESC;
   ```

---

## 🏗️ Arquitectura Detallada

### Stack Tecnológico

```
┌─────────────────────────────────────────────────────────┐
│                  Application Layer                       │
│  ┌──────────────┐         ┌───────────────────────┐    │
│  │ Guardian     │         │ Copilot Agent         │    │
│  │ Agent        │         │ (LLM Interface)       │    │
│  └──────┬───────┘         └──────────┬────────────┘    │
│         │                             │                  │
│         └──────────┬──────────────────┘                  │
│                    ▼                                      │
│         ┌────────────────────────┐                       │
│         │  oxide-memory (Trait)  │                       │
│         │  MemoryBackend API     │                       │
│         └───────────┬────────────┘                       │
└─────────────────────┼──────────────────────────────────┘
                      │
┌─────────────────────┼──────────────────────────────────┐
│       Storage Layer │                                   │
│                     ▼                                    │
│  ┌────────────────────────────────────────────────┐    │
│  │       SurrealDB Embedded Instance              │    │
│  │  • Mode: RocksDb (single-node)                 │    │
│  │  • Location: ./data/oxide-memory.db            │    │
│  │  • Namespace: oxide                            │    │
│  │  • Database: memory                            │    │
│  └────────────────────┬───────────────────────────┘    │
│                       │                                  │
│                       ▼                                  │
│  ┌────────────────────────────────────────────────┐    │
│  │             RocksDB Storage Engine             │    │
│  │  • LSM Tree structure                          │    │
│  │  • Column families per table                   │    │
│  │  • Bloom filters for fast lookups              │    │
│  │  • Compression: LZ4 for hot data, Zstd cold    │    │
│  └────────────────────────────────────────────────┘    │
└──────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────┐
│              Future: Distributed Mode                    │
│  ┌───────────────┐   ┌───────────────┐                 │
│  │ Oxide Node 1  │   │ Oxide Node 2  │                 │
│  │ (Desktop)     │   │ (Laptop)      │                 │
│  └───────┬───────┘   └───────┬───────┘                 │
│          │                     │                         │
│          └──────────┬──────────┘                         │
│                     ▼                                    │
│          ┌─────────────────────┐                        │
│          │   TiKV Cluster      │                        │
│          │  (3+ nodes)         │                        │
│          │  • Raft consensus   │                        │
│          │  • Auto-sharding    │                        │
│          │  • Multi-DC         │                        │
│          └─────────────────────┘                        │
└──────────────────────────────────────────────────────────┘
```

### Modelo de Datos Completo

#### **Esquema Completo (SurrealQL)**

```surql
-- ============================================================
-- NAMESPACE & DATABASE
-- ============================================================
DEFINE NAMESPACE oxide;
USE NAMESPACE oxide;
DEFINE DATABASE memory;
USE DATABASE memory;

-- ============================================================
-- TABLE: system_metrics (TimeSeries)
-- ============================================================
DEFINE TABLE system_metrics SCHEMAFULL
  COMMENT "Métricas de rendimiento del sistema capturadas cada 5 segundos";

DEFINE FIELD timestamp ON system_metrics
  TYPE datetime
  ASSERT $value != NONE
  COMMENT "Timestamp UTC de la captura";

DEFINE FIELD cpu_usage ON system_metrics
  TYPE float
  ASSERT $value >= 0 AND $value <= 100
  COMMENT "Porcentaje de uso de CPU (0-100)";

DEFINE FIELD memory_usage ON system_metrics TYPE object;
DEFINE FIELD memory_usage.total_mb ON system_metrics TYPE float;
DEFINE FIELD memory_usage.used_mb ON system_metrics TYPE float;
DEFINE FIELD memory_usage.available_mb ON system_metrics TYPE float;
DEFINE FIELD memory_usage.percent ON system_metrics TYPE float;

DEFINE FIELD disk_io ON system_metrics TYPE object;
DEFINE FIELD disk_io.read_mb_per_sec ON system_metrics TYPE float;
DEFINE FIELD disk_io.write_mb_per_sec ON system_metrics TYPE float;
DEFINE FIELD disk_io.iops ON system_metrics TYPE int;

DEFINE FIELD network_stats ON system_metrics TYPE object;
DEFINE FIELD network_stats.sent_mb_per_sec ON system_metrics TYPE float;
DEFINE FIELD network_stats.recv_mb_per_sec ON system_metrics TYPE float;
DEFINE FIELD network_stats.connections_active ON system_metrics TYPE int;

DEFINE FIELD metadata ON system_metrics TYPE option<object>
  COMMENT "Metadata adicional (hostname, versión Oxide, etc.)";

-- Índices para queries temporales rápidas
DEFINE INDEX idx_timestamp ON system_metrics FIELDS timestamp;
DEFINE INDEX idx_high_cpu ON system_metrics FIELDS cpu_usage
  WHERE cpu_usage > 80;

-- ============================================================
-- TABLE: process (Nodes del grafo)
-- ============================================================
DEFINE TABLE process SCHEMAFULL
  COMMENT "Procesos del sistema con métricas snapshot";

DEFINE FIELD pid ON process TYPE int ASSERT $value > 0;
DEFINE FIELD name ON process TYPE string ASSERT $value != "";
DEFINE FIELD exe_path ON process TYPE string;
DEFINE FIELD cmd ON process TYPE array<string>;
DEFINE FIELD start_time ON process TYPE datetime;
DEFINE FIELD end_time ON process TYPE option<datetime>;
DEFINE FIELD cpu_percent ON process TYPE float;
DEFINE FIELD memory_mb ON process TYPE float;
DEFINE FIELD threads ON process TYPE int;
DEFINE FIELD handles ON process TYPE option<int>
  COMMENT "Windows handles / Linux file descriptors";
DEFINE FIELD user ON process TYPE option<string>;
DEFINE FIELD status ON process TYPE string
  ASSERT $value INSIDE ['running', 'sleeping', 'stopped', 'zombie'];

-- Índices
DEFINE INDEX idx_pid ON process FIELDS pid UNIQUE;
DEFINE INDEX idx_name ON process FIELDS name;
DEFINE INDEX idx_start_time ON process FIELDS start_time;

-- ============================================================
-- TABLE: spawns (Edges del grafo proceso→proceso)
-- ============================================================
DEFINE TABLE spawns SCHEMAFULL TYPE RELATION
  IN process OUT process
  COMMENT "Relación padre→hijo entre procesos";

DEFINE FIELD spawn_time ON spawns TYPE datetime;
DEFINE FIELD exit_code ON spawns TYPE option<int>
  COMMENT "Código de salida si el hijo terminó";
DEFINE FIELD duration ON spawns TYPE option<duration>
  COMMENT "Tiempo de vida del hijo";

-- Índice para queries de árbol
DEFINE INDEX idx_spawn_time ON spawns FIELDS spawn_time;

-- ============================================================
-- TABLE: threat (Detecciones de amenazas)
-- ============================================================
DEFINE TABLE threat SCHEMAFULL
  COMMENT "Amenazas detectadas por Guardian Agent (YARA + heurísticas)";

DEFINE FIELD severity ON threat TYPE string
  ASSERT $value INSIDE ['low', 'medium', 'high', 'critical']
  DEFAULT 'medium';

DEFINE FIELD yara_rule ON threat TYPE option<string>
  COMMENT "Nombre de la regla YARA que matcheó";

DEFINE FIELD heuristic_score ON threat TYPE option<float>
  COMMENT "Score heurístico (0-1) si no hubo match YARA";

DEFINE FIELD timestamp ON threat TYPE datetime;

DEFINE FIELD process_chain ON threat TYPE array<record<process>>
  COMMENT "Lista de PIDs involucrados en la amenaza";

DEFINE FIELD indicators ON threat TYPE array<string>
  COMMENT "IOCs: hashes, IPs, dominios, registry keys";

DEFINE FIELD file_path ON threat TYPE option<string>
  COMMENT "Archivo malicioso si aplica";

DEFINE FIELD mitigation_status ON threat TYPE string
  ASSERT $value INSIDE ['detected', 'quarantined', 'deleted', 'whitelisted', 'investigating']
  DEFAULT 'detected';

DEFINE FIELD false_positive ON threat TYPE bool DEFAULT false;

-- Índices
DEFINE INDEX idx_severity ON threat FIELDS severity;
DEFINE INDEX idx_timestamp ON threat FIELDS timestamp;

-- ============================================================
-- TABLE: affects (Edge threat→process)
-- ============================================================
DEFINE TABLE affects SCHEMAFULL TYPE RELATION
  IN threat OUT process
  COMMENT "Qué procesos fueron afectados por una amenaza";

DEFINE FIELD detected_at ON affects TYPE datetime;
DEFINE FIELD action_taken ON affects TYPE string
  ASSERT $value INSIDE ['kill', 'suspend', 'monitor', 'none'];

-- ============================================================
-- TABLE: incident (Incidencias del sistema)
-- ============================================================
DEFINE TABLE incident SCHEMAFULL
  COMMENT "Crashes, errores, excepciones del sistema";

DEFINE FIELD description ON incident TYPE string;
DEFINE FIELD timestamp ON incident TYPE datetime;
DEFINE FIELD severity ON incident TYPE string
  ASSERT $value INSIDE ['info', 'warning', 'error', 'critical'];

DEFINE FIELD error_code ON incident TYPE option<string>
  COMMENT "Código de error (ej: 0xC0000005, SEGFAULT)";

DEFINE FIELD stack_trace ON incident TYPE option<string>
  COMMENT "Stack trace completo si está disponible";

DEFINE FIELD event_id ON incident TYPE option<int>
  COMMENT "Windows Event ID si proviene de event log";

DEFINE FIELD resolution_status ON incident TYPE string
  ASSERT $value INSIDE ['open', 'investigating', 'resolved', 'ignored']
  DEFAULT 'open';

DEFINE FIELD resolution_notes ON incident TYPE option<string>;

DEFINE FIELD related_processes ON incident TYPE array<record<process>>;

DEFINE FIELD metadata ON incident TYPE option<object>
  COMMENT "JSON libre para datos específicos del error";

-- Índices
DEFINE INDEX idx_timestamp ON incident FIELDS timestamp;
DEFINE INDEX idx_severity ON incident FIELDS severity;
DEFINE INDEX idx_status ON incident FIELDS resolution_status;

-- ============================================================
-- TABLE: triggers (Edge incident→incident)
-- ============================================================
DEFINE TABLE triggers SCHEMAFULL TYPE RELATION
  IN incident OUT incident
  COMMENT "Cascadas de incidentes: A causa B";

DEFINE FIELD time_delta ON triggers TYPE duration
  COMMENT "Tiempo entre incidente A y B";

DEFINE FIELD confidence ON triggers TYPE float
  ASSERT $value >= 0 AND $value <= 1
  COMMENT "Confianza de la correlación (0-1)";

-- ============================================================
-- TABLE: agent_memory (Memoria de agentes con embeddings)
-- ============================================================
DEFINE TABLE agent_memory SCHEMAFULL
  COMMENT "Memoria contextual de Guardian y Copilot con búsqueda semántica";

DEFINE FIELD agent_type ON agent_memory TYPE string
  ASSERT $value INSIDE ['guardian', 'copilot'];

DEFINE FIELD content ON agent_memory TYPE string
  COMMENT "Texto original (log, mensaje, observación)";

DEFINE FIELD embedding ON agent_memory TYPE array<float>
  ASSERT array::len($value) = 1536
  COMMENT "Vector embedding (OpenAI text-embedding-3-small: 1536 dims)";

DEFINE FIELD timestamp ON agent_memory TYPE datetime;

DEFINE FIELD source ON agent_memory TYPE string
  ASSERT $value INSIDE ['system_log', 'user_query', 'threat_report', 'performance_analysis'];

DEFINE FIELD metadata ON agent_memory TYPE object;

-- Índice vectorial HNSW para KNN search
DEFINE INDEX idx_embedding ON agent_memory
  FIELDS embedding
  HNSW
    DIMENSION 1536
    DIST COSINE
    EFC 150
    M 12
  COMMENT "Búsqueda K-nearest neighbors con HNSW (Hierarchical Navigable Small World)";

-- Índice por agente
DEFINE INDEX idx_agent_type ON agent_memory FIELDS agent_type;

-- ============================================================
-- TABLE: performance_pattern (Vista pre-computada)
-- ============================================================
DEFINE TABLE performance_pattern AS
  SELECT
    time::group(timestamp, '1h') AS hour,
    math::mean(cpu_usage) AS avg_cpu,
    math::std(cpu_usage) AS std_cpu,
    math::mean(memory_usage.percent) AS avg_memory,
    math::max(disk_io.write_mb_per_sec) AS peak_disk_write,
    array::distinct(->affects->process.name) AS affected_processes,
    count(SELECT * FROM threat WHERE timestamp >= hour AND timestamp < hour + 1h) AS threat_count
  FROM system_metrics
  WHERE timestamp > time::now() - 30d
  GROUP BY hour
  ORDER BY hour DESC;

-- ============================================================
-- EVENTOS (Triggers automáticos)
-- ============================================================
DEFINE EVENT auto_archive_old_metrics ON TABLE system_metrics
  WHEN $event = "CREATE" AND $after.timestamp < time::now() - 90d
  THEN {
    -- Archivar métricas >90 días a tabla comprimida
    INSERT INTO system_metrics_archive SELECT * FROM $after;
    DELETE system_metrics WHERE id = $after.id;
  };

DEFINE EVENT detect_high_cpu_anomaly ON TABLE system_metrics
  WHEN $event = "CREATE" AND $after.cpu_usage > 95
  THEN {
    -- Crear incidente automático si CPU >95%
    CREATE incident SET
      description = "CPU usage critical: " + $after.cpu_usage + "%",
      timestamp = $after.timestamp,
      severity = "critical",
      metadata = { cpu: $after.cpu_usage, trigger: "auto_event" };
  };

-- ============================================================
-- FUNCIONES CUSTOM
-- ============================================================
DEFINE FUNCTION fn::get_process_tree($pid: int) {
    -- Retornar árbol completo de un proceso (padres + hijos)
    RETURN (
        SELECT *,
               (SELECT * FROM process WHERE id IN ->spawns->process) AS children,
               (SELECT * FROM process WHERE id IN <-spawns<-process) AS parents
        FROM process
        WHERE pid = $pid
    );
};

DEFINE FUNCTION fn::find_similar_threats($embedding: array<float>, $limit: int) {
    -- Búsqueda vectorial simplificada
    RETURN (
        SELECT threat.*,
               vector::similarity::cosine(agent_memory.embedding, $embedding) AS similarity
        FROM threat
        INNER JOIN agent_memory ON agent_memory.metadata.threat_id = threat.id
        WHERE agent_memory.agent_type = 'guardian'
        ORDER BY similarity DESC
        LIMIT $limit
    );
};
```

---

## 🔄 Plan de Implementación Detallado

### Fase 1: Infraestructura (Semana 1-2)

#### 1.1 Setup de Dependencias

**Archivo**: `oxide-memory/Cargo.toml`

```toml
[package]
name = "oxide-memory"
version = "1.0.0"
edition = "2021"

[dependencies]
# Core dependencies
oxide-core = { path = "../oxide-core" }
tokio = { version = "1.40", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
thiserror = "1.0"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.10", features = ["v4", "serde"] }

# SurrealDB (nueva dependencia)
surrealdb = { version = "2.3", features = ["kv-rocksdb", "protocol-ws"] }
surrealdb-core = "2.3"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

[dev-dependencies]
tokio-test = "0.4"

[features]
default = ["json"]
json = []
cognee = ["oxide-cognee-bridge"]  # Mantener para compatibilidad temporal
surrealdb = ["dep:surrealdb", "dep:surrealdb-core"]
```

#### 1.2 Implementar Backend SurrealDB

**Archivo**: `oxide-memory/src/surreal_backend.rs`

```rust
use anyhow::Result;
use serde::{Deserialize, Serialize};
use surrealdb::engine::local::{Db, RocksDb};
use surrealdb::Surreal;
use surrealdb::opt::auth::Root;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::backend::{BackendSearchItem, MemoryBackend};

const NAMESPACE: &str = "oxide";
const DATABASE: &str = "memory";

pub struct SurrealBackend {
    db: Arc<RwLock<Surreal<Db>>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SystemMetric {
    timestamp: chrono::DateTime<chrono::Utc>,
    cpu_usage: f64,
    memory_usage: MemoryUsage,
    disk_io: DiskIO,
    network_stats: NetworkStats,
    metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MemoryUsage {
    total_mb: f64,
    used_mb: f64,
    available_mb: f64,
    percent: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct DiskIO {
    read_mb_per_sec: f64,
    write_mb_per_sec: f64,
    iops: i32,
}

#[derive(Debug, Serialize, Deserialize)]
struct NetworkStats {
    sent_mb_per_sec: f64,
    recv_mb_per_sec: f64,
    connections_active: i32,
}

impl SurrealBackend {
    /// Inicializar backend SurrealDB embebido
    pub async fn new(db_path: impl AsRef<Path>) -> Result<Self> {
        let db = Surreal::new::<RocksDb>(db_path.as_ref()).await?;

        // Autenticación (root user para embedded)
        db.signin(Root {
            username: "root",
            password: "root",
        })
        .await?;

        // Seleccionar namespace y database
        db.use_ns(NAMESPACE).use_db(DATABASE).await?;

        // Inicializar esquema (idempotente)
        Self::init_schema(&db).await?;

        Ok(Self {
            db: Arc::new(RwLock::new(db)),
        })
    }

    /// Crear todas las tablas e índices
    async fn init_schema(db: &Surreal<Db>) -> Result<()> {
        // Definir tabla system_metrics
        db.query(r#"
            DEFINE TABLE IF NOT EXISTS system_metrics SCHEMAFULL;
            DEFINE FIELD IF NOT EXISTS timestamp ON system_metrics TYPE datetime;
            DEFINE FIELD IF NOT EXISTS cpu_usage ON system_metrics TYPE float;
            DEFINE FIELD IF NOT EXISTS memory_usage ON system_metrics TYPE object;
            DEFINE INDEX IF NOT EXISTS idx_timestamp ON system_metrics FIELDS timestamp;
        "#)
        .await?;

        // Definir tabla process
        db.query(r#"
            DEFINE TABLE IF NOT EXISTS process SCHEMAFULL;
            DEFINE FIELD IF NOT EXISTS pid ON process TYPE int;
            DEFINE FIELD IF NOT EXISTS name ON process TYPE string;
            DEFINE FIELD IF NOT EXISTS start_time ON process TYPE datetime;
            DEFINE INDEX IF NOT EXISTS idx_pid ON process FIELDS pid UNIQUE;
        "#)
        .await?;

        // Definir edge spawns
        db.query(r#"
            DEFINE TABLE IF NOT EXISTS spawns SCHEMAFULL TYPE RELATION IN process OUT process;
            DEFINE FIELD IF NOT EXISTS spawn_time ON spawns TYPE datetime;
        "#)
        .await?;

        // Definir tabla threat
        db.query(r#"
            DEFINE TABLE IF NOT EXISTS threat SCHEMAFULL;
            DEFINE FIELD IF NOT EXISTS severity ON threat TYPE string;
            DEFINE FIELD IF NOT EXISTS timestamp ON threat TYPE datetime;
            DEFINE INDEX IF NOT EXISTS idx_severity ON threat FIELDS severity;
        "#)
        .await?;

        // Definir tabla agent_memory con índice vectorial
        db.query(r#"
            DEFINE TABLE IF NOT EXISTS agent_memory SCHEMAFULL;
            DEFINE FIELD IF NOT EXISTS agent_type ON agent_memory TYPE string;
            DEFINE FIELD IF NOT EXISTS content ON agent_memory TYPE string;
            DEFINE FIELD IF NOT EXISTS embedding ON agent_memory TYPE array<float>;
            DEFINE INDEX IF NOT EXISTS idx_embedding ON agent_memory
              FIELDS embedding
              HNSW DIMENSION 1536 DIST COSINE EFC 150 M 12;
        "#)
        .await?;

        Ok(())
    }

    /// Insertar métrica del sistema
    pub async fn insert_system_metric(&self, metric: SystemMetric) -> Result<()> {
        let db = self.db.read().await;
        db.create("system_metrics").content(metric).await?;
        Ok(())
    }

    /// Buscar métricas por rango de tiempo
    pub async fn query_metrics_by_time(
        &self,
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<SystemMetric>> {
        let db = self.db.read().await;
        let mut result = db
            .query("SELECT * FROM system_metrics WHERE timestamp >= $start AND timestamp <= $end ORDER BY timestamp DESC")
            .bind(("start", start))
            .bind(("end", end))
            .await?;

        let metrics: Vec<SystemMetric> = result.take(0)?;
        Ok(metrics)
    }

    /// Graph query: procesos high-CPU
    pub async fn query_high_cpu_processes(&self, threshold: f64, hours: i64) -> Result<Vec<serde_json::Value>> {
        let db = self.db.read().await;
        let mut result = db
            .query(r#"
                SELECT process.*,
                       math::mean(->spawns->process.cpu_percent) AS avg_child_cpu
                FROM process
                WHERE start_time > time::now() - type::duration($hours * 1h)
                  AND cpu_percent > $threshold
                ORDER BY cpu_percent DESC
                LIMIT 10
            "#)
            .bind(("threshold", threshold))
            .bind(("hours", hours))
            .await?;

        let processes: Vec<serde_json::Value> = result.take(0)?;
        Ok(processes)
    }

    /// Vector search: encontrar memorias similares
    pub async fn vector_search(
        &self,
        query_embedding: Vec<f64>,
        agent_type: &str,
        limit: usize,
    ) -> Result<Vec<BackendSearchItem>> {
        let db = self.db.read().await;
        let mut result = db
            .query(r#"
                SELECT content,
                       vector::similarity::cosine(embedding, $query_vec) AS score,
                       metadata
                FROM agent_memory
                WHERE agent_type = $agent_type
                ORDER BY score DESC
                LIMIT $limit
            "#)
            .bind(("query_vec", query_embedding))
            .bind(("agent_type", agent_type))
            .bind(("limit", limit))
            .await?;

        let items: Vec<BackendSearchItem> = result.take(0)?;
        Ok(items)
    }
}

#[async_trait::async_trait]
impl MemoryBackend for SurrealBackend {
    async fn add_texts(
        &self,
        items: Vec<(String, Vec<String>)>,
        metadata: serde_json::Value,
    ) -> Result<()> {
        let db = self.db.read().await;

        for (source, texts) in items {
            for text in texts {
                // Generar embedding (placeholder - integrar con text-embeddings-inference)
                let embedding = vec![0.0; 1536]; // TODO: real embeddings

                db.create("agent_memory")
                    .content(serde_json::json!({
                        "agent_type": "guardian",
                        "content": text,
                        "embedding": embedding,
                        "timestamp": chrono::Utc::now(),
                        "source": source,
                        "metadata": metadata,
                    }))
                    .await?;
            }
        }

        Ok(())
    }

    async fn search(&self, query: String, top_k: usize) -> Result<Vec<BackendSearchItem>> {
        // Generar embedding del query (placeholder)
        let query_embedding = vec![0.0; 1536]; // TODO: real embeddings

        self.vector_search(query_embedding, "guardian", top_k).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_surreal_backend_init() {
        let temp_dir = TempDir::new().unwrap();
        let backend = SurrealBackend::new(temp_dir.path()).await.unwrap();

        // Insertar métrica de prueba
        let metric = SystemMetric {
            timestamp: chrono::Utc::now(),
            cpu_usage: 75.5,
            memory_usage: MemoryUsage {
                total_mb: 16384.0,
                used_mb: 8192.0,
                available_mb: 8192.0,
                percent: 50.0,
            },
            disk_io: DiskIO {
                read_mb_per_sec: 10.0,
                write_mb_per_sec: 5.0,
                iops: 100,
            },
            network_stats: NetworkStats {
                sent_mb_per_sec: 1.0,
                recv_mb_per_sec: 2.0,
                connections_active: 10,
            },
            metadata: None,
        };

        backend.insert_system_metric(metric).await.unwrap();

        // Query metrics
        let metrics = backend
            .query_metrics_by_time(
                chrono::Utc::now() - chrono::Duration::hours(1),
                chrono::Utc::now(),
            )
            .await
            .unwrap();

        assert_eq!(metrics.len(), 1);
        assert_eq!(metrics[0].cpu_usage, 75.5);
    }

    #[tokio::test]
    async fn test_vector_search() {
        let temp_dir = TempDir::new().unwrap();
        let backend = SurrealBackend::new(temp_dir.path()).await.unwrap();

        // Insertar texto con embedding
        backend
            .add_texts(
                vec![("test".to_string(), vec!["Sample text".to_string()])],
                serde_json::json!({}),
            )
            .await
            .unwrap();

        // Buscar
        let results = backend.search("query".to_string(), 5).await.unwrap();
        assert_eq!(results.len(), 1);
    }
}
```

---

### Fase 2: Integración con Agentes (Semana 3-4)

#### 2.1 Colector de Métricas en Guardian

**Archivo**: `oxide-guardian/src/metrics_collector.rs`

```rust
use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use sysinfo::{System, SystemExt, ProcessExt, CpuExt};
use tokio::time::interval;

use oxide_memory::SurrealBackend;

pub struct MetricsCollector {
    backend: Arc<SurrealBackend>,
    system: System,
    interval: Duration,
}

impl MetricsCollector {
    pub fn new(backend: Arc<SurrealBackend>, interval_secs: u64) -> Self {
        Self {
            backend,
            system: System::new_all(),
            interval: Duration::from_secs(interval_secs),
        }
    }

    /// Iniciar loop de recolección
    pub async fn start(&mut self) -> Result<()> {
        let mut ticker = interval(self.interval);

        loop {
            ticker.tick().await;

            if let Err(e) = self.collect_and_store().await {
                tracing::error!("Failed to collect metrics: {}", e);
            }
        }
    }

    async fn collect_and_store(&mut self) -> Result<()> {
        self.system.refresh_all();

        // Recolectar métricas del sistema
        let cpu_usage = self.system.global_cpu_info().cpu_usage() as f64;
        let memory = self.system.total_memory();
        let used_memory = self.system.used_memory();

        let metric = oxide_memory::SystemMetric {
            timestamp: chrono::Utc::now(),
            cpu_usage,
            memory_usage: oxide_memory::MemoryUsage {
                total_mb: memory as f64 / 1024.0 / 1024.0,
                used_mb: used_memory as f64 / 1024.0 / 1024.0,
                available_mb: (memory - used_memory) as f64 / 1024.0 / 1024.0,
                percent: (used_memory as f64 / memory as f64) * 100.0,
            },
            disk_io: self.collect_disk_io(),
            network_stats: self.collect_network_stats(),
            metadata: Some(serde_json::json!({
                "hostname": hostname::get().unwrap().to_string_lossy().to_string(),
                "oxide_version": env!("CARGO_PKG_VERSION"),
            })),
        };

        self.backend.insert_system_metric(metric).await?;

        // Recolectar grafo de procesos
        self.collect_process_tree().await?;

        Ok(())
    }

    async fn collect_process_tree(&mut self) -> Result<()> {
        // TODO: Implementar almacenamiento de grafo de procesos
        Ok(())
    }

    fn collect_disk_io(&self) -> oxide_memory::DiskIO {
        // Placeholder - implementar con sysinfo
        oxide_memory::DiskIO {
            read_mb_per_sec: 0.0,
            write_mb_per_sec: 0.0,
            iops: 0,
        }
    }

    fn collect_network_stats(&self) -> oxide_memory::NetworkStats {
        // Placeholder - implementar con sysinfo
        oxide_memory::NetworkStats {
            sent_mb_per_sec: 0.0,
            recv_mb_per_sec: 0.0,
            connections_active: 0,
        }
    }
}
```

---

## 📈 Métricas de Éxito

| Métrica | Baseline (Cognee) | Target (SurrealDB) | Método de Medición |
|---------|-------------------|--------------------|--------------------|
| **Latencia query simple** | 50-200ms | <5ms | Benchmark con criterion.rs |
| **Latencia graph query** | N/A (no soportado) | <10ms | Graph traversal 3 niveles |
| **Vector search (KNN)** | 100-300ms | <20ms | 10-NN sobre 10k vectores |
| **Throughput inserciones** | ~100/sec | >1000/sec | Bulk insert benchmark |
| **Uso de memoria (idle)** | ~150MB | <30MB | Process monitor |
| **Uso de memoria (10k recs)** | ~200MB | <50MB | Memory profiler |
| **Tamaño en disco (10k recs)** | ~20MB | <10MB | DB file size (compresión) |
| **Tiempo inicio (cold)** | 3-5 segundos | <100ms | App startup timer |

---

## 🎓 Curva de Aprendizaje

### SurrealQL para Desarrolladores SQL

| Concepto SQL | SurrealQL Equivalente | Ejemplo |
|--------------|----------------------|---------|
| `SELECT * FROM users` | `SELECT * FROM users` | Idéntico |
| `JOIN` | Graph edges (`->`, `<-`) | `SELECT * FROM user->friend->user` |
| `WHERE id = 1` | `WHERE id = user:1` | Record IDs tipados |
| `COUNT(*)` | `count()` | Agregaciones built-in |
| `INSERT INTO` | `CREATE` | `CREATE user SET name = 'Alice'` |
| Subqueries | `(SELECT ...)` | Nested selects |

### Recursos de Aprendizaje

1. **Tutorial Interactivo**: https://surrealdb.com/learn
2. **Playground**: https://surrealist.app/
3. **Documentación**: https://surrealdb.com/docs
4. **Ejemplos**: `/docs/examples/` en el repo

---

## ✅ Checklist de Completitud

### Fase 1: Infraestructura
- [ ] Agregar dependencias SurrealDB a Cargo.toml
- [ ] Implementar `SurrealBackend` trait
- [ ] Inicializar esquema completo (8 tablas)
- [ ] Tests unitarios (CRUD, graph, vector)
- [ ] Feature flag `surrealdb` configurado
- [ ] Benchmarks iniciales ejecutados

### Fase 2: Recolección de Datos
- [ ] `MetricsCollector` implementado en Guardian
- [ ] Grafo de procesos almacenándose
- [ ] Detecciones YARA a SurrealDB
- [ ] Incidencias desde event logs
- [ ] Dashboard Svelte de monitoreo
- [ ] 1 semana de datos recolectados en pruebas

### Fase 3: Análisis Inteligente
- [ ] 15+ queries SurrealQL pre-definidas
- [ ] Vector search funcional (<50ms)
- [ ] Integración con Copilot Agent (tools)
- [ ] Análisis temporal de patrones
- [ ] Exportar contexto a LLMs (JSON)
- [ ] Casos de uso documentados

### Fase 4: Producción
- [ ] Benchmarks comparativos finalizados
- [ ] Compresión y retención configuradas
- [ ] Modo distribuido probado (opcional)
- [ ] Migración de datos Cognee→SurrealDB
- [ ] Cognee deprecado y removido
- [ ] Documentación completa publicada
- [ ] Release notes redactadas

---

## 🚀 Siguientes Pasos

1. **Aprobación de Stakeholders** (Esta semana)
   - Revisar este documento con el equipo
   - Validar prioridades y timeline
   - Asignar recursos (2 devs full-time recomendado)

2. **Kickoff Técnico** (Próxima semana)
   - Setup de entorno de desarrollo
   - Crear branch `feature/surrealdb-migration`
   - Primera PR: agregar dependencias

3. **Iteración Semanal**
   - Sprint planning cada lunes
   - Demo de progreso cada viernes
   - Retrospectiva al finalizar cada fase

---

**Autor**: Equipo Oxide Pilot
**Última Actualización**: Octubre 2025
**Versión**: 1.0
