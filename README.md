<!-- English README optimized for clarity and grant applications -->
# Oxide Pilot

Oxide Pilot is an agentic system assistant for secure system automation, monitoring and conversational assistance. It combines a Rust/Tauri backend with a Svelte frontend and integrates local system telemetry, a dual-agent architecture (Guardian + Copilot), a configurable RPA engine, and multi-LLM support.

Status: Active development — production-grade features implemented; integration and polish ongoing.

Core technologies: Rust, Tauri, Svelte, SurrealDB (optional), Playwright for E2E tests, and support for cloud LLMs (Vertex AI, OpenAI, Qwen).

Key highlights

- Guardian agent: continuous monitoring, EDR-style threat detection, process and resource analytics
- Copilot agent: conversational, voice-enabled assistant with function-calling tools and RPA controls
- Secure RPA: granular permission model, auditing and rollback for automated actions
- Persistent memory: optional SurrealDB backend for time-series, graph and vector storage
- Multi-LLM integration: usage of Vertex AI Gemini, OpenAI, Qwen and local models where available

Why this README was rewritten

This file has been translated and reorganized into English to improve clarity for contributors and for grant/funding applications (e.g., OpenAI Codex Open Source Fund). It focuses on the project's mission, quick start steps, architecture, and the repository state.

Quick start (developer)

Prerequisites

- Rust 1.70+ (cargo)
- Node.js 18+ (frontend build)
- PowerShell (Windows recommended)

Local development (PowerShell)

```powershell
# Start unified dev launcher (handles frontend and backend)
pwsh -File scripts/oxide-dev.ps1

# Run tests
cargo test --workspace

# Frontend E2E (from repo root)
cd src-frontend
npm install
npx playwright install
npm run test:e2e
```

Build & release notes

- Release automation is provided in scripts/create-release.ps1 and GitHub Actions. Pushing to `main` triggers CI that builds and validates artifacts.
- For Windows installers, use `pwsh -File scripts/build-windows.ps1` (see `docs/README-WINDOWS-BUILD.md`).

Repository structure (top-level)

- `src-tauri/` — Tauri native integration and system commands
- `oxide-core/`, `oxide-guardian/`, `oxide-copilot/`, `oxide-memory/`, `oxide-rpa/`, `oxide-voice/` — Rust crates
- `src-frontend/` — Svelte UI and Playwright tests
- `docs/` — design docs, deployment and onboarding guides

Testing & validation

- Unit tests: `cargo test --workspace`
- Frontend E2E: Playwright in `src-frontend`
- CI runs formatters, clippy, tests and build checks before commits/pushes via husky and GitHub Actions.

Security & privacy

- Default architecture favors local processing for sensitive operations (wake-word, basic analysis).
- All cloud interactions (LLMs or speech APIs) are configurable and optional; credentials are kept out of the repository and read from environment variables.

License

This project is licensed under the GNU Affero General Public License v3.0 (AGPL-3.0). See `LICENSE` for details.

Contact

- Email: iberi22@gmail.com

Short project summary for grant applications (copy for form fields)

Project name: Oxide Pilot

One-line pitch: An open-core agentic system assistant that combines secure system monitoring, automated RPA, and a conversational Copilot to diagnose, fix and automate tasks across desktop environments.

What we are building: Oxide Pilot delivers a dual-agent platform (Guardian for monitoring/security and Copilot for conversational assistance). It fuses system telemetry, programmatic tools (RPA), and multi-LLM integration, backed by optional persistent memory (SurrealDB) to provide context-aware, safe automation and remediation.

Why funding would help: Funding will accelerate integration of local vector search and memory, improve LLM-context tooling and data privacy controls, and fund production-grade packaging and testing across Windows/macOS/Linux. This shortens the path to a stable open-source core and increases adoption in privacy-conscious organizations.

Repository readiness: The repo contains automated CI, unit and E2E tests, and release scripts. Core features are implemented; remaining work focuses on SurrealDB migration, packaging, and docs.

How to evaluate quickly: Run `pwsh -File scripts/oxide-dev.ps1` to start a dev instance, or run `cargo test --workspace` to verify tests. See `docs/` for integration and architecture details.

Next steps after this commit

1. Keep the repo clean: finish translating and polishing docs and verify the SurrealDB feature on CI
2. Prepare a short demo & artifact build for reviewers
3. Provide a one-page grant summary and a short video demo (optional)

If you want, I can:

- produce a short, grant-ready one-paragraph pitch and a 500-character summary for the OpenAI form
- create a small `GRANT_SUMMARY.md` with the exact fields filled for copy/paste into the form

— End of README english —

---

## 🏗️ Arquitectura del Sistema

### Filosofía: "Open Core"

- **Núcleo Abierto**: Base potente, gratuita y de código abierto para la comunidad
- **Capa Comercial**: Funcionalidades avanzadas de IA y copiloto para sostenibilidad

### Pila Tecnológica

| Componente | Tecnología | Propósito |
|------------|------------|-----------|
| **Backend & Lógica Central** | ![Rust](https://img.shields.io/badge/-Rust-000000?style=flat&logo=rust) | Motor de alto rendimiento y seguridad |
| **Frontend & UI** | ![Tauri](https://img.shields.io/badge/-Tauri-24C8DB?style=flat&logo=tauri) + ![Svelte](https://img.shields.io/badge/-Svelte-FF3E00?style=flat&logo=svelte) | Interfaz nativa multiplataforma |
| **IA & Cloud** | ![Google Cloud](https://img.shields.io/badge/-Google%20Cloud-4285F4?style=flat&logo=google-cloud) | Vertex AI, Speech APIs, Firebase |
| **Base de Datos** | SurrealDB | Multi-modelo (Graph, Document, Vector) en Rust nativo |

---

## 🛠️ Resumen de Herramientas y Funcionalidades

Oxide Pilot ofrece un conjunto completo de herramientas para la gestión inteligente de tu sistema Windows. Aquí un resumen de sus principales funcionalidades:

### 👁️ Monitoreo y Seguridad (Agente Guardián)

- Vigilancia continua de procesos y recursos del sistema
- Detección automática de amenazas y anomalías
- Optimización proactiva del rendimiento del equipo
- Aplicación de políticas de seguridad personalizadas

### 🗣️ Asistencia Conversacional (Agente Copiloto)

- Interacción natural por voz con comandos como "Hey Oxide"
- Respuestas inteligentes a preguntas sobre el estado del sistema
- Análisis de problemas técnicos con sugerencias de solución
- Control directo del sistema mediante comandos de voz

### 📸 Análisis Visual Inteligente

- Captura y análisis automático de pantallas para diagnóstico
- Identificación de errores visuales en aplicaciones
- Comprensión contextual de interfaces y problemas en pantalla
- Asistencia visual para resolución de incidencias

### 🎯 Automatización de Tareas (RPA)

- Ejecución automática de tareas repetitivas
- Control preciso de mouse y teclado para workflows complejos
- Aprendizaje de patrones de uso personalizados
- Automatización de procesos administrativos

### 🧠 Memoria y Aprendizaje

- Historial completo de interacciones y preferencias
- Aprendizaje continuo de hábitos del usuario
- Correlación de eventos del sistema a lo largo del tiempo
- Personalización de respuestas basada en contexto histórico

### 🔊 Procesamiento de Voz

- Reconocimiento de voz de alta precisión
- Síntesis de voz natural para respuestas
- Detección de palabras clave para activación
- Soporte multimodal (voz + texto + imágenes)

### ⚙️ Gestión de Configuración

- Configuración sencilla de proveedores de IA (Gemini, Qwen)
- Gestión segura de credenciales y tokens
- Personalización de parámetros de seguridad y rendimiento
- Sincronización de configuraciones entre dispositivos

### 📊 Dashboards y Reportes

- Paneles visuales de estado del sistema
- Reportes de rendimiento y seguridad
- Métricas de uso y optimización
- Visualización de amenazas detectadas y acciones tomadas

### 🔄 Integraciones Externas

- Conexión con servicios de IA en la nube
- Integración con herramientas de desarrollo
- Compatibilidad con múltiples proveedores de IA (Google Vertex AI, OpenAI, Qwen)
- Sistema de memoria avanzada con SurrealDB

---

## 🤖 Sistema de Agentes Dual

### 👁️ Agente Guardián (Guardian Agent)

- **Función**: Sistema inmunológico del equipo
- **Operación**: 24/7 en segundo plano
- **Capacidades**:
  - Monitoreo continuo de procesos y recursos
  - Detección de amenazas en tiempo real
  - Optimización automática del rendimiento
  - Aplicación de políticas de seguridad

### 🗣️ Agente Copiloto (Copilot Agent)

- **Función**: Interfaz conversacional inteligente
- **Activación**: Bajo demanda ("Hey, Oxide")
- **Capacidades**:
  - Conversación natural por voz
  - Análisis multimodal (texto + imágenes)
  - Control directo del sistema (RPA)
  - Resolución proactiva de problemas

---

## 🚀 Capacidades Revolucionarias

### 🎙️ Interacción Natural

```text
Usuario: "Hey Oxide, ¿por qué se congela Visual Studio?"

Oxide: "Detecté que Visual Studio está esperando una operación de Git
        bloqueada por Windows Defender. ¿Quieres que configure una
        exclusión automáticamente?"
```

### 📸 Análisis Visual

- Capturas de pantalla automáticas para diagnóstico
- Análisis de interfaces y errores visuales
- Comprensión contextual de problemas en pantalla

### 🎯 Automatización Inteligente

- Control preciso de mouse y teclado
- Ejecución de tareas complejas paso a paso
- Aprendizaje de patrones de uso del usuario

### 🧠 Memoria Contextual

- Historial completo de interacciones
- Aprendizaje de preferencias del usuario
- Correlación de eventos del sistema a lo largo del tiempo

---

## 🛠️ Tecnologías Clave Integradas

### Detección de Voz

- **Picovoice**: Wake word detection de alta precisión
- **Google Speech-to-Text**: Transcripción en tiempo real

### Control del Sistema

- **rdev**: Control de bajo nivel de mouse y teclado
- **screenshots**: Captura multiplataforma de pantalla
- **windows-rs**: Integración profunda con Windows APIs

### Inteligencia Artificial

- **Vertex AI Gemini 1.5 Pro**: Modelo multimodal avanzado
- **Function Calling**: Ejecución inteligente de herramientas
- **RAG (Retrieval-Augmented Generation)**: Respuestas contextualizadas

---

## 🎨 Experiencia de Usuario

### Animación del Agente

- **Rive/Lottie**: Animaciones vectoriales fluidas
- Estados visuales reactivos (pensando, escuchando, hablando)
- Integración perfecta con la interfaz del sistema

### Interfaz Adaptativa

- Modo compacto para monitoreo pasivo
- Modo conversacional para interacción activa
- Dashboards personalizables según el rol del usuario

---

## 🔒 Seguridad y Privacidad

### Procesamiento Local

- Detección de wake word en el dispositivo
- Análisis de sistema sin envío de datos sensibles
- Encriptación end-to-end para comunicaciones cloud

### Arquitectura Zero-Trust

- Autenticación robusta con Firebase
- Permisos granulares por funcionalidad
- Auditoría completa de acciones del agente

---

## 🗄️ Roadmap: Sistema de Memoria Avanzado con SurrealDB

### 📌 Visión General

Oxide Pilot utiliza **SurrealDB** (Rust nativo) como base de datos multi-modelo embebida, proporcionando capacidades avanzadas de grafo, documentos y búsqueda vectorial directamente en Rust. Este enfoque elimina dependencias externas y ofrece rendimiento excepcional.

### 🎯 Objetivos Estratégicos

| Objetivo | Descripción | Impacto |
|----------|-------------|---------|
| **🚀 100% Rust Nativo** | Base de datos embebida sin sidecars | -50% uso de memoria, +300% velocidad de inicio |
| **📊 Almacenamiento Inteligente** | Datos del sistema como grafo de conocimiento | Análisis contextual avanzado para agentes |
| **🧠 Memoria Persistente** | Relaciones temporales entre eventos | Diagnóstico predictivo y correlación de incidencias |
| **⚡ Rendimiento** | Base de datos embebida en proceso | Latencia <5ms, throughput >1000 ops/s |
| **🔍 Capacidades Avanzadas** | Graph queries + Vector search + Full-text | Búsquedas híbridas para análisis multi-dimensional |

### 🏗️ Arquitectura Propuesta

```
┌─────────────────────────────────────────────────────────────────┐
│                     OXIDE PILOT AGENTS                          │
│  ┌────────────┐              ┌─────────────────────────┐       │
│  │  Guardian  │              │       Copilot           │       │
│  │   Agent    │              │        Agent            │       │
│  └─────┬──────┘              └───────────┬─────────────┘       │
│        │                                  │                      │
│        └──────────────┬───────────────────┘                      │
│                       ▼                                          │
│         ┌──────────────────────────────┐                        │
│         │   oxide-memory (Rust Trait)  │                        │
│         │   MemoryBackend Interface    │                        │
│         └────────────┬─────────────────┘                        │
│                      │                                           │
│                      ▼                                           │
│         ┌──────────────────────────────┐                        │
│         │   SurrealDB Backend (NEW)    │                        │
│         │  • Embedded (in-process)     │                        │
│         │  • RocksDB/TiKV storage      │                        │
│         │  • ACID transactions         │                        │
│         │  • Graph + Document + Vector │                        │
│         └────────────┬─────────────────┘                        │
│                      │                                           │
│                      ▼                                           │
│         ┌──────────────────────────────┐                        │
│         │    Data Storage Layer        │                        │
│         │  ┌────────────────────────┐  │                        │
│         │  │ System Metrics (Time)  │  │  ← CPU, RAM, Disk I/O │
│         │  ├────────────────────────┤  │                        │
│         │  │ Process Graph          │  │  ← Relaciones entre   │
│         │  │ (parent→child)         │  │    procesos           │
│         │  ├────────────────────────┤  │                        │
│         │  │ Threat Detections      │  │  ← YARA matches       │
│         │  ├────────────────────────┤  │                        │
│         │  │ User Interactions      │  │  ← Comandos, queries  │
│         │  ├────────────────────────┤  │                        │
│         │  │ Incident History       │  │  ← Errores, crashes   │
│         │  ├────────────────────────┤  │                        │
│         │  │ Performance Patterns   │  │  ← Análisis temporal  │
│         │  ├────────────────────────┤  │                        │
│         │  │ LLM Context Vectors    │  │  ← Embeddings para    │
│         │  │                        │  │    análisis semántico │
│         │  └────────────────────────┘  │                        │
│         └──────────────────────────────┘                        │
└─────────────────────────────────────────────────────────────────┘
```

### 📦 Modelo de Datos SurrealDB

#### **1. Métricas del Sistema (TimeSeries + Document)**
```surql
-- Métricas de rendimiento con metadata contextual
DEFINE TABLE system_metrics SCHEMAFULL;
DEFINE FIELD timestamp ON system_metrics TYPE datetime;
DEFINE FIELD cpu_usage ON system_metrics TYPE float;
DEFINE FIELD memory_usage ON system_metrics TYPE object;
DEFINE FIELD disk_io ON system_metrics TYPE object;
DEFINE FIELD network_stats ON system_metrics TYPE object;
DEFINE FIELD metadata ON system_metrics TYPE object;
DEFINE INDEX idx_timestamp ON system_metrics FIELDS timestamp;
```

#### **2. Grafo de Procesos (Graph Relationships)**
```surql
-- Procesos y sus relaciones padre-hijo
DEFINE TABLE process SCHEMAFULL;
DEFINE FIELD pid ON process TYPE int;
DEFINE FIELD name ON process TYPE string;
DEFINE FIELD cmd ON process TYPE string;
DEFINE FIELD start_time ON process TYPE datetime;
DEFINE FIELD cpu_percent ON process TYPE float;
DEFINE FIELD memory_mb ON process TYPE float;

-- Relación directed graph: parent→spawns→child
DEFINE TABLE spawns SCHEMAFULL TYPE RELATION IN process OUT process;
DEFINE FIELD spawn_time ON spawns TYPE datetime;
DEFINE FIELD exit_code ON spawns TYPE option<int>;
```

#### **3. Detecciones de Amenazas (Document + Graph)**
```surql
DEFINE TABLE threat SCHEMAFULL;
DEFINE FIELD severity ON threat TYPE string
  ASSERT $value INSIDE ['low', 'medium', 'high', 'critical'];
DEFINE FIELD yara_rule ON threat TYPE string;
DEFINE FIELD timestamp ON threat TYPE datetime;
DEFINE FIELD process_chain ON threat TYPE array<record<process>>;
DEFINE FIELD indicators ON threat TYPE array<string>;
DEFINE FIELD mitigation_status ON threat TYPE string;

-- Relación: threat→affects→process
DEFINE TABLE affects SCHEMAFULL TYPE RELATION IN threat OUT process;
```

#### **4. Historial de Incidencias (Temporal Graph)**
```surql
DEFINE TABLE incident SCHEMAFULL;
DEFINE FIELD description ON incident TYPE string;
DEFINE FIELD timestamp ON incident TYPE datetime;
DEFINE FIELD error_code ON incident TYPE option<string>;
DEFINE FIELD stack_trace ON incident TYPE option<string>;
DEFINE FIELD resolution_status ON incident TYPE string;
DEFINE FIELD related_processes ON incident TYPE array<record<process>>;

-- Relación temporal: incident→triggers→incident (cascadas)
DEFINE TABLE triggers SCHEMAFULL TYPE RELATION IN incident OUT incident;
DEFINE FIELD time_delta ON triggers TYPE duration;
```

#### **5. Memoria de Agentes (Vector Embeddings)**
```surql
DEFINE TABLE agent_memory SCHEMAFULL;
DEFINE FIELD agent_type ON agent_memory TYPE string
  ASSERT $value INSIDE ['guardian', 'copilot'];
DEFINE FIELD content ON agent_memory TYPE string;
DEFINE FIELD embedding ON agent_memory TYPE array<float>;  -- 1536 dims
DEFINE FIELD timestamp ON agent_memory TYPE datetime;
DEFINE FIELD metadata ON agent_memory TYPE object;

-- Índice vectorial para búsqueda semántica (KNN)
DEFINE INDEX idx_embedding ON agent_memory
  FIELDS embedding
  HNSW DIMENSION 1536
  DIST COSINE
  EFC 150
  M 12;
```

#### **6. Patrones de Comportamiento (Analytical Views)**
```surql
-- Vista pre-computada para patrones recurrentes
DEFINE TABLE performance_pattern AS
  SELECT
    time::group(timestamp, '1h') AS hour,
    math::mean(cpu_usage) AS avg_cpu,
    math::mean(memory_usage.percent) AS avg_memory,
    array::distinct(->affects->process.name) AS affected_processes
  FROM threat
  WHERE timestamp > time::now() - 7d
  GROUP BY hour;
```

### 🔄 Plan de Migración (Fases)

#### **Fase 1: Infraestructura Base (1-2 semanas)**

**Objetivo**: Implementar backend SurrealDB embebido sin romper funcionalidad actual

**Tareas**:
- [ ] **1.1** Agregar dependencia `surrealdb` al workspace
  ```toml
  [dependencies]
  surrealdb = { version = "2.3", features = ["kv-rocksdb", "scripting"] }
  surrealdb-core = { version = "2.3" }
  ```
- [ ] **1.2** Crear `oxide-memory/src/surreal_backend.rs`
  - Implementar `MemoryBackend` trait para SurrealDB
  - Modo embebido: `Surreal::new::<RocksDb>("./data/oxide.db")`
  - Namespace: `oxide`, Database: `memory`
- [ ] **1.3** Migración de esquema
  - Definir tablas (system_metrics, process, threat, incident, agent_memory)
  - Crear índices (timestamp, embeddings, graph edges)
- [ ] **1.4** Feature flag `surrealdb` en `Cargo.toml`
  ```toml
  [features]
  default = ["surrealdb"]
  json = []
  surrealdb = ["dep:surrealdb", "dep:surrealdb-core"]
  ```
- [ ] **1.5** Tests unitarios de backend
  - CRUD básico con transacciones ACID
  - Graph queries (procesos padre-hijo)
  - Vector search (similitud de embeddings)

**Entregables**:
- Backend funcional en modo embebido
- Tests passing al 100%
- Zero regresión en funcionalidad actual

---

#### **Fase 2: Recolección de Datos del Sistema (2-3 semanas)**

**Objetivo**: Capturar telemetría del sistema y almacenarla en SurrealDB

**Tareas**:
- [ ] **2.1** Extender `oxide-guardian` para recolección avanzada
  ```rust
  // oxide-guardian/src/metrics_collector.rs
  pub struct MetricsCollector {
      surreal: Arc<SurrealBackend>,
      interval: Duration,
  }

  impl MetricsCollector {
      pub async fn collect_and_store(&self) -> Result<()> {
          let metrics = self.gather_system_metrics().await?;
          let processes = self.gather_process_tree().await?;

          // Transacción ACID para consistencia
          self.surreal.transaction(|txn| async {
              txn.create("system_metrics", metrics).await?;
              txn.upsert_process_graph(processes).await?;
          }).await
      }
  }
  ```
- [ ] **2.2** Almacenar grafo de procesos
  - Capturar árbol de procesos cada 5 segundos
  - Crear edges `spawns` entre procesos padre-hijo
  - Metadata: CPU%, memoria, I/O, sockets abiertos
- [ ] **2.3** Telemetría de amenazas
  - Integración con detecciones YARA
  - Almacenar cadena de procesos afectados
  - Severity scoring automático
- [ ] **2.4** Histórico de incidencias
  - Capturar errores de aplicaciones (event logs Windows)
  - Stack traces de crashes
  - Relaciones temporales entre incidentes relacionados
- [ ] **2.5** Dashboard de monitoreo interno
  - UI Svelte para visualizar datos recolectados
  - Gráficos de series temporales (CPU, RAM)
  - Vista de grafo de procesos interactivo

**Entregables**:
- Sistema recolectando 50+ tipos de métricas
- DB creciendo ~1MB/día en uso normal
- Queries de ejemplo funcionando

---

#### **Fase 3: Análisis Inteligente para Agentes (2-3 semanas)**

**Objetivo**: LLMs consultan SurrealDB para diagnósticos contextuales

**Tareas**:
- [ ] **3.1** Queries SurrealQL para agentes
  ```rust
  // Ejemplo: "¿Qué procesos consumen más CPU en las últimas 2 horas?"
  pub async fn query_high_cpu_processes(&self, hours: u32) -> Result<Vec<ProcessInfo>> {
      self.surreal.query(r#"
          SELECT
              process.*,
              math::mean(->spawns->process.cpu_percent) AS avg_child_cpu
          FROM process
          WHERE start_time > time::now() - type::duration({hours}h)
          ORDER BY cpu_percent DESC
          LIMIT 10
      "#).bind(("hours", hours)).await
  }

  // Ejemplo: "¿Hay amenazas relacionadas con este proceso?"
  pub async fn query_related_threats(&self, pid: i32) -> Result<Vec<ThreatInfo>> {
      self.surreal.query(r#"
          SELECT threat.*,
                 array::len(threat.process_chain) AS chain_length
          FROM threat
          WHERE process_chain CONTAINS (SELECT * FROM process WHERE pid = {pid})
          ORDER BY timestamp DESC
      "#).bind(("pid", pid)).await
  }
  ```
- [ ] **3.2** Vector search para análisis semántico
  - Generar embeddings de logs con `text-embeddings-inference` (Rust)
  - Almacenar en tabla `agent_memory` con índice HNSW
  - Búsqueda KNN para "incidentes similares pasados"
  ```rust
  pub async fn find_similar_incidents(&self, query: &str, k: usize) -> Result<Vec<Incident>> {
      let embedding = self.generate_embedding(query).await?;
      self.surreal.query(r#"
          SELECT incident.*,
                 vector::similarity::cosine(embedding, {query_vec}) AS score
          FROM agent_memory
          WHERE agent_type = 'guardian'
          ORDER BY score DESC
          LIMIT {k}
      "#).bind(("query_vec", embedding)).bind(("k", k)).await
  }
  ```
- [ ] **3.3** Integración con Copilot Agent
  - Tool/Function: `analyze_system_performance(time_range)`
  - Tool/Function: `find_root_cause(error_message)`
  - Tool/Function: `predict_resource_exhaustion()`
- [ ] **3.4** Análisis temporal de patrones
  - Detección de anomalías: CPU spikes recurrentes
  - Correlación de eventos: "Cuando el proceso X crashea, Y siempre falla después"
  - Predicciones: "RAM se agotará en ~4 horas a este ritmo"
- [ ] **3.5** Exportar contexto para LLMs
  - Serializar subgrafo relevante a JSON compacto
  - Incluir en prompts de Gemini/Qwen/OpenAI
  - Ejemplo: "Últimos 10 procesos high-CPU + amenazas + incidentes relacionados"

**Entregables**:
- 15+ queries pre-definidas para agentes
- Vector search funcional con <50ms latencia
- Agentes respondiendo preguntas complejas con datos históricos

---

#### **Fase 4: Optimización y Producción (1-2 semanas)**

**Objetivo**: Preparar para release con performance óptima

**Tareas**:
- [ ] **4.1** Benchmarks de rendimiento
  - Comparar latencia SurrealDB vs Cognee (HTTP)
  - Medir throughput: inserciones/segundo
  - Target: <5ms queries, >1000 inserts/sec
- [ ] **4.2** Compresión y retención de datos
  - Comprimir métricas >30 días con agregaciones horarias
  - Purgar datos >6 meses automáticamente
  - Backup incremental a archivos `.surreal`
- [ ] **4.3** Modo distribuido (opcional)
  - Configurar cluster TiKV para enterprise
  - Multi-nodo para alta disponibilidad
  - Sincronización cross-device
- [ ] **4.4** Deprecar Cognee (breaking change)
  - Migrar datos históricos JSON → SurrealDB (`cargo run -p oxide-memory --bin migrate-json-to-surreal -- --json-dir oxide_memory --surreal-db ./data/oxide.db`; ver `docs/MEMORY_MIGRATION.md`)
  - Eliminar `oxide-cognee-bridge` del workspace
  - Actualizar docs con ejemplos SurrealDB
- [ ] **4.5** UI de administración
  - Panel Svelte para explorar DB
  - Editor de queries SurrealQL
  - Exportar datos para debugging

**Entregables**:
- Performance 10x mejor
- Sistema listo para escalar a 100k+ registros/día
- Documentación completa de implementación

---

### 📊 Especificaciones Técnicas: SurrealDB

| Característica | Valor | Beneficio |
|----------------|-------|-----------|
| **Lenguaje** | Rust (embebido) | 🟢 Sin overhead de red, rendimiento máximo |
| **Latencia típica** | <5ms (in-process) | 🟢 Respuesta instantánea |
| **Uso de memoria** | ~30MB (Rust nativo) | 🟢 Huella mínima |
| **Inicio en frío** | <100ms | 🟢 Arranque ultrarrápido |
| **Modelo de datos** | Graph + Document + Vector + Time-series | 🟢 Multi-modelo completo |
| **Queries complejas** | SurrealQL (SQL-like avanzado) | 🟢 Graph traversal nativo |
| **Transacciones** | ACID completas | 🟢 Consistencia garantizada |
| **Búsqueda vectorial** | HNSW integrado | 🟢 Sin deps externas |
| **Escalabilidad** | Horizontal (TiKV cluster opcional) | 🟢 Distribuido cuando necesario |
| **Tamaño despliegue** | +20MB (binary Rust) | 🟢 Minimal footprint |
| **Dependencias** | 0 (autocontenido) | 🟢 Zero deps |
**Implementaciones recientes**

- ✅ Búsqueda vectorial con embeddings reales e índice HNSW autoconfigurable.
- ✅ Stream guardian://metrics para suscripciones en tiempo real (subscribe_guardian_metrics).
- ✅ Analítica SurrealML (predict_threat_risk) y dataset 	hreat_training.
- ✅ Vistas computadas (iew_hourly_metrics) y función JS n::risk::resource.
- ✅ Formulario GuardianThreatTrainer para etiquetar muestras y mejorar SurrealML.


### 🎯 Beneficios Clave

1. **🚀 Performance Extrema**
   - Queries graph en <5ms
   - Embeddings search con HNSW nativo
   - Sin latencia de red (in-process)

2. **🧠 Análisis Contextual Avanzado**
   ```surql
   -- Ejemplo: "¿Qué procesos maliciosos infectaron otros procesos?"
   SELECT process.name,
          array::len(->spawns->process) AS children_spawned,
          <-affects<-threat AS threats
   FROM process
   WHERE threats IS NOT EMPTY
   ORDER BY children_spawned DESC;
   ```

3. **📈 Escalabilidad Ilimitada**
   - Embedded: 1 dispositivo, millones de registros
   - Cluster: sincronizar datos entre múltiples PCs
   - Cloud: futuro SaaS con SurrealDB serverless

4. **🔍 Búsquedas Híbridas**
   ```rust
   // Combinar graph + vector + full-text en una query
   "Procesos relacionados con 'ransomware' semánticamente +
    que spawnearon >5 hijos +
    con CPU >80%"
   ```

5. **🎨 Developer Experience**
   - SurrealQL es SQL familiar + graph extensions
   - Rust SDK con macros ergonómicas
   - Migraciones automáticas de esquema

### 🚧 Riesgos y Mitigaciones

| Riesgo | Probabilidad | Impacto | Mitigación |
|--------|--------------|---------|-----------|
| **Curva de aprendizaje SurrealQL** | Media | Medio | Queries pre-hechas + docs extensos |
| **Bugs en SurrealDB (ecosystem joven)** | Media | Alto | Tests exhaustivos + versión LTS (2.3.x) |
| **Performance no cumple target** | Baja | Alto | Benchmarks tempranos + optimización índices |

### 📚 Recursos y Referencias

- **SurrealDB Docs**: https://surrealdb.com/docs
- **SurrealQL Tutorial**: https://surrealdb.com/learn
- **Rust SDK**: https://docs.rs/surrealdb/latest/surrealdb/
- **Graph Queries**: https://surrealdb.com/docs/surrealql/statements/relate
- **Vector Search**: https://surrealdb.com/docs/surrealql/functions/vector
- **Embedding Integration**: `text-embeddings-inference` (Rust) - https://github.com/huggingface/text-embeddings-inference

### 🎉 Resultado Final

Al completar la migración, Oxide Pilot tendrá:

✅ **Sistema de Memoria 100% Rust**: Sin Python, sin HTTP, sin latencia
✅ **Knowledge Graph del Sistema**: Relaciones complejas entre procesos, amenazas, incidentes
✅ **Análisis Predictivo**: Patrones temporales para prevenir problemas
✅ **Búsqueda Híbrida**: Graph + Vector + Full-text en una sola query
✅ **Escalabilidad Enterprise**: De embedded a cluster distribuido
✅ **Performance 40x Superior**: <5ms queries vs 50-200ms actual

---

## �📊 Estado del Proyecto

**Estado Actual**: 🟢 Production Ready (92% Complete)
**Fase**: Final Integration & Polish

El proyecto está prácticamente completo con todas las funcionalidades principales implementadas. Para obtener información detallada sobre el progreso actual, consulte [TASK.md](TASK.md).

---

## 🤝 Contribución

### Nota

Actualmente en desarrollo interno para proteger la innovación.
El núcleo open source será liberado una vez completado el MVP.

### Para Colaboradores Internos

1. Clona el repositorio privado
2. Configura el entorno de desarrollo Rust + Tauri
3. Revisa la documentación técnica interna

<p align="center">
  <a href="https://github.com/usuario/oxide-pilot">
    <img src="https://img.shields.io/badge/-GitHub-181717?style=flat&logo=github" alt="GitHub">
  </a>
</p>

## Licencia

Este proyecto está licenciado bajo la **GNU Affero General Public License (AGPL) 3.0**.

### ¿Qué significa esto?

- **Uso Libre para Propósitos No Comerciales**: Puedes usar, modificar y distribuir el software libremente para uso personal, educativo o no comercial sin costo alguno.
- **Obligaciones para Uso Comercial**: Si utilizas este software en un producto o servicio comercial, debes liberar el código fuente completo de tus modificaciones bajo la misma licencia AGPL 3.0. Esto asegura que las mejoras y lógicas desarrolladas beneficien a la comunidad.
- **Protección de la Tecnología**: La AGPL garantiza que cualquier despliegue en red (como servicios web o aplicaciones SaaS) que incorpore este código deba proporcionar acceso al código fuente modificado.

Para más detalles, consulta el archivo [LICENSE](LICENSE) completo.

### Modelo de Negocio

- **Núcleo Open Source**: Gratuito y accesible para todos.
- **Servicios Empresariales**: Soporte premium, hosting gestionado, integraciones personalizadas y funcionalidades avanzadas disponibles bajo acuerdos comerciales.
- **Contacto**: Para licencias comerciales o soporte empresarial, contacta al equipo de desarrollo.

### Contacto

#### Equipo Oxide Pilot

- 📧 Email: [iberi22@gmail.com]
- 🐦 Twitter: [x_donberi]
- 💬 Discord: [Pendiente]

---

<div align="center">

**🔥 Oxide Pilot - Redefiniendo la Asistencia de Sistema con IA 🔥**

*"No solo monitoreamos tu sistema, lo entendemos"*

</div>
