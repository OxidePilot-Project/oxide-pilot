<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/tauri';

  interface Alert {
    text: string;
    score: number;
    timestamp: string;
    metadata?: {
      alert_type?: string;
      auto_generated?: boolean;
    };
  }

  let alerts: Alert[] = [];
  let loading = true;
  let error: string | null = null;
  let refreshInterval: number;
  let filterType: 'all' | 'performance' | 'security' = 'all';

  async function fetchAlerts() {
    try {
      const response = await invoke<{ results: Alert[]; count: number }>('search_agent_memory', {
        query: 'alert high usage warning',
        limit: 50
      });

      alerts = response.results.sort((a, b) =>
        new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime()
      );

      loading = false;
      error = null;
    } catch (e) {
      error = `Failed to fetch alerts: ${e}`;
      loading = false;
      console.error(error);
    }
  }

  onMount(() => {
    fetchAlerts();
    refreshInterval = window.setInterval(fetchAlerts, 10000); // Refresh every 10 seconds
  });

  onDestroy(() => {
    if (refreshInterval) {
      clearInterval(refreshInterval);
    }
  });

  function getAlertSeverity(text: string): 'critical' | 'warning' | 'info' {
    const lowerText = text.toLowerCase();
    if (lowerText.includes('critical') || lowerText.includes('95') || lowerText.includes('100')) {
      return 'critical';
    }
    if (lowerText.includes('warning') || lowerText.includes('high') || lowerText.includes('90')) {
      return 'warning';
    }
    return 'info';
  }

  function getSeverityColor(severity: string): string {
    switch (severity) {
      case 'critical': return 'text-red-600';
      case 'warning': return 'text-yellow-600';
      case 'info': return 'text-blue-600';
      default: return 'text-gray-600';
    }
  }

  function getSeverityBg(severity: string): string {
    switch (severity) {
      case 'critical': return 'bg-red-50 border-red-200';
      case 'warning': return 'bg-yellow-50 border-yellow-200';
      case 'info': return 'bg-blue-50 border-blue-200';
      default: return 'bg-gray-50 border-gray-200';
    }
  }

  function getSeverityIcon(severity: string): string {
    switch (severity) {
      case 'critical': return '🔴';
      case 'warning': return '⚠️';
      case 'info': return 'ℹ️';
      default: return '📋';
    }
  }

  function formatTimestamp(timestamp: string): string {
    const date = new Date(timestamp);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / 60000);

    if (diffMins < 1) return 'Just now';
    if (diffMins < 60) return `${diffMins}m ago`;

    const diffHours = Math.floor(diffMins / 60);
    if (diffHours < 24) return `${diffHours}h ago`;

    const diffDays = Math.floor(diffHours / 24);
    return `${diffDays}d ago`;
  }

  $: filteredAlerts = alerts.filter(alert => {
    if (filterType === 'all') return true;
    return alert.metadata?.alert_type === filterType;
  });

  $: criticalCount = alerts.filter(a => getAlertSeverity(a.text) === 'critical').length;
  $: warningCount = alerts.filter(a => getAlertSeverity(a.text) === 'warning').length;
</script>

<div class="alerts-panel">
  <div class="panel-header">
    <h3>🚨 System Alerts</h3>
    <button class="refresh-btn" onclick={fetchAlerts} disabled={loading}>
      <span class="refresh-icon {loading ? 'spinning' : ''}">↻</span>
      Refresh
    </button>
  </div>

  <div class="alert-summary">
    <div class="summary-item critical">
      <span class="count">{criticalCount}</span>
      <span class="label">Critical</span>
    </div>
    <div class="summary-item warning">
      <span class="count">{warningCount}</span>
      <span class="label">Warnings</span>
    </div>
    <div class="summary-item info">
      <span class="count">{alerts.length - criticalCount - warningCount}</span>
      <span class="label">Info</span>
    </div>
  </div>

  <div class="filter-tabs">
    <button
      class="filter-tab {filterType === 'all' ? 'active' : ''}"
      onclick={() => filterType = 'all'}
    >
      All ({alerts.length})
    </button>
    <button
      class="filter-tab {filterType === 'performance' ? 'active' : ''}"
      onclick={() => filterType = 'performance'}
    >
      Performance
    </button>
    <button
      class="filter-tab {filterType === 'security' ? 'active' : ''}"
      onclick={() => filterType = 'security'}
    >
      Security
    </button>
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
      <p>Loading alerts...</p>
    </div>
  {:else if filteredAlerts.length === 0}
    <div class="empty-state">
      <span class="empty-icon">✅</span>
      <h4>No Alerts</h4>
      <p>Your system is running smoothly with no active alerts.</p>
    </div>
  {:else}
    <div class="alerts-list">
      {#each filteredAlerts as alert}
        {@const severity = getAlertSeverity(alert.text)}
        <div class="alert-item {getSeverityBg(severity)}">
          <div class="alert-icon">
            {getSeverityIcon(severity)}
          </div>
          <div class="alert-content">
            <div class="alert-header">
              <span class="alert-severity {getSeverityColor(severity)}">
                {severity.toUpperCase()}
              </span>
              <span class="alert-time">{formatTimestamp(alert.timestamp)}</span>
            </div>
            <p class="alert-text">{alert.text}</p>
            {#if alert.metadata?.auto_generated}
              <span class="alert-badge">Auto-generated</span>
            {/if}
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .alerts-panel {
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

  .alert-summary {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 1rem;
    margin-bottom: 1.5rem;
  }

  .summary-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 1rem;
    border-radius: 0.5rem;
    gap: 0.5rem;
  }

  .summary-item.critical {
    background: #fee2e2;
    border: 1px solid #fecaca;
  }

  .summary-item.warning {
    background: #fef3c7;
    border: 1px solid #fde68a;
  }

  .summary-item.info {
    background: #dbeafe;
    border: 1px solid #bfdbfe;
  }

  .summary-item .count {
    font-size: 2rem;
    font-weight: 700;
  }

  .summary-item.critical .count {
    color: #dc2626;
  }

  .summary-item.warning .count {
    color: #d97706;
  }

  .summary-item.info .count {
    color: #2563eb;
  }

  .summary-item .label {
    font-size: 0.875rem;
    color: #6b7280;
    font-weight: 500;
  }

  .filter-tabs {
    display: flex;
    gap: 0.5rem;
    margin-bottom: 1.5rem;
    border-bottom: 2px solid #e5e7eb;
  }

  .filter-tab {
    padding: 0.75rem 1rem;
    background: none;
    border: none;
    border-bottom: 2px solid transparent;
    cursor: pointer;
    font-weight: 500;
    color: #6b7280;
    transition: all 0.2s;
    margin-bottom: -2px;
  }

  .filter-tab:hover {
    color: #3b82f6;
  }

  .filter-tab.active {
    color: #3b82f6;
    border-bottom-color: #3b82f6;
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

  .alerts-list {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .alert-item {
    display: flex;
    gap: 1rem;
    padding: 1rem;
    border: 1px solid;
    border-radius: 0.5rem;
    transition: transform 0.2s;
  }

  .alert-item:hover {
    transform: translateX(4px);
  }

  .alert-icon {
    font-size: 1.5rem;
    flex-shrink: 0;
  }

  .alert-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .alert-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .alert-severity {
    font-size: 0.75rem;
    font-weight: 700;
    letter-spacing: 0.05em;
  }

  .alert-time {
    font-size: 0.875rem;
    color: #6b7280;
  }

  .alert-text {
    color: #374151;
    line-height: 1.5;
  }

  .alert-badge {
    display: inline-block;
    padding: 0.25rem 0.5rem;
    background: #e5e7eb;
    color: #6b7280;
    border-radius: 0.25rem;
    font-size: 0.75rem;
    font-weight: 500;
    width: fit-content;
  }

  @media (max-width: 768px) {
    .alerts-panel {
      padding: 1rem;
    }

    .panel-header {
      flex-direction: column;
      align-items: flex-start;
      gap: 1rem;
    }

    .alert-summary {
      grid-template-columns: 1fr;
    }

    .filter-tabs {
      overflow-x: auto;
      -webkit-overflow-scrolling: touch;
    }

    .filter-tab {
      white-space: nowrap;
    }
  }
</style>
