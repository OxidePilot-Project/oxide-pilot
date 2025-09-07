<script lang="ts">
import { onDestroy, onMount } from "svelte";
import { writable } from "svelte/store";
import { isTauri } from "$lib/utils/env";

// Centralized invoke utility
import PerformancePanel from "./PerformancePanel.svelte";
import AudioControls from "./AudioControls.svelte";
import { tauriInvoke } from "$lib/utils/tauri";

interface SystemStatus {
  cpu_usage: number;
  memory_usage: [number, number]; // [used, total]
  process_count: number;
  threat_count: number;
}

interface ThreatEvent {
  id: string;
  timestamp: string;
  threat_type: string;
  severity: string;
  description: string;
  process_name?: string;
  process_id?: number;
}

interface MemoryStats {
  total_entries: number;
  total_patterns: number;
  storage_path: string;
  max_entries: number;
}

const systemStatus = writable<SystemStatus | null>(null);
const threats = writable<ThreatEvent[]>([]);
const memoryStats = writable<MemoryStats | null>(null);
const isSystemInitialized = writable(false);

let updateInterval: number;

onMount(async () => {
  // Check if system is initialized and start monitoring
  await checkSystemStatus();

  // Update every 5 seconds
  updateInterval = setInterval(async () => {
    await updateDashboard();
  }, 5000);
});

onDestroy(() => {
  if (updateInterval) {
    clearInterval(updateInterval);
  }
});

async function checkSystemStatus() {
  try {
    const status = await tauriInvoke("get_system_status");
    systemStatus.set(status as SystemStatus);
    isSystemInitialized.set(true);
    await updateDashboard();
  } catch (_error) {
    console.log("System not initialized yet");
    isSystemInitialized.set(false);
  }
}

async function updateDashboard() {
  try {
    const [status, threatHistory, memory] = await Promise.all([
      tauriInvoke("get_system_status"),
      tauriInvoke("get_threat_history"),
      tauriInvoke("get_memory_stats"),
    ]);

    systemStatus.set(status as SystemStatus);
    threats.set(threatHistory as ThreatEvent[]);
    memoryStats.set(memory as MemoryStats);
  } catch (error) {
    console.error("Failed to update dashboard:", error);
  }
}

async function initializeSystem() {
  try {
    const defaultConfig = {
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
          api_key: "your-google-api-key",
        },
        openai: null,
        anthropic: null,
        azure_openai: null,
        ollama: null,
      },
    };

    await tauriInvoke("initialize_system", { config: defaultConfig });
    isSystemInitialized.set(true);
    await updateDashboard();
  } catch (error) {
    console.error("Failed to initialize system:", error);
    alert(`Failed to initialize system: ${error}`);
  }
}

function formatBytes(bytes: number): string {
  const sizes = ["Bytes", "KB", "MB", "GB"];
  if (bytes === 0) return "0 Bytes";
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  return `${Math.round((bytes / 1024 ** i) * 100) / 100} ${sizes[i]}`;
}

function getSeverityColor(severity: string): string {
  switch (severity.toLowerCase()) {
    case "critical":
      return "#dc3545";
    case "high":
      return "#fd7e14";
    case "medium":
      return "#ffc107";
    case "low":
      return "#28a745";
    default:
      return "#6c757d";
  }
}
</script>

<div class="dashboard">
  <h2>Oxide Pilot System Dashboard</h2>

  {#if !$isSystemInitialized}
    <div class="initialization-panel">
      <h3>System Not Initialized</h3>
      <p>The Oxide Pilot system needs to be initialized before use.</p>
      <button on:click={initializeSystem} class="init-button">Initialize System</button>
    </div>
  {:else}
    <!-- System Status Panel -->
    {#if $systemStatus}
      <div class="status-panel">
        <h3>System Status</h3>
        <div class="status-grid">
          <div class="status-item">
            <span class="label">CPU Usage:</span>
            <span class="value">{$systemStatus.cpu_usage.toFixed(1)}%</span>
            <div class="progress-bar">
              <div class="progress" style="width: {$systemStatus.cpu_usage}%"></div>
            </div>
          </div>
          <div class="status-item">
            <span class="label">Memory:</span>
            <span class="value">
              {formatBytes($systemStatus.memory_usage[0])} / {formatBytes($systemStatus.memory_usage[1])}
            </span>
            <div class="progress-bar">
              <div class="progress" style="width: {($systemStatus.memory_usage[0] / $systemStatus.memory_usage[1]) * 100}%"></div>
            </div>
          </div>
          <div class="status-item">
            <span class="label">Processes:</span>
            <span class="value">{$systemStatus.process_count}</span>
          </div>
          <div class="status-item">
            <span class="label">Threats Detected:</span>
            <span class="value threat-count">{$systemStatus.threat_count}</span>
          </div>
        </div>
      </div>
    {/if}

    <!-- Memory Statistics Panel -->
    {#if $memoryStats}
      <div class="memory-panel">
        <h3>Memory System</h3>
        <div class="memory-grid">
          <div class="memory-item">
            <span class="label">Total Entries:</span>
            <span class="value">{$memoryStats.total_entries}</span>
          </div>
          <div class="memory-item">
            <span class="label">User Patterns:</span>
            <span class="value">{$memoryStats.total_patterns}</span>
          </div>
          <div class="memory-item">
            <span class="label">Storage Path:</span>
            <span class="value small">{$memoryStats.storage_path}</span>
          </div>
          <div class="memory-item">
            <span class="label">Capacity:</span>
            <span class="value">{$memoryStats.total_entries} / {$memoryStats.max_entries}</span>
          </div>
        </div>
      </div>
    {/if}

    <!-- Threats Panel -->
    <div class="threats-panel">
      <h3>Recent Threats ({$threats.length})</h3>
      {#if $threats.length === 0}
        <p class="no-threats">No threats detected</p>
      {:else}
        <div class="threats-list">
          {#each $threats.slice(-10) as threat (threat.id)}
            <div class="threat-item" style="border-left-color: {getSeverityColor(threat.severity)}">
              <div class="threat-header">
                <span class="threat-type">{threat.threat_type}</span>
                <span class="threat-severity" style="color: {getSeverityColor(threat.severity)}">
                  {threat.severity.toUpperCase()}
                </span>
                <span class="threat-time">{new Date(threat.timestamp).toLocaleTimeString()}</span>
              </div>
              <div class="threat-description">{threat.description}</div>
              {#if threat.process_name}
                <div class="threat-process">Process: {threat.process_name} (PID: {threat.process_id})</div>
              {/if}
            </div>
          {/each}
        </div>
      {/if}
    </div>

    <!-- Performance Panel -->
    <PerformancePanel />

    <!-- Audio Controls Panel -->
    <AudioControls />
  {/if}
</div>

<style>
  .dashboard {
    padding: 20px;
    max-width: 1200px;
    margin: 0 auto;
  }

  .initialization-panel {
    text-align: center;
    padding: 40px;
    background: #f8f9fa;
    border-radius: 8px;
    margin: 20px 0;
  }

  .init-button {
    padding: 12px 24px;
    background: #007bff;
    color: white;
    border: none;
    border-radius: 6px;
    font-size: 16px;
    cursor: pointer;
    margin-top: 20px;
  }

  .init-button:hover {
    background: #0056b3;
  }

  .status-panel, .memory-panel, .threats-panel {
    background: var(--color-surface);
    border-radius: 12px;
    padding: 20px;
    margin: 20px 0;
    box-shadow: var(--shadow-md);
    border: 1px solid rgba(0,0,0,0.06);
  }

  .status-grid, .memory-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
    gap: 15px;
    margin-top: 15px;
  }

  .status-item, .memory-item {
    display: flex;
    flex-direction: column;
    gap: 5px;
  }

  .label { font-weight: 600; color: var(--color-muted); font-size: 14px; }

  .value { font-size: 18px; font-weight: 700; color: var(--color-text); }

  .value.small { font-size: 12px; font-weight: 400; color: var(--color-muted); }

  .threat-count {
    color: #dc3545;
  }

  .progress-bar {
    width: 100%;
    height: 8px;
    background: #e9ecef;
    border-radius: 4px;
    overflow: hidden;
  }

  .progress {
    height: 100%;
    background: linear-gradient(90deg, #28a745, #ffc107, #dc3545);
    transition: width 0.3s ease;
  }

  .no-threats {
    color: #28a745;
    font-style: italic;
    text-align: center;
    padding: 20px;
  }

  .threats-list {
    max-height: 400px;
    overflow-y: auto;
  }

  .threat-item {
    border-left: 4px solid;
    padding: 12px;
    margin: 8px 0;
    background: var(--color-surface);
    border-radius: 10px;
    box-shadow: var(--shadow-sm, 0 2px 6px rgba(0,0,0,0.06));
    border: 1px solid rgba(0,0,0,0.06);
  }

  .threat-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 8px;
  }

  .threat-type { font-weight: 600; color: var(--color-text); }

  .threat-severity {
    font-weight: 700;
    font-size: 12px;
    padding: 2px 6px;
    border-radius: 3px;
    background: rgba(255,255,255,0.8);
  }

  .threat-time { font-size: 12px; color: var(--color-muted); }

  .threat-description { color: var(--color-text); margin-bottom: 4px; }

  .threat-process { font-size: 12px; color: var(--color-muted); font-family: monospace; }

  h2, h3 { color: var(--color-text); margin-bottom: 15px; }

  h2 { text-align: center; border-bottom: 1px solid rgba(0,0,0,0.08); padding-bottom: 10px; }

  .init-button { transition: background-color 0.15s ease, transform 0.05s ease; }
  .init-button:focus-visible { outline: 2px solid #3b82f6; outline-offset: 2px; }
  .init-button:active { transform: translateY(1px); }
</style>
