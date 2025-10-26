<script lang="ts">
  import { invoke } from '@tauri-apps/api/tauri';
  import { onMount } from 'svelte';

  interface RollbackAction {
    timestamp: string;
    action_type: string;
    details: string;
    reversible: boolean;
  }

  let rollbackHistory: RollbackAction[] = [];
  let reversibleCount = 0;
  let loading = false;
  let error = '';
  let success = '';

  async function loadRollbackData() {
    loading = true;
    error = '';

    try {
      const history = await invoke<RollbackAction[]>('rpa_get_rollback_history');
      rollbackHistory = history;

      const count = await invoke<number>('rpa_get_reversible_count');
      reversibleCount = count;
    } catch (err) {
      error = `Failed to load rollback data: ${err}`;
    } finally {
      loading = false;
    }
  }

  async function rollbackLast() {
    if (reversibleCount === 0) {
      error = 'No reversible actions available';
      return;
    }

    loading = true;
    error = '';
    success = '';

    try {
      await invoke('rpa_rollback_last');
      success = 'Successfully rolled back last action';

      // Reload data
      await loadRollbackData();
    } catch (err) {
      error = `Failed to rollback: ${err}`;
    } finally {
      loading = false;
    }
  }

  function formatTimestamp(timestamp: string): string {
    const date = new Date(timestamp);
    return date.toLocaleString();
  }

  onMount(() => {
    loadRollbackData();
  });
</script>

<div class="bg-white rounded-lg shadow p-6">
  <!-- Header -->
  <div class="flex items-center justify-between mb-6">
    <div>
      <h2 class="text-xl font-semibold text-gray-900">Rollback History</h2>
      <p class="text-sm text-gray-600 mt-1">
        {reversibleCount} reversible action(s) available
      </p>
    </div>
    <div class="flex gap-2">
      <button
        on:click={loadRollbackData}
        disabled={loading}
        class="px-4 py-2 bg-gray-200 text-gray-800 rounded hover:bg-gray-300 disabled:opacity-50 transition-colors"
      >
        Refresh
      </button>
      <button
        on:click={rollbackLast}
        disabled={loading || reversibleCount === 0}
        class="px-4 py-2 bg-red-600 text-white rounded hover:bg-red-700 disabled:opacity-50 transition-colors"
      >
        {loading ? 'Rolling back...' : 'Rollback Last'}
      </button>
    </div>
  </div>

  <!-- Success message -->
  {#if success}
    <div class="mb-4 p-3 bg-green-50 border border-green-200 rounded text-sm text-green-700">
      {success}
    </div>
  {/if}

  <!-- Error message -->
  {#if error}
    <div class="mb-4 p-3 bg-red-50 border border-red-200 rounded text-sm text-red-700">
      {error}
    </div>
  {/if}

  <!-- Info Box -->
  <div class="mb-6 p-4 bg-blue-50 border border-blue-200 rounded">
    <h3 class="text-sm font-medium text-blue-900 mb-2">About Rollback</h3>
    <p class="text-sm text-blue-700">
      Rollback allows you to undo recent RPA actions. Only certain actions like mouse movements,
      file writes, and file deletions can be reversed. Actions are rolled back in reverse order (LIFO).
    </p>
  </div>

  <!-- Rollback History Table -->
  <div class="overflow-x-auto">
    <table class="min-w-full divide-y divide-gray-200">
      <thead class="bg-gray-50">
        <tr>
          <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
            Timestamp
          </th>
          <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
            Action Type
          </th>
          <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
            Details
          </th>
          <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
            Reversible
          </th>
        </tr>
      </thead>
      <tbody class="bg-white divide-y divide-gray-200">
        {#if rollbackHistory.length === 0}
          <tr>
            <td colspan="4" class="px-4 py-8 text-center text-gray-500">
              No rollback history available
            </td>
          </tr>
        {:else}
          {#each rollbackHistory as action, index}
            <tr class="hover:bg-gray-50" class:bg-green-50={index === 0 && action.reversible}>
              <td class="px-4 py-3 text-sm text-gray-900 whitespace-nowrap">
                {formatTimestamp(action.timestamp)}
              </td>
              <td class="px-4 py-3 text-sm text-gray-900">
                <code class="bg-gray-100 px-2 py-1 rounded text-xs">{action.action_type}</code>
              </td>
              <td class="px-4 py-3 text-sm text-gray-600">
                {action.details}
              </td>
              <td class="px-4 py-3 text-sm">
                {#if action.reversible}
                  <span class="px-2 py-1 bg-green-100 text-green-800 rounded text-xs font-medium">
                    Yes
                  </span>
                {:else}
                  <span class="px-2 py-1 bg-gray-100 text-gray-800 rounded text-xs font-medium">
                    No
                  </span>
                {/if}
              </td>
            </tr>
          {/each}
        {/if}
      </tbody>
    </table>
  </div>

  <!-- Legend -->
  {#if rollbackHistory.length > 0}
    <div class="mt-4 flex items-center gap-4 text-sm text-gray-600">
      <div class="flex items-center gap-2">
        <div class="w-4 h-4 bg-green-50 border border-green-200 rounded"></div>
        <span>Next action to rollback</span>
      </div>
    </div>
  {/if}
</div>
