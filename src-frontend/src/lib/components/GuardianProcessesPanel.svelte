<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/tauri';

  interface Process {
    name: string;
    pid: number;
    cpu_usage: number;
    memory_mb: number;
    timestamp: string;
  }

  let processes: Process[] = [];
  let loading = true;
  let error: string | null = null;
  let refreshInterval: number;
  let cpuThreshold = 50.0;
  let timeRange = 1; // hours

  async function fetchHighCpuProcesses() {
    try {
      const response = await invoke<{ processes: any[]; count: number }>('get_high_cpu_processes', {
        threshold: cpuThreshold,
        hours: timeRange
      });

      // Parse and deduplicate processes
      const processMap = new Map<number, Process>();

      for (const proc of response.processes) {
        const pid = proc.pid || 0;
        const existing = processMap.get(pid);

        if (!existing || new Date(proc.timestamp) > new Date(existing.timestamp)) {
          processMap.set(pid, {
            name: proc.name || 'Unknown',
            pid: pid,
            cpu_usage: proc.cpu_usage || 0,
            memory_mb: proc.memory_mb || 0,
            timestamp: proc.timestamp || new Date().toISOString()
          });
        }
      }

      processes = Array.from(processMap.values())
        .sort((a, b) => b.cpu_usage - a.cpu_usage)
        .slice(0, 20); // Top 20 processes

      loading = false;
      error = null;
    } catch (e) {
      error = `Failed to fetch processes: ${e}`;
      loading = false;
      console.error(error);
    }
  }

  onMount(() => {
    fetchHighCpuProcesses();
    refreshInterval = window.setInterval(fetchHighCpuProcesses, 10000); // Refresh every 10 seconds
  });

  onDestroy(() => {
    if (refreshInterval) {
      clearInterval(refreshInterval);
    }
  });

  function formatMemory(mb: number): string {
    if (mb >= 1024) {
      return `${(mb / 1024).toFixed(2)} GB`;
    }
    return `${mb.toFixed(0)} MB`;
  }

  function getCpuColor(usage: number): string {
    if (usage >= 90) return '#ef4444';
    if (usage >= 70) return '#f59e0b';
    if (usage >= 50) return '#3b82f6';
    return '#10b981';
  }

  function handleThresholdChange(event: Event) {
    const target = event.target as HTMLInputElement;
    cpuThreshold = parseFloat(target.value);
    fetchHighCpuProcesses();
  }

  function handleTimeRangeChange(event: Event) {
    const target = event.target as HTMLSelectElement;
    timeRange = parseInt(target.value);
    fetchHighCpuProcesses();
  }
</script>

<div class="processes-panel">
  <div class="panel-header">
    <h3>⚡ High CPU Processes</h3>
    <button class="refresh-btn" onclick={fetchHighCpuProcesses} disabled={loading}>
      <span class="refresh-icon {loading ? 'spinning' : ''}">↻</span>
      Refresh
    </button>
  </div>

  <div class="controls">
    <div class="control-group">
      <label for="threshold">CPU Threshold: {cpuThreshold.toFixed(0)}%</label>
      <input
        id="threshold"
        type="range"
        min="10"
        max="100"
        step="5"
        value={cpuThreshold}
        oninput={handleThresholdChange}
      />
    </div>
    <div class="control-group">
      <label for="timeRange">Time Range</label>
      <select id="timeRange" value={timeRange} onchange={handleTimeRangeChange}>
        <option value="1">Last Hour</option>
        <option value="6">Last 6 Hours</option>
        <option value="24">Last 24 Hours</option>
        <option value="168">Last Week</option>
      </select>
    </div>
  </div>

  {#if error}
    <div class="error-message">
      <span>⚠️</span>
      <p>{error}</p>
    </div>
  {/if}

  {#if loading}
    <div class="loading-state">
      <div class="spinner"></div>
      <p>Loading processes...</p>
    </div>
  {:else if processes.length === 0}
    <div class="empty-state">
      <span class="empty-icon">✅</span>
      <h4>No High CPU Processes</h4>
      <p>No processes found exceeding {cpuThreshold.toFixed(0)}% CPU usage in the selected time range.</p>
    </div>
  {:else}
    <div class="processes-table">
      <div class="table-header">
        <div class="col-name">Process Name</div>
        <div class="col-pid">PID</div>
        <div class="col-cpu">CPU Usage</div>
        <div class="col-memory">Memory</div>
      </div>
      <div class="table-body">
        {#each processes as process}
          <div class="table-row">
            <div class="col-name">
              <span class="process-name" title={process.name}>{process.name}</span>
            </div>
            <div class="col-pid">
              <span class="process-pid">{process.pid}</span>
            </div>
            <div class="col-cpu">
              <div class="cpu-bar-container">
                <div
                  class="cpu-bar"
                  style="width: {Math.min(process.cpu_usage, 100)}%; background: {getCpuColor(process.cpu_usage)}"
                ></div>
                <span class="cpu-value">{process.cpu_usage.toFixed(1)}%</span>
              </div>
            </div>
            <div class="col-memory">
              <span class="memory-value">{formatMemory(process.memory_mb)}</span>
            </div>
          </div>
        {/each}
      </div>
    </div>

    <div class="summary">
      <p>Showing {processes.length} process{processes.length !== 1 ? 'es' : ''} exceeding {cpuThreshold.toFixed(0)}% CPU usage</p>
    </div>
  {/if}
</div>

<style>
  .processes-panel {
    background: white;
    border: 1px solid #e5e7eb;
    border-radius: 0.75rem;
    padding: 1.5rem;
    max-width: 1200px;
    margin: 0 auto;
  }

  .panel-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1.5rem;
  }

  .panel-header h3 {
    font-size: 1.5rem;
    font-weight: 700;
    color: #1f2937;
  }

  .refresh-btn {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 1rem;
    background: #3b82f6;
    color: white;
    border: none;
    border-radius: 0.5rem;
    cursor: pointer;
    font-weight: 500;
    transition: background 0.2s;
  }

  .refresh-btn:hover:not(:disabled) {
    background: #2563eb;
  }

  .refresh-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .refresh-icon {
    font-size: 1.25rem;
    display: inline-block;
  }

  .refresh-icon.spinning {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .controls {
    display: flex;
    gap: 2rem;
    margin-bottom: 1.5rem;
    padding: 1rem;
    background: #f9fafb;
    border-radius: 0.5rem;
  }

  .control-group {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    flex: 1;
  }

  .control-group label {
    font-size: 0.875rem;
    font-weight: 500;
    color: #374151;
  }

  .control-group input[type="range"] {
    width: 100%;
    height: 0.5rem;
    background: #e5e7eb;
    border-radius: 0.25rem;
    outline: none;
    -webkit-appearance: none;
  }

  .control-group input[type="range"]::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    width: 1rem;
    height: 1rem;
    background: #3b82f6;
    border-radius: 50%;
    cursor: pointer;
  }

  .control-group input[type="range"]::-moz-range-thumb {
    width: 1rem;
    height: 1rem;
    background: #3b82f6;
    border-radius: 50%;
    cursor: pointer;
    border: none;
  }

  .control-group select {
    padding: 0.5rem;
    border: 1px solid #d1d5db;
    border-radius: 0.375rem;
    background: white;
    color: #374151;
    font-size: 0.875rem;
    cursor: pointer;
  }

  .error-message {
    background: #fee2e2;
    border: 1px solid #fecaca;
    border-radius: 0.5rem;
    padding: 1rem;
    display: flex;
    align-items: center;
    gap: 0.75rem;
    color: #991b1b;
    margin-bottom: 1rem;
  }

  .loading-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 3rem;
    gap: 1rem;
  }

  .spinner {
    width: 2.5rem;
    height: 2.5rem;
    border: 3px solid #e5e7eb;
    border-top-color: #3b82f6;
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 3rem;
    gap: 1rem;
    text-align: center;
  }

  .empty-icon {
    font-size: 3rem;
  }

  .empty-state h4 {
    font-size: 1.25rem;
    font-weight: 600;
    color: #1f2937;
  }

  .empty-state p {
    color: #6b7280;
  }

  .processes-table {
    border: 1px solid #e5e7eb;
    border-radius: 0.5rem;
    overflow: hidden;
  }

  .table-header {
    display: grid;
    grid-template-columns: 2fr 1fr 2fr 1fr;
    gap: 1rem;
    padding: 0.75rem 1rem;
    background: #f9fafb;
    border-bottom: 1px solid #e5e7eb;
    font-weight: 600;
    font-size: 0.875rem;
    color: #6b7280;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .table-body {
    max-height: 600px;
    overflow-y: auto;
  }

  .table-row {
    display: grid;
    grid-template-columns: 2fr 1fr 2fr 1fr;
    gap: 1rem;
    padding: 1rem;
    border-bottom: 1px solid #e5e7eb;
    transition: background 0.2s;
  }

  .table-row:hover {
    background: #f9fafb;
  }

  .table-row:last-child {
    border-bottom: none;
  }

  .col-name,
  .col-pid,
  .col-cpu,
  .col-memory {
    display: flex;
    align-items: center;
  }

  .process-name {
    font-weight: 500;
    color: #1f2937;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .process-pid {
    font-family: monospace;
    color: #6b7280;
    font-size: 0.875rem;
  }

  .cpu-bar-container {
    position: relative;
    width: 100%;
    height: 1.5rem;
    background: #e5e7eb;
    border-radius: 0.25rem;
    overflow: hidden;
  }

  .cpu-bar {
    height: 100%;
    transition: width 0.3s ease;
  }

  .cpu-value {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    font-size: 0.75rem;
    font-weight: 600;
    color: #1f2937;
    text-shadow: 0 0 2px white;
  }

  .memory-value {
    font-weight: 500;
    color: #374151;
  }

  .summary {
    padding: 1rem;
    text-align: center;
    color: #6b7280;
    font-size: 0.875rem;
  }

  @media (max-width: 768px) {
    .processes-panel {
      padding: 1rem;
    }

    .panel-header {
      flex-direction: column;
      align-items: flex-start;
      gap: 1rem;
    }

    .controls {
      flex-direction: column;
      gap: 1rem;
    }

    .table-header,
    .table-row {
      grid-template-columns: 1fr;
      gap: 0.5rem;
    }

    .table-header {
      display: none;
    }

    .table-row {
      padding: 1rem;
      display: flex;
      flex-direction: column;
      gap: 0.75rem;
    }

    .col-name::before {
      content: 'Process: ';
      font-weight: 600;
      color: #6b7280;
    }

    .col-pid::before {
      content: 'PID: ';
      font-weight: 600;
      color: #6b7280;
    }

    .col-memory::before {
      content: 'Memory: ';
      font-weight: 600;
      color: #6b7280;
    }
  }
</style>
