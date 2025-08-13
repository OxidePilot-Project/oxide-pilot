---
description: Authoritative Oxide Pilot context and implementation rule for Cascade
---

# Purpose
Single source of truth for architecture, constraints, and implementation status so Cascade always operates with correct context. This rule supersedes speculative progress claims and ties plans to the actual codebase.

# Architecture Snapshot
- Backend: Rust (multi-crate), Tauri host
- Frontend: Svelte
- Dual agents: Guardian (monitoring/EDR), Copilot (conversational)
- AI: Primary Google Vertex AI (Gemini 1.5 Pro), optional others
- Memory: Pluggable backend (JSON local today) → Cognee via sidecar next
- Storage: Local-first, encryption; future SQLite + vector DB

# Current Ground Truth (code-verified)
- `oxide-memory/`: JSON-backed store (`memory.json`, `patterns.json`) via `tokio::fs` and in-memory `HashMap`. No Cognee/SQLite/vector DB in code.
- `.profraw` files: LLVM coverage artifacts; safe to ignore/remove. Not memory dumps.
- Providers: Gemini integration exists elsewhere (validate in `oxide-copilot/`).
- RPA/Voice/Guardian: basic implementations present; several advanced features are placeholders.

# Target State (from Kiro specs, validated and adjusted)
- Memory uses Cognee for knowledge architecture, semantic retrieval, and knowledge graph.
- Keep Local First; security-first: auth, encryption, audit of memory actions.
- Performance budgets: <5% CPU background, <100MB idle, low-latency voice.

# Implementation Plan (authoritative)
- Memory backend trait in `oxide-memory/`:
  - `trait MemoryBackend { store_system_event; store_interaction; retrieve_context; get_user_patterns; save/load; stats }`
  - `JsonBackend` (existing) + `CogneeBackend` (new).
- Cognee integration via local sidecar (Python FastAPI) and Rust bridge:
  - See workflow: `/cognee-integration` for end-to-end steps and endpoints.
  - Auth: localhost-only + bearer token in OS keychain.
  - Migration tool: import `oxide_memory/*.json` to Cognee.
- Config switch:
  - `memory.backend = "json" | "cognee"`
  - UI toggle in `oxide-ui/AdvancedSettings.svelte`; show sidecar status.
- Security:
  - Local-only sidecar, minimal logging, opt-in cloud usage.
  - Encrypt sensitive data at rest; add audit trail for memory writes/reads.
- Observability:
  - Structured logs; add metrics for memory ops latency and size.

# File/Module Mapping (where to change)
- `oxide-memory/src/memory.rs`: introduce trait + split into backends
- New crate `oxide-cognee-bridge/`: HTTP client to sidecar
- New folder `cognee-sidecar/` (Python) with FastAPI app
- `oxide-core/src/config.rs`: add memory backend and sidecar config
- `oxide-ui/AdvancedSettings.svelte`: backend toggle + status

# Milestones
1) Backend abstraction + keep JSON operational
2) Sidecar MVP (add/cognify/search) + Rust bridge
3) Wire `CogneeBackend` into `oxide-memory` + config switch
4) Migration tool + tests (unit/integration)
5) Security hardening + packaging story

# Testing
- Unit tests: backends conform to `MemoryBackend` contract
- Integration: spin sidecar in CI; run store/search flows
- Performance: ensure budgets (CPU/RAM) respected under load

# Requirements Tie-in
- Maps to Rules: Security-First, Local First, Performance Rules, AI Integration Rules, Voice Processing, RPA Safety.
- Treat Kiro specs (`.kiro/specs/...`) as target design; prioritize this rule when conflicts with code reality.

# Operational Notes
- If `.profraw` files appear, they come from LLVM coverage; clean and ignore.
- Sidecar process supervision from Rust with safe retries; user can disable auto-spawn.

# Next Actions (tracked)
- Implement step 1 (backend trait + JSON refactor)
- Scaffold sidecar + bridge per `/cognee-integration`
- Add config/UI toggles
- Write tests and CI job
