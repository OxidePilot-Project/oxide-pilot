# Oxide Pilot

> **AI-powered research platform for next-generation agentic system interfaces**

Oxide Pilot is an experimental framework exploring how AI agents can securely interact with and manage computer systems through conversational interfaces—eliminating traditional mouse and keyboard interactions. This project serves as a research testbed for the community to study AI behavior in system administration, security monitoring, and automated task execution within virtual environments.

## 🎯 Purpose

 **community research project** designed to:

- **Explore agentic interfaces**: Test how conversational AI can replace traditional GUIs for system management
- **Study AI behavior**: Observe and analyze how LLMs make decisions in complex system environments
- **Advance security research**: Develop AI-driven threat detection and automated response systems
- **Bridge human-computer interaction**: Create intuitive voice/text interfaces for technical operations
- **Open experimentation platform**: Provide a complete testbed for AI safety and capabilities research

## 🏗️ Architecture

**Dual-Agent Design:**

- **Guardian Agent**: Autonomous background monitoring, security analysis, and performance optimization
- **Copilot Agent**: Conversational assistant for on-demand system queries and task execution

**Technology Stack:**

- **Backend**: Rust (high-performance, memory-safe system integration)
- **Frontend**: Tauri + Svelte (native cross-platform UI)
- **Memory**: SurrealDB (embedded graph/vector database for context persistence)
- **AI Integration**: Multi-provider support (Google Vertex AI, OpenAI, Qwen)
- **RPA Engine**: Secure automation with granular permissions and audit trails

## 🚀 Quick Start

### Prerequisites

- Rust 1.70+ (`cargo`)
- Node.js 18+ (frontend build)
- PowerShell (Windows) or Bash (Linux/macOS)

### Run Development Environment

```powershell
# Start unified dev launcher
pwsh -File scripts/oxide-dev.ps1

# Run test suite
cargo test --workspace

# Frontend E2E tests
cd src-frontend && npm install && npx playwright install && npm run test:e2e
```

### Build Releases

```powershell
# Windows installer
pwsh -File scripts/build-windows.ps1

# See docs/README-WINDOWS-BUILD.md for multi-platform builds
```

## 📂 Project Structure

```
oxide-pilot/
├── oxide-core/          # Configuration, authentication, utilities
├── oxide-guardian/      # Security monitoring, EDR, threat detection
├── oxide-copilot/       # Conversational AI agent, voice processing
├── oxide-memory/        # SurrealDB backend, context persistence
├── oxide-rpa/           # Secure automation engine with permissions
├── oxide-voice/         # Speech-to-text, wake word detection
├── src-tauri/           # Native system integration
├── src-frontend/        # Svelte UI, E2E tests (Playwright)
└── docs/                # Architecture docs, guides
```

## 🔬 Research Focus Areas

### 1. **Agentic System Control**

How can AI safely execute privileged system operations through conversational commands?

**Experiment**: Ask "Why is Visual Studio freezing?" → Agent analyzes processes, identifies Git timeout, proposes Windows Defender exclusion, and executes fix (with user confirmation).

### 2. **AI-Driven Security**

Can LLMs effectively detect threats by analyzing system behavior patterns?

**Approach**: Guardian agent uses YARA rules + LLM reasoning to identify zero-day malware through behavioral heuristics instead of signatures.

### 3. **Multimodal Context Understanding**

How well can AI interpret system state from screenshots + logs + telemetry?

**Method**: Agent captures visual errors, correlates with event logs, and provides diagnosis using image + text analysis (Gemini Pro Vision).

### 4. **Human-in-the-Loop Automation**

What's the optimal balance between autonomous execution and user confirmation?

**Design**: RPA engine with tiered permissions (auto-execute safe operations, require approval for destructive changes, audit all actions).

### 5. **Long-Term Memory Systems**

Can graph databases enable AI agents to learn from historical system behavior?

**Implementation**: SurrealDB stores process relationships, incident patterns, and user preferences to improve future recommendations.

## 🤝 For Researchers & Contributors

This project is **community-driven and open for experimentation**:

### Ways to Contribute

- **Test agent capabilities**: Try complex system management tasks, report results
- **Security research**: Attempt privilege escalation, test permission boundaries
- **AI behavior analysis**: Study decision-making patterns, identify failure modes
- **Performance benchmarking**: Measure resource usage, response times, accuracy
- **Integration experiments**: Add support for new LLM providers or system APIs

### Research Questions to Explore

- At what complexity do LLMs fail to understand system state?
- Can AI agents detect adversarial inputs designed to trick them?
- How do different LLM providers compare for system diagnostics?
- What minimum context is needed for accurate troubleshooting?
- Can agents learn user preferences without compromising privacy?

## 🔒 Security & Privacy

**Design Principles:**

- Local-first processing (wake words, basic analysis run on-device)
- Cloud LLM calls are optional and configurable
- All credentials stored in environment variables (never in code)
- Complete audit trail for every agent action
- User confirmation required for destructive operations

**Threat Model**: This is a research platform—**not hardened for production use**. Use in isolated virtual environments only.

## 📊 Current Status

**Maturity**: ~92% feature-complete (see [TASK.md](TASK.md) for details)

**Working Features:**

- ✅ Dual-agent architecture (Guardian + Copilot)
- ✅ Voice interaction with wake word detection
- ✅ Multi-LLM support (Vertex AI, OpenAI, Qwen)
- ✅ Secure RPA engine with permission system
- ✅ SurrealDB memory backend (graph + vector + timeseries)
- ✅ Cross-platform builds (Windows/macOS/Linux)
- ✅ Automated CI/CD pipelines

**In Progress:**

- ⚠️ SurrealDB migration (from JSON fallback)
- ⚠️ Advanced vector search for semantic memory
- ⚠️ Production packaging and installers

## 📚 Documentation

- **[AGENTS.md](AGENTS.md)** - Project overview, development guide
- **[TASK.md](TASK.md)** - Progress tracker, milestones
- **[RULES.md](RULES.md)** - Architecture guidelines, constraints
- **[docs/](docs/)** - Setup guides, technical specs, API references
- **[WORKFLOW_ANALYSIS.md](WORKFLOW_ANALYSIS.md)** - CI/CD pipeline analysis

## 📜 License

**GNU Affero General Public License v3.0 (AGPL-3.0)**

**Key Points:**

- ✅ Free for personal, educational, and research use
- ✅ Modifications must be shared under same license
- ✅ Network use (e.g., SaaS) requires source code disclosure
- ℹ️ Commercial support and dual licensing available on request

See [LICENSE](LICENSE) for full terms.

## 🌐 Community

**Contact:**

- Email: <iberi22@gmail.com>
- Twitter: [@x_donberi](https://twitter.com/x_donberi)
- GitHub Issues: [Report bugs or propose features](https://github.com/OxidePilot-Project/oxide-pilot/issues)

**Join the Research:**

This project thrives on community experimentation. Whether you're an AI researcher, security analyst, or curious developer—try pushing the system's limits and share your findings!

---

<div align="center">

**"Bridging humans and machines through conversational AI"**

*An open platform for studying the future of human-computer interaction*

</div>

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
- ✅ Analítica SurrealML (predict_threat_risk) y dataset  hreat_training.
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

- **SurrealDB Docs**: <https://surrealdb.com/docs>
- **SurrealQL Tutorial**: <https://surrealdb.com/learn>
- **Rust SDK**: <https://docs.rs/surrealdb/latest/surrealdb/>
- **Graph Queries**: <https://surrealdb.com/docs/surrealql/statements/relate>
- **Vector Search**: <https://surrealdb.com/docs/surrealql/functions/vector>
- **Embedding Integration**: `text-embeddings-inference` (Rust) - <https://github.com/huggingface/text-embeddings-inference>

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
