---
trigger: always_on
---

# 📋 Oxide Pilot - Project Rules & Guidelines

## 🎯 Project Overview

**Oxide Pilot** is an advanced AI-powered system assistant with agentic capabilities, combining system monitoring, security EDR, and conversational AI. Built with a Rust backend using the Tauri framework and a Svelte frontend, it integrates Google's Gemini 1.5 Pro, speech APIs, and a custom Cognee-based memory system.

**Status**: 🟢 Production Ready (92% Complete)  
**Architecture**: Dual-agent system (Guardian + Copilot)  
**Philosophy**: Open Core model with commercial AI layer

---

## 🏗️ Core Architecture Rules

### 1. Dual Agent System

- **Guardian Agent**: 24/7 background system monitoring and security
- **Copilot Agent**: On-demand conversational AI interface ("Hey, Oxide")
- Agents must operate independently but share context when needed
- Clear separation of concerns between monitoring and interaction

### 2. Technology Stack Requirements

- **Backend**: Rust only - no exceptions for core functionality
- **Frontend**: Svelte + Tauri for native cross-platform UI
- **AI Provider**: Google Vertex AI (Gemini 1.5 Pro) as primary
- **Memory System**: Cognee for cognitive knowledge architecture
- **Database**: SQLite for local storage, Vector DB for contextual retrieval

### 3. Security-First Design

- Zero-trust architecture with granular permissions
- End-to-end encryption for all cloud communications
- Local processing for sensitive operations (wake word, system analysis)
- YARA-based malware detection with heuristic analysis
- Complete audit trail for all agent actions

---

## 🔒 Security Rules

### 1. Data Protection

- **Local First**: Sensitive system data never leaves the device
- **Encryption**: AES-256 for local storage, TLS 1.3 for network
- **Authentication**: Firebase OAuth2 with multi-factor support
- **Privacy**: Minimal data collection, user consent for all cloud operations

### 2. Threat Detection

- Real-time YARA signature scanning
- Behavioral analysis for suspicious processes
- Automatic threat scoring and alert prioritization
- Quarantine capabilities for detected threats

### 3. Access Control

- Role-based permissions for enterprise deployment
- Granular RPA action controls with rollback mechanisms
- User approval required for system-modifying operations
- Audit logging for all security events

---

## ⚡ Performance Rules

### 1. Resource Constraints

- **CPU Usage**: < 5% average in background operation
- **Memory Usage**: < 100MB RAM in idle state
- **Wake Word Latency**: < 500ms detection time
- **Response Time**: < 2 seconds for standard queries

### 2. Optimization Requirements

- Automatic performance tuning based on system capabilities
- Resource pooling for expensive operations
- Lazy loading for UI components
- Background task prioritization

### 3. Cross-Platform Compatibility

- Windows, macOS, and Linux support
- Native system API integration per platform
- Consistent UI/UX across all platforms
- Platform-specific optimizations where beneficial

---

## 🎨 UI/UX Rules

### 1. Design Principles

- **Minimalist**: Clean, uncluttered interface
- **Accessible**: WCAG 2.1 AA compliance
- **Responsive**: Adaptive to different screen sizes
- **Consistent**: Unified color scheme and typography

### 2. Component Structure

```
AppLayout.svelte (Main container)
├── SystemDashboard.svelte (Monitoring overview)
├── ConversationInterface.svelte (AI chat)
├── PerformancePanel.svelte (System metrics)
├── AudioControls.svelte (Voice interaction)
├── AdvancedSettings.svelte (Configuration)
└── GoogleAuthSetup.svelte (Authentication)
```

### 3. Required Features

- Real-time system status visualization
- Voice input/output with visual feedback
- Conversation history with context awareness
- Multi-modal response handling (text, images, files)
- Comprehensive settings management
- Loading states and smooth transitions

---

## 🤖 AI Integration Rules

### 1. Provider Management

- Support for multiple AI providers (Google, OpenAI, Anthropic, Azure, Ollama)
- Graceful fallback between providers
- API key management with secure storage
- Rate limiting and quota management

### 2. Conversation System

- Context-aware responses using Cognee memory
- Multi-modal capabilities (text, voice, images)
- Function calling for system operations
- Conversation history persistence

### 3. Voice Processing

- Wake word detection ("Hey, Oxide")
- Real-time speech-to-text transcription
- Natural text-to-speech synthesis
- Audio device management and selection

---

## 🔧 Development Rules

### 1. Code Quality

- **Testing**: Minimum 80% code coverage
- **Documentation**: Comprehensive inline and API docs
- **Error Handling**: Detailed error types and recovery
- **Logging**: Structured logging with appropriate levels

### 2. Project Structure

```
oxide-pilot/
├── oxide-core/          # Core system functionality
├── oxide-guardian/      # Security and monitoring agent
├── oxide-copilot/       # Conversational AI agent
├── oxide-memory/        # Cognee memory integration
├── oxide-rpa/          # Robotic process automation
├── oxide-voice/        # Speech processing
├── oxide-ui/           # Svelte frontend components
└── oxide-config/       # Configuration management
```

### 3. Build & Deployment

- Automated CI/CD pipeline
- Cross-platform builds
- Enterprise deployment scripts
- Uninstallation cleanup tools
- Hot-reloading for development

---

## 🚀 RPA (Robotic Process Automation) Rules

### 1. Safety First

- User approval required for system modifications
- Rollback mechanisms for all automated actions
- Granular permission system
- Action logging and audit trail

### 2. Capabilities

- Precise mouse and keyboard control using rdev
- Screenshot analysis for visual automation
- Multi-step task execution
- Context-aware action sequences

### 3. Limitations

- No destructive operations without explicit consent
- Respect system security boundaries
- Timeout mechanisms for long-running tasks
- User override capabilities at any time

---

## 📊 Monitoring & Analytics Rules

### 1. System Metrics

- Real-time CPU, memory, disk, and network monitoring
- Process and service status tracking
- Performance trend analysis
- Automated optimization recommendations

### 2. Security Monitoring

- Continuous threat detection
- Behavioral analysis of system processes
- Network traffic analysis
- Vulnerability assessment

### 3. User Analytics

- Usage patterns (with privacy protection)
- Feature adoption metrics
- Performance impact measurement
- Error and crash reporting

---

## 🌐 Enterprise Rules

### 1. Deployment

- Group Policy template support
- Centralized management capabilities
- Silent installation options
- Configuration inheritance

### 2. Administration

- IT administrator role-based access
- Bulk configuration management
- Compliance reporting
- License management

### 3. Integration

- Active Directory integration
- SIEM system compatibility
- API endpoints for external tools
- Custom plugin architecture

---

## 📈 Business Rules

### 1. Open Core Model

- Core monitoring and security features: Open source
- Advanced AI and copilot features: Commercial license
- Clear feature differentiation
- Community contribution guidelines

### 2. Pricing Strategy

- Free tier: Basic system monitoring
- Pro tier: Full AI assistant capabilities
- Enterprise tier: Advanced management and compliance
- Educational discounts available

### 3. Support Structure

- Community support for open core
- Professional support for commercial licenses
- Documentation and knowledge base
- Training and certification programs

---

## 🔄 Update & Maintenance Rules

### 1. Version Control

- Semantic versioning (MAJOR.MINOR.PATCH)
- Backward compatibility for configuration
- Migration scripts for breaking changes
- Rollback capabilities

### 2. Security Updates

- Automatic security patch deployment
- Zero-day vulnerability response procedures
- Regular security audits
- Penetration testing schedule

### 3. Feature Updates

- User feedback integration
- A/B testing for new features
- Gradual rollout mechanisms
- Feature flag management

---

## ⚠️ Compliance & Legal Rules

### 1. Data Privacy

- GDPR compliance for EU users
- CCPA compliance for California users
- Data minimization principles
- User consent management

### 2. Security Standards

- SOC 2 Type II compliance
- ISO 27001 alignment
- NIST Cybersecurity Framework
- Industry-specific regulations

### 3. Licensing

- Open source components: MIT/Apache 2.0
- Commercial features: Proprietary license
- Third-party license compliance
- Patent protection strategy

---

## 🎯 Success Metrics

### 1. Technical KPIs

- System uptime: > 99.9%
- Response time: < 2 seconds average
- Resource usage: Within defined limits
- Security incidents: Zero tolerance

### 2. User Experience KPIs

- User satisfaction: > 4.5/5 rating
- Feature adoption: > 70% for core features
- Support ticket volume: < 5% of user base
- Onboarding completion: > 90%

### 3. Business KPIs

- Market penetration: Target metrics by segment
- Revenue growth: Quarterly targets
- Customer retention: > 95% annual
- Community engagement: Active contributor growth

---

*This document serves as the authoritative guide for all Oxide Pilot development, deployment, and operational decisions. All team members must adhere to these rules to ensure project success and user satisfaction.*
