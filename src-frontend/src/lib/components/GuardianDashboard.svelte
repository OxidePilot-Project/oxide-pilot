<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/tauri';

  interface SystemMetric {
    timestamp: string;
    cpu_usage: number;
    memory_usage: {
      total_mb: number;
      used_mb: number;
      available_mb: number;
      percent: number;
    };
    disk_io: {
      read_mb_per_sec: number;
      write_mb_per_sec: number;
      iops: number;
    };
    network_stats: {
      sent_mb_per_sec: number;
      recv_mb_per_sec: number;
      connections_active: number;
    };
  }

  interface SystemStatus {
    status: 'healthy' | 'caution' | 'warning' | 'no_data';
    timestamp?: string;
    cpu_usage?: number;
    memory_usage?: { percent: number };
    message?: string;
  }

  let currentStatus: SystemStatus | null = null;
  let recentMetrics: SystemMetric[] = [];
  let loading = true;
  let error: string | null = null;
  let refreshInterval: number;
  let activeTab: 'overview' | 'cpu' | 'memory' | 'disk' | 'network' = 'overview';

  async function fetchSystemStatus() {
    try {
      currentStatus = await invoke<SystemStatus>('get_guardian_status');
      error = null;
    } catch (e) {
      error = `Failed to fetch status: ${e}`;
      console.error(error);
    }
  }

  async function fetchRecentMetrics() {
    try {
      const response = await invoke<{ metrics: SystemMetric[]; count: number }>('get_recent_metrics', {
        hours: 1
      });
      recentMetrics = response.metrics.slice(0, 60); // Last 5 minutes (60 samples at 5s interval)
      loading = false;
      error = null;
    } catch (e) {
      error = `Failed to fetch metrics: ${e}`;
      loading = false;
      console.error(error);
    }
  }

  async function refreshData() {
    await Promise.all([fetchSystemStatus(), fetchRecentMetrics()]);
  }

  onMount(() => {
    refreshData();
    refreshInterval = window.setInterval(refreshData, 5000); // Refresh every 5 seconds
  });

  onDestroy(() => {
    if (refreshInterval) {
      clearInterval(refreshInterval);
    }
  });

  function getStatusColor(status: string): string {
    switch (status) {
      case 'healthy': return 'text-green-500';
      case 'caution': return 'text-yellow-500';
      case 'warning': return 'text-red-500';
      default: return 'text-gray-500';
    }
  }

  function getStatusBg(status: string): string {
    switch (status) {
      case 'healthy': return 'bg-green-500/10';
      case 'caution': return 'bg-yellow-500/10';
      case 'warning': return 'bg-red-500/10';
      default: return 'bg-gray-500/10';
    }
  }

  function formatBytes(mb: number): string {
    if (mb >= 1024) {
      return `${(mb / 1024).toFixed(2)} GB`;
    }
    return `${mb.toFixed(2)} MB`;
  }

  function getLatestMetric(): SystemMetric | null {
    return recentMetrics.length > 0 ? recentMetrics[0] : null;
  }

  function calculateAverage(key: keyof SystemMetric): number {
    if (recentMetrics.length === 0) return 0;

    if (key === 'cpu_usage') {
      const sum = recentMetrics.reduce((acc, m) => acc + m.cpu_usage, 0);
      return sum / recentMetrics.length;
    }

    return 0;
  }
</script>

<div class="guardian-dashboard">
  <div class="dashboard-header">
    <h2>🛡️ Guardian Agent Dashboard</h2>
    <div class="status-indicator">
      {#if currentStatus}
        <span class="status-badge {getStatusBg(currentStatus.status)} {getStatusColor(currentStatus.status)}">
          {currentStatus.status.toUpperCase()}
        </span>
      {/if}
    </div>
  </div>

  {#if error}
    <div class="error-banner">
      <span>⚠️</span>
      <p>{error}</p>
    </div>
  {/if}

  {#if loading}
    <div class="loading-state">
      <div class="spinner"></div>
      <p>Loading metrics...</p>
    </div>
  {:else}
    <div class="tabs">
      <button
        class="tab {activeTab === 'overview' ? 'active' : ''}"
        onclick={() => activeTab = 'overview'}
      >
        Overview
      </button>
      <button
        class="tab {activeTab === 'cpu' ? 'active' : ''}"
        onclick={() => activeTab = 'cpu'}
      >
        CPU
      </button>
      <button
        class="tab {activeTab === 'memory' ? 'active' : ''}"
        onclick={() => activeTab = 'memory'}
      >
        Memory
      </button>
      <button
        class="tab {activeTab === 'disk' ? 'active' : ''}"
        onclick={() => activeTab = 'disk'}
      >
        Disk I/O
      </button>
      <button
        class="tab {activeTab === 'network' ? 'active' : ''}"
        onclick={() => activeTab = 'network'}
      >
        Network
      </button>
    </div>

    <div class="tab-content">
      {#if activeTab === 'overview'}
        <div class="overview-grid">
          {#if getLatestMetric()}
            {@const latest = getLatestMetric()}
            <div class="metric-card">
              <div class="metric-header">
                <span class="metric-icon">💻</span>
                <h3>CPU Usage</h3>
              </div>
              <div class="metric-value">
                <span class="value">{latest.cpu_usage.toFixed(1)}%</span>
                <span class="label">Current</span>
              </div>
              <div class="metric-bar">
                <div class="bar-fill" style="width: {latest.cpu_usage}%; background: {latest.cpu_usage > 90 ? '#ef4444' : latest.cpu_usage > 70 ? '#f59e0b' : '#10b981'}"></div>
              </div>
            </div>

            <div class="metric-card">
              <div class="metric-header">
                <span class="metric-icon">🧠</span>
                <h3>Memory Usage</h3>
              </div>
              <div class="metric-value">
                <span class="value">{latest.memory_usage.percent.toFixed(1)}%</span>
                <span class="label">{formatBytes(latest.memory_usage.used_mb)} / {formatBytes(latest.memory_usage.total_mb)}</span>
              </div>
              <div class="metric-bar">
                <div class="bar-fill" style="width: {latest.memory_usage.percent}%; background: {latest.memory_usage.percent > 90 ? '#ef4444' : latest.memory_usage.percent > 70 ? '#f59e0b' : '#10b981'}"></div>
              </div>
            </div>

            <div class="metric-card">
              <div class="metric-header">
                <span class="metric-icon">💾</span>
                <h3>Disk I/O</h3>
              </div>
              <div class="metric-value">
                <span class="value">{latest.disk_io.iops}</span>
                <span class="label">IOPS</span>
              </div>
              <div class="metric-stats">
                <div class="stat">
                  <span class="stat-label">Read</span>
                  <span class="stat-value">{latest.disk_io.read_mb_per_sec.toFixed(2)} MB/s</span>
                </div>
                <div class="stat">
                  <span class="stat-label">Write</span>
                  <span class="stat-value">{latest.disk_io.write_mb_per_sec.toFixed(2)} MB/s</span>
                </div>
              </div>
            </div>

            <div class="metric-card">
              <div class="metric-header">
                <span class="metric-icon">🌐</span>
                <h3>Network</h3>
              </div>
              <div class="metric-value">
                <span class="value">{latest.network_stats.connections_active}</span>
                <span class="label">Active Connections</span>
              </div>
              <div class="metric-stats">
                <div class="stat">
                  <span class="stat-label">Sent</span>
                  <span class="stat-value">{latest.network_stats.sent_mb_per_sec.toFixed(2)} MB/s</span>
                </div>
                <div class="stat">
                  <span class="stat-label">Received</span>
                  <span class="stat-value">{latest.network_stats.recv_mb_per_sec.toFixed(2)} MB/s</span>
                </div>
              </div>
            </div>
          {/if}
        </div>
      {:else if activeTab === 'cpu'}
        <div class="detail-view">
          <h3>CPU Usage History (Last 5 Minutes)</h3>
          <div class="chart-container">
            <div class="simple-chart">
              {#each recentMetrics.slice().reverse() as metric, i}
                <div class="chart-bar" style="height: {metric.cpu_usage}%; background: {metric.cpu_usage > 90 ? '#ef4444' : metric.cpu_usage > 70 ? '#f59e0b' : '#10b981'}">
                  <span class="bar-tooltip">{metric.cpu_usage.toFixed(1)}%</span>
                </div>
              {/each}
            </div>
          </div>
          <div class="stats-summary">
            <div class="stat-item">
              <span class="stat-label">Average</span>
              <span class="stat-value">{calculateAverage('cpu_usage').toFixed(1)}%</span>
            </div>
            <div class="stat-item">
              <span class="stat-label">Current</span>
              <span class="stat-value">{getLatestMetric()?.cpu_usage.toFixed(1)}%</span>
            </div>
          </div>
        </div>
      {:else if activeTab === 'memory'}
        <div class="detail-view">
          <h3>Memory Usage Details</h3>
          {#if getLatestMetric()}
            {@const latest = getLatestMetric()}
            <div class="memory-details">
              <div class="memory-stat">
                <span class="label">Total Memory</span>
                <span class="value">{formatBytes(latest.memory_usage.total_mb)}</span>
              </div>
              <div class="memory-stat">
                <span class="label">Used Memory</span>
                <span class="value">{formatBytes(latest.memory_usage.used_mb)}</span>
              </div>
              <div class="memory-stat">
                <span class="label">Available Memory</span>
                <span class="value">{formatBytes(latest.memory_usage.available_mb)}</span>
              </div>
              <div class="memory-stat">
                <span class="label">Usage Percentage</span>
                <span class="value">{latest.memory_usage.percent.toFixed(1)}%</span>
              </div>
            </div>
          {/if}
        </div>
      {:else if activeTab === 'disk'}
        <div class="detail-view">
          <h3>Disk I/O Performance</h3>
          {#if getLatestMetric()}
            {@const latest = getLatestMetric()}
            <div class="disk-details">
              <div class="disk-stat">
                <span class="label">Read Speed</span>
                <span class="value">{latest.disk_io.read_mb_per_sec.toFixed(2)} MB/s</span>
              </div>
              <div class="disk-stat">
                <span class="label">Write Speed</span>
                <span class="value">{latest.disk_io.write_mb_per_sec.toFixed(2)} MB/s</span>
              </div>
              <div class="disk-stat">
                <span class="label">IOPS</span>
                <span class="value">{latest.disk_io.iops}</span>
              </div>
            </div>
          {/if}
        </div>
      {:else if activeTab === 'network'}
        <div class="detail-view">
          <h3>Network Statistics</h3>
          {#if getLatestMetric()}
            {@const latest = getLatestMetric()}
            <div class="network-details">
              <div class="network-stat">
                <span class="label">Active Connections</span>
                <span class="value">{latest.network_stats.connections_active}</span>
              </div>
              <div class="network-stat">
                <span class="label">Upload Speed</span>
                <span class="value">{latest.network_stats.sent_mb_per_sec.toFixed(2)} MB/s</span>
              </div>
              <div class="network-stat">
                <span class="label">Download Speed</span>
                <span class="value">{latest.network_stats.recv_mb_per_sec.toFixed(2)} MB/s</span>
              </div>
            </div>
          {/if}
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .guardian-dashboard {
    padding: 1.5rem;
    max-width: 1400px;
    margin: 0 auto;
  }

  .dashboard-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 2rem;
  }

  .dashboard-header h2 {
    font-size: 1.75rem;
    font-weight: 700;
    color: #1f2937;
  }

  .status-indicator {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .status-badge {
    padding: 0.5rem 1rem;
    border-radius: 0.5rem;
    font-weight: 600;
    font-size: 0.875rem;
  }

  .error-banner {
    background: #fee2e2;
    border: 1px solid #fecaca;
    border-radius: 0.5rem;
    padding: 1rem;
    margin-bottom: 1.5rem;
    display: flex;
    align-items: center;
    gap: 0.75rem;
    color: #991b1b;
  }

  .loading-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 4rem;
    gap: 1rem;
  }

  .spinner {
    width: 3rem;
    height: 3rem;
    border: 4px solid #e5e7eb;
    border-top-color: #3b82f6;
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .tabs {
    display: flex;
    gap: 0.5rem;
    border-bottom: 2px solid #e5e7eb;
    margin-bottom: 1.5rem;
  }

  .tab {
    padding: 0.75rem 1.5rem;
    background: none;
    border: none;
    border-bottom: 2px solid transparent;
    cursor: pointer;
    font-weight: 500;
    color: #6b7280;
    transition: all 0.2s;
    margin-bottom: -2px;
  }

  .tab:hover {
    color: #3b82f6;
  }

  .tab.active {
    color: #3b82f6;
    border-bottom-color: #3b82f6;
  }

  .overview-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
    gap: 1.5rem;
  }

  .metric-card {
    background: white;
    border: 1px solid #e5e7eb;
    border-radius: 0.75rem;
    padding: 1.5rem;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
  }

  .metric-header {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    margin-bottom: 1rem;
  }

  .metric-icon {
    font-size: 1.5rem;
  }

  .metric-header h3 {
    font-size: 1rem;
    font-weight: 600;
    color: #374151;
  }

  .metric-value {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    margin-bottom: 1rem;
  }

  .metric-value .value {
    font-size: 2rem;
    font-weight: 700;
    color: #1f2937;
  }

  .metric-value .label {
    font-size: 0.875rem;
    color: #6b7280;
  }

  .metric-bar {
    height: 0.5rem;
    background: #e5e7eb;
    border-radius: 0.25rem;
    overflow: hidden;
  }

  .bar-fill {
    height: 100%;
    transition: width 0.3s ease;
  }

  .metric-stats {
    display: flex;
    gap: 1rem;
  }

  .stat {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .stat-label {
    font-size: 0.75rem;
    color: #6b7280;
    text-transform: uppercase;
    font-weight: 600;
  }

  .stat-value {
    font-size: 1rem;
    font-weight: 600;
    color: #1f2937;
  }

  .detail-view {
    background: white;
    border: 1px solid #e5e7eb;
    border-radius: 0.75rem;
    padding: 1.5rem;
  }

  .detail-view h3 {
    font-size: 1.25rem;
    font-weight: 600;
    color: #1f2937;
    margin-bottom: 1.5rem;
  }

  .chart-container {
    margin-bottom: 1.5rem;
  }

  .simple-chart {
    display: flex;
    align-items: flex-end;
    gap: 2px;
    height: 200px;
    padding: 1rem;
    background: #f9fafb;
    border-radius: 0.5rem;
  }

  .chart-bar {
    flex: 1;
    min-width: 4px;
    position: relative;
    transition: all 0.3s ease;
    border-radius: 2px 2px 0 0;
  }

  .chart-bar:hover {
    opacity: 0.8;
  }

  .bar-tooltip {
    display: none;
    position: absolute;
    bottom: 100%;
    left: 50%;
    transform: translateX(-50%);
    background: #1f2937;
    color: white;
    padding: 0.25rem 0.5rem;
    border-radius: 0.25rem;
    font-size: 0.75rem;
    white-space: nowrap;
    margin-bottom: 0.25rem;
  }

  .chart-bar:hover .bar-tooltip {
    display: block;
  }

  .stats-summary {
    display: flex;
    gap: 2rem;
    padding: 1rem;
    background: #f9fafb;
    border-radius: 0.5rem;
  }

  .stat-item {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .stat-item .stat-label {
    font-size: 0.875rem;
    color: #6b7280;
  }

  .stat-item .stat-value {
    font-size: 1.5rem;
    font-weight: 700;
    color: #1f2937;
  }

  .memory-details,
  .disk-details,
  .network-details {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 1.5rem;
  }

  .memory-stat,
  .disk-stat,
  .network-stat {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    padding: 1rem;
    background: #f9fafb;
    border-radius: 0.5rem;
  }

  .memory-stat .label,
  .disk-stat .label,
  .network-stat .label {
    font-size: 0.875rem;
    color: #6b7280;
    font-weight: 500;
  }

  .memory-stat .value,
  .disk-stat .value,
  .network-stat .value {
    font-size: 1.25rem;
    font-weight: 700;
    color: #1f2937;
  }

  @media (max-width: 768px) {
    .guardian-dashboard {
      padding: 1rem;
    }

    .dashboard-header {
      flex-direction: column;
      align-items: flex-start;
      gap: 1rem;
    }

    .overview-grid {
      grid-template-columns: 1fr;
    }

    .tabs {
      overflow-x: auto;
      -webkit-overflow-scrolling: touch;
    }

    .tab {
      white-space: nowrap;
    }
  }
</style>
