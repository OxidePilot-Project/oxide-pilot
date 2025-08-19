# Plan de Integración Dual: GGUF (Local) + Gemini

## Objetivos
- Unir un modelo local GGUF (vía LM Studio/OpenAI API o CLI `lms`) con Google Gemini bajo una sola UI.
- Enrutamiento por proveedor estable y determinista (Tauri vs Web Preview).
- Pruebas E2E robustas para chat, UI de configuración y fallback.

## Arquitectura
- Frontend (Svelte): `AppLayout.svelte` selecciona proveedor y persiste en `localStorage` (`oxide.provider`). `ConversationInterface.svelte` enruta envío de mensajes.
- Backend (Tauri):
  - Gemini/Qwen: OAuth/Device Flow y comandos Tauri existentes.
  - Local (GGUF via LM Studio):
    - API OpenAI-compatible (`LM Studio` REST) o CLI `lms` con wrapper Tauri.
  - Configuración vía `.env` y/o archivo de config seguro.

## Enrutamiento de Chat
- Web Preview:
  - `local`: mensaje fijo "Local models are only available in the desktop (Tauri) app."
  - `gemini`/`qwen`: mensaje de eco determinista: `Web preview: I received your message: ...`
- Tauri:
  - `local`: `local_llm_chat(userPrompt)` (ya implementado)
  - `gemini`/`qwen`: `handle_user_input_command(user_input)` (ruta genérica). Futuro: incluir `provider` explícito si el backend requiere distinción.

## Configuración y Variables
- Gemini: `GEMINI_API_KEY` (en `src-tauri/.env`).
- Qwen (Device Code): `QWEN_DEVICE_AUTH_URL`, `QWEN_DEVICE_TOKEN_URL`, `QWEN_CLIENT_ID`, `QWEN_CLIENT_SECRET`, `QWEN_SCOPE`.
- Local (LM Studio):
  - REST: `LOCAL_LLM_BASE_URL`, `LOCAL_LLM_API_KEY`, `LOCAL_LLM_MODEL`.
  - CLI: `LMS_BIN` (ruta a `lms`).
- Seguridad: No versionar secretos. Usar `.env` local y cifrado donde aplique.

## UI/UX
- `LocalModelsPanel.svelte`:
  - Campos: Base URL, API Key, Modelo, Estado/Start/Stop.
  - Persistencia local y validación mínima.
- `GoogleAuthSetup.svelte` / `QwenAuthSetup.svelte`:
  - Estados de setup, errores claros y `authComplete`.
- `ConversationInterface.svelte`:
  - Badge del proveedor + advertencia para `local` en Web Preview.

## Pruebas
- E2E (Playwright):
  - Routing por proveedor (local/gemini/qwen) en Web Preview (determinista).
  - UI de setup: Google OAuth, Qwen Device Code (éxito/error/clear).
  - Selección y persistencia de proveedor (`providerInitialized` evita overwrite durante tests).
- Opcional: pruebas Tauri para rutas reales (requiere entorno local preparado).

## Roadmap de Implementación
1. Backend local LLM
   - Finalizar wrapper Tauri REST/CLI y estado (`local_llm_server_status`).
   - Manejo de errores (timeout, 4xx/5xx) y logs.
2. Frontend
   - `LocalModelsPanel.svelte`: guardar/leer configuración; comprobar conectividad.
   - `ConversationInterface.svelte`: si es necesario, enviar `provider` al backend (parámetro opcional) para ruteo explícito.
3. Configuración
   - Ampliar `src-tauri/.env.example` si faltan claves del LLM local.
   - Documentación de variables en README/Docs.
4. Pruebas
   - E2E para UI Local Models (estado/arranque/parada/config).
   - Visual snapshots (opcional) bajo `PW_VRT=1`.

## Consideraciones de Seguridad
- No exponer claves en frontend ni en logs.
- Cifrar tokens en reposo (cuando aplique) y validar scopes mínimos.
- Rate limiting para endpoints sensibles.

## Monitoreo y Observabilidad
- Métricas: tiempos de respuesta por proveedor, tasa de errores.
- Logs de enrutamiento y errores de provider.

## Éxito / Criterios de Aceptación
- Enviar/recibir respuestas desde Gemini y Local bajo Tauri sin errores.
- Web Preview ofrece respuestas deterministas y 0 flakes en E2E.
- UI de configuración clara y persistente.
