<script lang="ts">
import { invoke } from "@tauri-apps/api/tauri";
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
  ai_providers: {
    google: {
      api_key: string;
    } | null;
    openai: {
      api_key: string;
    } | null;
    anthropic: {
      api_key: string;
    } | null;
    azure_openai: {
      api_key: string;
      endpoint: string;
    } | null;
    ollama: {
      url: string;
    } | null;
  };
}

const config = writable<SystemConfig | null>(null);
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
    const currentConfig = (await invoke("get_system_config")) as SystemConfig;
    config.set(currentConfig);
  } catch (error) {
    console.error("Failed to load config:", error);
    saveStatus.set({
      message: `Failed to load configuration: ${error}`,
      type: "error",
    });
  }
}

async function _saveConfig() {
  if (!$config) return;

  isSaving.set(true);
  saveStatus.set({
    message: "Saving configuration...",
    type: "info",
  });

  try {
    await invoke("update_system_config", { config: $config });
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

function _resetToDefaults() {
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
      ai_providers: {
        google: {
          api_key: "",
        },
        openai: {
          api_key: "",
        },
        anthropic: {
          api_key: "",
        },
        azure_openai: {
          api_key: "",
          endpoint: "",
        },
        ollama: {
          url: "http://localhost:11434",
        },
      },
    });
  }
}

function _exportConfig() {
  if (!$config) return;

  const dataStr = JSON.stringify($config, null, 2);
  const dataBlob = new Blob([dataStr], { type: "application/json" });
  const url = URL.createObjectURL(dataBlob);

  const link = document.createElement("a");
  link.href = url;
  link.download = "oxide-pilot-config.json";
  link.click();

  URL.revokeObjectURL(url);
}

function _importConfig(event: Event) {
  const input = event.target as HTMLInputElement;
  const file = input.files?.[0];

  if (!file) return;

  const reader = new FileReader();
  reader.onload = (e) => {
    try {
      const importedConfig = JSON.parse(e.target?.result as string);
      config.set(importedConfig);
      saveStatus.set({
        message: "Configuration imported successfully!",
        type: "success",
      });
    } catch (_error) {
      saveStatus.set({
        message: "Failed to import configuration: Invalid JSON",
        type: "error",
      });
    }
  };
  reader.readAsText(file);
}
</script>

<div class="advanced-settings">
  <div class="settings-header">
    <h2>⚙️ Advanced Settings</h2>
    <div class="header-actions">
      <button class="export-button" on:click={exportConfig}>
        📤 Export Config
      </button>
      <label class="import-button">
        📥 Import Config
        <input type="file" accept=".json" on:change={importConfig} style="display: none;">
      </label>
      <button class="reset-button" on:click={resetToDefaults}>
        🔄 Reset Defaults
      </button>
    </div>
  </div>

  {#if $saveStatus}
    <div class="status-message {$saveStatus.type}">
      {$saveStatus.message}
    </div>
  {/if}

  {#if $config}
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

      <!-- AI Provider Settings -->
      <div class="settings-section">
        <h3>🧠 AI Providers</h3>
        
        <!-- Google AI -->
        <div class="setting-group">
          <label class="text-setting">
            <span class="setting-label">Google API Key</span>
            <input
              type="password"
              bind:value={$config.ai_providers.google?.api_key || ""}
              placeholder="Enter your Google API key"
            />
          </label>
          <p class="setting-description">
            API key for Google Gemini AI services (Speech, Text-to-Speech, Generative AI)
          </p>
        </div>
        
        <!-- OpenAI -->
        <div class="setting-group">
          <label class="text-setting">
            <span class="setting-label">OpenAI API Key</span>
            <input
              type="password"
              bind:value={$config.ai_providers.openai?.api_key || ""}
              placeholder="Enter your OpenAI API key"
            />
          </label>
          <p class="setting-description">
            API key for OpenAI GPT models
          </p>
        </div>
        
        <!-- Anthropic -->
        <div class="setting-group">
          <label class="text-setting">
            <span class="setting-label">Anthropic API Key</span>
            <input
              type="password"
              bind:value={$config.ai_providers.anthropic?.api_key || ""}
              placeholder="Enter your Anthropic API key"
            />
          </label>
          <p class="setting-description">
            API key for Anthropic Claude models
          </p>
        </div>
        
        <!-- Azure OpenAI -->
        <div class="setting-group">
          <label class="text-setting">
            <span class="setting-label">Azure OpenAI API Key</span>
            <input
              type="password"
              bind:value={$config.ai_providers.azure_openai?.api_key || ""}
              placeholder="Enter your Azure OpenAI API key"
            />
          </label>
          <p class="setting-description">
            API key for Azure OpenAI services
          </p>
        </div>
        
        <div class="setting-group">
          <label class="text-setting">
            <span class="setting-label">Azure OpenAI Endpoint</span>
            <input
              type="text"
              bind:value={$config.ai_providers.azure_openai?.endpoint || ""}
              placeholder="Enter your Azure OpenAI endpoint"
            />
          </label>
          <p class="setting-description">
            Endpoint URL for your Azure OpenAI resource
          </p>
        </div>
        
        <!-- Ollama -->
        <div class="setting-group">
          <label class="text-setting">
            <span class="setting-label">Ollama URL</span>
            <input
              type="text"
              bind:value={$config.ai_providers.ollama?.url || ""}
              placeholder="http://localhost:11434"
            />
          </label>
          <p class="setting-description">
            URL for your local Ollama server
          </p>
        </div>
      </div>

      <!-- Performance Settings -->
      <div class="settings-section">
        <h3>⚡ Performance</h3>
        <div class="setting-group">
          <div class="performance-info">
            <div class="info-item">
              <span class="info-label">CPU Usage Target:</span>
              <span class="info-value">< 5%</span>
            </div>
            <div class="info-item">
              <span class="info-label">Memory Usage Target:</span>
              <span class="info-value">< 100MB</span>
            </div>
            <div class="info-item">
              <span class="info-label">Response Time Target:</span>
              <span class="info-value">< 500ms</span>
            </div>
          </div>
          <p class="setting-description">
            System automatically optimizes performance when these targets are exceeded
          </p>
        </div>
      </div>

      <!-- Security Settings -->
      <div class="settings-section">
        <h3>🔒 Security</h3>
        <div class="setting-group">
          <div class="security-info">
            <div class="security-item enabled">
              ✅ End-to-end encryption for API communications
            </div>
            <div class="security-item enabled">
              ✅ Secure credential storage using OS keychain
            </div>
            <div class="security-item enabled">
              ✅ YARA-based malware detection
            </div>
            <div class="security-item enabled">
              ✅ Behavioral analysis for threat detection
            </div>
            <div class="security-item enabled">
              ✅ Audit logging for all agent actions
            </div>
          </div>
          <p class="setting-description">
            All security features are enabled by default and cannot be disabled
          </p>
        </div>
      </div>
    </div>

    <div class="settings-footer">
      <button
        class="save-button"
        class:saving={$isSaving}
        on:click={saveConfig}
        disabled={$isSaving}
      >
        {#if $isSaving}
          💾 Saving...
        {:else}
          💾 Save Configuration
        {/if}
      </button>
    </div>
  {:else}
    <div class="loading-state">
      <div class="loading-spinner"></div>
      <p>Loading configuration...</p>
    </div>
  {/if}
</div>

<style>
  .advanced-settings {
    max-width: 800px;
    margin: 0 auto;
    padding: 20px;
  }

  .settings-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 30px;
    padding-bottom: 20px;
    border-bottom: 2px solid #ecf0f1;
  }

  .settings-header h2 {
    color: #2c3e50;
    margin: 0;
    font-size: 28px;
  }

  .header-actions {
    display: flex;
    gap: 10px;
  }

  .export-button, .import-button, .reset-button {
    padding: 8px 16px;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-size: 14px;
    font-weight: 500;
    transition: all 0.2s ease;
  }

  .export-button {
    background: #3498db;
    color: white;
  }

  .import-button {
    background: #27ae60;
    color: white;
  }

  .reset-button {
    background: #e74c3c;
    color: white;
  }

  .export-button:hover, .import-button:hover, .reset-button:hover {
    transform: translateY(-1px);
    opacity: 0.9;
  }

  .status-message {
    padding: 12px 16px;
    border-radius: 6px;
    margin-bottom: 20px;
    font-weight: 500;
  }

  .status-message.success {
    background: #d5f4e6;
    color: #27ae60;
    border: 1px solid #27ae60;
  }

  .status-message.error {
    background: #fce8e6;
    color: #e74c3c;
    border: 1px solid #e74c3c;
  }

  .status-message.info {
    background: #e8f0fe;
    color: #1967d2;
    border: 1px solid #1967d2;
  }

  .settings-sections {
    display: flex;
    flex-direction: column;
    gap: 30px;
  }

  .settings-section {
    background: white;
    border-radius: 12px;
    padding: 25px;
    box-shadow: 0 4px 15px rgba(0, 0, 0, 0.1);
  }

  .settings-section h3 {
    color: #2c3e50;
    margin: 0 0 20px 0;
    font-size: 20px;
    border-bottom: 1px solid #ecf0f1;
    padding-bottom: 10px;
  }

  .setting-group {
    margin-bottom: 20px;
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
    height: 24px;
    background: #bdc3c7;
    border-radius: 12px;
    position: relative;
    transition: background 0.3s ease;
  }

  .toggle-slider::before {
    content: '';
    position: absolute;
    width: 20px;
    height: 20px;
    background: white;
    border-radius: 50%;
    top: 2px;
    left: 2px;
    transition: transform 0.3s ease;
  }

  .toggle-setting input[type="checkbox"]:checked + .toggle-slider {
    background: #27ae60;
  }

  .toggle-setting input[type="checkbox"]:checked + .toggle-slider::before {
    transform: translateX(26px);
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
    background: #ecf0f1;
    border-radius: 3px;
    outline: none;
  }

  .range-value {
    min-width: 40px;
    font-weight: 600;
    color: #2c3e50;
  }

  .text-setting {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .text-setting input {
    padding: 12px 16px;
    border: 2px solid #ecf0f1;
    border-radius: 6px;
    font-size: 14px;
    transition: border-color 0.2s ease;
  }

  .text-setting input:focus {
    outline: none;
    border-color: #3498db;
  }

  .setting-label {
    font-weight: 600;
    color: #2c3e50;
    font-size: 16px;
  }

  .setting-description {
    color: #7f8c8d;
    font-size: 14px;
    margin: 8px 0 0 0;
    line-height: 1.4;
  }

  .performance-info, .security-info {
    background: #f8f9fa;
    border-radius: 8px;
    padding: 15px;
  }

  .info-item {
    display: flex;
    justify-content: space-between;
    margin: 8px 0;
  }

  .info-label {
    color: #7f8c8d;
  }

  .info-value {
    font-weight: 600;
    color: #27ae60;
  }

  .security-item {
    margin: 8px 0;
    color: #2c3e50;
  }

  .security-item.enabled {
    color: #27ae60;
  }

  .settings-footer {
    margin-top: 30px;
    text-align: center;
  }

  .save-button {
    padding: 15px 30px;
    background: #27ae60;
    color: white;
    border: none;
    border-radius: 8px;
    font-size: 16px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .save-button:hover:not(:disabled) {
    background: #229954;
    transform: translateY(-2px);
    box-shadow: 0 4px 15px rgba(39, 174, 96, 0.3);
  }

  .save-button:disabled {
    background: #bdc3c7;
    cursor: not-allowed;
    transform: none;
  }

  .save-button.saving {
    animation: pulse 1s infinite;
  }

  .loading-state {
    text-align: center;
    padding: 60px;
    color: #7f8c8d;
  }

  .loading-spinner {
    width: 40px;
    height: 40px;
    border: 4px solid #ecf0f1;
    border-top: 4px solid #3498db;
    border-radius: 50%;
    animation: spin 1s linear infinite;
    margin: 0 auto 20px;
  }

  @keyframes spin {
    0% { transform: rotate(0deg); }
    100% { transform: rotate(360deg); }
  }

  @keyframes pulse {
    0% { opacity: 1; }
    50% { opacity: 0.7; }
    100% { opacity: 1; }
  }

  @media (max-width: 768px) {
    .settings-header {
      flex-direction: column;
      gap: 20px;
      align-items: stretch;
    }

    .header-actions {
      justify-content: center;
      flex-wrap: wrap;
    }

    .range-container {
      flex-direction: column;
      align-items: stretch;
    }
  }
</style>