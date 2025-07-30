<script lang="ts">
import { invoke } from "@tauri-apps/api/tauri";
import { createEventDispatcher } from "svelte";
import { writable } from "svelte/store";

const dispatch = createEventDispatcher();

const clientId = "";
const clientSecret = "";
const authStatus = writable<{
  message: string;
  type: "success" | "error" | "info" | "warning";
}>({ message: "", type: "info" });
let _isAuthenticating = false;
let showInstructions = false;

async function _saveClientCredentials() {
  if (!clientId.trim() || !clientSecret.trim()) {
    authStatus.set({
      message: "Please enter both Client ID and Client Secret",
      type: "warning",
    });
    return;
  }

  authStatus.set({ message: "Saving client credentials...", type: "info" });
  try {
    await invoke("set_google_client_credentials", { clientId, clientSecret });
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

async function _startGoogleAuth() {
  if (!clientId.trim() || !clientSecret.trim()) {
    authStatus.set({
      message: "Please save client credentials first",
      type: "warning",
    });
    return;
  }

  _isAuthenticating = true;
  authStatus.set({
    message: "Initiating Google authentication flow...",
    type: "info",
  });

  try {
    const _accessToken = await invoke("authenticate_google_command");
    authStatus.set({
      message: `Google authentication successful! System is now ready to use.`,
      type: "success",
    });

    // Dispatch event to parent component
    dispatch("authComplete");
  } catch (error) {
    authStatus.set({
      message: `Error during Google authentication: ${error}`,
      type: "error",
    });
    console.error("Error during Google authentication:", error);
  } finally {
    _isAuthenticating = false;
  }
}

function _toggleInstructions() {
  showInstructions = !showInstructions;
}
</script>

<div class="google-auth-setup">
  <div class="header">
    <h2>🔐 Google API Configuration</h2>
    <button class="help-button" on:click={toggleInstructions}>
      {showInstructions ? '❌' : '❓'} {showInstructions ? 'Hide' : 'Help'}
    </button>
  </div>

  {#if showInstructions}
    <div class="instructions">
      <h3>📋 Setup Instructions</h3>
      <ol>
        <li>Go to the <a href="https://console.cloud.google.com/" target="_blank">Google Cloud Console</a></li>
        <li>Create a new project or select an existing one</li>
        <li>Enable the following APIs:
          <ul>
            <li>Google AI Generative Language API</li>
            <li>Google Speech-to-Text API</li>
            <li>Google Text-to-Speech API</li>
          </ul>
        </li>
        <li>Go to "Credentials" → "Create Credentials" → "OAuth 2.0 Client IDs"</li>
        <li>Set application type to "Desktop application"</li>
        <li>Copy the Client ID and Client Secret below</li>
      </ol>
    </div>
  {/if}

  <div class="form-section">
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
      <input
        type="password"
        id="clientSecret"
        bind:value={clientSecret}
        placeholder="GOCSPX-xxxxxxxxxxxxxxxxxxxxxxxx"
        class="credential-input"
      />
    </div>

    <div class="button-group">
      <button
        class="save-button"
        on:click={saveClientCredentials}
        disabled={!clientId.trim() || !clientSecret.trim()}
      >
        💾 Save Credentials
      </button>

      <button
        class="auth-button"
        on:click={startGoogleAuth}
        disabled={isAuthenticating || !clientId.trim() || !clientSecret.trim()}
      >
        {#if isAuthenticating}
          🔄 Authenticating...
        {:else}
          🚀 Authenticate with Google
        {/if}
      </button>
    </div>
  </div>

  {#if $authStatus.message}
    <div class="auth-status {$authStatus.type}">
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
    border-radius: 12px;
    padding: 30px;
    max-width: 600px;
    margin: 0 auto;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.1);
    border: 1px solid #e1e8ed;
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 25px;
    padding-bottom: 15px;
    border-bottom: 2px solid #f1f3f4;
  }

  h2 {
    color: #1a73e8;
    margin: 0;
    font-size: 24px;
    font-weight: 600;
  }

  .help-button {
    background: #f8f9fa;
    border: 1px solid #dadce0;
    border-radius: 20px;
    padding: 8px 16px;
    font-size: 14px;
    color: #5f6368;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .help-button:hover {
    background: #e8f0fe;
    border-color: #1a73e8;
    color: #1a73e8;
  }

  .instructions {
    background: #f8f9fa;
    border-radius: 8px;
    padding: 20px;
    margin-bottom: 25px;
    border-left: 4px solid #1a73e8;
  }

  .instructions h3 {
    color: #1a73e8;
    margin: 0 0 15px 0;
    font-size: 18px;
  }

  .instructions ol {
    margin: 0;
    padding-left: 20px;
  }

  .instructions li {
    margin-bottom: 8px;
    line-height: 1.5;
    color: #3c4043;
  }

  .instructions ul {
    margin: 5px 0;
    padding-left: 20px;
  }

  .instructions a {
    color: #1a73e8;
    text-decoration: none;
  }

  .instructions a:hover {
    text-decoration: underline;
  }

  .form-section {
    margin-top: 20px;
  }

  .input-group {
    margin-bottom: 20px;
  }

  label {
    display: flex;
    align-items: center;
    margin-bottom: 8px;
    font-weight: 500;
    color: #3c4043;
  }

  .label-text {
    font-size: 14px;
  }

  .required {
    color: #ea4335;
    margin-left: 4px;
    font-weight: bold;
  }

  .credential-input {
    width: 100%;
    padding: 12px 16px;
    border: 2px solid #dadce0;
    border-radius: 8px;
    font-size: 14px;
    font-family: 'Courier New', monospace;
    transition: border-color 0.2s ease;
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

  .button-group {
    display: flex;
    gap: 12px;
    margin-top: 25px;
    flex-wrap: wrap;
  }

  .save-button, .auth-button {
    flex: 1;
    min-width: 200px;
    padding: 12px 24px;
    border: none;
    border-radius: 8px;
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
  }

  .save-button {
    background: #34a853;
    color: white;
  }

  .save-button:hover:not(:disabled) {
    background: #2d8f47;
    transform: translateY(-1px);
    box-shadow: 0 4px 12px rgba(52, 168, 83, 0.3);
  }

  .auth-button {
    background: #1a73e8;
    color: white;
  }

  .auth-button:hover:not(:disabled) {
    background: #1557b0;
    transform: translateY(-1px);
    box-shadow: 0 4px 12px rgba(26, 115, 232, 0.3);
  }

  .save-button:disabled, .auth-button:disabled {
    background: #f1f3f4;
    color: #9aa0a6;
    cursor: not-allowed;
    transform: none;
    box-shadow: none;
  }

  .auth-status {
    margin-top: 20px;
    padding: 16px;
    border-radius: 8px;
    display: flex;
    align-items: flex-start;
    gap: 12px;
    animation: slideIn 0.3s ease;
  }

  .status-icon {
    font-size: 18px;
    flex-shrink: 0;
  }

  .status-message {
    flex: 1;
    line-height: 1.5;
  }

  .auth-status.success {
    background: #e6f4ea;
    color: #137333;
    border: 1px solid #ceead6;
  }

  .auth-status.error {
    background: #fce8e6;
    color: #c5221f;
    border: 1px solid #f9dedc;
  }

  .auth-status.warning {
    background: #fef7e0;
    color: #b06000;
    border: 1px solid #feefc3;
  }

  .auth-status.info {
    background: #e8f0fe;
    color: #1967d2;
    border: 1px solid #d2e3fc;
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

  @media (max-width: 600px) {
    .google-auth-setup {
      margin: 10px;
      padding: 20px;
    }

    .header {
      flex-direction: column;
      gap: 15px;
      align-items: stretch;
    }

    .button-group {
      flex-direction: column;
    }

    .save-button, .auth-button {
      min-width: auto;
    }
  }
</style>