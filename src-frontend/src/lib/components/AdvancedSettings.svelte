<script lang="ts">
import { onMount } from "svelte";
import { writable } from "svelte/store";

interface SystemConfig {
  guardian: {
    enabled: boolean;
    monitor_interval_secs: number;
  };
  copilot: {
    enabled: boolean;
    wake_word: string;
  };
}

const config = writable<SystemConfig>({
  guardian: {
    enabled: true,
    monitor_interval_secs: 10,
  },
  copilot: {
    enabled: true,
    wake_word: "Hey Oxide",
  },
});

const isSaving = writable(false);
const saveStatus = writable<{
  message: string;
  type: "success" | "error" | "info";
} | null>(null);

onMount(async () => {
  await loadConfig();
});

async function loadConfig() {
  try {
    // For now, we'll use default config since the backend might not be fully implemented
    console.log("Loading configuration...");
  } catch (error) {
    console.error("Failed to load config:", error);
    saveStatus.set({
      message: `Failed to load configuration: ${error}`,
      type: "error",
    });
  }
}

async function saveConfig() {
  isSaving.set(true);
  saveStatus.set({
    message: "Saving configuration...",
    type: "info",
  });

  try {
    // For now, we'll just simulate saving
    await new Promise(resolve => setTimeout(resolve, 1000));
    saveStatus.set({
      message: "Configuration saved successfully!",
      type: "success",
    });
  } catch (error) {
    console.error("Failed to save config:", error);
    saveStatus.set({
      message: `Failed to save configuration: ${error}`,
      type: "error",
    });
  } finally {
    isSaving.set(false);
  }
}

function resetToDefaults() {
  if (confirm("Are you sure you want to reset all settings to defaults?")) {
    config.set({
      guardian: {
        enabled: true,
        monitor_interval_secs: 10,
      },
      copilot: {
        enabled: true,
        wake_word: "Hey Oxide",
      },
    });
  }
}
</script>

<div class="advanced-settings">
  <div class="settings-header">
    <h2>⚙️ Advanced Settings</h2>
    <div class="header-actions">
      <button class="reset-button" on:click={resetToDefaults}>
        🔄 Reset Defaults
      </button>
      <button
        class="save-button"
        on:click={saveConfig}
        disabled={$isSaving}
      >
        {$isSaving ? "💾 Saving..." : "💾 Save Config"}
      </button>
    </div>
  </div>

  {#if $saveStatus}
    <div class="status-message {$saveStatus.type}">
      {$saveStatus.message}
    </div>
  {/if}

  <div class="settings-sections">
    <!-- Guardian Agent Settings -->
    <div class="settings-section">
      <h3>🛡️ Guardian Agent</h3>
      <div class="setting-group">
        <label class="toggle-setting">
          <input
            type="checkbox"
            bind:checked={$config.guardian.enabled}
          />
          <span class="toggle-slider"></span>
          <span class="setting-label">Enable Guardian Agent</span>
        </label>
        <p class="setting-description">
          Continuously monitors system for threats and performance issues
        </p>
      </div>

      <div class="setting-group">
        <label class="range-setting">
          <span class="setting-label">Monitor Interval</span>
          <div class="range-container">
            <input
              type="range"
              min="1"
              max="60"
              bind:value={$config.guardian.monitor_interval_secs}
            />
            <span class="range-value">{$config.guardian.monitor_interval_secs}s</span>
          </div>
        </label>
        <p class="setting-description">
          How often the Guardian checks system status (lower = more frequent)
        </p>
      </div>
    </div>

    <!-- Copilot Agent Settings -->
    <div class="settings-section">
      <h3>🤖 Copilot Agent</h3>
      <div class="setting-group">
        <label class="toggle-setting">
          <input
            type="checkbox"
            bind:checked={$config.copilot.enabled}
          />
          <span class="toggle-slider"></span>
          <span class="setting-label">Enable Copilot Agent</span>
        </label>
        <p class="setting-description">
          AI-powered conversational assistant with system control capabilities
        </p>
      </div>

      <div class="setting-group">
        <label class="text-setting">
          <span class="setting-label">Wake Word</span>
          <input
            type="text"
            bind:value={$config.copilot.wake_word}
            placeholder="Hey Oxide"
          />
        </label>
        <p class="setting-description">
          Phrase to activate voice interaction (e.g., "Hey Oxide", "Computer")
        </p>
      </div>
    </div>

    <!-- Performance Info -->
    <div class="settings-section">
      <h3>⚡ Performance Targets</h3>
      <div class="performance-info">
        <div class="info-item">
          <span class="info-label">CPU Usage Target:</span>
          <span class="info-value">&lt; 5%</span>
        </div>
        <div class="info-item">
          <span class="info-label">Memory Usage Target:</span>
          <span class="info-value">&lt; 100MB</span>
        </div>
        <div class="info-item">
          <span class="info-label">Response Time Target:</span>
          <span class="info-value">&lt; 500ms</span>
        </div>
      </div>
      <p class="setting-description">
        System automatically optimizes performance when these targets are exceeded
      </p>
    </div>
  </div>
</div>

<style>
  .advanced-settings {
    padding: 30px;
    max-width: 1000px;
    margin: 0 auto;
    background: var(--color-surface);
    border-radius: 15px;
    box-shadow: var(--shadow-md);
    border: 1px solid rgba(0,0,0,0.06);
  }

  .settings-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 30px;
    padding-bottom: 20px;
    border-bottom: 2px solid #f1f3f4;
  }

  .settings-header h2 { color: var(--color-text); margin: 0; font-size: 28px; font-weight: 600; }

  .header-actions {
    display: flex;
    gap: 15px;
  }

  .reset-button, .save-button { padding: 12px 24px; border: none; border-radius: 8px; font-size: 14px; font-weight: 500; cursor: pointer; transition: background-color 0.15s ease, transform 0.05s ease; }

  .reset-button {
    background: #e74c3c;
    color: white;
  }

  .reset-button:hover { background: #c0392b; }
  .reset-button:focus-visible { outline: 2px solid #3b82f6; outline-offset: 2px; }
  .reset-button:active { transform: translateY(1px); }

  .save-button {
    background: #27ae60;
    color: white;
  }

  .save-button:hover:not(:disabled) { background: #229954; }
  .save-button:focus-visible { outline: 2px solid #3b82f6; outline-offset: 2px; }
  .save-button:active { transform: translateY(1px); }

  .save-button:disabled {
    background: #95a5a6;
    cursor: not-allowed;
    transform: none;
  }

  .status-message {
    padding: 15px;
    border-radius: 8px;
    margin-bottom: 20px;
    font-weight: 500;
  }

  .status-message.success { background: #ecfdf5; color: #065f46; border: 1px solid #a7f3d0; }

  .status-message.error { background: #fef2f2; color: #b91c1c; border: 1px solid #fecaca; }

  .status-message.info { background: #eff6ff; color: #1d4ed8; border: 1px solid #bfdbfe; }

  .settings-sections {
    display: flex;
    flex-direction: column;
    gap: 30px;
  }

  .settings-section { background: var(--color-surface); border-radius: 12px; padding: 25px; border: 1px solid rgba(0,0,0,0.06); box-shadow: var(--shadow-sm, 0 2px 6px rgba(0,0,0,0.06)); }

  .settings-section h3 { color: var(--color-text); margin: 0 0 20px 0; font-size: 20px; font-weight: 600; }

  .setting-group {
    margin-bottom: 20px;
  }

  .setting-group:last-child {
    margin-bottom: 0;
  }

  .toggle-setting {
    display: flex;
    align-items: center;
    gap: 15px;
    cursor: pointer;
  }

  .toggle-setting input[type="checkbox"] {
    display: none;
  }

  .toggle-slider {
    width: 50px;
    height: 26px;
    background: #ccc;
    border-radius: 13px;
    position: relative;
    transition: background 0.3s ease;
  }

  .toggle-slider::before {
    content: '';
    position: absolute;
    width: 22px;
    height: 22px;
    border-radius: 50%;
    background: white;
    top: 2px;
    left: 2px;
    transition: transform 0.3s ease;
  }

  .toggle-setting input[type="checkbox"]:checked + .toggle-slider {
    background: #27ae60;
  }

  .toggle-setting input[type="checkbox"]:checked + .toggle-slider::before {
    transform: translateX(24px);
  }

  .setting-label {
    font-weight: 500;
    color: #2c3e50;
    font-size: 16px;
  }

  .range-setting {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .range-container {
    display: flex;
    align-items: center;
    gap: 15px;
  }

  .range-container input[type="range"] {
    flex: 1;
    height: 6px;
    border-radius: 3px;
    background: #ddd;
    outline: none;
    -webkit-appearance: none;
  }

  .range-container input[type="range"]::-webkit-slider-thumb {
    -webkit-appearance: none;
    width: 20px;
    height: 20px;
    border-radius: 50%;
    background: #3498db;
    cursor: pointer;
  }

  .range-value {
    font-weight: 600;
    color: #3498db;
    min-width: 40px;
    text-align: center;
  }

  .text-setting {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .text-setting input[type="text"] {
    padding: 12px 16px;
    border: 2px solid #e9ecef;
    border-radius: 8px;
    font-size: 14px;
    transition: border-color 0.3s ease;
  }

  .text-setting input[type="text"]:focus {
    outline: none;
    border-color: #3498db;
  }

  .setting-description {
    color: #6c757d;
    font-size: 14px;
    margin: 8px 0 0 0;
    line-height: 1.4;
  }

  .performance-info {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 15px;
    margin-bottom: 15px;
  }

  .info-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 16px;
    background: white;
    border-radius: 8px;
    border: 1px solid #e9ecef;
  }

  .info-label {
    font-weight: 500;
    color: #495057;
  }

  .info-value {
    font-weight: 600;
    color: #28a745;
  }

  @media (max-width: 768px) {
    .advanced-settings {
      margin: 10px;
      padding: 20px;
    }

    .settings-header {
      flex-direction: column;
      gap: 15px;
      align-items: stretch;
    }

    .header-actions {
      justify-content: center;
    }

    .performance-info {
      grid-template-columns: 1fr;
    }
  }
</style>
