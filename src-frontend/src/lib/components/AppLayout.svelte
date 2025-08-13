<script lang="ts">
import { onMount } from "svelte";
import { writable } from "svelte/store";
import GoogleAuthSetup from "./GoogleAuthSetup.svelte";
import QwenAuthSetup from "./QwenAuthSetup.svelte";
import SystemDashboard from "./SystemDashboard.svelte";
import AdvancedSettings from "./AdvancedSettings.svelte";
import ConversationInterface from "./ConversationInterface.svelte";
import PatternDashboard from "./PatternDashboard.svelte";
import { isTauri } from "$lib/utils/env";

type ActiveTab = "dashboard" | "conversation" | "settings" | "advanced";

const activeTab = writable<ActiveTab>("dashboard");
let isAuthSetupComplete = false;
let selectedProvider: "gemini" | "qwen" = "gemini";

// Persist and restore provider selection
const PROVIDER_KEY = "oxide.provider";

// Lazy-load Tauri invoke to avoid SSR importing '@tauri-apps/api/tauri'
type InvokeFn = <T = any>(cmd: string, args?: Record<string, unknown>) => Promise<T>;
let invokeFn: InvokeFn | null = null;
async function tauriInvoke<T = any>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (!isTauri) throw new Error("Not running in Tauri context");
  if (!invokeFn) {
    const mod = await import("@tauri-apps/api/tauri");
    invokeFn = mod.invoke as InvokeFn;
  }
  return invokeFn<T>(cmd, args);
}

onMount(async () => {
  try {
    const saved = typeof localStorage !== 'undefined' ? localStorage.getItem(PROVIDER_KEY) : null;
    if (saved === 'gemini' || saved === 'qwen') {
      selectedProvider = saved;
    }
  } catch {}
  await checkAuthStatus();
});

async function checkAuthStatus() {
  // If running under Tauri, check both providers; mark complete if either is authenticated
  if (!isTauri) {
    isAuthSetupComplete = false;
    return;
  }
  try {
    const [geminiStatus, qwenStatus] = await Promise.allSettled([
      tauriInvoke<string>("get_auth_status"),
      tauriInvoke<string>("qwen_get_auth_status"),
    ]);

    const geminiOk = geminiStatus.status === 'fulfilled' && /auth/i.test(geminiStatus.value) && !/not\s+auth/i.test(geminiStatus.value);
    const qwenOk = qwenStatus.status === 'fulfilled' && /auth/i.test(qwenStatus.value) && !/not\s+auth/i.test(qwenStatus.value);
    isAuthSetupComplete = !!(geminiOk || qwenOk);
  } catch (_) {
    isAuthSetupComplete = false;
  }
}

function setActiveTab(tab: ActiveTab) {
  activeTab.set(tab);
}

function onAuthComplete() {
  isAuthSetupComplete = true;
  activeTab.set("dashboard");
}

// Write provider changes to localStorage
$: try {
  if (typeof localStorage !== 'undefined' && (selectedProvider === 'gemini' || selectedProvider === 'qwen')) {
    localStorage.setItem(PROVIDER_KEY, selectedProvider);
  }
} catch {}
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
        <p>Please configure your AI provider to get started. Choose between Google Gemini or Qwen.</p>
        <div class="provider-selector">
          <button class="prov-btn" class:active={selectedProvider === 'gemini'} on:click={() => selectedProvider = 'gemini'}>
            🌟 Google Gemini
          </button>
          <button class="prov-btn" class:active={selectedProvider === 'qwen'} on:click={() => selectedProvider = 'qwen'}>
            🤖 Qwen (Device Code)
          </button>
        </div>
        {#if selectedProvider === 'gemini'}
          <GoogleAuthSetup on:authComplete={onAuthComplete} />
        {:else}
          <QwenAuthSetup on:authComplete={onAuthComplete} />
        {/if}
      </div>
    {:else}
      {#if $activeTab === 'dashboard'}
        <PatternDashboard />
      {:else if $activeTab === 'conversation'}
        <div class="conversation-container">
          <ConversationInterface />
        </div>
      {:else if $activeTab === 'settings'}
        <div class="settings-container">
          <h2>⚙️ Settings</h2>
          <div class="settings-panel">
            <h3>Authentication</h3>
            <div class="auth-grid">
              <div>
                <h4>Google Gemini</h4>
                <GoogleAuthSetup on:authComplete={() => {}} />
              </div>
              <div>
                <h4>Qwen</h4>
                <QwenAuthSetup on:authComplete={() => {}} />
              </div>
            </div>

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
    background: var(--color-bg);
  }

  .app-header {
    background: var(--color-surface);
    backdrop-filter: blur(10px);
    padding: var(--space-5);
    box-shadow: var(--shadow-md);
    display: flex;
    justify-content: space-between;
    align-items: center;
    flex-wrap: wrap;
  }

  .logo h1 {
    margin: 0;
    color: var(--color-text);
    font-size: 28px;
    font-weight: 700;
  }

  .logo p {
    margin: 5px 0 0 0;
    color: var(--color-muted);
    font-size: 14px;
  }

  .tab-navigation {
    display: flex;
    gap: 10px;
  }

  .tab-button {
    padding: 12px 20px;
    border: 1px solid rgba(0,0,0,0.06);
    border-radius: var(--radius-pill);
    background: var(--color-surface);
    color: var(--color-text);
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
    background: var(--color-primary);
    color: white;
    box-shadow: 0 4px 15px rgba(79, 70, 229, 0.35);
  }

  .app-main {
    flex: 1;
    overflow-y: auto;
    background: var(--color-surface);
    margin: 20px;
    border-radius: 15px;
    box-shadow: var(--shadow-lg);
    backdrop-filter: blur(10px);
  }

  .setup-container {
    padding: 60px 40px;
    text-align: center;
    max-width: 600px;
    margin: 0 auto;
  }

  .setup-container h2 {
    color: var(--color-text);
    margin-bottom: 20px;
    font-size: 32px;
  }

  .setup-container p {
    color: var(--color-muted);
    font-size: 16px;
    line-height: 1.6;
    margin-bottom: 40px;
  }

  .provider-selector {
    display: flex;
    gap: 10px;
    justify-content: center;
    margin-bottom: 20px;
  }

  .prov-btn {
    padding: 10px 16px;
    border-radius: var(--radius-pill);
    border: 1px solid rgba(0,0,0,0.08);
    background: var(--color-surface);
    color: var(--color-text);
    cursor: pointer;
  }

  .prov-btn.active {
    background: var(--color-primary);
    color: #fff;
    border-color: var(--color-primary);
    box-shadow: 0 4px 12px rgba(79,70,229,0.25);
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
    color: var(--color-text);
    margin-bottom: 30px;
    font-size: 28px;
  }

  .settings-panel {
    background: var(--color-surface);
    border-radius: var(--radius-md);
    padding: 25px;
    box-shadow: var(--shadow-md);
  }

  .auth-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 20px;
    margin-bottom: 20px;
  }

  .auth-grid h4 {
    margin: 10px 0;
    color: var(--color-text);
  }

  .settings-panel h3 {
    color: var(--color-text);
    margin: 0 0 20px 0;
    font-size: 20px;
    border-bottom: 1px solid rgba(0,0,0,0.08);
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
    color: var(--color-text);
    min-width: 150px;
  }

  .setting-item input[type="text"],
  .setting-item input[type="number"] {
    padding: 8px 12px;
    border: 2px solid rgba(0,0,0,0.08);
    border-radius: var(--radius-sm);
    font-size: 14px;
    transition: border-color 0.3s ease;
    background: var(--color-surface);
    color: var(--color-text);
  }

  .setting-item input[type="text"]:focus,
  .setting-item input[type="number"]:focus {
    outline: none;
    border-color: var(--color-primary);
  }

  .setting-item input[type="checkbox"] {
    transform: scale(1.2);
    margin-right: 8px;
  }

  .app-footer {
    background: var(--color-surface);
    color: var(--color-text);
    padding: 15px 20px;
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 14px;
    box-shadow: var(--shadow-sm);
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
