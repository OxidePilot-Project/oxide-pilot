# Qwen OAuth (Device Code) en Oxide Pilot

Guía breve para configurar y usar Qwen mediante flujo de código de dispositivo en la app de escritorio (Tauri) y validar el flujo con Playwright.

## Requisitos
* App de escritorio (Tauri) en ejecución.
* Conexión a Internet para el intercambio de tokens.
* Variables de entorno configuradas (ver abajo).

## Variables de entorno (src-tauri/.env)
Defina estas variables (consulte `src-tauri/.env.example`):

```
QWEN_DEVICE_AUTH_URL=...        # Endpoint para iniciar device auth
QWEN_DEVICE_TOKEN_URL=...       # Endpoint para canjear device_code
QWEN_CLIENT_ID=...              # Client ID de su app Qwen
QWEN_CLIENT_SECRET=...          # Client Secret (mantener seguro)
QWEN_SCOPE="openid profile email"   # Alcances recomendados
```

> Nota: No comprometa secretos en el repositorio. Use `.env` local y/o gestor de secretos.

## Comandos Tauri
Registrados en `src-tauri/src/main.rs`:

* `qwen_start_device_auth()`
  - Inicia el flujo y devuelve `{ verification_uri, user_code, device_code, interval, expires_in }`.
* `qwen_poll_device_auth(device_code)`
  - Sondea hasta éxito/expiración. En éxito, persiste sesión/token.
* `qwen_get_auth_status()`
  - Estado legible: "authenticated" / "not authenticated".
* `qwen_clear_auth()`
  - Revoca/borra sesión local.

## Flujo en la UI
Componente `src-frontend/src/lib/components/QwenAuthSetup.svelte`:
1. Inicia: llama `qwen_start_device_auth` y muestra `verification_uri` y `user_code`.
2. Usuario verifica el código en la URL indicada.
3. Sondeo: llama periódicamente `qwen_poll_device_auth` hasta completar.
4. Éxito: emite `authComplete`; `AppLayout.svelte` navega a Dashboard.
5. Limpieza: "Clear Session" invoca `qwen_clear_auth`.

`AppLayout.svelte` marca `isAuthSetupComplete=true` si Gemini o Qwen están autenticados o si Local está activo.

## Chat y enrutamiento por proveedor
* Tauri: tras autenticar Qwen, el chat usa backend Tauri (ruta remota genérica actual).
* Web Preview/tests: no hay llamadas reales; el asistente responde con mensaje determinista de preview.

## Pruebas E2E (Playwright)
* Ejecutar: `npm run test:e2e` desde `src-frontend/`.
* Cobertura relevante:
  - `tests/google-oauth-ui.spec.ts`: flujo Google.
  - `tests/provider-routing.spec.ts`: incluye Qwen (preview) — badge "Qwen" y respuesta "Web preview: ...".
  - Pruebas de Qwen Device Flow (si aplica): éxito, error y clear session.
* Para evitar conflictos de puertos, puede usar `node scripts/run-e2e.mjs ...` (asigna puerto libre).

## Troubleshooting
* "not authenticated":
  - Verifique `.env` y reintente; los códigos expiran (`expires_in`).
* Sondeo infinito:
  - Revise `interval` y conectividad; consulte logs de Tauri.
* El chat no responde con Qwen (Tauri):
  - Compruebe `qwen_get_auth_status` y errores del backend.
* Flaky selectors en tests:
  - Use scoping por contenedor `.card` + `getByRole`/regex.

## Seguridad
* No exponga `QWEN_CLIENT_SECRET`.
* Use HTTPS y almacenamiento cifrado cuando aplique.
