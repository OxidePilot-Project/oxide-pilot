<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/tauri';
  import RPAAuditPanel from './RPAAuditPanel.svelte';
  import RPARollbackPanel from './RPARollbackPanel.svelte';

  type RPATab = 'overview' | 'audit' | 'rollback' | 'permissions';

  let activeTab: RPATab = 'overview';
  let rpaInitialized = false;
  let loading = false;
  let error = '';
  let success = '';

  // Overview stats
  let auditStats: any = null;
  let rollbackCount = 0;
  let pendingConfirmations = 0;

  async function initializeRPA() {
    loading = true;
    error = '';
    success = '';

    try {
      await invoke('rpa_initialize');
      rpaInitialized = true;
      success = 'RPA system initialized successfully';
      await loadOverviewStats();
    } catch (err) {
      error = `Failed to initialize RPA: ${err}`;
    } finally {
      loading = false;
    }
  }

  async function shutdownRPA() {
    loading = true;
    error = '';
    success = '';

    try {
      await invoke('rpa_shutdown');
      rpaInitialized = false;
      success = 'RPA system shutdown successfully';
    } catch (err) {
      error = `Failed to shutdown RPA: ${err}`;
    } finally {
      loading = false;
    }
  }

  async function loadOverviewStats() {
    try {
      const [stats, count, confirmations] = await Promise.all([
        invoke('rpa_get_audit_stats'),
        invoke('rpa_get_reversible_count'),
        invoke('rpa_get_pending_confirmations')
      ]);

      auditStats = stats;
      rollbackCount = count as number;
      pendingConfirmations = (confirmations as any[]).length;
    } catch (err) {
      console.error('Failed to load overview stats:', err);
    }
  }

  function setActiveTab(tab: RPATab) {
    activeTab = tab;
    if (tab === 'overview' && rpaInitialized) {
      loadOverviewStats();
    }
  }

  onMount(() => {
    // Check if RPA is already initialized
    loadOverviewStats().then(() => {
      // If we can load stats, RPA is probably initialized
      rpaInitialized = true;
    }).catch(() => {
      // If we can't load stats, RPA is not initialized
      rpaInitialized = false;
    });
  });
</script>

<div class="rpa-dashboard">
  <!-- Header -->
  <div class="dashboard-header">
    <div class="header-info">
      <h2>🤖 RPA Control Center</h2>
      <p>Robotic Process Automation with security controls</p>
    </div>

    <div class="header-actions">
      <div class="status-indicator">
        <span class="status-dot {rpaInitialized ? 'online' : 'offline'}"></span>
        <span>{rpaInitialized ? 'RPA Active' : 'RPA Inactive'}</span>
      </div>

      {#if rpaInitialized}
        <button
          on:click={shutdownRPA}
          disabled={loading}
          class="btn btn-danger"
        >
          {loading ? 'Shutting down...' : 'Shutdown RPA'}
        </button>
      {:else}
        <button
          on:click={initializeRPA}
          disabled={loading}
          class="btn btn-primary"
        >
          {loading ? 'Initializing...' : 'Initialize RPA'}
        </button>
      {/if}
    </div>
  </div>

  <!-- Messages -->
  {#if success}
    <div class="message success">
      {success}
    </div>
  {/if}

  {#if error}
    <div class="message error">
      {error}
    </div>
  {/if}

  <!-- Tab Navigation -->
  <div class="tab-navigation">
    <button
      class="tab-btn"
      class:active={activeTab === 'overview'}
      on:click={() => setActiveTab('overview')}
    >
      📊 Overview
    </button>
    <button
      class="tab-btn"
      class:active={activeTab === 'audit'}
      on:click={() => setActiveTab('audit')}
      disabled={!rpaInitialized}
    >
      📋 Audit Log
    </button>
    <button
      class="tab-btn"
      class:active={activeTab === 'rollback'}
      on:click={() => setActiveTab('rollback')}
      disabled={!rpaInitialized}
    >
      ↩️ Rollback
    </button>
    <button
      class="tab-btn"
      class:active={activeTab === 'permissions'}
      on:click={() => setActiveTab('permissions')}
      disabled={!rpaInitialized}
    >
      🔐 Permissions
    </button>
  </div>

  <!-- Tab Content -->
  <div class="tab-content">
    {#if activeTab === 'overview'}
      <div class="overview-content">
        {#if !rpaInitialized}
          <div class="welcome-card">
            <div class="welcome-icon">🤖</div>
            <h3>Welcome to RPA Control Center</h3>
            <p>
              Initialize the RPA system to start automating tasks with built-in security controls.
              The system includes permission management, audit logging, and rollback capabilities.
            </p>
            <button
              on:click={initializeRPA}
              disabled={loading}
              class="btn btn-primary btn-large"
            >
              {loading ? 'Initializing...' : 'Get Started'}
            </button>
          </div>
        {:else}
          <!-- Stats Cards -->
          <div class="stats-grid">
            {#if auditStats}
              <div class="stat-card">
                <div class="stat-icon">📊</div>
                <div class="stat-info">
                  <h4>Total Actions</h4>
                  <p class="stat-value">{auditStats.total_actions}</p>
                </div>
              </div>

              <div class="stat-card success">
                <div class="stat-icon">✅</div>
                <div class="stat-info">
                  <h4>Successful</h4>
                  <p class="stat-value">{auditStats.successful_actions}</p>
                </div>
              </div>

              <div class="stat-card danger">
                <div class="stat-icon">❌</div>
                <div class="stat-info">
                  <h4>Failed</h4>
                  <p class="stat-value">{auditStats.failed_actions}</p>
                </div>
              </div>

              <div class="stat-card warning">
                <div class="stat-icon">🚫</div>
                <div class="stat-info">
                  <h4>Denied</h4>
                  <p class="stat-value">{auditStats.denied_actions}</p>
                </div>
              </div>
            {/if}

            <div class="stat-card info">
              <div class="stat-icon">↩️</div>
              <div class="stat-info">
                <h4>Reversible Actions</h4>
                <p class="stat-value">{rollbackCount}</p>
              </div>
            </div>

            <div class="stat-card {pendingConfirmations > 0 ? 'warning' : 'info'}">
              <div class="stat-icon">⏳</div>
              <div class="stat-info">
                <h4>Pending Confirmations</h4>
                <p class="stat-value">{pendingConfirmations}</p>
              </div>
            </div>
          </div>

          <!-- Quick Actions -->
          <div class="quick-actions">
            <h3>Quick Actions</h3>
            <div class="action-grid">
              <button
                class="action-btn"
                on:click={() => setActiveTab('audit')}
              >
                <div class="action-icon">📋</div>
                <div class="action-info">
                  <h4>View Audit Log</h4>
                  <p>Review all RPA actions and their status</p>
                </div>
              </button>

              <button
                class="action-btn"
                on:click={() => setActiveTab('rollback')}
                disabled={rollbackCount === 0}
              >
                <div class="action-icon">↩️</div>
                <div class="action-info">
                  <h4>Rollback Actions</h4>
                  <p>Undo recent RPA actions if needed</p>
                </div>
              </button>

              <button
                class="action-btn"
                on:click={() => setActiveTab('permissions')}
              >
                <div class="action-icon">🔐</div>
                <div class="action-info">
                  <h4>Manage Permissions</h4>
                  <p>Configure RPA security policies</p>
                </div>
              </button>
            </div>
          </div>
        {/if}
      </div>
    {:else if activeTab === 'audit'}
      <RPAAuditPanel />
    {:else if activeTab === 'rollback'}
      <RPARollbackPanel />
    {:else if activeTab === 'permissions'}
      <div class="permissions-content">
        <div class="coming-soon">
          <div class="coming-soon-icon">🔐</div>
          <h3>Permission Management</h3>
          <p>
            Advanced permission management interface is coming soon.
            For now, permissions are managed through the RPA confirmation dialogs.
          </p>
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  .rpa-dashboard {
    padding: 20px;
    max-width: 1200px;
    margin: 0 auto;
  }

  .dashboard-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 24px;
    padding-bottom: 16px;
    border-bottom: 1px solid rgba(0, 0, 0, 0.1);
  }

  .header-info h2 {
    margin: 0 0 4px 0;
    color: var(--color-text);
    font-size: 24px;
  }

  .header-info p {
    margin: 0;
    color: var(--color-muted);
    font-size: 14px;
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 16px;
  }

  .status-indicator {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 14px;
    color: var(--color-text);
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

  .btn {
    padding: 8px 16px;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-weight: 500;
    transition: all 0.2s;
    font-size: 14px;
  }

  .btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .btn-primary {
    background: #3b82f6;
    color: white;
  }

  .btn-primary:hover:not(:disabled) {
    background: #2563eb;
  }

  .btn-danger {
    background: #ef4444;
    color: white;
  }

  .btn-danger:hover:not(:disabled) {
    background: #dc2626;
  }

  .btn-large {
    padding: 12px 24px;
    font-size: 16px;
  }

  .message {
    padding: 12px 16px;
    border-radius: 6px;
    margin-bottom: 16px;
    font-size: 14px;
  }

  .message.success {
    background: #dcfce7;
    color: #166534;
    border: 1px solid #bbf7d0;
  }

  .message.error {
    background: #fef2f2;
    color: #991b1b;
    border: 1px solid #fecaca;
  }

  .tab-navigation {
    display: flex;
    gap: 4px;
    margin-bottom: 24px;
    border-bottom: 1px solid rgba(0, 0, 0, 0.1);
  }

  .tab-btn {
    padding: 12px 16px;
    border: none;
    background: transparent;
    cursor: pointer;
    font-weight: 500;
    color: var(--color-muted);
    border-bottom: 2px solid transparent;
    transition: all 0.2s;
  }

  .tab-btn:hover:not(:disabled) {
    color: var(--color-text);
    background: rgba(0, 0, 0, 0.05);
  }

  .tab-btn.active {
    color: #3b82f6;
    border-bottom-color: #3b82f6;
  }

  .tab-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .tab-content {
    min-height: 400px;
  }

  .welcome-card {
    text-align: center;
    padding: 48px 24px;
    background: var(--color-surface);
    border-radius: 12px;
    box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
  }

  .welcome-icon {
    font-size: 48px;
    margin-bottom: 16px;
  }

  .welcome-card h3 {
    margin: 0 0 12px 0;
    color: var(--color-text);
    font-size: 24px;
  }

  .welcome-card p {
    margin: 0 0 24px 0;
    color: var(--color-muted);
    font-size: 16px;
    line-height: 1.5;
    max-width: 500px;
    margin-left: auto;
    margin-right: auto;
  }

  .stats-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 16px;
    margin-bottom: 32px;
  }

  .stat-card {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 16px;
    background: var(--color-surface);
    border-radius: 8px;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
    border-left: 4px solid #e5e7eb;
  }

  .stat-card.success {
    border-left-color: #10b981;
  }

  .stat-card.danger {
    border-left-color: #ef4444;
  }

  .stat-card.warning {
    border-left-color: #f59e0b;
  }

  .stat-card.info {
    border-left-color: #3b82f6;
  }

  .stat-icon {
    font-size: 24px;
  }

  .stat-info h4 {
    margin: 0 0 4px 0;
    color: var(--color-text);
    font-size: 14px;
    font-weight: 500;
  }

  .stat-value {
    margin: 0;
    color: var(--color-text);
    font-size: 24px;
    font-weight: 700;
  }

  .quick-actions h3 {
    margin: 0 0 16px 0;
    color: var(--color-text);
    font-size: 20px;
  }

  .action-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
    gap: 16px;
  }

  .action-btn {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 16px;
    background: var(--color-surface);
    border: 1px solid rgba(0, 0, 0, 0.1);
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.2s;
    text-align: left;
  }

  .action-btn:hover:not(:disabled) {
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.15);
    transform: translateY(-1px);
  }

  .action-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .action-icon {
    font-size: 24px;
  }

  .action-info h4 {
    margin: 0 0 4px 0;
    color: var(--color-text);
    font-size: 16px;
    font-weight: 600;
  }

  .action-info p {
    margin: 0;
    color: var(--color-muted);
    font-size: 14px;
  }

  .coming-soon {
    text-align: center;
    padding: 48px 24px;
    background: var(--color-surface);
    border-radius: 12px;
    box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
  }

  .coming-soon-icon {
    font-size: 48px;
    margin-bottom: 16px;
  }

  .coming-soon h3 {
    margin: 0 0 12px 0;
    color: var(--color-text);
    font-size: 24px;
  }

  .coming-soon p {
    margin: 0;
    color: var(--color-muted);
    font-size: 16px;
    line-height: 1.5;
    max-width: 500px;
    margin-left: auto;
    margin-right: auto;
  }

  @keyframes pulse {
    0% { opacity: 1; }
    50% { opacity: 0.5; }
    100% { opacity: 1; }
  }

  @media (max-width: 768px) {
    .rpa-dashboard {
      padding: 16px;
    }

    .dashboard-header {
      flex-direction: column;
      align-items: flex-start;
      gap: 16px;
    }

    .header-actions {
      width: 100%;
      justify-content: space-between;
    }

    .tab-navigation {
      overflow-x: auto;
      -webkit-overflow-scrolling: touch;
    }

    .tab-btn {
      white-space: nowrap;
    }

    .stats-grid {
      grid-template-columns: 1fr;
    }

    .action-grid {
      grid-template-columns: 1fr;
    }
  }
</style>