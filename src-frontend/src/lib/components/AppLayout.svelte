<script lang="ts">
import { onMount } from "svelte";
import { writable } from "svelte/store";

type ActiveTab = "dashboard" | "conversation" | "settings" | "advanced";

const activeTab = writable<ActiveTab>("dashboard");
let _isAuthSetupComplete = false;

onMount(async () => {
  // Check if Google authentication is set up
  checkAuthStatus();
});

async function checkAuthStatus() {
  // This would check if Google credentials are configured
  // For now, we'll assume they need to be set up
  _isAuthSetupComplete = false;
}

function _setActiveTab(tab: ActiveTab) {
  activeTab.set(tab);
}

function _onAuthComplete() {
  _isAuthSetupComplete = true;
  activeTab.set("dashboard");
}
</script>

<div class="app-container">
  <header class="app-header">
    <div class="logo">
      <h1>🛡️ Oxide Pilot</h1>
      <p>AI-Powered System Guardian & Assistant</p>
    </div>

    {#if isAuthSetupComplete}
      <nav class="tab-navigation">
        <button
          class="tab-button"
          class:active={$activeTab === 'dashboard'}
          on:click={() => setActiveTab('dashboard')}
        >
          📊 Dashboard
        </button>
        <button
          class="tab-button"
          class:active={$activeTab === 'conversation'}
          on:click={() => setActiveTab('conversation')}
        >
          💬 Chat
        </button>
        <button
          class="tab-button"
          class:active={$activeTab === 'settings'}
          on:click={() => setActiveTab('settings')}
        >
          ⚙️ Settings
        </button>
        <button
          class="tab-button"
          class:active={$activeTab === 'advanced'}
          on:click={() => setActiveTab('advanced')}
        >
          🔧 Advanced
        </button>
      </nav>
    {/if}
  </header>

  <main class="app-main">
    {#if !isAuthSetupComplete}
      <div class="setup-container">
        <h2>🔐 Setup Required</h2>
        <p>Please configure your Google API credentials to get started with Oxide Pilot.</p>
        <GoogleAuthSetup on:authComplete={onAuthComplete} />
      </div>
    {:else}
      {#if $activeTab === 'dashboard'}
        <SystemDashboard />
      {:else if $activeTab === 'conversation'}
        <div class="conversation-container">
          <ConversationInterface />
        </div>
      {:else if $activeTab === 'settings'}
        <div class="settings-container">
          <h2>⚙️ Settings</h2>
          <div class="settings-panel">
            <h3>Authentication</h3>
            <GoogleAuthSetup on:authComplete={() => {}} />

            <h3>System Configuration</h3>
            <p>Advanced system configuration options will be available here.</p>

            <div class="setting-item">
              <label>
                <input type="checkbox" checked /> Enable Guardian Agent
              </label>
            </div>

            <div class="setting-item">
              <label>
                <input type="checkbox" checked /> Enable Voice Processing
              </label>
            </div>

            <div class="setting-item">
              <label for="wake-word">Wake Word:</label>
              <input type="text" id="wake-word" value="Hey Oxide" />
            </div>

            <div class="setting-item">
              <label for="monitor-interval">Monitor Interval (seconds):</label>
              <input type="number" id="monitor-interval" value="10" min="1" max="300" />
            </div>
          </div>
        </div>
      {:else if $activeTab === 'advanced'}
        <AdvancedSettings />
      {/if}
    {/if}
  </main>

  <footer class="app-footer">
    <p>Oxide Pilot v0.1.0 - AI-Powered System Guardian</p>
    <div class="status-indicator">
      <span class="status-dot {isAuthSetupComplete ? 'online' : 'offline'}"></span>
      <span>{isAuthSetupComplete ? 'System Ready' : 'Setup Required'}</span>
    </div>
  </footer>
</div>

<style>
  .app-container {
    display: flex;
    flex-direction: column;
    height: 100vh;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  }

  .app-header {
    background: rgba(255, 255, 255, 0.95);
    backdrop-filter: blur(10px);
    padding: 20px;
    box-shadow: 0 2px 20px rgba(0, 0, 0, 0.1);
    display: flex;
    justify-content: space-between;
    align-items: center;
    flex-wrap: wrap;
  }

  .logo h1 {
    margin: 0;
    color: #2c3e50;
    font-size: 28px;
    font-weight: 700;
  }

  .logo p {
    margin: 5px 0 0 0;
    color: #7f8c8d;
    font-size: 14px;
  }

  .tab-navigation {
    display: flex;
    gap: 10px;
  }

  .tab-button {
    padding: 12px 20px;
    border: none;
    border-radius: 25px;
    background: #ecf0f1;
    color: #2c3e50;
    cursor: pointer;
    font-weight: 500;
    transition: all 0.3s ease;
    font-size: 14px;
  }

  .tab-button:hover {
    background: #d5dbdb;
    transform: translateY(-2px);
  }

  .tab-button.active {
    background: #3498db;
    color: white;
    box-shadow: 0 4px 15px rgba(52, 152, 219, 0.3);
  }

  .app-main {
    flex: 1;
    overflow-y: auto;
    background: rgba(255, 255, 255, 0.9);
    margin: 20px;
    border-radius: 15px;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.1);
    backdrop-filter: blur(10px);
  }

  .setup-container {
    padding: 60px 40px;
    text-align: center;
    max-width: 600px;
    margin: 0 auto;
  }

  .setup-container h2 {
    color: #2c3e50;
    margin-bottom: 20px;
    font-size: 32px;
  }

  .setup-container p {
    color: #7f8c8d;
    font-size: 16px;
    line-height: 1.6;
    margin-bottom: 40px;
  }

  .conversation-container {
    height: 100%;
    padding: 20px;
  }

  .settings-container {
    padding: 30px;
    max-width: 800px;
    margin: 0 auto;
  }

  .settings-container h2 {
    color: #2c3e50;
    margin-bottom: 30px;
    font-size: 28px;
  }

  .settings-panel {
    background: white;
    border-radius: 10px;
    padding: 25px;
    box-shadow: 0 4px 15px rgba(0, 0, 0, 0.1);
  }

  .settings-panel h3 {
    color: #34495e;
    margin: 0 0 20px 0;
    font-size: 20px;
    border-bottom: 2px solid #ecf0f1;
    padding-bottom: 10px;
  }

  .setting-item {
    margin: 15px 0;
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .setting-item label {
    font-weight: 500;
    color: #2c3e50;
    min-width: 150px;
  }

  .setting-item input[type="text"],
  .setting-item input[type="number"] {
    padding: 8px 12px;
    border: 2px solid #ecf0f1;
    border-radius: 6px;
    font-size: 14px;
    transition: border-color 0.3s ease;
  }

  .setting-item input[type="text"]:focus,
  .setting-item input[type="number"]:focus {
    outline: none;
    border-color: #3498db;
  }

  .setting-item input[type="checkbox"] {
    transform: scale(1.2);
    margin-right: 8px;
  }

  .app-footer {
    background: rgba(44, 62, 80, 0.9);
    color: white;
    padding: 15px 20px;
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 14px;
  }

  .status-indicator {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .status-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    animation: pulse 2s infinite;
  }

  .status-dot.online {
    background: #27ae60;
  }

  .status-dot.offline {
    background: #e74c3c;
  }

  @keyframes pulse {
    0% { opacity: 1; }
    50% { opacity: 0.5; }
    100% { opacity: 1; }
  }

  @media (max-width: 768px) {
    .app-header {
      flex-direction: column;
      gap: 20px;
    }

    .tab-navigation {
      width: 100%;
      justify-content: center;
    }

    .tab-button {
      flex: 1;
      text-align: center;
    }

    .app-main {
      margin: 10px;
    }

    .app-footer {
      flex-direction: column;
      gap: 10px;
      text-align: center;
    }
  }
</style>
