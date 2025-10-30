<script lang="ts">
import { createEventDispatcher, onDestroy, onMount } from "svelte";
import { writable } from "svelte/store";
import { isTauri } from "$lib/utils/env";
import { tauriInvoke } from "$lib/utils/tauri";

const dispatch = createEventDispatcher();

export const provider: "gemini" | "qwen" | "local" = "gemini";

type AuthStatus = {
  message: string;
  type: "success" | "error" | "info" | "warning";
};

const authStatus = writable<AuthStatus>({ message: "", type: "info" });
let isAuthenticating = false;
let hasExistingSession = false;

// Gemini specific (OAuth-only)
const apiKey = "";
const activeAuthTab: "oauth" = "oauth";

// Qwen Device Flow specific
let deviceCode = "";
let userCode = "";
let verificationUri = "";
let expiresIn: number | null = null;
let remainingSec = 0;
let pollTimer: number | null = null;
let countdownTimer: number | null = null;

function clearTimers(): void {
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
});

async function checkAuthStatus() {
  if (!isTauri) return;

  try {
    if (provider === "gemini") {
      const result = await tauriInvoke("get_auth_status");
      hasExistingSession =
        result &&
        typeof result === "string" &&
        result.includes("authenticated");
    } else if (provider === "qwen") {
      const result = await tauriInvoke("qwen_get_auth_status");
      hasExistingSession =
        result &&
        typeof result === "string" &&
        /auth/i.test(result) &&
        !/not\s+auth/i.test(result);
    }

    if (hasExistingSession) {
      authStatus.set({
        message: `${provider.charAt(0).toUpperCase() + provider.slice(1)} session is active.`,
        type: "success",
      });
    }
  } catch (e) {
    hasExistingSession = false;
  }
}

async function handleGoogleOAuth() {
  if (!isTauri) {
    authStatus.set({
      message: "Desktop application required for authentication",
      type: "error",
    });
    return;
  }

  isAuthenticating = true;
  authStatus.set({
    message: "Starting Google OAuth2 authentication...",
    type: "info",
  });

  try {
    const result = await tauriInvoke("authenticate_google_command");
    authStatus.set({
      message: "OAuth2 authentication completed successfully!",
      type: "success",
    });
    hasExistingSession = true;
    dispatch("authComplete");
  } catch (error) {
    authStatus.set({
      message: `OAuth2 authentication failed: ${error}`,
      type: "error",
    });
  } finally {
    isAuthenticating = false;
  }
}

// API key flow removed (OAuth-only)

async function startQwenDeviceFlow() {
  if (!isTauri) {
    authStatus.set({
      message: "Desktop application required for authentication",
      type: "error",
    });
    return;
  }

  isAuthenticating = true;
  clearTimers();
  authStatus.set({
    message: "Starting Qwen device authentication...",
    type: "info",
  });

  try {
    const result = await tauriInvoke("qwen_start_device_auth");
    const data = typeof result === "string" ? JSON.parse(result) : result;

    deviceCode = data.device_code || "";
    userCode = data.user_code || "";
    verificationUri = data.verification_uri || "";
    expiresIn = data.expires_in || 300;
    remainingSec = expiresIn as number;

    authStatus.set({
      message:
        "Device flow started. Please complete authentication in your browser.",
      type: "info",
    });

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
    if (
      errorMessage.includes("QWEN_DEVICE_AUTH_URL") ||
      errorMessage.includes("Missing env var")
    ) {
      errorMessage =
        "Qwen OAuth configuration missing. Please configure environment variables in src-tauri/.env file. See docs/QWEN.md for setup instructions.";
    }

    authStatus.set({ message: errorMessage, type: "error" });
    isAuthenticating = false;
  }
}

function startPolling() {
  const poll = async () => {
    try {
      const result = await tauriInvoke("qwen_poll_device_auth");
      const data = typeof result === "string" ? JSON.parse(result) : result;

      if (data.status === "success") {
        clearTimers();
        isAuthenticating = false;
        hasExistingSession = true;
        authStatus.set({
          message: "Qwen authentication completed successfully!",
          type: "success",
        });
        dispatch("authComplete");
        return;
      } else if (data.status === "pending") {
        // Continue polling
        pollTimer = setTimeout(poll, 5000) as unknown as number;
      } else if (data.status === "slow_down") {
        // Slow down polling
        pollTimer = setTimeout(poll, 10000) as unknown as number;
      } else {
        throw new Error(data.message || "Authentication failed");
      }
    } catch (error) {
      clearTimers();
      isAuthenticating = false;
      authStatus.set({
        message: `Authentication failed: ${error}`,
        type: "error",
      });
    }
  };

  pollTimer = setTimeout(poll, 5000) as unknown as number;
}

function startCountdown() {
  countdownTimer = setInterval(() => {
    remainingSec--;
    if (remainingSec <= 0) {
      clearTimers();
      isAuthenticating = false;
      authStatus.set({
        message: "Authentication timeout. Please try again.",
        type: "warning",
      });
    }
  }, 1000) as unknown as number;
}

async function clearSession() {
  if (!isTauri) {
    authStatus.set({
      message: "Clear session is a desktop-only operation",
      type: "info",
    });
    return;
  }

  try {
    if (provider === "gemini") {
      await tauriInvoke("clear_google_auth");
    } else if (provider === "qwen") {
      await tauriInvoke("qwen_clear_auth");
    }

    hasExistingSession = false;
    authStatus.set({ message: "Session cleared successfully", type: "info" });
  } catch (error) {
    authStatus.set({
      message: `Failed to clear session: ${error}`,
      type: "error",
    });
  }
}

onMount(() => {
  checkAuthStatus();
});

function formatTime(seconds: number): string {
  const mins = Math.floor(seconds / 60);
  const secs = seconds % 60;
  return `${mins}:${secs.toString().padStart(2, "0")}`;
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
    margin: 0 auto;
  }
</style>
