# 🔐 OAuth Setup Guide for Oxide Pilot

## Overview

Oxide Pilot uses **🔐 OAuth 2.0 only** for Google Gemini AI. API Key authentication has been disabled in the backend and UI to enforce a single, secure method for production.

## API Key Authentication (Deprecated)

API Key flow is deprecated and disabled. If you had previously configured `GEMINI_API_KEY` or `GOOGLE_GEMINI_API_KEY`, they will be ignored. Please use the OAuth 2.0 setup below.

## Method 2: OAuth 2.0 Setup (Production Ready)

### Step 1: Create Google Cloud Project
1. Go to [Google Cloud Console](https://console.cloud.google.com/)
2. Create a new project or select existing one
3. Enable the "Generative Language API"

### Step 2: Configure OAuth Consent Screen
1. Go to "APIs & Services" > "OAuth consent screen"
2. Choose "External" user type
3. Fill in required information:
   - App name: "Oxide Pilot"
   - User support email: your email
   - Developer contact: your email
4. Add scopes:
   - `https://www.googleapis.com/auth/generative-language`
   - `openid`
   - `email`
   - `profile`

### Step 3: Create OAuth Credentials
1. Go to "APIs & Services" > "Credentials"
2. Click "Create Credentials" > "OAuth 2.0 Client IDs"
3. Choose "Desktop application"
4. Name: "Oxide Pilot Desktop"
5. Add authorized redirect URIs (default path is `/callback`):
   - `http://localhost:8080/callback`
   - `http://127.0.0.1:8080/callback`
   
   You can customize the port and path used locally via environment variables below. If you change them, be sure to register matching redirect URIs in Cloud Console.

### Step 4: Download Credentials
1. Download the JSON file with your credentials
2. Note the `client_id` and `client_secret`

### Step 5: Set Environment Variables
```bash
# Windows (PowerShell)
$env:GOOGLE_OAUTH_CLIENT_ID="your-client-id.apps.googleusercontent.com"
$env:GOOGLE_OAUTH_CLIENT_SECRET="your-client-secret"
# Optional overrides for the local redirect listener
# Default path is /callback; default port tries 8080 then falls back to a random free port
$env:GOOGLE_REDIRECT_PORT="8080"            # optional
$env:GOOGLE_REDIRECT_PATH="/callback"        # optional
# Headless/CI: do not auto-open browser; copy URL from logs instead
$env:GOOGLE_OAUTH_NO_BROWSER="1"             # optional

# Windows (Command Prompt)
set GOOGLE_OAUTH_CLIENT_ID=your-client-id.apps.googleusercontent.com
set GOOGLE_OAUTH_CLIENT_SECRET=your-client-secret
rem Optional
set GOOGLE_REDIRECT_PORT=8080
set GOOGLE_REDIRECT_PATH=/callback
set GOOGLE_OAUTH_NO_BROWSER=1

# Linux/macOS
export GOOGLE_OAUTH_CLIENT_ID="your-client-id.apps.googleusercontent.com"
export GOOGLE_OAUTH_CLIENT_SECRET="your-client-secret"
# Optional
export GOOGLE_REDIRECT_PORT=8080
export GOOGLE_REDIRECT_PATH="/callback"
export GOOGLE_OAUTH_NO_BROWSER=1
```

### Step 6: Run Oxide Pilot
The application will use OAuth authentication and open a browser for login.

Notes:
- The backend listens on `http://127.0.0.1:<port><path>` and exchanges the auth code for tokens using PKCE. Tokens are stored via OS keyring.
- If `GOOGLE_OAUTH_NO_BROWSER` is set, the app will NOT auto-open a browser. Copy the authorization URL from the console logs and open it manually.

---

## Testing Authentication

### Test OAuth Authentication
```bash
# First, remove API key to force OAuth
unset GEMINI_API_KEY  # Linux/macOS
# or
Remove-Item Env:GEMINI_API_KEY  # Windows PowerShell

# Then run OAuth test
cargo run --bin test_oauth
```

---

## OpenAI (GPT-5) OAuth 2.0

Oxide Pilot supports OpenAI via OAuth 2.0 with PKCE using a local redirect listener.

### Environment Variables
- `OPENAI_REDIRECT_PATH` (optional): Override the redirect path. Default: `/callback-openai`.
- `OPENAI_REDIRECT_PORT` (optional): Preferred local port for the redirect listener. If unavailable, the app tries `8081` then falls back to a random free port.
- `OPENAI_OAUTH_NO_BROWSER` (optional): Set to `1` to suppress auto-opening the browser (headless/CI). The authorization URL is printed to logs.
- `OPENAI_API_BASE` (optional): Override the API base (for enterprise tenants). Default: `https://api.openai.com/v1`.
- `OPENAI_MODEL` (optional): Model name. Default: `gpt-5`.

### UI Flow (Settings → OpenAI)
1. Open the app and go to `Settings` → `Authentication` → `OpenAI (GPT-5)`.
2. Enter your `Client ID` and `Client Secret` then click `Connect with OpenAI OAuth`.
3. A browser window opens for login and consent.
4. After consent, the app captures the redirect and stores tokens securely in the OS keyring.
5. The OpenAI card shows a live status badge and a “Clear Session” button.

### Troubleshooting
- Browser does not open:
  - Set `OPENAI_OAUTH_NO_BROWSER=1` and open the printed URL manually.
- Redirect mismatch:
  - Ensure the OAuth client in OpenAI Console allows `http://127.0.0.1:<PORT><PATH>` (e.g., `http://127.0.0.1:8081/callback-openai`). If the port is randomized, use a Desktop application client type which permits loopback.
- Port conflicts:
  - Use `OPENAI_REDIRECT_PORT` to request a specific port. The app tries this port, then 8081, then any free port.
- “Not authenticated” in UI after login:
  - Verify tokens are present in OS keyring under service `oxide_pilot_openai`. Use “Clear Session” and retry.

## Authentication Flow

### API Key Flow
1. Check for `GEMINI_API_KEY` environment variable
2. If found, use directly for API calls
3. If not found, prompt user for manual entry

### OAuth Flow
1. Check for saved OAuth token
2. If valid token exists, use it
3. If token expired, refresh using refresh token
4. If no token or refresh fails:
   - Open browser to Google OAuth
   - User grants permissions
   - Receive authorization code
   - Exchange code for access token
   - Save token for future use

---

## Qwen OAuth 2.0 Device Code Flow

This project supports Qwen (or any OAuth2-compliant provider) via the Device Authorization Grant.

### Environment Variables
Set these in `src-tauri/.env` (see `src-tauri/.env.example`):
- `QWEN_DEVICE_AUTH_URL` — Device Authorization endpoint (POST)
- `QWEN_DEVICE_TOKEN_URL` — Token endpoint for device grant polling (POST)
- `QWEN_CLIENT_ID` — OAuth client ID
- `QWEN_CLIENT_SECRET` — Optional client secret
- `QWEN_SCOPE` — Optional scopes (default: `openid profile email`)

### Flow Overview
1. Call backend command `qwen_start_device_auth()`
   - Returns: `device_code`, `user_code`, `verification_uri`, `expires_in`, `interval`
2. Show `user_code` and open `verification_uri` for the user to authorize.
3. Poll `qwen_poll_device_auth(device_code)` every `interval` seconds:
   - `{"status":"pending"}` — keep polling
   - `{"status":"slow_down"}` — increase delay between polls
   - `{"status":"success"}` — tokens stored securely (OS keyring)
   - `{"status":"error","message":"..."}` — stop and display message
4. Check status with `qwen_get_auth_status()` or clear tokens via `qwen_clear_auth()`.

### Frontend usage (Svelte + Tauri)
```ts
import { invoke } from '@tauri-apps/api/tauri';

// 1) Start device auth
const start = await invoke('qwen_start_device_auth');
// start: { device_code, user_code, verification_uri, expires_in, interval }

// 2) Direct user to verify
window.open(start.verification_uri, '_blank');

// 3) Poll until success/error
const delay = (ms:number) => new Promise(r => setTimeout(r, ms));
let intervalMs = (start.interval ?? 5) * 1000;
let done = false;
while (!done) {
  const res = await invoke('qwen_poll_device_auth', { device_code: start.device_code });
  // res: { status: 'pending' | 'slow_down' | 'success' | 'error', message?: string }
  if (res.status === 'pending') {
    await delay(intervalMs);
  } else if (res.status === 'slow_down') {
    intervalMs += 2000; // back off a bit
    await delay(intervalMs);
  } else if (res.status === 'success') {
    done = true;
  } else {
    throw new Error(res.message ?? 'Qwen auth error');
  }
}

// 4) Verify status
const status = await invoke('qwen_get_auth_status');
console.log('Qwen auth status:', status);
```

### Notes
- Tokens are stored via OS keyring under service `oxide_pilot_qwen`.
- Ensure the Device and Token endpoints are correct for your provider.
- Handle `expires_in` on the UI to time out the flow gracefully.

#### Frontend UI components
- The Svelte UI for Qwen Device Code is implemented in `src-frontend/src/lib/components/QwenAuthSetup.svelte`.
- Provider selection (Gemini vs Qwen) is available in `src-frontend/src/lib/components/AppLayout.svelte` within the initial setup and Settings sections.
  
### Gemini Clear Session (Tauri Backend + UI)

Oxide Pilot incluye un comando Tauri para limpiar la sesión de Google/Gemini:

```ts
// Rust command name
invoke('clear_google_auth');
```

UI: En `src-frontend/src/lib/components/GoogleAuthSetup.svelte` hay un botón “Clear Session” que invoca este comando para eliminar credenciales almacenadas de forma segura.

### Provider persistence y Dark Mode

- La selección de proveedor (Gemini/Qwen) se persiste en `localStorage` bajo la clave `oxide.provider` desde `AppLayout.svelte`.
- El tema utiliza tokens de diseño y soporta dark mode mediante `prefers-color-scheme` definidos en `src-frontend/src/app.html`.

### E2E Tests (Playwright)

Se han añadido pruebas E2E de frontend con Playwright en `src-frontend/`.

Comandos:

```powershell
cd src-frontend
npm install
npx playwright install
npm run test:e2e
```

Detalles:
- Configuración: `src-frontend/playwright.config.ts` (levanta `vite dev` en el puerto 5173 y ejecuta pruebas en Chromium/Firefox/WebKit).
- Prueba de humo: `src-frontend/tests/smoke.spec.ts` valida carga de la app, encabezado y estado “Setup Required” o “System Ready”.

---

## Security Considerations

### API Key
- ✅ Simple to set up
- ✅ Good for development/testing
- ⚠️ No automatic expiration
- ⚠️ Manual key management required

### OAuth
- ✅ Automatic token refresh
- ✅ User controls permissions
- ✅ Tokens expire automatically
- ✅ More secure for production
- ⚠️ More complex setup

---

## Troubleshooting

### Common Issues

#### "Authentication failed: OAuth error"
- Check that OAuth credentials are correctly set
- Verify redirect URI matches exactly
- Ensure Generative Language API is enabled

#### "Invalid API key"
- Verify API key is correct
- Check that API key has proper permissions
- Ensure Generative Language API is enabled for your project

#### "Browser doesn't open"
- Check firewall settings
- If running in CI or headless mode, set `GOOGLE_OAUTH_NO_BROWSER=1` and open the URL printed in logs manually.
- Ensure port 8080 is available or set a custom `GOOGLE_REDIRECT_PORT`.
- If you change the path using `GOOGLE_REDIRECT_PATH`, make sure the corresponding redirect URI is registered in Google Cloud.

#### "Token refresh failed"
- Clear saved authentication: `cargo run --bin test_auth` and choose clear option
- Re-authenticate with fresh OAuth flow

---

## Production Deployment

For production deployment:

1. **Use OAuth** instead of API keys
2. **Secure credential storage** (not environment variables)
3. **Implement proper error handling**
4. **Monitor token usage and refresh**
5. **Consider service account authentication** for server deployments

---

## Next Steps

Once authentication is working:

1. **Test AI conversations**: Use the chat interface
2. **Enable voice features**: Configure microphone permissions
3. **Set up monitoring**: Configure security alerts
4. **Customize settings**: Adjust AI behavior and security policies

For support, check the main README.md or create an issue on GitHub.
