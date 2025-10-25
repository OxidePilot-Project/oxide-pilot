<script lang="ts">
import { createEventDispatcher, onMount, onDestroy } from "svelte";
import { writable } from "svelte/store";
import { isTauri } from "$lib/utils/env";
import { tauriInvoke } from "$lib/utils/tauri";

const dispatch = createEventDispatcher();

export let provider: 'gemini' | 'qwen' | 'local' = 'gemini';

type AuthStatus = {
  message: string;
  type: "success" | "error" | "info" | "warning";
};

const authStatus = writable<AuthStatus>({ message: "", type: "info" });
let isAuthenticating = false;
let hasExistingSession = false;

// Gemini specific (OAuth-only)
let apiKey = "";
let activeAuthTab: 'oauth' = 'oauth';

// Qwen Device Flow specific
let deviceCode = "";
let userCode = "";
let verificationUri = "";
let expiresIn: number | null = null;
let remainingSec = 0;
let pollTimer: number | null = null;
let countdownTimer: number | null = null;

// Event listeners
let unlisten: null | (() => void | Promise<void>) = null;

function clearTimers() {
  if (pollTimer !== null) {
    clearTimeout(pollTimer);
    pollTimer = null;
  }
  if (countdownTimer !== null) {
    clearInterval(countdownTimer);
    countdownTimer = null;
  }
}

onDestroy(() => {
  clearTimers();
  try { if (unlisten) { unlisten(); } } catch {}
});

async function checkAuthStatus() {
  if (!isTauri) return;

  try {
    if (provider === 'gemini') {
      const result = await tauriInvoke("get_auth_status");
      hasExistingSession = result && typeof result === 'string' && result.includes('authenticated');
    } else if (provider === 'qwen') {
      const result = await tauriInvoke("qwen_get_auth_status");
      hasExistingSession = result && typeof result === 'string' && /auth/i.test(result) && !/not\s+auth/i.test(result);
    }

    if (hasExistingSession) {
      authStatus.set({ message: `${provider.charAt(0).toUpperCase() + provider.slice(1)} session is active.`, type: "success" });
    }
  } catch (e) {
    hasExistingSession = false;
  }
}

async function handleGoogleOAuth() {
  if (!isTauri) {
    authStatus.set({ message: "Desktop application required for authentication", type: "error" });
    return;
  }

  isAuthenticating = true;
  authStatus.set({ message: "Starting Google OAuth2 authentication...", type: "info" });

  try {
    const result = await tauriInvoke("authenticate_google_command");
    authStatus.set({ message: "OAuth2 authentication completed successfully!", type: "success" });
    hasExistingSession = true;
    dispatch("authComplete");
  } catch (error) {
    authStatus.set({ message: `OAuth2 authentication failed: ${error}`, type: "error" });
  } finally {
    isAuthenticating = false;
  }
}

// API key flow removed (OAuth-only)

async function startQwenDeviceFlow() {
  if (!isTauri) {
    authStatus.set({ message: "Desktop application required for authentication", type: "error" });
    return;
  }

  isAuthenticating = true;
  clearTimers();
  authStatus.set({ message: "Starting Qwen device authentication...", type: "info" });

  try {
    const result = await tauriInvoke("qwen_start_device_auth");
    const data = typeof result === 'string' ? JSON.parse(result) : result;

    deviceCode = data.device_code || "";
    userCode = data.user_code || "";
    verificationUri = data.verification_uri || "";
    expiresIn = data.expires_in || 300;
    remainingSec = expiresIn;

    authStatus.set({ message: "Device flow started. Please complete authentication in your browser.", type: "info" });

    // Open verification URL
    if (verificationUri) {
      await tauriInvoke("open_url", { url: verificationUri });
    }

    // Start polling
    startPolling();
    startCountdown();
  } catch (error) {
    let errorMessage = `Failed to start device flow: ${error}`;

    // Check for specific environment variable errors
    if (errorMessage.includes("QWEN_DEVICE_AUTH_URL") || errorMessage.includes("Missing env var")) {
      errorMessage = "Qwen OAuth configuration missing. Please configure environment variables in src-tauri/.env file. See docs/QWEN.md for setup instructions.";
    }

    authStatus.set({ message: errorMessage, type: "error" });
    isAuthenticating = false;
  }
}

function startPolling() {
  const poll = async () => {
    try {
      const result = await tauriInvoke("qwen_poll_device_auth");
      const data = typeof result === 'string' ? JSON.parse(result) : result;

      if (data.status === 'success') {
        clearTimers();
        isAuthenticating = false;
        hasExistingSession = true;
        authStatus.set({ message: "Qwen authentication completed successfully!", type: "success" });
        dispatch("authComplete");
        return;
      } else if (data.status === 'pending') {
        // Continue polling
        pollTimer = setTimeout(poll, 5000);
      } else if (data.status === 'slow_down') {
        // Slow down polling
        pollTimer = setTimeout(poll, 10000);
      } else {
        throw new Error(data.message || 'Authentication failed');
      }
    } catch (error) {
      clearTimers();
      isAuthenticating = false;
      authStatus.set({ message: `Authentication failed: ${error}`, type: "error" });
    }
  };

  pollTimer = setTimeout(poll, 5000);
}

function startCountdown() {
  countdownTimer = setInterval(() => {
    remainingSec--;
    if (remainingSec <= 0) {
      clearTimers();
      isAuthenticating = false;
      authStatus.set({ message: "Authentication timeout. Please try again.", type: "warning" });
    }
  }, 1000);
}

async function clearSession() {
  if (!isTauri) {
    authStatus.set({ message: "Clear session is a desktop-only operation", type: "info" });
    return;
  }

  try {
    if (provider === 'gemini') {
      await tauriInvoke("clear_google_auth");
    } else if (provider === 'qwen') {
      await tauriInvoke("qwen_clear_auth");
    }

    hasExistingSession = false;
    authStatus.set({ message: "Session cleared successfully", type: "info" });
  } catch (error) {
    authStatus.set({ message: `Failed to clear session: ${error}`, type: "error" });
  }
}

onMount(() => {
  checkAuthStatus();
});

function formatTime(seconds: number): string {
  const mins = Math.floor(seconds / 60);
  const secs = seconds % 60;
  return `${mins}:${secs.toString().padStart(2, '0')}`;
}
</script>

<div class="modern-auth-setup">
  <div class="auth-header">
    <div class="provider-info">
      {#if provider === 'gemini'}
        <div class="provider-icon">🌟</div>
        <div>
          <h2>Google Gemini</h2>
          <p>Connect to Google's powerful AI models</p>
        </div>
      {:else if provider === 'qwen'}
        <div class="provider-icon">🤖</div>
        <div>
          <h2>Qwen AI</h2>
          <p>Secure OAuth2 device authentication</p>
        </div>
      {:else}
        <div class="provider-icon">💻</div>
        <div>
          <h2>Local Models</h2>
          <p>Run AI models locally on your device</p>
        </div>
      {/if}
    </div>

    {#if hasExistingSession}
      <div class="session-status">
        <span class="status-indicator active"></span>
        <span>Connected</span>
        <button class="clear-session-btn" on:click={clearSession}>
          Disconnect
        </button>
      </div>
    {:else}
      <div class="session-status">
        <span class="status-indicator inactive"></span>
        <span>Not connected</span>
      </div>
    {/if}
  </div>

  {#if !hasExistingSession}
    <div class="auth-content">
      {#if provider === 'gemini'}
        <div class="auth-method">
          <div class="method-header">
            <h3>🔑 Google Authentication</h3>
            <p>Choose your preferred authentication method</p>
          </div>

          <!-- OAuth-only -->
          <div class="auth-tabs">
            <button
              class="tab-button active"
              disabled
            >
              🔐 OAuth2 (Recommended)
            </button>
          </div>

          <!-- Tab Content -->
          <div class="tab-content">
            {#if activeAuthTab === 'oauth'}
              <div class="auth-option">
                <div class="option-header">
                  <h4>🔐 OAuth2 (Recommended)</h4>
                  <p>Secure authentication without storing credentials</p>
                </div>
                <button
                  class="auth-button primary"
                  on:click={handleGoogleOAuth}
                  disabled={isAuthenticating}
                >
                  {#if isAuthenticating}
                    <div class="spinner"></div>
                    Authenticating...
                  {:else}
                    🔗 Connect with Google OAuth2
                  {/if}
                </button>
              </div>
            {/if}
          </div>
        </div>
      {:else if provider === 'qwen'}
        <div class="auth-method">
          <div class="method-header">
            <h3>🔐 OAuth2 Device Flow</h3>
            <p>Secure authentication without storing credentials</p>
          </div>

          {#if isAuthenticating && userCode}
            <div class="device-flow-active">
              <div class="verification-info">
                <h4>Complete authentication in your browser</h4>
                <div class="user-code">
                  <span class="code-label">Enter this code:</span>
                  <span class="code-value">{userCode}</span>
                </div>
                <div class="verification-link">
                  <a href={verificationUri} target="_blank" class="open-browser-btn">
                    🌐 Open Browser
                  </a>
                </div>
                {#if remainingSec > 0}
                  <div class="countdown">
                    Time remaining: {formatTime(remainingSec)}
                  </div>
                {/if}
              </div>
            </div>
          {:else}
            <div class="qwen-setup-info">
              <div class="setup-notice">
                <h4>📋 Setup Required</h4>
                <p>To use Qwen authentication, you need to configure environment variables in <code>src-tauri/.env</code></p>
                <div class="required-vars">
                  <strong>Required variables:</strong>
                  <ul>
                    <li><code>QWEN_DEVICE_AUTH_URL</code></li>
                    <li><code>QWEN_DEVICE_TOKEN_URL</code></li>
                    <li><code>QWEN_CLIENT_ID</code></li>
                    <li><code>QWEN_CLIENT_SECRET</code></li>
                  </ul>
                </div>
                <p class="setup-link">
                  See <a href="docs/ENVIRONMENT_SETUP.md" target="_blank">Environment Setup Guide</a> for detailed instructions.
                </p>
              </div>
              <button
                class="auth-button primary"
                on:click={startQwenDeviceFlow}
                disabled={isAuthenticating}
              >
                {#if isAuthenticating}
                  <div class="spinner"></div>
                  Starting...
                {:else}
                  Start OAuth2 Authentication
                {/if}
              </button>
            </div>
          {/if}
        </div>
      {:else}
        <div class="auth-method">
          <div class="method-header">
            <h3>💻 Local Models</h3>
            <p>Run AI models directly on your device</p>
          </div>
          <div class="local-info">
            <p>Local model support is coming soon. This will allow you to run AI models directly on your device without internet connectivity.</p>
            <button class="auth-button" disabled>
              Coming Soon
            </button>
          </div>
        </div>
      {/if}
    </div>
  {:else}
    <div class="connected-state">
      <div class="success-icon">✅</div>
      <h3>Successfully Connected!</h3>
      <p>Your {provider.charAt(0).toUpperCase() + provider.slice(1)} authentication is active and ready to use.</p>
      <button class="auth-button primary" on:click={() => dispatch("authComplete")}>
        Continue to Dashboard
      </button>
    </div>
  {/if}

  {#if $authStatus.message}
    <div class="status-message {$authStatus.type}">
      <div class="status-icon">
        {#if $authStatus.type === 'success'}✅
        {:else if $authStatus.type === 'error'}❌
        {:else if $authStatus.type === 'warning'}⚠️
        {:else}ℹ️{/if}
      </div>
      <span>{$authStatus.message}</span>
    </div>
  {/if}
</div>

<style>
  .modern-auth-setup {
    max-width: 600px;
    width: 100%;
    margin: 0 auto;
    padding: clamp(16px, 2vh, 24px);
    background: white;
    border-radius: 12px;
    box-shadow: 0 6px 24px rgba(0, 0, 0, 0.1);
    border: 1px solid #e5e7eb;
    box-sizing: border-box;
    overflow: hidden;
  }

  .auth-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 2rem;
    padding-bottom: 1.5rem;
    border-bottom: 1px solid #f3f4f6;
  }

  .provider-info {
    display: flex;
    align-items: center;
    gap: 1rem;
  }

  .provider-icon {
    font-size: 2.5rem;
  }

  .provider-info h2 {
    margin: 0;
    font-size: 1.5rem;
    font-weight: 700;
    color: #111827;
  }

  .provider-info p {
    margin: 0.25rem 0 0 0;
    color: #6b7280;
    font-size: 0.875rem;
  }

  .session-status {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.875rem;
  }

  .status-indicator {
    width: 8px;
    height: 8px;
    border-radius: 50%;
  }

  .status-indicator.active {
    background: #10b981;
  }

  .status-indicator.inactive {
    background: #ef4444;
  }

  .clear-session-btn {
    background: #f3f4f6;
    border: 1px solid #d1d5db;
    border-radius: 6px;
    padding: 0.25rem 0.75rem;
    font-size: 0.75rem;
    cursor: pointer;
    transition: all 0.2s;
  }

  .clear-session-btn:hover {
    background: #e5e7eb;
  }

  .auth-content {
    margin-bottom: 1.5rem;
  }

  .auth-method {
    background: #f9fafb;
    border-radius: 12px;
    padding: 1.5rem;
    border: 1px solid #e5e7eb;
    width: 100%;
    box-sizing: border-box;
  }

  .method-header {
    margin-bottom: 1.5rem;
  }

  .method-header h3 {
    margin: 0 0 0.5rem 0;
    font-size: 1.125rem;
    font-weight: 600;
    color: #111827;
  }

  .method-header p {
    margin: 0;
    color: #6b7280;
    font-size: 0.875rem;
  }

  .auth-tabs {
    display: flex;
    background: #f8fafc;
    border-radius: 10px;
    padding: 4px;
    margin-bottom: 1.5rem;
    border: 1px solid #e2e8f0;
  }

  .auth-tabs .tab-button {
    flex: 1;
    padding: 0.75rem 1rem;
    border: none;
    border-radius: 8px;
    background: transparent;
    color: #64748b;
    font-size: 0.875rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
  }

  .auth-tabs .tab-button:hover {
    background: rgba(59, 130, 246, 0.1);
    color: #3b82f6;
  }

  .auth-tabs .tab-button.active {
    background: white;
    color: #3b82f6;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
    font-weight: 600;
  }

  .tab-content {
    animation: fadeIn 0.3s ease-in-out;
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
      transform: translateY(10px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .auth-options {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  .auth-option {
    background: white;
    border-radius: 8px;
    padding: 1.25rem;
    border: 1px solid #e5e7eb;
    transition: all 0.2s ease;
  }

  .auth-option:hover {
    border-color: #3b82f6;
    box-shadow: 0 4px 12px rgba(59, 130, 246, 0.1);
  }

  .option-header {
    margin-bottom: 1rem;
  }

  .option-header h4 {
    margin: 0 0 0.25rem 0;
    color: #111827;
    font-size: 1rem;
    font-weight: 600;
  }

  .option-header p {
    margin: 0;
    color: #6b7280;
    font-size: 0.8rem;
  }

  .divider {
    display: flex;
    align-items: center;
    text-align: center;
    margin: 0.5rem 0;
  }

  .divider::before,
  .divider::after {
    content: '';
    flex: 1;
    height: 1px;
    background: #e5e7eb;
  }

  .divider span {
    padding: 0 1rem;
    color: #6b7280;
    font-size: 0.75rem;
    font-weight: 500;
    background: #f9fafb;
  }

  .input-group {
    margin-bottom: 1.5rem;
  }

  .input-group label {
    display: block;
    margin-bottom: 0.5rem;
    font-weight: 500;
    color: #374151;
    font-size: 0.875rem;
  }

  .input-wrapper {
    position: relative;
  }

  .auth-input {
    width: 100%;
    padding: 0.75rem 3rem 0.75rem 1rem;
    border: 2px solid #d1d5db;
    border-radius: 8px;
    font-size: 0.875rem;
    font-family: 'SF Mono', monospace;
    transition: border-color 0.2s;
    box-sizing: border-box;
  }

  .auth-input:focus {
    outline: none;
    border-color: #3b82f6;
    box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
  }

  .toggle-visibility {
    position: absolute;
    right: 0.75rem;
    top: 50%;
    transform: translateY(-50%);
    background: none;
    border: none;
    cursor: pointer;
    padding: 0.25rem;
    border-radius: 4px;
  }

  .toggle-visibility:hover {
    background: #f3f4f6;
  }

  .input-hint {
    display: block;
    margin-top: 0.5rem;
    color: #6b7280;
    font-size: 0.75rem;
  }

  .input-hint a {
    color: #3b82f6;
    text-decoration: none;
  }

  .input-hint a:hover {
    text-decoration: underline;
  }

  .auth-button {
    width: 100%;
    padding: 0.875rem 1.5rem;
    border: none;
    border-radius: 8px;
    font-size: 0.875rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    background: #f3f4f6;
    color: #374151;
  }

  .auth-button.primary {
    background: #3b82f6;
    color: white;
  }

  .auth-button.secondary {
    background: #f8fafc;
    color: #374151;
    border: 1px solid #d1d5db;
  }

  .auth-button:hover:not(:disabled) {
    transform: translateY(-1px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  }

  .auth-button.primary:hover:not(:disabled) {
    background: #2563eb;
  }

  .auth-button.secondary:hover:not(:disabled) {
    background: #f1f5f9;
    border-color: #9ca3af;
  }

  .auth-button:disabled {
    background: #f3f4f6;
    color: #9ca3af;
    cursor: not-allowed;
    transform: none;
    box-shadow: none;
  }

  .spinner {
    width: 16px;
    height: 16px;
    border: 2px solid rgba(255, 255, 255, 0.3);
    border-top: 2px solid white;
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    0% { transform: rotate(0deg); }
    100% { transform: rotate(360deg); }
  }

  .device-flow-active {
    text-align: center;
  }

  .verification-info h4 {
    margin: 0 0 1rem 0;
    color: #111827;
  }

  .user-code {
    background: #eff6ff;
    border: 2px solid #3b82f6;
    border-radius: 12px;
    padding: 1rem;
    margin: 1rem 0;
  }

  .code-label {
    display: block;
    font-size: 0.875rem;
    color: #1e40af;
    margin-bottom: 0.5rem;
  }

  .code-value {
    font-size: 1.5rem;
    font-weight: 700;
    font-family: 'SF Mono', monospace;
    color: #1e40af;
    letter-spacing: 0.1em;
  }

  .open-browser-btn {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.75rem 1.5rem;
    background: #3b82f6;
    color: white;
    text-decoration: none;
    border-radius: 8px;
    font-weight: 500;
    transition: all 0.2s;
  }

  .open-browser-btn:hover {
    background: #2563eb;
    transform: translateY(-1px);
  }

  .countdown {
    margin-top: 1rem;
    font-size: 0.875rem;
    color: #6b7280;
  }

  .connected-state {
    text-align: center;
    padding: 2rem 0;
  }

  .success-icon {
    font-size: 3rem;
    margin-bottom: 1rem;
  }

  .connected-state h3 {
    margin: 0 0 0.5rem 0;
    color: #111827;
  }

  .connected-state p {
    margin: 0 0 1.5rem 0;
    color: #6b7280;
  }

  .local-info {
    text-align: center;
    padding: 1rem 0;
  }

  .local-info p {
    margin-bottom: 1.5rem;
    color: #6b7280;
    line-height: 1.5;
  }

  .status-message {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 1rem;
    border-radius: 8px;
    margin-top: 1rem;
    font-size: 0.875rem;
    border: 1px solid;
  }

  .status-message.success {
    background: #f0fdf4;
    color: #166534;
    border-color: #bbf7d0;
  }

  .status-message.error {
    background: #fef2f2;
    color: #dc2626;
    border-color: #fecaca;
  }

  .status-message.warning {
    background: #fffbeb;
    color: #d97706;
    border-color: #fed7aa;
  }

  .status-message.info {
    background: #eff6ff;
    color: #2563eb;
    border-color: #bfdbfe;
  }

  .qwen-setup-info {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .setup-notice {
    background: #f8fafc;
    border: 1px solid #e2e8f0;
    border-radius: 8px;
    padding: 1rem;
    margin-bottom: 1rem;
  }

  .setup-notice h4 {
    margin: 0 0 0.5rem 0;
    color: #1e293b;
    font-size: 0.875rem;
    font-weight: 600;
  }

  .setup-notice p {
    margin: 0 0 0.75rem 0;
    color: #64748b;
    font-size: 0.8rem;
    line-height: 1.4;
  }

  .setup-notice code {
    background: #e2e8f0;
    padding: 0.125rem 0.25rem;
    border-radius: 3px;
    font-size: 0.75rem;
    font-family: 'SF Mono', monospace;
  }

  .required-vars {
    margin: 0.75rem 0;
  }

  .required-vars strong {
    display: block;
    margin-bottom: 0.5rem;
    color: #374151;
    font-size: 0.8rem;
  }

  .required-vars ul {
    margin: 0;
    padding-left: 1.25rem;
    list-style-type: disc;
  }

  .required-vars li {
    margin: 0.25rem 0;
    color: #64748b;
    font-size: 0.75rem;
  }

  .required-vars code {
    background: #f1f5f9;
    color: #1e40af;
    padding: 0.125rem 0.25rem;
    border-radius: 3px;
    font-size: 0.7rem;
    font-family: 'SF Mono', monospace;
  }

  .setup-link {
    margin: 0.75rem 0 0 0;
    font-size: 0.8rem;
  }

  .setup-link a {
    color: #3b82f6;
    text-decoration: none;
    font-weight: 500;
  }

  .setup-link a:hover {
    text-decoration: underline;
  }

  @media (max-width: 640px) {
    .modern-auth-setup {
      margin: 1rem;
      padding: 1.5rem;
    }

    .auth-header {
      flex-direction: column;
      align-items: flex-start;
      gap: 1rem;
    }

    .provider-info {
      flex-direction: column;
      text-align: center;
      gap: 0.5rem;
    }

    .session-status {
      align-self: flex-end;
    }

    .auth-tabs {
      flex-direction: column;
      gap: 4px;
    }

    .auth-tabs .tab-button {
      padding: 0.875rem 1rem;
      font-size: 0.8rem;
    }

    .qwen-setup-info {
      gap: 0.75rem;
    }

    .setup-notice {
      padding: 0.75rem;
    }
  }
</style>
