<script lang="ts">
import { createEventDispatcher, onDestroy, onMount } from "svelte";
import { isTauri } from "$lib/utils/env";
import { tauriInvoke } from "$lib/utils/tauri";

const dispatch = createEventDispatcher();

// Preferred: API Key flow
const apiKey = "";
// Optional: OAuth flow (advanced)
let clientId = "";
let clientSecret = "";
let isAuthenticating = false;
let hasExistingSession = false;
let status: {
  message: string;
  type: "success" | "error" | "info" | "warning";
} = { message: "", type: "info" };
const useAdvancedOAuth = false;

async function checkStatus() {
  if (!isTauri) return;
  try {
    const res = await tauriInvoke<string>("openai_get_auth_status");
    const s = String(res || "");
    hasExistingSession =
      (/auth/i.test(s) && !/not\s+auth/i.test(s)) || /api\s*key/i.test(s);
    if (hasExistingSession)
      status = { message: "OpenAI session is active.", type: "success" };
  } catch {
    hasExistingSession = false;
  }
}

async function connectWithApiKey() {
  if (!isTauri) {
    status = {
      message: "Desktop application required for authentication",
      type: "error",
    };
    return;
  }
  if (!apiKey.trim()) {
    status = { message: "Enter your OpenAI API Key", type: "warning" };
    return;
  }
  isAuthenticating = true;
  status = { message: "Saving API key...", type: "info" };
  try {
    await tauriInvoke("openai_set_api_key", { api_key: apiKey });
    await checkStatus();
    if (hasExistingSession) {
      status = { message: "API key saved. OpenAI is ready.", type: "success" };
      dispatch("authComplete");
    } else {
      status = {
        message: "API key saved but not active; please try again.",
        type: "warning",
      };
    }
  } catch (e) {
    status = { message: `Failed to save API key: ${e}`, type: "error" };
  } finally {
    isAuthenticating = false;
  }
}

async function startOAuth() {
  if (!isTauri) {
    status = {
      message: "Desktop application required for authentication",
      type: "error",
    };
    return;
  }
  if (!clientId.trim() || !clientSecret.trim()) {
    status = { message: "Enter Client ID and Client Secret", type: "warning" };
    return;
  }

  isAuthenticating = true;
  status = {
    message: "Starting OpenAI OAuth2 authentication...",
    type: "info",
  };
  try {
    await tauriInvoke("openai_start_oauth", {
      client_id: clientId,
      client_secret: clientSecret,
    });
    status = {
      message: "OAuth2 authentication completed successfully!",
      type: "success",
    };
    hasExistingSession = true;
    dispatch("authComplete");
  } catch (e) {
    status = { message: `OAuth2 authentication failed: ${e}`, type: "error" };
  } finally {
    isAuthenticating = false;
  }
}

async function clearSession() {
  if (!isTauri) {
    status = { message: "Desktop-only operation", type: "info" };
    return;
  }
  try {
    await tauriInvoke("openai_clear_auth");
    hasExistingSession = false;
    status = { message: "OpenAI session cleared", type: "info" };
  } catch (e) {
    status = { message: `Failed to clear session: ${e}`, type: "error" };
  }
}

let unlisten: null | (() => void) = null;
onMount(() => {
  checkStatus();
  // Browser-mode simulation for E2E tests
  if (!isTauri && typeof window !== "undefined") {
    const handler = (ev: Event) => {
      try {
        const detail: any = (ev as CustomEvent).detail || {};
        const action = String(detail.action || "").toLowerCase();
        if (action === "success") {
          hasExistingSession = true;
          status = {
            message:
              "OAuth2 authentication completed successfully! (simulated)",
            type: "success",
          };
          dispatch("authComplete");
        } else if (action === "error") {
          status = {
            message:
              detail.message || "OAuth2 authentication failed (simulated)",
            type: "error",
          };
        } else if (action === "clear") {
          hasExistingSession = false;
          status = {
            message: "OpenAI session cleared (simulated)",
            type: "info",
          };
        }
      } catch {}
    };
    window.addEventListener("openai_oauth", handler as EventListener);
    unlisten = () =>
      window.removeEventListener("openai_oauth", handler as EventListener);
  }
});
onDestroy(() => {
  try {
    unlisten && unlisten();
  } catch {}
});
</script>

<div class="openai-auth-setup">
  <div class="header">
    <div class="provider">
      <div class="icon">🧠</div>
      <div>
        <h3>OpenAI (GPT‑5)</h3>
        <p>Sign in with OAuth 2.0 (PKCE)</p>
      </div>
    </div>
    <div class="session">
      <span class="dot {hasExistingSession ? 'on' : 'off'}"></span>
      <span>{hasExistingSession ? 'Connected' : 'Not connected'}</span>
      <button class="btn" on:click={clearSession}>Clear Session</button>
    </div>
  </div>

  {#if !hasExistingSession}
    <div class="content">
      <div class="grid">
        <div class="field">
          <label for="openai-client-id">Client ID</label>
          <input id="openai-client-id" type="text" bind:value={clientId} placeholder="your-client-id" />
        </div>
        <div class="field">
          <label for="openai-client-secret">Client Secret</label>
          <input id="openai-client-secret" type="password" bind:value={clientSecret} placeholder="your-client-secret" />
        </div>
      </div>
      <button class="primary" on:click={startOAuth} disabled={isAuthenticating}>
        {#if isAuthenticating}Authenticating…{:else}Connect with OpenAI OAuth{/if}
      </button>
    </div>
  {:else}
    <div class="connected">
      <div class="big">✅</div>
      <p>OpenAI authentication is active and ready.</p>
    </div>
  {/if}

  {#if status.message}
    <div class="status {status.type}">
      <span class="badge">{status.type.toUpperCase()}</span>
      <span>{status.message}</span>
    </div>
  {/if}
</div>

<style>
  .openai-auth-setup { background: #fff; border: 1px solid #e5e7eb; border-radius: 12px; padding: 14px; box-shadow: 0 6px 20px rgba(0,0,0,0.06); max-width: 720px; margin-inline: auto; }
  .header { display: flex; justify-content: space-between; align-items: center; gap: 10px; border-bottom: 1px solid #f3f4f6; padding-bottom: 10px; margin-bottom: 12px; }
  .provider { display: flex; align-items: center; gap: 10px; }
  .icon { font-size: 28px; }
  .session { display: flex; align-items: center; gap: 10px; }
  .dot { width: 8px; height: 8px; border-radius: 50%; }
  .dot.on { background: #10b981; }
  .dot.off { background: #ef4444; }
  .btn { padding: 6px 10px; border: 1px solid #e5e7eb; border-radius: 6px; background: #f3f4f6; cursor: pointer; font-size: 12px; }
  .btn:hover { background: #e5e7eb; }
  .btn:focus-visible { outline: 2px solid #3b82f6; outline-offset: 2px; }
  .content { display: grid; gap: 12px; }
  .grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(220px, 1fr)); gap: 10px; }
  .field { display: flex; flex-direction: column; gap: 6px; }
  label { font-size: 12px; color: #6b7280; }
  input { padding: 8px 10px; border: 1px solid #e5e7eb; border-radius: 8px; font-size: 14px; transition: border-color 0.15s, box-shadow 0.15s; }
  input:hover { border-color: #cbd5e1; }
  input:focus-visible { outline: none; border-color: #3b82f6; box-shadow: 0 0 0 3px rgba(59,130,246,0.2); }
  .primary { padding: 10px 12px; border-radius: 8px; background: #111827; color: #fff; border: none; cursor: pointer; transition: transform 0.05s ease, background-color 0.15s ease; }
  .primary:hover { background: #0b1220; }
  .primary:active { transform: translateY(1px); }
  .primary:focus-visible { outline: 2px solid #3b82f6; outline-offset: 2px; }
  .connected { text-align: center; padding: 12px; }
  .connected .big { font-size: 36px; }
  .status { display: flex; align-items: center; gap: 8px; margin-top: 10px; padding: 10px; border-radius: 8px; border: 1px solid; font-size: 13px; box-shadow: 0 2px 8px rgba(0,0,0,0.04); }
  .status.success { background: #f0fdf4; color: #166534; border-color: #bbf7d0; }
  .status.error { background: #fef2f2; color: #b91c1c; border-color: #fecaca; }
  .status.info { background: #eff6ff; color: #1d4ed8; border-color: #bfdbfe; }
  .status.warning { background: #fffbeb; color: #d97706; border-color: #fed7aa; }
  .badge { font-size: 10px; border: 1px solid currentColor; padding: 2px 6px; border-radius: 999px; }
  @media (max-width: 640px) {
    .openai-auth-setup { margin: 0 8px; padding: 12px; }
    .header { flex-direction: column; align-items: flex-start; gap: 8px; }
  }
</style>
