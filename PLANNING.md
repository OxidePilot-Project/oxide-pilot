# 📐 Oxide Pilot - Project Planning

> Notas de Infra (Ago 2025): ver `docs/README.md` (índice), `docs/REPO_CLEANUP.md` (limpieza y target unificado) y CI en `.github/workflows/ci.yml`.

## 🎯 Vision & Objectives

Oxide Pilot represents the evolution of traditional system assistants towards a new era of **agent intelligence**. We combine the power and security of Rust with the most advanced conversational AI to create an assistant that doesn't just monitor your system, but **understands, learns, and acts** proactively.

### Core Objectives

1. **Next-Generation Security**: Integrated EDR (Endpoint Detection & Response) for comprehensive protection
2. **Intelligent Optimization**: Automatic analysis and improvement of system performance
3. **Natural Conversational Assistance**: Voice interaction with multimodal capabilities
4. **Agent Control**: Direct system actions when necessary
5. **Persistent Memory**: Learning from every interaction for personalized assistance

## 🏗️ Architecture Overview

### Philosophy: "Open Core"
- **Open Core**: Powerful, free, and open-source base for the community
- **Commercial Layer**: Advanced AI and copilot functionalities for sustainability

### Dual Agent System

#### 👁️ Guardian Agent
- **Function**: System immune system
- **Operation**: 24/7 background operation
- **Capabilities**:
  - Continuous process and resource monitoring
  - Real-time threat detection
  - Automatic performance optimization
  - Security policy enforcement

#### 🗣️ Copilot Agent
- **Function**: Intelligent conversational interface
- **Activation**: On-demand ("Hey, Oxide")
- **Capabilities**:
  - Natural voice conversation
  - Multimodal analysis (text + images)
  - Direct system control (RPA)
  - Proactive problem resolution

## 🛠️ Technology Stack

### Backend & Core Logic
- **Rust**: High-performance, secure backend engine
- **Tauri**: Desktop application framework
- **sysinfo**: Cross-platform system information
- **windows-rs**: Deep Windows API integration
- **yara-rust**: Malware signature detection
- **rdev**: Low-level mouse and keyboard control
- **screenshots**: Cross-platform screen capture

### Frontend & UI
- **Svelte**: Modern, reactive frontend framework
- **Tauri**: Native multi-platform interface
- **Rive/Lottie**: Smooth vector animations

### AI & Cloud Services
- **Google Cloud Vertex AI**: Gemini 1.5 Pro multimodal model
- **Google Speech-to-Text**: Real-time transcription
- **Google Text-to-Speech**: Natural voice synthesis
- **Firebase**: Authentication and backend services
- **Cognee**: Advanced knowledge architecture

### Memory & Data
- **Cognee**: Cognitive memory system
- **SQLite**: Local data storage
- **Vector Database**: Contextual retrieval

## 🔒 Security Framework

### Local Processing
- Wake word detection on-device
- System analysis without sending sensitive data
- End-to-end encryption for cloud communications

### Zero-Trust Architecture
- Robust authentication with Firebase
- Granular permissions by functionality
- Complete audit of agent actions

### Threat Detection
- YARA-based malware detection
- Heuristic analysis for suspicious behavior
- Real-time threat scoring and alerts

## 🎯 Implementation Constraints

### Technical Constraints
1. **Performance**: < 5% CPU average usage
2. **Memory**: < 100MB RAM in idle state
3. **Latency**: < 500ms wake word detection
4. **Cross-platform**: Windows, macOS, and Linux support
5. **Offline capability**: Core functions without internet

### Business Constraints
1. **Open Core Model**: Balance between open-source and commercial features
2. **User Privacy**: Minimal data collection and processing
3. **Enterprise Ready**: IT administration and deployment tools
4. **Scalable Architecture**: Support for future AI model integration

## 📈 Development Roadmap

### 🏁 Phase 1: Core MVP (Completed)
- [x] Rust monitoring engine
- [x] Basic Tauri interface
- [x] System API integration
- [x] Basic threat detection

### 🚀 Phase 2: Agent Capabilities (In Progress)
- [x] Vertex AI integration
- [x] Cognee memory system
- [x] Basic RPA control
- [x] Conversational interface
- [ ] Performance optimization
- [ ] Advanced security features

### 🌟 Phase 3: Advanced Intelligence (Pending)
- [ ] Complete multimodal analysis
- [ ] Complex automation
- [ ] Personalized learning
- [ ] Plugin ecosystem

## 🧪 Testing Strategy

### Unit Testing
- Comprehensive coverage for all core components
- Mock implementations for external dependencies
- Property-based testing for data validation

### Integration Testing
- End-to-end user workflow scenarios
- AI provider switching and fallback testing
- RPA and voice processing integration

### Performance Testing
- Concurrent operations and memory usage
- Cross-platform compatibility
- Resource impact on host system

### Security Testing
- Permission validation before critical actions
- End-to-end data encryption validation
- API key management security

## 🚀 Deployment Architecture

### Local Deployment
- Standalone desktop application
- Local model support for offline use
- Secure configuration management

### Enterprise Deployment
- Group policy templates
- Centralized management capabilities
- Deployment scripts for automated installation
- Monitoring and reporting tools

## 📊 Success Metrics

### Performance Metrics
- CPU usage < 5% average
- Memory usage < 100MB idle
- Response time < 2s for local functions
- Performance score 70-95

### User Experience Metrics
- Task completion rate > 90%
- User satisfaction score > 4.5/5
- First-time setup < 5 minutes
- Voice recognition accuracy > 95%

### Security Metrics
- Threat detection rate > 99%
- False positive rate < 1%
- Security incident response time < 1s
- Compliance with industry standards

## 🎨 UX/UI Design Principles

### Interface Design
- Clean, modern aesthetic
- Intuitive navigation
- Responsive layout
- Consistent visual language

### Agent Interaction
- Smooth animations for state changes
- Clear visual feedback
- Contextual information display
- Minimal cognitive load

### Accessibility
- Keyboard navigation support
- Screen reader compatibility
- High contrast mode
- Customizable interface elements

## 📈 Future Expansion

### AI Model Integration
- Support for multiple AI providers
- Local model optimization
- Hybrid cloud/local processing
- Continuous model updates

### Platform Expansion
- Mobile application development
- Web-based interface
- Cloud service integration
- IoT device support

### Ecosystem Development
- Plugin architecture
- Third-party integrations
- Developer API
- Community contribution framework
  
## 🔐 Plan de Autenticación (Qwen + Gemini)
  
### Objetivos
- Ofrecer un flujo de autenticación profesional, seguro y claro para proveedores Qwen y Gemini.
- Minimizar fricción en el onboarding mediante Device Code Flow (Qwen) y API Key (Gemini) con fallback.
- Unificar la experiencia de login con selección de proveedor, estado de sesión y cierre de sesión.
  
### Alcance y Arquitectura
- Backend: Qwen OAuth2 Device Code Flow expuesto por comandos Tauri ya integrados en `src-tauri/src/main.rs`.
- Frontend: Componentes Svelte para iniciar, visualizar y completar el flujo Qwen, más soporte API Key para Gemini.
- Configuración: Variables en `src-tauri/.env.example` para Qwen y `GEMINI_API_KEY`; documentación en `docs/OAUTH_SETUP.md`.
- Seguridad: Tokens guardados en el llavero del SO; no exponer secretos en logs o UI.
  
### Tareas (Frontend y Docs)
- Qwen Device Flow
  - `QwenAuthSetup.svelte`: iniciar `qwen_start_device_auth`, mostrar `user_code`, abrir `verification_uri`, poll con `qwen_poll_device_auth` respetando `interval` y `slow_down`.
  - Manejar `expires_in` con timeout visual y reintentos limitados.
  - `qwen_get_auth_status` para reflejar estado; `qwen_clear_auth` para cerrar sesión.
- Login/Index Unificado
  - Selector de proveedor (Qwen / Gemini).
  - Estados consistentes (no autenticado / autenticado / en curso / error) y mensajes claros.
- Gemini
  - UI de API Key (entrada, validación mínima, persistencia segura si aplica); documentar OAuth2 como futuro y fallback actual.
- Documentación
  - Extender `OAUTH_SETUP.md` con guía Qwen (ya agregado) y enlazar desde `README.md`.
  - Mantener `src-tauri/.env.example` sincronizado.
  
### Criterios de Aceptación
- El usuario completa Qwen Device Flow desde la UI y obtiene estado “autenticado”.
- Cierre de sesión Qwen elimina sesión y vuelve a “no autenticado”.
- Para Gemini, si no hay OAuth funcional, API Key permite uso; UI informa claramente el modo activo.
- Errores y timeouts se manejan sin bloquear la app ni filtrar secretos.
  
### Riesgos y Mitigaciones
- Configuración incorrecta de endpoints Qwen → Validar presencia de env vars, mensajes de ayuda y enlaces a docs.
- Polling excesivo → Aplicar backoff en `slow_down`, intervalos mínimos, límites de reintentos y cancelación por timeout.
- Confusión UX entre proveedores → Instrucciones breves por proveedor, etiquetas claras y estados visibles.
  
### Dependencias
- Comandos Tauri Qwen ya integrados en backend.
- `@tauri-apps/api` disponible en la UI Svelte.
- Variables de entorno configuradas según `src-tauri/.env.example`.
  
### Hitos y Timeline orientativo
- Día 1: `QwenAuthSetup.svelte` + integración en login; manejo de errores y timeouts.
- Día 2: UX unificada proveedor + Gemini API Key; QA básico.
- Día 3: Pulido, documentación y validación cruzada en Windows.
  
---
  
*This document outlines the strategic and technical planning for the Oxide Pilot project.*
