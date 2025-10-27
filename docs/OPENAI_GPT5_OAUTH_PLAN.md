# OpenAI GPT‑5 OAuth Integration Plan

## Overview

We will add first‑class support for OpenAI GPT‑5 using OAuth 2.0 (no API keys). The goal is to enable a seamless “link-based” onboarding flow similar to our Google OAuth experience and the Codex example referenced, and then integrate GPT‑5 as a collaborative provider in Oxide Pilot’s LLM orchestrator and the new threat consensus engine.

Constraints and principles:
- OAuth‑only (no API keys).
- Desktop redirect listener (127.0.0.1:<port>/<path>) with PKCE.
- Secure token storage in OS keyring.
- Clear user flow and Playwright E2E.
- No external reputation APIs required for threat analysis.

## Deliverables

- Backend (Rust, Tauri):
  - `oxide_core/openai_auth.rs`: OAuth client (PKCE, token exchange/refresh), keyring storage, auth status helpers.
  - New Tauri commands in `src-tauri/src/main.rs`:
    - `openai_start_oauth()` → opens browser or prints link; spins local listener.
    - `openai_get_auth_status()` → "OAuth Token" | "OAuth Token Expired" | "Not authenticated".
    - `openai_clear_auth()` → deletes tokens from keyring.
  - Provider integration:
    - Add `openai_chat_completions(...)` (OpenAI‑compatible API) using OAuth bearer.
    - Register OpenAI as a provider in `oxide_copilot::collaborative_providers` and in the threat consensus engine (optional second model).
- Frontend (Svelte):
  - `OpenAIAuthSetup.svelte` component with onboarding flow (like GoogleAuthSetup/QwenAuthSetup).
  - Settings + provider selector update to show OpenAI.
  - E2E tests: UI flow success, error, clear session, and provider routing.
- Docs:
  - `docs/OPENAI_GPT5_OAUTH.md`: usage, env hints, troubleshooting.
  - Update `docs/OAUTH_SETUP.md` cross‑links.

## OAuth flow

- Grant type: Authorization Code + PKCE.
- Local listener: prefer port 8081 (fallback to random), default path `/callback-openai`.
- Scopes (example; confirm per provider): `openid profile email offline_access`.
- Endpoints (example; confirm in provider documentation):
  - Authorization: `https://auth.openai.com/authorize`
  - Token: `https://auth.openai.com/oauth/token`
- Redirect URIs to register (desktop client):
  - `http://localhost:8081/callback-openai`
  - `http://127.0.0.1:8081/callback-openai`

## Backend design

- `oxide_core/openai_auth.rs`:
  - Store under keyring service `oxide_pilot_openai` with entries: `client_id`, `client_secret`, `access_token`, `refresh_token`, `access_token_expiry`.
  - Public fns:
    - `store_client_credentials(client_id, client_secret)`
    - `get_client_credentials() -> (String, String)`
    - `authenticate_openai() -> Result<String, Error>` (opens browser or headless link mode) → stores tokens
    - `get_access_token() -> Option<String>` (auto‑refresh if near expiry)
    - `clear_auth()`
    - `get_auth_status()`
- `src-tauri/src/main.rs`:
  - Commands: `openai_start_oauth`, `openai_get_auth_status`, `openai_clear_auth`.
  - Wire to keyring + listener logic (pattern identical to `google_auth.rs`).
- HTTP client:
  - `openai_chat_completions(base, model, messages, temperature) -> String` using `Authorization: Bearer <access_token>`.
  - Base URL default: `https://api.openai.com/v1` (override via env if needed for enterprise tenants).

## Threat consensus and orchestrator

- Register OpenAI as a provider in `oxide_copilot::collaborative_providers` with role configs.
- Extend `src-tauri/src/threat_consensus.rs` to use OpenAI when available:
  - If Gemini+OpenAI available → dual.
  - If OpenAI only → single with self‑consistency multi‑pass.
- Reuse the strict JSON schema used for Gemini/Qwen.

## Frontend UI/UX

- `OpenAIAuthSetup.svelte`:
  - “Sign in with OpenAI” button → invokes `openai_start_oauth`.
  - Status area bound to `openai_get_auth_status`.
  - Clear session button.
- Update Settings page and provider chooser to include OpenAI.
- Add Playwright tests mirroring Google/Qwen patterns.

## Env and config

- `.env` (no secrets):
  - `OPENAI_REDIRECT_PORT=8081` (optional)
  - `OPENAI_REDIRECT_PATH=/callback-openai` (optional)
  - `OPENAI_OAUTH_NO_BROWSER=0|1` (optional)
- Credentials are stored in OS keyring via UI/commands, not in env.

## Migration and compatibility

- OAuth‑only posture already applied to Gemini.
- OpenAI added without API keys; mirrors Gemini OAuth path.
- No change required for existing Qwen device flow.

## Testing strategy

- Unit tests:
  - Token storage/refresh (mock token responses via feature‑gated test client).
  - Chat completion request assembly (bearer header, payload shape).
  - Threat consensus aggregation when OpenAI replaces/augments Qwen.
- E2E tests:
  - Auth flow UI (success, error, headless link mode).
  - Provider routing and deterministic mock response.
  - Security Center consensus end‑to‑end with OpenAI present.

## Timeline (phased)

- Phase 1 (1–2 days): Backend OAuth client + Tauri commands, unit tests.
- Phase 2 (1 day): Frontend auth UI + E2E tests.
- Phase 3 (0.5–1 day): Provider wiring (orchestrator + consensus), docs, final pass.

## Risks

- Redirect URI registration mismatch → mitigate with clear logs and env overrides.
- Token refresh edge cases → robust retry and pre‑expiry refresh.
- API shape differences → normalize to internal JSON schema.

## Acceptance criteria

- OpenAI OAuth flow completes and stores tokens securely.
- `openai_chat_completions` works with bearer token only.
- Threat consensus uses OpenAI when available and passes unit tests.
- UI shows OpenAI provider and passes E2E.
