<script lang="ts">
  import { invoke } from '@tauri-apps/api/tauri';
  import { writable } from 'svelte/store';

  let clientId = '';
  let clientSecret = '';
  let authStatus = writable<{ message: string; type: 'success' | 'error' | 'info' }>({ message: '', type: 'info' });

  async function saveClientCredentials() {
    authStatus.set({ message: 'Saving client credentials...', type: 'info' });
    try {
      await invoke('set_google_client_credentials', { clientId, clientSecret });
      authStatus.set({ message: 'Client credentials saved successfully!', type: 'success' });
    } catch (error) {
      authStatus.set({ message: `Error saving client credentials: ${error}`, type: 'error' });
      console.error('Error saving client credentials:', error);
    }
  }

  async function startGoogleAuth() {
    authStatus.set({ message: 'Initiating Google authentication flow...', type: 'info' });
    try {
      const accessToken = await invoke('authenticate_google_command');
      authStatus.set({ message: `Google authentication successful! Access Token: ${accessToken.substring(0, 10)}...`, type: 'success' });
    } catch (error) {
      authStatus.set({ message: `Error during Google authentication: ${error}`, type: 'error' });
      console.error('Error during Google authentication:', error);
    }
  }
</script>

<div class="google-auth-setup">
  <h2>Google Authentication Setup</h2>

  <div class="input-group">
    <label for="clientId">Google Client ID:</label>
    <input type="text" id="clientId" bind:value={clientId} placeholder="Enter your Google Client ID" />
  </div>

  <div class="input-group">
    <label for="clientSecret">Google Client Secret:</label>
    <input type="password" id="clientSecret" bind:value={clientSecret} placeholder="Enter your Google Client Secret" />
  </div>

  <button on:click={saveClientCredentials}>Save Client Credentials</button>
  <button on:click={startGoogleAuth}>Authenticate with Google</button>

  {#if $authStatus.message}
    <p class="auth-status {$authStatus.type}">{$authStatus.message}</p>
  {/if}
</div>

<style>
  .google-auth-setup {
    padding: 20px;
    border: 1px solid #ccc;
    border-radius: 8px;
    max-width: 500px;
    margin: 20px auto;
    background-color: #f9f9f9;
  }

  h2 {
    text-align: center;
    color: #333;
    margin-bottom: 20px;
  }

  .input-group {
    margin-bottom: 15px;
  }

  label {
    display: block;
    margin-bottom: 5px;
    font-weight: bold;
    color: #555;
  }

  input[type="text"],
  input[type="password"] {
    width: calc(100% - 20px);
    padding: 10px;
    border: 1px solid #ddd;
    border-radius: 4px;
    font-size: 1em;
  }

  button {
    background-color: #4CAF50;
    color: white;
    padding: 10px 15px;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 1em;
    margin-right: 10px;
    margin-top: 10px;
  }

  button:hover {
    background-color: #45a049;
  }

  .auth-status {
    margin-top: 20px;
    padding: 10px;
    border-radius: 4px;
  }

  .auth-status.success {
    background-color: #e7f3e7;
    color: #3c763d;
    border: 1px solid #d6e9c6;
  }

  .auth-status.error {
    background-color: #f8d7da;
    color: #721c24;
    border: 1px solid #f5c6cb;
  }

  .auth-status.info {
    background-color: #d1ecf1;
    color: #0c5460;
    border: 1px solid #bee5eb;
  }
</style>