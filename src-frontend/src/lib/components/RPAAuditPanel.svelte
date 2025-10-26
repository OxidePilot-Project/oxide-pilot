<script lang="ts">
  import { invoke } from '@tauri-apps/api/tauri';
  import { onMount } from 'svelte';

  interface AuditEntry {
    timestamp: string;
    permission: string;
    action: string;
    status: string;
    error: string | null;
  }

  interface AuditStats {
    total_actions: number;
    successful_actions: number;
    failed_actions: number;
    denied_actions: number;
  }

  let auditEntries: AuditEntry[] = [];
  let auditStats: AuditStats | null = null;
  let loading = false;
  let error = '';
  let filterPermission = '';
  let filterStatus = '';

  // Status colors
  const statusColors: Record<string, string> = {
    Success: 'bg-green-100 text-green-800',
    Failed: 'bg-red-100 text-red-800',
    Denied: 'bg-orange-100 text-orange-800',
    Pending: 'bg-yellow-100 text-yellow-800'
  };

  async function loadAuditData() {
    loading = true;
    error = '';

    try {
      // Load audit entries
      const entries = await invoke<AuditEntry[]>('rpa_get_audit_entries', {
        permission: filterPermission || null,
        status: filterStatus || null,
        limit: 100
      });
      auditEntries = entries;

      // Load audit stats
      const stats = await invoke<AuditStats>('rpa_get_audit_stats');
      auditStats = stats;
    } catch (err) {
      error = `Failed to load audit data: ${err}`;
    } finally {
      loading = false;
    }
  }

  function formatTimestamp(timestamp: string): string {
    const date = new Date(timestamp);
    return date.toLocaleString();
  }

  function getSuccessRate(): number {
    if (!auditStats || auditStats.total_actions === 0) return 0;
    return (auditStats.successful_actions / auditStats.total_actions) * 100;
  }

  onMount(() => {
    loadAuditData();
  });
</script>

<div class="bg-white rounded-lg shadow p-6">
  <!-- Header -->
  <div class="flex items-center justify-between mb-6">
    <h2 class="text-xl font-semibold text-gray-900">RPA Audit Log</h2>
    <button
      on:click={loadAuditData}
      disabled={loading}
      class="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50 transition-colors"
    >
      {loading ? 'Loading...' : 'Refresh'}
    </button>
  </div>

  <!-- Stats Cards -->
  {#if auditStats}
    <div class="grid grid-cols-4 gap-4 mb-6">
      <div class="bg-gray-50 rounded p-4">
        <p class="text-sm text-gray-600 mb-1">Total Actions</p>
        <p class="text-2xl font-bold text-gray-900">{auditStats.total_actions}</p>
      </div>
      <div class="bg-green-50 rounded p-4">
        <p class="text-sm text-gray-600 mb-1">Successful</p>
        <p class="text-2xl font-bold text-green-700">{auditStats.successful_actions}</p>
      </div>
      <div class="bg-red-50 rounded p-4">
        <p class="text-sm text-gray-600 mb-1">Failed</p>
        <p class="text-2xl font-bold text-red-700">{auditStats.failed_actions}</p>
      </div>
      <div class="bg-orange-50 rounded p-4">
        <p class="text-sm text-gray-600 mb-1">Denied</p>
        <p class="text-2xl font-bold text-orange-700">{auditStats.denied_actions}</p>
      </div>
    </div>

    <!-- Success Rate -->
    <div class="mb-6">
      <div class="flex items-center justify-between mb-2">
        <span class="text-sm font-medium text-gray-700">Success Rate</span>
        <span class="text-sm font-bold text-gray-900">{getSuccessRate().toFixed(1)}%</span>
      </div>
      <div class="w-full bg-gray-200 rounded-full h-2">
        <div
          class="bg-green-600 h-2 rounded-full transition-all"
          style="width: {getSuccessRate()}%"
        ></div>
      </div>
    </div>
  {/if}

  <!-- Filters -->
  <div class="flex gap-4 mb-4">
    <div class="flex-1">
      <label class="block text-sm font-medium text-gray-700 mb-1">
        Filter by Permission
      </label>
      <select
        bind:value={filterPermission}
        on:change={loadAuditData}
        class="w-full px-3 py-2 border border-gray-300 rounded focus:ring-2 focus:ring-blue-500 focus:border-transparent"
      >
        <option value="">All Permissions</option>
        <option value="MouseMove">Mouse Move</option>
        <option value="MouseClick">Mouse Click</option>
        <option value="KeyPress">Key Press</option>
        <option value="TypeText">Type Text</option>
        <option value="ScreenCapture">Screen Capture</option>
        <option value="FileWrite">File Write</option>
        <option value="FileDelete">File Delete</option>
      </select>
    </div>

    <div class="flex-1">
      <label class="block text-sm font-medium text-gray-700 mb-1">
        Filter by Status
      </label>
      <select
        bind:value={filterStatus}
        on:change={loadAuditData}
        class="w-full px-3 py-2 border border-gray-300 rounded focus:ring-2 focus:ring-blue-500 focus:border-transparent"
      >
        <option value="">All Statuses</option>
        <option value="Success">Success</option>
        <option value="Failed">Failed</option>
        <option value="Denied">Denied</option>
      </select>
    </div>
  </div>

  <!-- Error message -->
  {#if error}
    <div class="mb-4 p-3 bg-red-50 border border-red-200 rounded text-sm text-red-700">
      {error}
    </div>
  {/if}

  <!-- Audit Entries Table -->
  <div class="overflow-x-auto">
    <table class="min-w-full divide-y divide-gray-200">
      <thead class="bg-gray-50">
        <tr>
          <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
            Timestamp
          </th>
          <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
            Permission
          </th>
          <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
            Action
          </th>
          <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
            Status
          </th>
          <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
            Error
          </th>
        </tr>
      </thead>
      <tbody class="bg-white divide-y divide-gray-200">
        {#if auditEntries.length === 0}
          <tr>
            <td colspan="5" class="px-4 py-8 text-center text-gray-500">
              No audit entries found
            </td>
          </tr>
        {:else}
          {#each auditEntries as entry}
            <tr class="hover:bg-gray-50">
              <td class="px-4 py-3 text-sm text-gray-900 whitespace-nowrap">
                {formatTimestamp(entry.timestamp)}
              </td>
              <td class="px-4 py-3 text-sm text-gray-900">
                <code class="bg-gray-100 px-2 py-1 rounded text-xs">{entry.permission}</code>
              </td>
              <td class="px-4 py-3 text-sm text-gray-600">
                {entry.action}
              </td>
              <td class="px-4 py-3 text-sm">
                <span class={`px-2 py-1 rounded text-xs font-medium ${statusColors[entry.status] || 'bg-gray-100 text-gray-800'}`}>
                  {entry.status}
                </span>
              </td>
              <td class="px-4 py-3 text-sm text-red-600">
                {entry.error || '-'}
              </td>
            </tr>
          {/each}
        {/if}
      </tbody>
    </table>
  </div>
</div>
