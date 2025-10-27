# Oxide Pilot

> ⚠️ **ESTADO DE DESARROLLO**: Este proyecto está actualmente en **fase de desarrollo activo**. Aunque funcional, algunas características pueden ser experimentales o estar sujetas a cambios.

[![CI/CD Pipeline](https://github.com/yourusername/oxide-pilot/actions/workflows/ci.yml/badge.svg)](https://github.com/yourusername/oxide-pilot/actions/workflows/ci.yml)
[![Build Release](https://github.com/yourusername/oxide-pilot/actions/workflows/build-release.yml/badge.svg)](https://github.com/yourusername/oxide-pilot/actions/workflows/build-release.yml)
[![License: AGPL v3](https://img.shields.io/badge/License-AGPL%20v3-blue.svg)](https://www.gnu.org/licenses/agpl-3.0)

Oxide Pilot es un **asistente de automatización empresarial** con backend en Rust (Tauri) y UI Svelte. Incluye un sistema avanzado de permisos RPA, agentes Guardian/Copilot, memoria local y soporte para múltiples LLMs.

## 🚀 Características Principales

- 🔒 **Sistema de Permisos RPA** - Control granular de automatización con seguridad enterprise
- 📊 **Auditoría Completa** - Logging automático y monitoreo de todas las acciones
- 🔄 **Rollback Inteligente** - Deshacer operaciones reversibles de forma segura
- ✋ **Confirmación de Usuario** - Sistema asíncrono de aprobación para acciones críticas
- 🧠 **Integración Multi-LLM** - Soporte para OpenAI, Gemini, Qwen y modelos locales
- 🛡️ **Análisis de Amenazas** - Consenso multi-agente para detección de seguridad
- 📈 **Monitoreo de Performance** - Métricas en tiempo real del sistema

## 📋 Requisitos del Sistema

### Mínimos
- **OS**: Windows 10/11 x64, macOS 10.15+, Ubuntu 20.04+
- **Rust**: 1.70+ con cargo
- **Node.js**: 18+ (para construir la UI)
- **Memoria**: 4GB RAM mínimo, 8GB recomendado

### Opcionales
- **Python**: 3.8–3.12 (para sidecar Cognee)
- **GPU**: Para aceleración de modelos locales

## 📥 Descarga e Instalación

### Releases Oficiales (Automáticas)

Las releases se generan automáticamente en cada push a `main` y están disponibles en [GitHub Releases](https://github.com/yourusername/oxide-pilot/releases):

- **Windows x64**: Descarga el instalador `.msi` o `.exe`
- **Etiquetado Automático**: Cada release incluye timestamp y commit SHA
- **Pre-releases**: Las builds de `main` se marcan como pre-release
- **Releases Estables**: Las versiones etiquetadas con `v*` son releases estables

#### Formatos de Etiquetas

- `v1.0.0` - Release estable con versionado semántico
- `bootstrap-YYYYMMDD-HHmmss-commit` - Build automática desde main
- `bootstrap-*` - Build específica con etiqueta personalizada

### Verificación de Integridad

Cada release incluye múltiples checksums para verificar la integridad de los archivos:

```powershell
# Verificar checksum SHA256 (Windows PowerShell)
Get-FileHash oxide-pilot-setup.exe -Algorithm SHA256
# Comparar con CHECKSUMS-sha256.txt

# También disponibles: MD5, SHA512
Get-FileHash oxide-pilot-setup.exe -Algorithm MD5
Get-FileHash oxide-pilot-setup.exe -Algorithm SHA512
```

```bash
# Verificar checksum (Linux/macOS)
sha256sum -c CHECKSUMS-sha256.txt
md5sum -c CHECKSUMS-md5.txt
sha512sum -c CHECKSUMS-sha512.txt
```

### Instalación desde Código Fuente

Si prefieres compilar desde el código fuente:

## Desarrollo rápido

PowerShell:

```powershell
# Lanzador unificado de desarrollo (gestiona .profraw y frontend)
pwsh -File scripts/oxide-dev.ps1

# Con memoria Cognee y sidecar Python
pwsh -File scripts/oxide-dev.ps1 -UseCognee -StartSidecar

# Opcional: qué hacer con artefactos *.profraw (move|delete|none)
pwsh -File scripts/oxide-dev.ps1 -ProfrawAction move -ProfrawDir dev-artifacts/coverage
```

Notas:
- El script crea/ajusta `src-tauri/.env` y construye `src-frontend/` si existe.
- Los artefactos `*.profraw` (LLVM coverage) se mueven o eliminan según parámetros.

## Pruebas

```bash
cargo test --workspace
```

### Pruebas E2E (Frontend - Playwright)

En `src-frontend/` se han añadido pruebas E2E con Playwright.

```powershell
cd src-frontend
npm install
npx playwright install
npm run test:e2e
```

Notas:

- Configuración: `src-frontend/playwright.config.ts` (levanta Vite dev y prueba en Chromium/Firefox/WebKit).
- Prueba de humo: `src-frontend/tests/smoke.spec.ts`.

## Autenticación (Gemini y Qwen)

- Configure Google Gemini (API Key u OAuth) desde la UI en `Settings` o en el asistente inicial.
- Configure Qwen mediante el flujo Device Code desde el asistente inicial o `Settings`.
- Guía completa en `docs/OAUTH_SETUP.md`.

## Build de instalador Windows

PowerShell:

```powershell
# Requisitos: cargo-tauri, (opcional) WiX/NSIS según target
pwsh -File scripts/build-windows.ps1
# Con Cognee habilitado durante build
pwsh -File scripts/build-windows.ps1 -UseCognee
```

## 🏷️ Crear una Release

El proyecto incluye un sistema automatizado de releases. Usa el script helper para crear releases fácilmente:

### Release de Desarrollo (Automática)

```powershell
# Simplemente push a main - se crea automáticamente
git push origin main

# O usa el script helper
pwsh -File scripts/create-release.ps1 -Type dev
```

### Release Estable (Versionada)

```powershell
# Crear release estable con nueva versión
pwsh -File scripts/create-release.ps1 -Type stable -Version 1.0.0

# El script automáticamente:
# 1. Actualiza la versión en Cargo.toml
# 2. Crea commit de versión
# 3. Crea tag v1.0.0
# 4. Push para trigger el workflow
```

### Release Personalizada

```powershell
# Crear release con tag personalizado
pwsh -File scripts/create-release.ps1 -Type custom -CustomTag bootstrap-feature-xyz
```

Para más detalles sobre el sistema de releases, consulta [.github/RELEASE_AUTOMATION.md](.github/RELEASE_AUTOMATION.md).

## Plan de implementación y estado

Consulta docs/IMPLEMENTATION-TASKS.md para el desglose de tareas, estados y próximos pasos.

## 🚀 Oxide Pilot v1.0

> **El Primer Asistente de Sistema Agéntico del Mundo**

[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Tauri](https://img.shields.io/badge/tauri-%2324C8DB.svg?style=for-the-badge&logo=tauri&logoColor=%23FFFFFF)](https://tauri.app/)
[![Svelte](https://img.shields.io/badge/svelte-%23f1413d.svg?style=for-the-badge&logo=svelte&logoColor=white)](https://svelte.dev/)
[![SurrealDB](https://img.shields.io/badge/SurrealDB-FF00A0?style=for-the-badge&logo=surrealdb&logoColor=white)](https://surrealdb.com/)
[![Google Cloud](https://img.shields.io/badge/GoogleCloud-%234285F4.svg?style=for-the-badge&logo=google-cloud&logoColor=white)](https://cloud.google.com/)

---

## 📋 Documentación del Proyecto

Para obtener información detallada sobre el proyecto, consulte los siguientes documentos:

- [TASK.md](TASK.md) - Gestión de tareas y progreso actual
- [PLANNING.md](PLANNING.md) - Planificación estratégica y técnica
- [UI-UX-CHANGES.md](docs/UI-UX-CHANGES.md) - Cambios recientes de UI/UX, nuevos paneles de Seguridad y Rendimiento, y guía de uso

---

## 🎯 Visión del Proyecto

### Oxide Pilot representa la evolución de los asistentes de sistema tradicionales hacia una nueva era de inteligencia agéntica. Combinamos la potencia y seguridad de Rust con la inteligencia artificial conversacional más avanzada para crear un asistente que no solo monitorea tu sistema, sino que entiende, aprende y actúa de forma proactiva

### ¿Qué hace único a Oxide Pilot?

- 🛡️ Seguridad de Próxima Generación: EDR (Endpoint Detection & Response) integrado
- ⚡ Optimización Inteligente: Análisis y mejora automática del rendimiento del sistema
- 🤖 Asistencia Conversacional: Interacción natural por voz con capacidades multimodales
- 🎮 Control Agéntico: Capacidad de tomar acciones directas en el sistema cuando es necesario
- 🧠 Memoria Persistente: Aprende de cada interacción para brindar asistencia personalizada


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
| **Memoria Cognitiva** | [Cognee](https://github.com/topoteretes/cognee) | Arquitectura de conocimiento avanzada |

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
- Soporte para memoria cognitiva avanzada (Cognee)
- Compatibilidad con múltiples proveedores de IA

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

## �️ Roadmap: Migración a SurrealDB (Sistema de Memoria Avanzado)

### 📌 Visión General

Oxide Pilot está migrando de **Cognee** (Python) a **SurrealDB** (Rust nativo) para eliminar dependencias externas y aprovechar capacidades avanzadas de bases de datos multi-modelo directamente en Rust. Esta migración representa un salto significativo en rendimiento, seguridad y capacidades de análisis.

### 🎯 Objetivos Estratégicos

| Objetivo | Descripción | Impacto |
|----------|-------------|---------|
| **🚀 100% Rust Nativo** | Eliminar sidecar Python (Cognee) | -50% uso de memoria, +300% velocidad de inicio |
| **📊 Almacenamiento Inteligente** | Datos del sistema como grafo de conocimiento | Análisis contextual avanzado para agentes |
| **🧠 Memoria Persistente** | Relaciones temporales entre eventos | Diagnóstico predictivo y correlación de incidencias |
| **⚡ Rendimiento** | Base de datos embebida en proceso | Latencia <5ms vs 50-200ms actual (HTTP) |
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
  default = ["json"]
  json = []
  cognee = ["oxide-cognee-bridge"]  # mantener compatibilidad
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
  - Migrar datos históricos JSON → SurrealDB
  - Eliminar `oxide-cognee-bridge` del workspace
  - Actualizar docs con ejemplos SurrealDB
- [ ] **4.5** UI de administración
  - Panel Svelte para explorar DB
  - Editor de queries SurrealQL
  - Exportar datos para debugging

**Entregables**:
- Performance 10x mejor que Cognee
- Sistema listo para escalar a 100k+ registros/día
- Documentación completa de migración

---

### 📊 Comparativa: Cognee vs SurrealDB

| Característica | Cognee (Actual) | SurrealDB (Propuesto) | Mejora |
|----------------|-----------------|----------------------|--------|
| **Lenguaje** | Python (sidecar HTTP) | Rust (embebido) | 🟢 Sin overhead de red |
| **Latencia típica** | 50-200ms (HTTP) | <5ms (in-process) | 🟢 **40x más rápido** |
| **Uso de memoria** | ~150MB (Python + libs) | ~30MB (Rust nativo) | 🟢 **5x menos RAM** |
| **Inicio en frío** | 3-5 segundos | <100ms | 🟢 **50x más rápido** |
| **Modelo de datos** | Vector + JSON | Graph + Document + Vector + Time-series | 🟢 Multi-modelo |
| **Queries complejas** | Limitado (REST API) | SurrealQL (SQL-like avanzado) | 🟢 Graph traversal nativo |
| **Transacciones** | No ACID | ACID completas | 🟢 Consistencia garantizada |
| **Búsqueda vectorial** | ChromaDB (externo) | HNSW integrado | 🟢 Sin deps externas |
| **Escalabilidad** | Vertical (single node) | Horizontal (TiKV cluster) | 🟢 Distribuido |
| **Tamaño despliegue** | +200MB (Python runtime) | +20MB (binary Rust) | 🟢 **10x más ligero** |
| **Dependencias** | 50+ paquetes Python | 0 (autocontenido) | 🟢 Zero deps |

### 🎯 Beneficios Clave

1. **🚀 Performance Extrema**
   - Queries graph en <5ms vs 50-200ms actual
   - Embeddings search 10x más rápido (HNSW nativo)
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
| **Breaking changes en Cognee users** | Alta | Alto | Feature flag `cognee` mantener 1 versión más |
| **Curva de aprendizaje SurrealQL** | Media | Medio | Queries pre-hechas + docs extensos |
| **Bugs en SurrealDB (joven ecosystem)** | Media | Alto | Tests exhaustivos + versión LTS (2.3.x) |
| **Migración de datos fallida** | Baja | Crítico | Script de migración con rollback |
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
