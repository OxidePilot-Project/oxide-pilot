---
description: Integrate Cognee memory into Oxide Pilot via local Python sidecar and Rust bridge
---

# Goal
Adopt Cognee as the primary knowledge/memory backend while preserving Local First, security, and cross-platform constraints. Implement a Python sidecar exposing a minimal REST API that wraps Cognee, and a Rust client crate used by `oxide-memory` via a pluggable backend trait.

# Why this approach
- Cognee is a Python package; embedding via PyO3 in a desktop app is possible but complex to package on Windows/macOS/Linux.
- A local sidecar (FastAPI + Uvicorn) is simpler to ship, isolate, monitor, and secure (bind to 127.0.0.1 + token).
- Keeps Rust/Tauri core clean and compliant with the “Rust backend only” rule for core functionality while permitting an optional advanced AI memory layer.

# Architecture
- `cognee-sidecar/` (Python): wraps `cognee` functions: add(), cognify(), search() via REST.
- `oxide-cognee-bridge/` (Rust crate): HTTP client with typed models, retries, and auth token.
- `oxide-memory/`: define `MemoryBackend` trait; implement `JsonBackend` (existing) and `CogneeBackend` (uses bridge). Select via config.

# Endpoints (initial)
- POST /v1/add { items: [TextItem|FileItem|UrlItem], metadata?: {...} }
- POST /v1/cognify { pipeline?: string }
- POST /v1/search { query: string, top_k?: number }
- GET /v1/health

# Data contracts
- TextItem { kind: "text", text: string, tags?: [string] }
- SearchResult { text: string, score: float, source?: string, meta?: object }

# Security
- Bind to 127.0.0.1 only. TLS optional (localhost).
- Static bearer token in local secure storage (Windows Credential Manager, macOS Keychain, Linux Secret Service)
- Minimal logging; no payloads stored in logs. E2E encryption for any non-local comms.

# Storage & Paths
- Sidecar working dir under OS user data path, e.g.:
  - Windows: %APPDATA%/OxidePilot/cognee
  - macOS: ~/Library/Application Support/OxidePilot/cognee
  - Linux: ~/.local/share/oxide-pilot/cognee

# Steps
1. Create Python sidecar skeleton
   - Files: `cognee-sidecar/pyproject.toml`, `cognee_sidecar/app.py`, `.env.example`, `README.md`
   - Dependencies: `cognee`, `fastapi`, `uvicorn[standard]`, `pydantic`, `python-dotenv`
   - Implement endpoints mapping to: `await cognee.add(...)`, `await cognee.cognify()`, `await cognee.search(query)`
   - Health check and graceful shutdown

2. Local dev environment
   - Create venv and install deps
   - Run `uvicorn cognee_sidecar.app:app --host 127.0.0.1 --port 8765`
   - Verify /v1/health, /v1/search with curl

3. Rust bridge crate: `oxide-cognee-bridge`
   - Add `reqwest`, `tokio`, `serde`, `serde_json`, `thiserror`, `tracing`
   - Define client: `CogneeClient { base_url, token, timeout }`
   - Methods: `add(items)`, `cognify(pipeline)`, `search(query, top_k)`
   - Retries with backoff; map errors to `MemoryError`

4. Refactor `oxide-memory`
   - Introduce trait `MemoryBackend` with methods: `store_interaction`, `store_system_event`, `retrieve_context`, `get_user_patterns`, etc.
   - Implement `JsonBackend` (wrap current logic)
   - Implement `CogneeBackend` using `oxide-cognee-bridge`
   - Add config switch: `memory.backend = "json" | "cognee"`

5. Process management
   - Add a small supervisor in Rust: detect sidecar running; if not, spawn it (configurable). On exit, attempt restart with jitter; max retries.
   - Windows: spawn via `python -m uvicorn ...` or packaged Python.

6. Packaging
   - Option A: Require system Python (simpler dev). Detect and guide install.
   - Option B: Ship embedded Python per-OS for production.
   - Ensure all licenses included. Keep CPU/RAM within project limits (<5% idle, <100MB when idle).

7. Tests
   - Sidecar: pytest for endpoints, mock Cognee if needed
   - Bridge: Rust unit tests with `httpmock`/`wiremock-rs`
   - Integration: spin sidecar in CI, run memory flows (store/search)

8. Security & privacy
   - Bearer token required for all endpoints
   - Never send sensitive data to cloud from sidecar without explicit consent
   - Add allowlist/denylist for data ingestion sources

9. Migration
   - Provide migration tool from `oxide_memory/*.json` to Cognee by calling /add then /cognify

10. Observability
   - Structured logs; simple metrics endpoint `/v1/metrics` (optional)

# Commands (dev)
# Create venv and install deps
python -m venv .venv
. .venv/Scripts/activate  # Windows PowerShell: .venv\Scripts\Activate.ps1
pip install -U pip
pip install cognee fastapi "uvicorn[standard]" pydantic python-dotenv

# Run sidecar
uvicorn cognee_sidecar.app:app --host 127.0.0.1 --port 8765 --reload

# Validate
curl http://127.0.0.1:8765/v1/health

# Next
- After scaffolding, wire `oxide-memory` to use `CogneeBackend` behind a feature flag `memory-cognee`.
- Add config UI in `oxide-ui/AdvancedSettings.svelte` to switch backend and show sidecar status.
