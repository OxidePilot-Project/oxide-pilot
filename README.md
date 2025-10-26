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

## Plan de implementación y estado

Consulta docs/IMPLEMENTATION-TASKS.md para el desglose de tareas, estados y próximos pasos.

## 🚀 Oxide Pilot v1.0

> **El Primer Asistente de Sistema Agéntico del Mundo**

[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Tauri](https://img.shields.io/badge/tauri-%2324C8DB.svg?style=for-the-badge&logo=tauri&logoColor=%23FFFFFF)](https://tauri.app/)
[![Svelte](https://img.shields.io/badge/svelte-%23f1413d.svg?style=for-the-badge&logo=svelte&logoColor=white)](https://svelte.dev/)
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

## 📊 Estado del Proyecto

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

- 📧 Email: [Pendiente]
- 🐦 Twitter: [Pendiente]
- 💬 Discord: [Pendiente]

---

<div align="center">

**🔥 Oxide Pilot - Redefiniendo la Asistencia de Sistema con IA 🔥**

*"No solo monitoreamos tu sistema, lo entendemos"*

</div>
