# Oxide Pilot - AGENTS.md

## Project Overview

Oxide Pilot is an advanced AI-powered system assistant with dual-agent architecture (Guardian + Copilot) built with Rust/Tauri backend and Svelte frontend. It features:

- **Guardian Agent**: 24/7 background monitoring, security EDR, and system optimization
- **Copilot Agent**: Conversational AI with voice interaction and multimodal capabilities
- **Technology Stack**: Rust + Tauri + Svelte + Google Vertex AI (Gemini 1.5 Pro) + Cognee memory system

**Status**: Production Ready (92% Complete)
**Architecture**: Dual-agent system with modular Rust crates and cross-platform support (Windows, macOS, Linux)

## Development Environment Setup

### Prerequisites
- Windows 10/11 x64 (primary platform)
- Rust toolchain (`rustup`, `cargo`)
- Node.js (for frontend development)
- Python 3.8–3.12 (for Cognee sidecar, optional)

### Quick Start
```powershell
# Unified development launcher (handles frontend build and artifacts)
pwsh -File scripts/oxide-dev.ps1

# With Cognee memory system enabled
pwsh -File scripts/oxide-dev.ps1 -UseCognee -StartSidecar
```

### Build Commands
```bash
# Full workspace build
cargo build --workspace

# Development with Tauri hot reload
cargo tauri dev

# Release build for Windows
pwsh -File scripts/build-windows.ps1
```

## Project Structure & Key Locations

### Root Level Files
- `TASK.md` - Current progress, milestones, and task management (92% complete status)
- `RULES.md` - Authoritative project rules, architecture guidelines, and constraints
- `README.md` - Project overview, vision, and setup instructions
- `PLANNING.md` - Strategic planning, roadmap, and technical specifications
- `Cargo.toml` - Root workspace configuration with all member crates

### Core Architecture (`oxide-core/`)
- `src/config.rs` - Configuration management and validation
- `src/auth.rs` - Authentication providers (Google OAuth2, Qwen Device Code)
- `src/qwen_auth.rs` - Qwen OAuth2 implementation and token management

### Backend Modules
- `oxide-guardian/src/` - Security agent, EDR, threat detection (YARA integration)
- `oxide-copilot/src/` - Conversational AI agent, voice processing
- `oxide-memory/src/` - Memory system with JSON fallback and optional Cognee backend
- `oxide-rpa/src/` - Robotic Process Automation, mouse/keyboard control
- `oxide-voice/src/` - Speech-to-text and text-to-speech integration

### Frontend (`src-frontend/`)
- `src/lib/components/` - Svelte components (AppLayout, ConversationInterface, etc.)
- `src/lib/stores/` - Svelte stores for state management
- `tests/` - Playwright E2E tests (smoke, OAuth, security-performance)
- `playwright.config.ts` - Test configuration with Vite dev server

### Tauri Integration (`src-tauri/`)
- `src/main.rs` - Tauri command registration and app initialization
- `src/oxide_system.rs` - Core system integration and memory management
- `src/cognee_supervisor.rs` - Cognee sidecar process management
- `tauri.conf.json` - Tauri configuration with bundle settings
- `.env.example` - Environment variables template

### Documentation (`docs/`)
- `README.md` - Documentation index
- `OAUTH_SETUP.md` - Authentication setup (Google + Qwen)
- `ENVIRONMENT_SETUP.md` - Development environment configuration
- `IMPLEMENTATION-TASKS.md` - Detailed implementation guide
- `UI-UX-CHANGES.md` - Recent UI/UX updates and changes
- `COLLABORATIVE_LLM_SYSTEM.md` - Multi-provider AI integration
- `REPO_CLEANUP.md` - Repository maintenance and cleanup guide

## Code Quality & Standards

### Rust Standards
- Use `cargo fmt` automatically after any Rust code changes
- Run `cargo clippy` to check for linting issues before commits
- Follow Rust naming conventions: `snake_case` for functions/variables, `PascalCase` for types
- Use `format!()` for string formatting with inline variables: `format!("Hello {}", name)`
- Add comprehensive error handling with custom error types
- Document public APIs with `///` doc comments

### Testing Requirements
- Minimum 80% code coverage target
- Run `cargo test --workspace` for full test suite
- Integration tests for cross-crate functionality
- Use property-based testing where applicable

### Frontend Standards
- Svelte components follow consistent naming: `ComponentName.svelte`
- Use TypeScript for type safety
- Follow Svelte's reactive patterns and stores
- Maintain accessibility standards (WCAG 2.1 AA)

## Authentication & Security

### Google OAuth2 Setup
- Required env vars: `GOOGLE_OAUTH_CLIENT_ID`, `GOOGLE_OAUTH_CLIENT_SECRET`
- Optional: `GOOGLE_REDIRECT_PATH`, `GOOGLE_OAUTH_NO_BROWSER`
- API Key fallback: `GOOGLE_API_KEY` for simpler setup
- Scopes: email, profile, drive, generative-language

### Qwen Device Code Flow
- Backend commands: `qwen_start_device_auth`, `qwen_poll_device_auth`, `qwen_get_auth_status`
- Frontend component: `QwenAuthSetup.svelte` handles device code display and polling
- Env vars: `QWEN_DEVICE_AUTH_URL`, `QWEN_DEVICE_TOKEN_URL`, `QWEN_CLIENT_ID`, `QWEN_CLIENT_SECRET`

### Security Features
- End-to-end encryption for cloud communications
- Local processing for sensitive operations (wake word, system analysis)
- YARA-based malware detection with heuristic analysis
- Complete audit trail for all agent actions

## Build & Deployment

### Development Workflow
1. Use `scripts/oxide-dev.ps1` for unified development setup
2. Frontend builds automatically if `src-frontend/` exists
3. Hot reload enabled for both Rust backend and Svelte frontend
4. Clean artifacts handling (`.profraw` files moved to `dev-artifacts/coverage/`)

### Production Build
- Use `scripts/build-windows.ps1` for Windows installer
- Supports optional Cognee integration (`-UseCognee` flag)
- Includes enterprise deployment features
- Configurable signing for production releases

### Environment Variables
```env
# Core Configuration
OXIDE_COGNEE_ENABLE=true          # Enable Cognee memory system
OXIDE_COGNEE_URL=http://127.0.0.1:8765
OXIDE_COGNEE_TOKEN=<bearer-token>

# Authentication
GOOGLE_OAUTH_CLIENT_ID=<client-id>
GOOGLE_OAUTH_CLIENT_SECRET=<secret>
GEMINI_API_KEY=<api-key>

QWEN_CLIENT_ID=<client-id>
QWEN_CLIENT_SECRET=<secret>
QWEN_DEVICE_AUTH_URL=<auth-url>

# Development
RUST_LOG=debug                    # Enable debug logging
```

## Testing Guidelines

### Unit Tests
- Run specific crate tests: `cargo test -p oxide-core`
- Full workspace tests: `cargo test --workspace`
- Mock external dependencies for isolated testing

### E2E Tests (Frontend)
```bash
cd src-frontend
npm install
npx playwright install
npm run test:e2e
```

### Performance Testing
- CPU usage target: < 5% average
- Memory usage: < 100MB in idle state
- Response time: < 2 seconds for standard queries
- Wake word detection: < 500ms latency

## Common Development Tasks

### Adding New Features
1. Plan in `TASK.md` with component assignment
2. Implement in appropriate crate (`oxide-copilot/`, `oxide-guardian/`, etc.)
3. Add unit tests and integration tests
4. Update documentation in `docs/`
5. Test with full workspace build

### Authentication Integration
1. Add OAuth2 credentials to `.env` file
2. Implement Tauri commands in `src-tauri/src/main.rs`
3. Create Svelte component in `src-frontend/src/lib/components/`
4. Add E2E test in `src-frontend/tests/`
5. Document in `docs/OAUTH_SETUP.md`

### Memory System Changes
1. Modify `oxide-memory/src/backend.rs` for backend logic
2. Update `src-tauri/src/oxide_system.rs` for integration
3. Test with both JSON and Cognee backends
4. Update feature flags in `Cargo.toml` files

## Troubleshooting

### Build Issues
- Clean workspace: `cargo clean` in root directory
- Check Rust version: `rustc --version`
- Verify Node.js: `node --version` (for frontend)
- Clear caches: Delete `target/` and `node_modules/` if corrupted

### Authentication Problems
- Clear stored credentials from system keyring
- Verify OAuth2 consent screen configuration
- Check environment variables in `.env` file
- Enable debug logging: `$env:RUST_LOG="debug"`

### Memory System Issues
- Default to JSON backend if Cognee unavailable
- Check sidecar process status
- Verify network connectivity for Cognee URL
- Monitor logs for backend switching

## Commit Guidelines

### Commit Message Format
```
[<component>] <description>

- Detailed changes
- Impact on other components
- Testing notes
```

### Pre-commit Checks
- Run `cargo check` for all crates
- Execute relevant test suites
- Format code with `cargo fmt`
- Check for clippy warnings

## Key Project Locations Summary

| Location | Purpose | Key Files |
|----------|---------|-----------|
| `TASK.md` | Progress tracking, milestones | Current status (92% complete) |
| `RULES.md` | Project guidelines, architecture | Authoritative rules document |
| `oxide-core/` | Core functionality, config | `config.rs`, `auth.rs` |
| `oxide-guardian/` | Security agent, EDR | Threat detection, monitoring |
| `oxide-copilot/` | AI agent, conversation | Voice processing, multimodal |
| `oxide-memory/` | Memory system | JSON + optional Cognee backend |
| `src-frontend/` | Svelte UI | Components, E2E tests |
| `src-tauri/` | Tauri integration | Commands, system integration |
| `docs/` | Documentation | Setup guides, implementation |
| `scripts/` | Build tools | Dev launcher, Windows build |

This AGENTS.md file provides comprehensive context for AI coding agents to work effectively on the Oxide Pilot project. Always refer to the specific component documentation and maintain the established patterns and conventions.
