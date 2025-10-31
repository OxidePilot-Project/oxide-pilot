<script lang="ts">
import { createEventDispatcher, onDestroy, onMount } from "svelte";
import { writable } from "svelte/store";
import { isTauri } from "$lib/utils/env";

const dispatch = createEventDispatcher();

import { tauriInvoke } from "$lib/utils/tauri";

type AuthMethod = "api_key" | "oauth";

let authMethod: AuthMethod = "api_key";
let apiKey = "";
let clientId = "";
let clientSecret = "";
const authStatus = writable<{
  message: string;
  type: "success" | "error" | "info" | "warning";
}>({ message: "", type: "info" });
let isAuthenticating = false;
let showInstructions = false;
const availableModels: string[] = [];

// Listen for backend auth completion events (Tauri) or browser-dispatched CustomEvent fallback
let unlisten: null | (() => void | Promise<void>) = null;

type GoogleAuthEvent = {
  status?: string;
  provider?: string;
  message?: string;
  timestamp?: number;
};

function handleGoogleAuthComplete(payload: GoogleAuthEvent) {
  // Stop any spinner
  isAuthenticating = false;
  const status = (payload?.status || "").toLowerCase();
  if (status === "success") {
    authStatus.set({
      message: "Google authentication completed.",
      type: "success",
    });
    // Notify parent layout to proceed
    dispatch("authComplete");
  } else if (status === "error") {
    const msg = payload?.message || "Unknown error";
    authStatus.set({
      message: `Google authentication failed: ${msg}`,
      type: "error",
    });
  } else {
    // Unknown/other payloads
    authStatus.set({
      message: "Received authentication update.",
      type: "info",
    });
  }
}

onMount(async () => {
  try {
    if (isTauri) {
      const mod = await import("@tauri-apps/api/event");
      const stop = await mod.listen("google_auth_complete", (evt: any) => {
        handleGoogleAuthComplete(evt?.payload as GoogleAuthEvent);
      });
      unlisten = () => {
        try {
          (stop as any)();
        } catch {}
      };
    } else if (typeof window !== "undefined") {
      const browserHandler = (e: Event) => {
        const detail = (e as CustomEvent).detail as GoogleAuthEvent;
        handleGoogleAuthComplete(detail);
      };
      window.addEventListener(
        "google_auth_complete",
        browserHandler as EventListener,
      );
      unlisten = () =>
        window.removeEventListener(
          "google_auth_complete",
          browserHandler as EventListener,
        );
    }
  } catch (e) {
    // Non-fatal; just log to console
    console.error("Failed to initialize google_auth_complete listener:", e);
  }
});

onDestroy(() => {
  try {
    if (unlisten) {
      unlisten();
    }
  } catch {}
});

async function clearGoogleSession() {
  if (!isTauri) {
    authStatus.set({
      message: "Clear session is a desktop-only operation (browser test mode).",
      type: "info",
    });
    return;
  }
  try {
    await tauriInvoke("clear_google_auth");
    authStatus.set({ message: "Google session cleared.", type: "info" });
  } catch (e) {
    authStatus.set({ message: `Failed to clear session: ${e}`, type: "error" });
  }
}

async function saveApiKey() {
  if (!apiKey.trim()) {
    authStatus.set({
      message: "Please enter your Google API Key",
      type: "warning",
    });
    return;
  }

  authStatus.set({ message: "Validating and saving API key...", type: "info" });

  // Check if we're in Tauri context
  if (!isTauri) {
    authStatus.set({
      message:
        "Error: Not running in Tauri context. Please use the desktop application.",
      type: "error",
    });
    return;
  }

  try {
    // For API key method, use the correct function
    await tauriInvoke("set_google_api_key", {
      api_key: apiKey,
    });
    authStatus.set({
      message: "API key saved successfully!",
      type: "success",
    });

    // Dispatch success event
    dispatch("authSuccess", { method: "api_key", apiKey });

    // Dispatch event to parent component
    dispatch("authComplete");
  } catch (error) {
    console.error("API key validation failed:", error);
    authStatus.set({
      message: `Failed to save API key: ${error}`,
      type: "error",
    });
  }
}

async function saveClientCredentials() {
  if (!clientId.trim() || !clientSecret.trim()) {
    authStatus.set({
      message: "Please enter both Client ID and Client Secret",
      type: "warning",
    });
    return;
  }

  authStatus.set({ message: "Saving client credentials...", type: "info" });

  // Check if we're in Tauri context
  if (!isTauri) {
    authStatus.set({
      message:
        "Error: Not running in Tauri context. Please use the desktop application.",
      type: "error",
    });
    return;
  }

  try {
    await tauriInvoke("set_google_client_credentials", {
      client_id: clientId,
      client_secret: clientSecret,
    });
    authStatus.set({
      message:
        "Client credentials saved successfully! You can now authenticate.",
      type: "success",
    });
  } catch (error) {
    authStatus.set({
      message: `Error saving client credentials: ${error}`,
      type: "error",
    });
    console.error("Error saving client credentials:", error);
  }
}

async function startGoogleAuth() {
  if (authMethod === "oauth" && (!clientId.trim() || !clientSecret.trim())) {
    authStatus.set({
      message: "Please save client credentials first",
      type: "warning",
    });
    return;
  }

  isAuthenticating = true;
  authStatus.set({
    message: "Initiating Google authentication flow...",
    type: "info",
  });

  // Check if we're in Tauri context
  if (!isTauri) {
    authStatus.set({
      message:
        "Error: Not running in Tauri context. Please use the desktop application.",
      type: "error",
    });
    isAuthenticating = false;
    return;
  }

  try {
    const result = await tauriInvoke("authenticate_google_command");
    authStatus.set({
      message: `Authentication flow started: ${result}`,
      type: "info",
    });
  } catch (error) {
    authStatus.set({
      message: `Error during Google authentication: ${error}`,
      type: "error",
    });
    console.error("Error during Google authentication:", error);
  } finally {
    isAuthenticating = false;
  }
}

function toggleInstructions() {
  showInstructions = !showInstructions;
}

function switchAuthMethod(method: AuthMethod) {
  authMethod = method;
  authStatus.set({ message: "", type: "info" });
}
</script>

<div class="google-auth-setup">
  <div class="auth-header">
    <div class="header-content">
      <div class="logo-container">
        <div class="logo-icon">🔐</div>
        <h2>Google Gemini API Configuration</h2>
      </div>
      <div style="display:flex; gap:10px; align-items:center;">
        <button class="help-button" on:click={toggleInstructions} aria-label={showInstructions ? 'Hide help' : 'Show help'}>
          {showInstructions ? '✕' : '❓'}
        </button>
        <button class="clear-btn" on:click={clearGoogleSession} title="Clear saved Google/Gemini session" aria-label="Clear saved Google session">Clear Session</button>
      </div>
    </div>
    <p class="header-subtitle">Connect Oxide Pilot to Google's powerful AI models</p>
  </div>

  <!-- Auth Method Selection -->
  <div class="auth-method-selector">
    <button type="button" class="method-option {authMethod === 'api_key' ? 'active' : ''}" on:click={() => switchAuthMethod('api_key')} aria-pressed={authMethod === 'api_key'}>
      <div class="method-icon">🔑</div>
      <div class="method-content">
        <h3>API Key</h3>
        <p>Simple setup for personal use</p>
      </div>
    </button>
    <button type="button" class="method-option {authMethod === 'oauth' ? 'active' : ''}" on:click={() => switchAuthMethod('oauth')} aria-pressed={authMethod === 'oauth'}>
      <div class="method-icon">🔐</div>
      <div class="method-content">
        <h3>OAuth 2.0</h3>
        <p>Secure authentication for teams</p>
      </div>
    </button>
  </div>

  {#if showInstructions}
    <div class="instructions">
      <h3>📋 Setup Instructions</h3>
      {#if authMethod === 'api_key'}
        <div class="instruction-steps">
          <div class="step">
            <div class="step-number">1</div>
            <div class="step-content">
              <p>Go to <a href="https://aistudio.google.com/apikey" target="_blank" rel="noopener">Google AI Studio</a></p>
            </div>
          </div>
          <div class="step">
            <div class="step-number">2</div>
            <div class="step-content">
              <p>Click "Create API Key"</p>
            </div>
          </div>
          <div class="step">
            <div class="step-number">3</div>
            <div class="step-content">
              <p>Copy the generated API key</p>
            </div>
          </div>
          <div class="step">
            <div class="step-number">4</div>
            <div class="step-content">
              <p>Paste it in the field below</p>
            </div>
          </div>
        </div>
        <div class="info-box">
          <div class="info-icon">💡</div>
          <div class="info-content">
            <strong>Why API Key?</strong>
            <p>API keys are simpler to set up and perfect for personal use. They provide direct access to Google's Gemini AI models.</p>
          </div>
        </div>
      {:else}
        <div class="instruction-steps">
          <div class="step">
            <div class="step-number">1</div>
            <div class="step-content">
              <p>Go to the <a href="https://console.cloud.google.com/" target="_blank" rel="noopener">Google Cloud Console</a></p>
            </div>
          </div>
          <div class="step">
            <div class="step-number">2</div>
            <div class="step-content">
              <p>Create a new project or select an existing one</p>
            </div>
          </div>
          <div class="step">
            <div class="step-number">3</div>
            <div class="step-content">
              <p>Enable the "Generative Language API"</p>
            </div>
          </div>
          <div class="step">
            <div class="step-number">4</div>
            <div class="step-content">
              <p>Go to "Credentials" → "Create Credentials" → "OAuth 2.0 Client IDs"</p>
            </div>
          </div>
          <div class="step">
            <div class="step-number">5</div>
            <div class="step-content">
              <p>Set application type to "Desktop application"</p>
            </div>
          </div>
          <div class="step">
            <div class="step-number">6</div>
            <div class="step-content">
              <p>Copy the Client ID and Client Secret below</p>
            </div>
          </div>
        </div>
      {/if}
    </div>
  {/if}

  <div class="form-section">
    {#if authMethod === 'api_key'}
      <div class="input-group">
        <label for="apiKey">
          <span class="label-text">Google Gemini API Key</span>
          <span class="required">*</span>
        </label>
        <div class="input-wrapper">
          <input
            type="password"
            id="apiKey"
            bind:value={apiKey}
            placeholder="AIzaSy..."
            class="credential-input"
            autocomplete="off"
          />
          <button
            class="visibility-toggle"
            on:click="{() => {
              const input = document.getElementById('apiKey') as HTMLInputElement | null;
              if (input) {
                input.type = input.type === 'password' ? 'text' : 'password';
              }
            }}"
            aria-label="Toggle API key visibility"
          >
            👁️
          </button>
        </div>
        <small class="input-hint">Get your API key from <a href="https://aistudio.google.com/apikey" target="_blank" rel="noopener">Google AI Studio</a></small>
      </div>

      <div class="button-group">
        <button
          class="auth-button primary"
          on:click={saveApiKey}
          disabled={!apiKey.trim() || isAuthenticating}
          aria-busy={isAuthenticating}
        >
          {#if isAuthenticating}
            <div class="button-content">
              <div class="spinner"></div>
              <span>Validating...</span>
            </div>
          {:else}
            <div class="button-content">
              <span>🔑 Save & Validate API Key</span>
            </div>
          {/if}
        </button>
      </div>
    {:else}
      <div class="input-group">
        <label for="clientId">
          <span class="label-text">Google Client ID</span>
          <span class="required">*</span>
        </label>
        <input
          type="text"
          id="clientId"
          bind:value={clientId}
          placeholder="123456789-abcdefghijklmnop.apps.googleusercontent.com"
          class="credential-input"
        />
      </div>

      <div class="input-group">
        <label for="clientSecret">
          <span class="label-text">Google Client Secret</span>
          <span class="required">*</span>
        </label>
        <div class="input-wrapper">
          <input
            type="password"
            id="clientSecret"
            bind:value={clientSecret}
            placeholder="GOCSPX-xxxxxxxxxxxxxxxxxxxxxxxx"
            class="credential-input"
            autocomplete="off"
          />
          <button
            class="visibility-toggle"
            on:click="{() => {
              const input = document.getElementById('clientSecret') as HTMLInputElement | null;
              if (input) {
                input.type = input.type === 'password' ? 'text' : 'password';
              }
            }}"
            aria-label="Toggle client secret visibility"
          >
            👁️
          </button>
        </div>
      </div>

      <div class="button-group">
        <button
          class="auth-button"
          on:click={saveClientCredentials}
          disabled={!clientId.trim() || !clientSecret.trim() || isAuthenticating}
          aria-busy={isAuthenticating}
        >
          <div class="button-content">
            <span>💾 Save Credentials</span>
          </div>
        </button>

        <button
          class="auth-button primary"
          on:click={startGoogleAuth}
          disabled={isAuthenticating || !clientId.trim() || !clientSecret.trim()}
          aria-busy={isAuthenticating}
        >
          {#if isAuthenticating}
            <div class="button-content">
              <div class="spinner"></div>
              <span>Authenticating...</span>
            </div>
          {:else}
            <div class="button-content">
              <span>🚀 Authenticate with Google</span>
            </div>
          {/if}
        </button>
      </div>
    {/if}
  </div>

  {#if availableModels.length > 0}
    <div class="models-info">
      <h4>✅ Available Models ({availableModels.length})</h4>
      <div class="models-list">
        {#each availableModels.slice(0, 3) as model}
          <span class="model-tag">{model.split('/').pop()}</span>
        {/each}
        {#if availableModels.length > 3}
          <span class="model-tag more">+{availableModels.length - 3} more</span>
        {/if}
      </div>
    </div>
  {/if}

  {#if $authStatus.message}
    <div class="auth-status {$authStatus.type}" role="status" aria-live="polite" aria-atomic="true">
      <div class="status-icon">
        {#if $authStatus.type === 'success'}
          ✅
        {:else if $authStatus.type === 'error'}
          ❌
        {:else if $authStatus.type === 'warning'}
          ⚠️
        {:else}
          ℹ️
        {/if}
      </div>
      <div class="status-message">{$authStatus.message}</div>
    </div>
  {/if}
</div>

<style>
  .google-auth-setup {
    background: white;
    border-radius: 16px;
    padding: 30px;
    max-width: 700px;
    margin: 0 auto;
    box-shadow: 0 10px 40px rgba(0, 0, 0, 0.08);
    border: 1px solid #e1e8ed;
    backdrop-filter: blur(10px);
  }

  .auth-header {
    margin-bottom: 30px;
  }

  .header-content {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: 15px;
  }

  .logo-container {
    display: flex;
    align-items: center;
    gap: 15px;
  }

  .logo-icon {
    font-size: 32px;
  }

  .auth-header h2 {
    color: #1a73e8;
    margin: 0;
    font-size: 24px;
    font-weight: 700;
  }

  .header-subtitle {
    color: #5f6368;
    margin: 0 0 0 48px;
    font-size: 16px;
    line-height: 1.5;
  }

  .help-button {
    background: #f8f9fa;
    border: 1px solid #dadce0;
    border-radius: 50%;
    width: 40px;
    height: 40px;
    font-size: 18px;
    color: #5f6368;
    cursor: pointer;
    transition: all 0.2s ease;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .help-button:hover {
    background: #e8f0fe;
    border-color: #1a73e8;
    color: #1a73e8;
    transform: rotate(15deg);
  }

  .auth-method-selector {
    display: flex;
    gap: 15px;
    margin-bottom: 25px;
  }

  .method-option {
    flex: 1;
    padding: 20px;
    border: 2px solid #e8eaed;
    border-radius: 12px;
    cursor: pointer;
    transition: all 0.3s ease;
    background: #f8f9fa;
  }

  .method-option:hover {
    border-color: #d2e3fc;
    background: #f1f8ff;
    transform: translateY(-3px);
  }

  .method-option.active {
    border-color: #1a73e8;
    background: #e8f0fe;
    box-shadow: 0 6px 20px rgba(26, 115, 232, 0.15);
  }

  .method-icon {
    font-size: 24px;
    margin-bottom: 12px;
  }

  .method-content h3 {
    margin: 0 0 8px 0;
    color: #202124;
    font-size: 18px;
    font-weight: 600;
  }

  .method-content p {
    margin: 0;
    color: #5f6368;
    font-size: 14px;
  }

  .instructions {
    background: #f8f9fa;
    border-radius: 12px;
    padding: 25px;
    margin-bottom: 30px;
    border: 1px solid #e8eaed;
  }

  .instructions h3 {
    color: #1a73e8;
    margin: 0 0 20px 0;
    font-size: 20px;
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .instruction-steps {
    margin-bottom: 20px;
  }

  .step {
    display: flex;
    gap: 15px;
    margin-bottom: 15px;
  }

  .step-number {
    width: 28px;
    height: 28px;
    background: #1a73e8;
    color: white;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-weight: 600;
    font-size: 14px;
    flex-shrink: 0;
  }

  .step-content p {
    margin: 0;
    color: #3c4043;
    font-size: 15px;
    line-height: 1.5;
  }

  .step-content a {
    color: #1a73e8;
    text-decoration: none;
    font-weight: 500;
  }

  .step-content a:hover {
    text-decoration: underline;
  }

  .info-box {
    background: #e8f0fe;
    border: 1px solid #d2e3fc;
    border-radius: 12px;
    padding: 20px;
    display: flex;
    gap: 15px;
  }

  .info-icon {
    font-size: 24px;
    flex-shrink: 0;
  }

  .info-content strong {
    color: #1967d2;
    display: block;
    margin-bottom: 8px;
    font-size: 16px;
  }

  .info-content p {
    margin: 0;
    color: #3c4043;
    font-size: 14px;
    line-height: 1.5;
  }

  .form-section {
    margin-top: 20px;
  }

  .input-group {
    margin-bottom: 25px;
  }

  label {
    display: flex;
    align-items: center;
    margin-bottom: 10px;
    font-weight: 500;
    color: #202124;
  }

  .label-text {
    font-size: 15px;
  }

  .required {
    color: #ea4335;
    margin-left: 4px;
    font-weight: bold;
  }

  .input-wrapper {
    position: relative;
  }

  .credential-input {
    width: 100%;
    padding: 14px 50px 14px 16px;
    border: 2px solid #dadce0;
    border-radius: 12px;
    font-size: 15px;
    font-family: 'SF Mono', 'Monaco', 'Inconsolata', 'Fira Code', monospace;
    transition: all 0.2s ease;
    box-sizing: border-box;
  }

  .credential-input:focus {
    outline: none;
    border-color: #1a73e8;
    box-shadow: 0 0 0 3px rgba(26, 115, 232, 0.1);
  }

  .credential-input::placeholder {
    color: #9aa0a6;
    font-style: italic;
  }

  .visibility-toggle {
    position: absolute;
    right: 12px;
    top: 50%;
    transform: translateY(-50%);
    background: none;
    border: none;
    font-size: 18px;
    cursor: pointer;
    padding: 5px;
    border-radius: 4px;
  }

  .visibility-toggle:hover {
    background: #f1f3f4;
  }

  .input-hint {
    display: block;
    margin-top: 8px;
    color: #5f6368;
    font-size: 13px;
  }

  .input-hint a {
    color: #1a73e8;
    text-decoration: none;
    font-weight: 500;
  }

  .input-hint a:hover {
    text-decoration: underline;
  }

  .button-group {
    display: flex;
    gap: 15px;
    margin-top: 30px;
    flex-wrap: wrap;
  }

  .auth-button {
    flex: 1;
    min-width: 200px;
    padding: 0;
    border: none;
    border-radius: 12px;
    font-size: 15px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 50px;
    background: #f1f3f4;
    color: #3c4043;
    box-shadow: 0 2px 6px rgba(0, 0, 0, 0.05);
  }

  .auth-button.primary {
    background: #1a73e8;
    color: white;
    box-shadow: 0 4px 12px rgba(26, 115, 232, 0.3);
  }

  .auth-button:hover:not(:disabled) {
    transform: translateY(-2px);
    box-shadow: 0 6px 20px rgba(0, 0, 0, 0.1);
  }

  .auth-button.primary:hover:not(:disabled) {
    background: #1557b0;
    box-shadow: 0 6px 20px rgba(26, 115, 232, 0.4);
  }

  .auth-button:disabled {
    background: #f1f3f4;
    color: #9aa0a6;
    cursor: not-allowed;
    transform: none;
    box-shadow: none;
  }

  .button-content {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .spinner {
    width: 20px;
    height: 20px;
    border: 3px solid rgba(255, 255, 255, 0.3);
    border-top: 3px solid white;
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    0% { transform: rotate(0deg); }
    100% { transform: rotate(360deg); }
  }

  .models-info {
    background: #e6f4ea;
    border: 1px solid #c3e6cb;
    border-radius: 12px;
    padding: 20px;
    margin-top: 25px;
  }

  .models-info h4 {
    margin: 0 0 15px 0;
    color: #155724;
    font-size: 18px;
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .models-list {
    display: flex;
    flex-wrap: wrap;
    gap: 10px;
  }

  .model-tag {
    background: #4caf50;
    color: white;
    padding: 6px 12px;
    border-radius: 20px;
    font-size: 13px;
    font-weight: 500;
  }

  .model-tag.more {
    background: #9e9e9e;
  }

  .auth-status {
    margin-top: 25px;
    padding: 20px;
    border-radius: 12px;
    display: flex;
    align-items: flex-start;
    gap: 15px;
    animation: slideIn 0.3s ease;
    border: 1px solid;
  }

  .status-icon {
    font-size: 20px;
    flex-shrink: 0;
  }

  .status-message {
    flex: 1;
    line-height: 1.5;
    font-size: 15px;
  }

  .auth-status.success {
    background: #e6f4ea;
    color: #137333;
    border-color: #ceead6;
  }

  .auth-status.error {
    background: #fce8e6;
    color: #c5221f;
    border-color: #f9dedc;
  }

  .auth-status.warning {
    background: #fef7e0;
    color: #b06000;
    border-color: #feefc3;
  }

  .auth-status.info {
    background: #e8f0fe;
    color: #1967d2;
    border-color: #d2e3fc;
  }

  @keyframes slideIn {
    from {
      opacity: 0;
      transform: translateY(-10px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  @media (max-width: 768px) {
    .google-auth-setup {
      margin: 15px;
      padding: 25px;
    }

    .header-content {
      flex-direction: column;
      gap: 15px;
      align-items: stretch;
    }

    .auth-method-selector {
      flex-direction: column;
      gap: 12px;
    }

    .button-group {
      flex-direction: column;
    }

    .auth-button {
      min-width: auto;
    }

    .info-box {
      flex-direction: column;
      gap: 10px;
    }
  }

  @media (max-width: 480px) {
    .google-auth-setup {
      margin: 10px;
      padding: 20px;
    }

    .step {
      flex-direction: column;
      gap: 8px;
    }

    .step-number {
      align-self: flex-start;
    }
  }

  .clear-btn {
    background: #ffffff;
    border: 1px solid #dadce0;
    border-radius: 8px;
    padding: 8px 12px;
    cursor: pointer;
  }
  .clear-btn:hover {
    background: #f8f9fa;
  }
</style>