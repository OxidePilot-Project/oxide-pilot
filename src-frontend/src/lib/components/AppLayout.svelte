<script lang="ts">
import { onMount } from "svelte";
import { writable } from "svelte/store";
import ModernAuthSetup from "./ModernAuthSetup.svelte";
import GoogleAuthSetup from "./GoogleAuthSetup.svelte";
import QwenAuthSetup from "./QwenAuthSetup.svelte";
import LocalModelsPanel from "./LocalModelsPanel.svelte";
import OpenAIAuthSetup from "./OpenAIAuthSetup.svelte";
import SystemDashboard from "./SystemDashboard.svelte";
import SystemAnalysisPanel from "./SystemAnalysisPanel.svelte";
import AdvancedSettings from "./AdvancedSettings.svelte";
import ConversationInterface from "./ConversationInterface.svelte";
import PatternDashboard from "./PatternDashboard.svelte";
import CollaborativeAnalysis from "./CollaborativeAnalysis.svelte";
import RPAConfirmationDialog from "./RPAConfirmationDialog.svelte";
import RPADashboard from "./RPADashboard.svelte";
import { isTauri } from "$lib/utils/env";
import { tauriInvoke } from "$lib/utils/tauri";

type ActiveTab = "dashboard" | "conversation" | "analysis" | "collaborative" | "rpa" | "settings" | "advanced";

const activeTab = writable<ActiveTab>("dashboard");
let isAuthSetupComplete = false;
let selectedProvider: "gemini" | "qwen" | "openai" | "local" = "gemini";
let providerInitialized = false;
let geminiConnected = false;
let qwenConnected = false;
let openaiConnected = false;

// Persist and restore provider selection
const PROVIDER_KEY = "oxide.provider";

// Window sizing controls (Tauri only): fullscreen or fixed medium
let windowApi: any = null;
async function ensureWindowApi() {
  if (!isTauri) return null;
  if (!windowApi) {
    const mod = await import("@tauri-apps/api/window");
    windowApi = mod.appWindow;
  }
  return windowApi;
}

async function enterFullscreen() {
  const win = await ensureWindowApi();
  if (win) {
    await win.setFullscreen(true);
  }
}

async function exitToMedium() {
  const win = await ensureWindowApi();
  if (win) {
    await win.setFullscreen(false);
    // Set fixed medium size and center
    const mod = await import("@tauri-apps/api/window");
    const size = new mod.LogicalSize(1280, 800);
    await win.setSize(size);
    try { await win.center(); } catch {}
  }
}

onMount(async () => {
  try {
    const saved = typeof localStorage !== 'undefined' ? localStorage.getItem(PROVIDER_KEY) : null;
    if (saved === 'gemini' || saved === 'qwen' || saved === 'openai' || saved === 'local') {
      selectedProvider = saved as any;
    }
  } catch {}
  providerInitialized = true;
  // E2E test bypass: allow dashboard in browser mode when ?e2e=1 is present
  try {
    if (!isTauri && typeof window !== 'undefined') {
      const params = new URLSearchParams(window.location.search);
      if (params.get('e2e') === '1') {
        isAuthSetupComplete = true;
      }
    }
  } catch {}
  await checkAuthStatus();
  await refreshProviderStatuses();
});

async function checkAuthStatus() {
  // If running under Tauri, check both providers; mark complete if either is authenticated
  if (!isTauri) {
    // In browser mode default to setup incomplete unless E2E bypass set above
    return;
  }
  try {
    const [geminiStatus, qwenStatus, openaiStatus, localStatus] = await Promise.allSettled([
      tauriInvoke<string>("get_auth_status"),
      tauriInvoke<string>("qwen_get_auth_status"),
      tauriInvoke<string>("openai_get_auth_status"),
      tauriInvoke<{ running: boolean }>("local_llm_server_status"),
    ]);

    const geminiOk = geminiStatus.status === 'fulfilled' && /auth/i.test(geminiStatus.value) && !/not\s+auth/i.test(geminiStatus.value);
    const qwenOk = qwenStatus.status === 'fulfilled' && /auth/i.test(qwenStatus.value) && !/not\s+auth/i.test(qwenStatus.value);
    const openaiVal = openaiStatus.status === 'fulfilled' ? String(openaiStatus.value) : '';
    const openaiOk = (/auth/i.test(openaiVal) && !/not\s+auth/i.test(openaiVal)) || /api\s*key/i.test(openaiVal);
    const localOk = localStatus.status === 'fulfilled' && !!localStatus.value?.running;
    isAuthSetupComplete = !!(geminiOk || qwenOk || openaiOk || localOk);
  } catch (_) {
    isAuthSetupComplete = false;
  }
}

async function refreshProviderStatuses() {
  if (!isTauri) return;
  try {
    const [geminiStatus, qwenStatus, openaiStatus] = await Promise.allSettled([
      tauriInvoke<string>("get_auth_status"),
      tauriInvoke<string>("qwen_get_auth_status"),
      tauriInvoke<string>("openai_get_auth_status"),
    ]);
    geminiConnected = geminiStatus.status === 'fulfilled' && /auth/i.test(geminiStatus.value) && !/not\s+auth/i.test(geminiStatus.value);
    qwenConnected = qwenStatus.status === 'fulfilled' && /auth/i.test(qwenStatus.value) && !/not\s+auth/i.test(qwenStatus.value);
    openaiConnected = openaiStatus.status === 'fulfilled' && /auth/i.test(openaiStatus.value) && !/not\s+auth/i.test(openaiStatus.value);
  } catch {
    geminiConnected = false;
    qwenConnected = false;
    openaiConnected = false;
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
  if (providerInitialized && typeof localStorage !== 'undefined' && (selectedProvider === 'gemini' || selectedProvider === 'qwen' || selectedProvider === 'openai' || selectedProvider === 'local')) {
    localStorage.setItem(PROVIDER_KEY, selectedProvider);
  }
} catch {}

// When switching to Settings, refresh provider badges
$: if ($activeTab === 'settings') {
  void refreshProviderStatuses();
}
</script>

<div class="app-container">
  <header class="app-header">
    <div class="logo">
      <h1>🛡️ Oxide Pilot</h1>
      <p>AI-Powered System Guardian & Assistant</p>
    </div>

    <div class="header-actions">
      <div class="provider-badge" aria-label="Active provider">
        <span class="dot {isAuthSetupComplete ? 'on' : 'off'}"></span>
        <span class="label">{selectedProvider === 'gemini' ? 'Gemini' : selectedProvider === 'qwen' ? 'Qwen' : selectedProvider === 'openai' ? 'OpenAI' : 'Local'}</span>
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
            class:active={$activeTab === 'analysis'}
            on:click={() => setActiveTab('analysis')}
          >
            🧠 Analysis
          </button>
          <button
            class="tab-button"
            class:active={$activeTab === 'collaborative'}
            on:click={() => setActiveTab('collaborative')}
          >
            🤝 Collaborative
          </button>
          <button
            class="tab-button"
            class:active={$activeTab === 'rpa'}
            on:click={() => setActiveTab('rpa')}
          >
            🤖 RPA
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

      {#if isTauri}
        <div class="window-controls">
          <button class="win-btn" on:click={enterFullscreen} title="Fullscreen">⛶ Fullscreen</button>
          <button class="win-btn" on:click={exitToMedium} title="Medium window">◻ Medium</button>
        </div>
      {/if}
    </div>
  </header>

  <main class="app-main">
    {#if !isAuthSetupComplete}
      <div class="setup-container">
        <div class="setup-header">
          <h2>🔐 Setup Required</h2>
          <p>Choose your AI provider to get started with Oxide Pilot</p>
        </div>

        <div class="provider-selector">
          <button
            class="provider-btn"
            class:active={selectedProvider === 'gemini'}
            on:click={() => selectedProvider = 'gemini'}
          >
            <div class="provider-icon">🌟</div>
            <div class="provider-info">
              <h3>Google Gemini</h3>
              <p>OAuth2 secure authentication</p>
            </div>
          </button>

          <button
            class="provider-btn"
            class:active={selectedProvider === 'qwen'}
            on:click={() => selectedProvider = 'qwen'}
          >
            <div class="provider-icon">🤖</div>
            <div class="provider-info">
              <h3>Qwen AI</h3>
              <p>OAuth2 secure authentication</p>
            </div>
          </button>

          <button
            class="provider-btn"
            class:active={selectedProvider === 'openai'}
            on:click={() => selectedProvider = 'openai'}
          >
            <div class="provider-icon">🧠</div>
            <div class="provider-info">
              <h3>OpenAI (GPT‑5)</h3>
              <p>OAuth2 secure authentication</p>
            </div>
          </button>

          <button
            class="provider-btn"
            class:active={selectedProvider === 'local'}
            on:click={() => selectedProvider = 'local'}
          >
            <div class="provider-icon">💻</div>
            <div class="provider-info">
              <h3>Local Models</h3>
              <p>Run AI locally (coming soon)</p>
            </div>
          </button>
        </div>

        <div class="auth-setup-wrapper auth-container no-x-overflow">
          {#if selectedProvider === 'openai'}
            <OpenAIAuthSetup on:authComplete={onAuthComplete} />
          {:else}
            <ModernAuthSetup provider={selectedProvider as 'gemini' | 'qwen' | 'local'} on:authComplete={onAuthComplete} />
          {/if}
        </div>
      </div>
    {:else}
      {#if $activeTab === 'dashboard'}
        <PatternDashboard />
      {:else if $activeTab === 'conversation'}
        <div class="conversation-container">
          <ConversationInterface provider={selectedProvider} />
        </div>
      {:else if $activeTab === 'analysis'}
        <SystemAnalysisPanel />
      {:else if $activeTab === 'collaborative'}
        <CollaborativeAnalysis />
      {:else if $activeTab === 'rpa'}
        <RPADashboard />
      {:else if $activeTab === 'settings'}
        <div class="settings-container">
          <h2>⚙️ Settings</h2>
          <div class="settings-panel">
            <h3>Authentication</h3>
            <div class="auth-grid">
              <div>
                <h4>
                  Google Gemini
                  <span class="status-badge" data-testid="status-gemini">
                    <span class="dot {geminiConnected ? 'on' : 'off'}"></span>
                    {geminiConnected ? 'Connected' : 'Not connected'}
                  </span>
                </h4>
                <GoogleAuthSetup on:authComplete={() => {}} />
              </div>
              <div>
                <h4>
                  Qwen
                  <span class="status-badge" data-testid="status-qwen">
                    <span class="dot {qwenConnected ? 'on' : 'off'}"></span>
                    {qwenConnected ? 'Connected' : 'Not connected'}
                  </span>
                </h4>
                <QwenAuthSetup on:authComplete={() => {}} />
              </div>
              <div>
                <h4>
                  OpenAI (GPT-5)
                  <span class="status-badge" data-testid="status-openai">
                    <span class="dot {openaiConnected ? 'on' : 'off'}"></span>
                    {openaiConnected ? 'Connected' : 'Not connected'}
                  </span>
                </h4>
                <OpenAIAuthSetup />
              </div>
              <div>
                <h4>Local Models (LM Studio)</h4>
                <LocalModelsPanel />
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

<!-- Global RPA Confirmation Dialog -->
{#if isAuthSetupComplete && isTauri}
  <RPAConfirmationDialog />
{/if}

<style>
  :global(html, body) {
    height: 100%;
    overflow-x: hidden;
    overflow-y: hidden;
  }

  .app-container {
    display: grid;
    grid-template-rows: 60px 1fr 40px;
    height: 100vh;
    max-height: 100vh;
    overflow: hidden;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    background: var(--color-bg);
    width: 100vw;
    box-sizing: border-box;
  }

  .app-header {
    background: var(--color-surface);
    backdrop-filter: blur(10px);
    padding: 8px 16px;
    box-shadow: var(--shadow-md);
    display: flex;
    justify-content: space-between;
    align-items: center;
    flex-wrap: wrap;
    height: 60px;
    box-sizing: border-box;
  }

  .logo h1 {
    margin: 0;
    color: var(--color-text);
    font-size: 18px;
    font-weight: 700;
    line-height: 1.1;
  }

  .logo p {
    margin: 1px 0 0 0;
    color: var(--color-muted);
    font-size: 11px;
    line-height: 1.1;
  }

  .header-actions { display: flex; align-items: center; gap: 12px; flex-wrap: wrap; }
  .window-controls { display: flex; gap: 8px; }
  .win-btn { padding: 8px 10px; border: 1px solid rgba(0,0,0,0.08); border-radius: 8px; background: var(--color-surface); cursor: pointer; font-size: 12px; }

  .tab-navigation {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    max-width: 100%;
    min-width: 0;
  }

  .tab-button {
    padding: 6px 12px;
    border: 1px solid rgba(0,0,0,0.06);
    border-radius: var(--radius-pill);
    background: var(--color-surface);
    color: var(--color-text);
    cursor: pointer;
    font-weight: 500;
    transition: all 0.3s ease;
    font-size: 12px;
    white-space: nowrap;
    min-width: 0;
    flex-shrink: 1;
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

  /* Prevent flex/grid children from forcing horizontal overflow */
  .header-actions,
  .tab-navigation,
  .setup-container,
  .settings-container,
  .settings-panel,
  .conversation-container {
    min-width: 0;
  }

  .app-main {
    min-height: 0; /* allow this area to shrink and be the only scroller */
    overflow-y: auto;
    overflow-x: hidden;
    background: var(--color-surface);
    padding: 8px;
    border-radius: 8px;
    box-shadow: var(--shadow-lg);
    backdrop-filter: blur(10px);
    box-sizing: border-box;
    max-width: 100%;
    margin: 4px;
    height: calc(100vh - 108px); /* 60px header + 40px footer + 8px margins */
  }

  .setup-container {
    padding: 20px 16px;
    text-align: center;
    max-width: 600px;
    margin-inline: auto;
    height: 100%;
    display: flex;
    flex-direction: column;
    justify-content: center;
    box-sizing: border-box;
  }

  .setup-container h2 {
    color: var(--color-text);
    margin-bottom: 12px;
    font-size: 20px;
    line-height: 1.2;
  }

  .setup-container p {
    color: var(--color-muted);
    font-size: 13px;
    line-height: 1.5;
    margin-bottom: 20px;
  }

  .provider-selector {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
    gap: 16px;
    margin-bottom: 24px;
    max-width: 600px;
    margin-left: auto;
    margin-right: auto;
  }

  .provider-btn {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 12px;
    border: 1px solid rgba(255,255,255,0.12);
    border-radius: 10px;
    background: var(--color-surface);
    cursor: pointer;
    transition: all 0.3s ease;
    text-align: left;
  }

  .provider-btn:hover {
    border-color: var(--color-primary);
    transform: translateY(-2px);
    box-shadow: 0 8px 25px rgba(0, 0, 0, 0.18);
  }

  .provider-btn.active {
    border-color: var(--color-primary);
    background: var(--color-primary);
    color: white;
    box-shadow: 0 8px 25px rgba(59, 130, 246, 0.3);
  }

  .provider-icon {
    font-size: clamp(1.5rem, 3vw, 1.8rem);
    flex-shrink: 0;
  }

  .provider-info h3 {
    margin: 0 0 0.2rem 0;
    font-size: clamp(1rem, 2vw, 1.1rem);
    font-weight: 600;
  }

  .provider-info p {
    margin: 0;
    font-size: clamp(0.75rem, 1.5vw, 0.8rem);
    color: var(--color-muted);
  }

  .provider-btn.active .provider-info h3,
  .provider-btn.active .provider-info p {
    color: white;
  }

  .auth-setup-wrapper {
    flex: 1;
    display: flex;
    align-items: flex-start;
    justify-content: center;
    width: 100%;
    max-width: 100vw;
    overflow-x: hidden;
    padding: 1rem;
    box-sizing: border-box;
  }

  .conversation-container {
    min-height: 0;
    padding: clamp(12px, 2vh, 20px);
  }

  .settings-container {
    padding: clamp(16px, 2.5vh, 30px);
    max-width: min(900px, 100% - 32px);
    margin-inline: auto;
  }

  .settings-container h2 {
    color: var(--color-text);
    margin-bottom: 30px;
    font-size: 28px;
  }

  .settings-panel {
    background: var(--color-surface);
    border-radius: var(--radius-md);
    padding: clamp(14px, 2vh, 25px);
    box-shadow: var(--shadow-md);
  }

  /* Responsive design for smaller screens */
  @media (max-width: 768px) {
    .app-header {
      flex-direction: column;
      gap: 1rem;
      padding: 1rem;
    }

    .tab-navigation {
      justify-content: center;
      width: 100%;
    }

    .tab-button {
      padding: 8px 12px;
      font-size: 13px;
    }

    .setup-container {
      padding: 1.5rem 1rem;
    }

    .setup-header h2 {
      font-size: 2rem;
    }

    .provider-selector {
      grid-template-columns: 1fr;
      gap: 1rem;
      margin-bottom: 2rem;
    }

    .provider-btn {
      padding: 1rem;
    }

    .provider-icon {
      font-size: 1.5rem;
    }
  }

  /* Compact vertical spacing on short viewports */
  @media (max-height: 840px) {
    .app-header { padding: var(--space-4); }
    .logo h1 { font-size: 24px; }
    .tab-button { padding: 10px 14px; font-size: 13px; }
    .setup-container h2 { font-size: 28px; }
    .setup-container p { font-size: 15px; margin-bottom: 28px; }
  }

  @media (max-width: 480px) {
    .setup-header h2 {
      font-size: 1.75rem;
    }

    .setup-header p {
      font-size: 1rem;
    }

    .provider-btn {
      flex-direction: column;
      text-align: center;
      gap: 0.5rem;
    }

    .provider-info h3 {
      font-size: 1rem;
    }

    .provider-info p {
      font-size: 0.8rem;
    }
  }

  .auth-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
    gap: 16px;
    margin-bottom: 20px;
    align-items: start;
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
    padding: clamp(8px, 1vh, 12px) clamp(12px, 2vw, 16px);
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: clamp(11px, 1.5vw, 13px);
    box-shadow: var(--shadow-sm);
    min-height: 35px;
    max-height: 45px;
  }

  .status-indicator {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  /* Prevent horizontal scroll globally */
  :global(html, body) { overflow-x: hidden; width: 100%; }
  :global(#app), :global(#svelte) { max-width: 100vw; overflow-x: hidden; }

  .status-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    animation: pulse 2s infinite;
  }

  .provider-badge {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 6px 10px;
    border-radius: 999px;
    border: 1px solid rgba(255,255,255,0.12);
    background: var(--color-surface);
    color: var(--color-text);
  }
  .provider-badge .dot {
    width: 8px; height: 8px; border-radius: 50%;
  }
  .provider-badge .dot.on { background: #27ae60; }
  .provider-badge .dot.off { background: #e74c3c; }
  .provider-badge .label { font-size: 12px; }

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
