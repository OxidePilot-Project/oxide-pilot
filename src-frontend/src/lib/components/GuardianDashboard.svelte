<script lang="ts">
import { onDestroy, onMount } from "svelte";
import {
  getGuardianStatus,
  getHourlyMetrics,
  getMetricsSummary,
  getRecentMetrics,
  type HourlyMetricsRow,
  type MetricsSummary,
  predictThreatRisk,
  type SystemMetric,
  subscribeGuardianMetrics,
  type ThreatPrediction,
} from "../utils/guardian";

interface SystemStatus {
  status: "healthy" | "caution" | "warning" | "no_data";
  timestamp?: string;
  cpu_usage?: number;
  memory_usage?: { percent: number };
  message?: string;
}

let currentStatus: SystemStatus | null = null;
let metricsSummary: MetricsSummary | null = null;
let hourlyMetrics: HourlyMetricsRow[] = [];
let recentMetrics: SystemMetric[] = [];
let liveMetric: SystemMetric | null = null;
let threatPrediction: ThreatPrediction | null = null;
let loading = true;
let error: string | null = null;
let refreshInterval: number;
let hourlyInterval: number;
let metricsUnlisten: (() => void) | null = null;
let activeTab: "overview" | "cpu" | "memory" | "disk" | "network" =
  "overview";

async function fetchSystemStatus() {
  try {
    currentStatus = (await getGuardianStatus()) as SystemStatus;
    error = null;
  } catch (e) {
    console.error(e);
    error = `Failed to fetch status: ${e}`;
  }
}

async function fetchRecentMetrics(hours = 1) {
  try {
    const response = await getRecentMetrics(hours);
    recentMetrics = response.metrics.slice(0, 120);
    loading = false;
    error = null;
  } catch (e) {
    console.error(e);
    error = `Failed to fetch metrics: ${e}`;
    loading = false;
  }
}

async function fetchSummary(hours = 6) {
  try {
    metricsSummary = await getMetricsSummary(hours);
  } catch (e) {
    console.warn("Failed to fetch metrics summary", e);
  }
}

async function fetchHourly(hours = 12) {
  try {
    hourlyMetrics = await getHourlyMetrics(hours);
  } catch (e) {
    console.warn("Failed to fetch hourly metrics", e);
  }
}

function buildFeatureVector(
  metric: SystemMetric | null,
): Record<string, number> | null {
  if (!metric) return null;
  const avgCpu = metricsSummary?.avg_cpu ?? metric.cpu_usage;
  const anomaly = Math.abs(metric.cpu_usage - avgCpu);
  return {
    cpu_usage: metric.cpu_usage,
    memory_pressure: metric.memory_usage.percent,
    network_score: metric.network_stats.connections_active,
    anomaly_score: anomaly,
  };
}

async function refreshThreatPrediction(metric?: SystemMetric) {
  const latest = metric ?? liveMetric ?? recentMetrics[0] ?? null;
  const features = buildFeatureVector(latest);
  if (!features) return;

  try {
    threatPrediction = await predictThreatRisk(features);
  } catch (e) {
    console.warn("Threat prediction failed", e);
  }
}

async function refreshData() {
  await Promise.all([
    fetchSystemStatus(),
    fetchRecentMetrics(),
    fetchSummary(),
    fetchHourly(),
  ]);
  await refreshThreatPrediction();
}

onMount(async () => {
  await refreshData();
  refreshInterval = window.setInterval(fetchSystemStatus, 15000);
  hourlyInterval = window.setInterval(async () => {
    await Promise.all([fetchSummary(), fetchHourly()]);
    await refreshThreatPrediction();
  }, 60000);

  metricsUnlisten = await subscribeGuardianMetrics(async (metric) => {
    liveMetric = metric;
    recentMetrics = [metric, ...recentMetrics].slice(0, 120);
    await refreshThreatPrediction(metric);
  });
});

onDestroy(() => {
  if (refreshInterval) clearInterval(refreshInterval);
  if (hourlyInterval) clearInterval(hourlyInterval);
  metricsUnlisten?.();
});

function getStatusColor(status: string): string {
  switch (status) {
    case "healthy":
      return "text-green-500";
    case "caution":
      return "text-yellow-500";
    case "warning":
      return "text-red-500";
    default:
      return "text-gray-500";
  }
}

function getStatusBg(status: string): string {
  switch (status) {
    case "healthy":
      return "bg-green-500/10";
    case "caution":
      return "bg-yellow-500/10";
    case "warning":
      return "bg-red-500/10";
    default:
      return "bg-gray-500/10";
  }
}

function formatBytes(mb: number): string {
  if (mb >= 1024) {
    return `${(mb / 1024).toFixed(2)} GB`;
  }
  return `${mb.toFixed(2)} MB`;
}

function getLatestMetric(): SystemMetric | null {
  return liveMetric ?? (recentMetrics.length > 0 ? recentMetrics[0] : null);
}

function calculateAverage(key: keyof SystemMetric): number {
  if (key === "cpu_usage") {
    if (metricsSummary) {
      return metricsSummary.avg_cpu;
    }
    if (recentMetrics.length > 0) {
      const sum = recentMetrics.reduce((acc, m) => acc + m.cpu_usage, 0);
      return sum / recentMetrics.length;
    }
  }
  return 0;
}

function formatHourBucket(bucket: string): string {
  try {
    return new Date(bucket).toLocaleTimeString([], {
      hour: "2-digit",
      minute: "2-digit",
    });
  } catch {
    return bucket;
  }
}

function getThreatColor(severity: string | undefined): string {
  switch (severity) {
    case "critical":
      return "text-red-500";
    case "high":
      return "text-orange-500";
    case "medium":
      return "text-yellow-500";
    case "low":
      return "text-green-500";
    default:
      return "text-gray-500";
  }
}

function getThreatBg(severity: string | undefined): string {
  switch (severity) {
    case "critical":
      return "bg-red-500/10 border border-red-200";
    case "high":
      return "bg-orange-500/10 border border-orange-200";
    case "medium":
      return "bg-yellow-500/10 border border-yellow-200";
    case "low":
      return "bg-green-500/10 border border-green-200";
    default:
      return "bg-gray-500/10 border border-gray-200";
  }
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
        {@const latest = getLatestMetric()}
        <div class="overview-grid">
          {#if latest}
            <div class="metric-card">
              <div class="metric-header">
                <span class="metric-icon">CPU</span>
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
                <span class="metric-icon">MEM</span>
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
                <span class="metric-icon">IO</span>
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
                <span class="metric-icon">NET</span>
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

          {#if metricsSummary}
            <div class="metric-card">
              <div class="metric-header">
                <span class="metric-icon">AVG</span>
                <h3>Last {metricsSummary.sample_count} Samples</h3>
              </div>
              <div class="metric-value">
                <span class="value">{metricsSummary.avg_cpu.toFixed(1)}%</span>
                <span class="label">Average CPU</span>
              </div>
              <div class="metric-stats">
                <div class="stat">
                  <span class="stat-label">Peak CPU</span>
                  <span class="stat-value">{metricsSummary.max_cpu.toFixed(1)}%</span>
                </div>
                <div class="stat">
                  <span class="stat-label">Avg Memory</span>
                  <span class="stat-value">{metricsSummary.avg_memory_percent.toFixed(1)}%</span>
                </div>
              </div>
              <p class="metric-footnote">
                Window: {metricsSummary.window_start
                  ? new Date(metricsSummary.window_start).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
                  : 'unknown'} - {metricsSummary.window_end
                  ? new Date(metricsSummary.window_end).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
                  : 'now'}
              </p>
            </div>
          {/if}

          {#if threatPrediction}
            <div class="metric-card threat-card {getThreatBg(threatPrediction.severity)}">
              <div class="metric-header">
                <span class="metric-icon">RISK</span>
                <h3>Threat Risk</h3>
              </div>
              <div class="metric-value">
                <span class="value {getThreatColor(threatPrediction.severity)}">
                  {threatPrediction.severity?.toUpperCase() ?? 'UNKNOWN'}
                </span>
                <span class="label">
                  Score {(threatPrediction.score * 100).toFixed(0)}%
                  {#if threatPrediction.confidence !== undefined}
                    + Confidence {(threatPrediction.confidence * 100).toFixed(0)}%
                  {/if}
                </span>
              </div>
              <p class="metric-footnote">
                Provider: {threatPrediction.provider ?? 'heuristic fallback'}
              </p>
            </div>
          {/if}
        </div>
      {:else if activeTab === 'cpu'}
        <div class="detail-view">
          <h3>CPU Trend (last {hourlyMetrics.length} hours)</h3>
          <div class="chart-container">
            {#if hourlyMetrics.length > 0}
              <div class="simple-chart">
                {#each hourlyMetrics.slice().reverse() as metric, i}
                  <div
                    class="chart-bar"
                    style="height: {Math.min(metric.peak_cpu, 100)}%; background: {metric.peak_cpu > 90 ? '#ef4444' : metric.peak_cpu > 70 ? '#f59e0b' : '#10b981'}"
                  >
                    <div class="bar-tooltip">
                      <div>{metric.peak_cpu.toFixed(1)}%</div>
                      <div>{formatHourBucket(metric.hour_bucket)}</div>
                    </div>
                  </div>
                {/each}
              </div>
            {:else}
              <div class="empty-state small">
                <p>No hourly metrics available.</p>
              </div>
            {/if}
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
            {@const latest = getLatestMetric()!}
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
            {@const latest = getLatestMetric()!}
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
            {@const latest = getLatestMetric()!}
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

  .metric-footnote {
    margin-top: 0.75rem;
    font-size: 0.75rem;
    color: #6b7280;
  }

  .threat-card {
    border: 1px solid #e5e7eb;
  }

  .empty-state.small {
    padding: 1.5rem;
    background: #f9fafb;
    border-radius: 0.75rem;
    color: #6b7280;
    text-align: center;
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
