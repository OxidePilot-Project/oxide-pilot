<script lang="ts">
  import { invoke } from '@tauri-apps/api/tauri';
  import { onMount, onDestroy } from 'svelte';

  interface ConfirmationRequest {
    id: string;
    permission: string;
    risk_level: string;
    timeout_secs: number;
    created_at: string;
  }

  let pendingConfirmations: ConfirmationRequest[] = [];
  let currentConfirmation: ConfirmationRequest | null = null;
  let loading = false;
  let error = '';
  let pollInterval: number;

  // Risk level colors
  const riskColors: Record<string, string> = {
    Low: 'bg-green-100 text-green-800 border-green-300',
    Medium: 'bg-yellow-100 text-yellow-800 border-yellow-300',
    High: 'bg-orange-100 text-orange-800 border-orange-300',
    Critical: 'bg-red-100 text-red-800 border-red-300'
  };

  // Permission descriptions
  const permissionDescriptions: Record<string, string> = {
    MouseMove: 'Move the mouse cursor',
    MouseClick: 'Click the mouse button',
    MouseScroll: 'Scroll the mouse wheel',
    KeyPress: 'Press keyboard keys',
    TypeText: 'Type text',
    ScreenCapture: 'Capture screenshots',
    FileRead: 'Read files',
    FileWrite: 'Write files',
    FileDelete: 'Delete files',
    ProcessStart: 'Start processes',
    ProcessKill: 'Kill processes',
    NetworkRequest: 'Make network requests',
    ClipboardRead: 'Read clipboard',
    ClipboardWrite: 'Write to clipboard',
    SystemCommand: 'Execute system commands',
    RegistryAccess: 'Access system registry'
  };

  async function fetchPendingConfirmations() {
    try {
      const confirmations = await invoke<ConfirmationRequest[]>('rpa_get_pending_confirmations');
      pendingConfirmations = confirmations;

      // Show the first pending confirmation if available
      if (confirmations.length > 0 && !currentConfirmation) {
        currentConfirmation = confirmations[0];
      }
    } catch (err) {
      console.error('Failed to fetch pending confirmations:', err);
    }
  }

  async function handleResponse(approved: boolean) {
    if (!currentConfirmation) return;

    loading = true;
    error = '';

    try {
      await invoke('rpa_respond_confirmation', {
        requestId: currentConfirmation.id,
        approved
      });

      // Remove from pending list
      pendingConfirmations = pendingConfirmations.filter(c => c.id !== currentConfirmation!.id);

      // Show next confirmation if available
      currentConfirmation = pendingConfirmations.length > 0 ? pendingConfirmations[0] : null;
    } catch (err) {
      error = `Failed to respond: ${err}`;
    } finally {
      loading = false;
    }
  }

  function formatTimeRemaining(createdAt: string, timeoutSecs: number): string {
    const created = new Date(createdAt).getTime();
    const now = Date.now();
    const elapsed = (now - created) / 1000;
    const remaining = Math.max(0, timeoutSecs - elapsed);

    if (remaining < 60) {
      return `${Math.floor(remaining)}s`;
    } else {
      return `${Math.floor(remaining / 60)}m ${Math.floor(remaining % 60)}s`;
    }
  }

  onMount(() => {
    // Poll for pending confirmations every 2 seconds
    fetchPendingConfirmations();
    pollInterval = setInterval(fetchPendingConfirmations, 2000) as unknown as number;
  });

  onDestroy(() => {
    if (pollInterval) {
      clearInterval(pollInterval);
    }
  });
</script>

{#if currentConfirmation}
  <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
    <div class="bg-white rounded-lg shadow-xl max-w-md w-full p-6">
      <!-- Header -->
      <div class="flex items-center justify-between mb-4">
        <h3 class="text-lg font-semibold text-gray-900">
          RPA Permission Request
        </h3>
        <span class={`px-2 py-1 rounded text-xs font-medium border ${riskColors[currentConfirmation.risk_level] || riskColors.Medium}`}>
          {currentConfirmation.risk_level} Risk
        </span>
      </div>

      <!-- Content -->
      <div class="mb-6">
        <p class="text-sm text-gray-600 mb-2">
          The system is requesting permission to:
        </p>
        <p class="text-base font-medium text-gray-900 mb-4">
          {permissionDescriptions[currentConfirmation.permission] || currentConfirmation.permission}
        </p>

        <div class="bg-gray-50 rounded p-3 text-sm">
          <div class="flex justify-between mb-1">
            <span class="text-gray-600">Permission:</span>
            <span class="font-mono text-gray-900">{currentConfirmation.permission}</span>
          </div>
          <div class="flex justify-between">
            <span class="text-gray-600">Time remaining:</span>
            <span class="font-mono text-gray-900">
              {formatTimeRemaining(currentConfirmation.created_at, currentConfirmation.timeout_secs)}
            </span>
          </div>
        </div>

        {#if pendingConfirmations.length > 1}
          <p class="text-xs text-gray-500 mt-2">
            {pendingConfirmations.length - 1} more request(s) pending
          </p>
        {/if}
      </div>

      <!-- Error message -->
      {#if error}
        <div class="mb-4 p-3 bg-red-50 border border-red-200 rounded text-sm text-red-700">
          {error}
        </div>
      {/if}

      <!-- Actions -->
      <div class="flex gap-3">
        <button
          on:click={() => handleResponse(false)}
          disabled={loading}
          class="flex-1 px-4 py-2 bg-gray-200 text-gray-800 rounded hover:bg-gray-300 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
        >
          Deny
        </button>
        <button
          on:click={() => handleResponse(true)}
          disabled={loading}
          class="flex-1 px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
        >
          {loading ? 'Processing...' : 'Approve'}
        </button>
      </div>
    </div>
  </div>
{/if}
