<script lang="ts">
  import { onMount } from "svelte";
  import { writable } from "svelte/store";
  import { isTauri } from "$lib/utils/env";
  import { tauriInvoke } from "$lib/utils/tauri";

  const loading = writable(false);
  const error = writable<string | null>(null);
  const status = writable<string | null>(null);

  // Monitoring toggle
  let monitoringEnabled = true;

  // Data stores
  const alerts = writable<any[]>([]);
  const errorStats = writable<any | null>(null);
  const recentErrors = writable<any[]>([]);
  const operationProfiles = writable<any[]>([]);

  let errorsLimit = 25;

  async function loadData() {
    if (!isTauri) return;
    loading.set(true);
    error.set(null);
    status.set(null);
    try {
      const [alertsList, stats, errors, profiles] = await Promise.all([
        tauriInvoke<any[]>("get_performance_alerts"),
        tauriInvoke<any>("get_error_statistics").catch(() => null),
        tauriInvoke<any[]>("get_recent_errors", { limit: errorsLimit }).catch(() => []),
        tauriInvoke<any[]>("get_operation_profiles").catch(() => []),
      ]);
      alerts.set(alertsList ?? []);
      errorStats.set(stats);
      recentErrors.set(errors ?? []);
      operationProfiles.set(profiles ?? []);
    } catch (e: any) {
      error.set(e?.message ?? String(e));
    } finally {
      loading.set(false);
    }
  }

  async function toggleMonitoring() {
    if (!isTauri) return;
    loading.set(true);
    error.set(null);
    status.set(null);
    try {
      await tauriInvoke("set_performance_monitoring", { enabled: monitoringEnabled });
      status.set(monitoringEnabled ? "Performance monitoring enabled." : "Performance monitoring disabled.");
    } catch (e: any) {
      error.set(e?.message ?? String(e));
    } finally {
      loading.set(false);
    }
  }

  async function clearAlerts() {
    if (!isTauri) return;
    loading.set(true);
    error.set(null);
    status.set(null);
    try {
      await tauriInvoke("clear_performance_alerts");
      await loadData();
      status.set("Performance alerts cleared.");
    } catch (e: any) {
      error.set(e?.message ?? String(e));
    } finally {
      loading.set(false);
    }
  }

  onMount(loadData);
</script>

<div class="perf-alerts-panel">
  <div class="header">
    <h3>⚠️ Performance Alerts & Errors</h3>
    <div class="actions">
      <button class="btn" on:click={loadData} disabled={$loading}>⟳ Refresh</button>
      <button class="btn danger" on:click={clearAlerts} disabled={$loading || !isTauri}>🧹 Clear Alerts</button>
    </div>
  </div>

  {#if $status}
    <div class="status success">{$status}</div>
  {/if}
  {#if $error}
    <div class="status error">{$error}</div>
  {/if}

  <div class="grid">
    <div class="card">
      <div class="row between">
        <h4>Monitoring</h4>
        <label class="switch">
          <input type="checkbox" bind:checked={monitoringEnabled} on:change={toggleMonitoring} disabled={$loading || !isTauri} />
          <span class="slider"></span>
        </label>
      </div>
      <p class="muted">Toggle backend performance monitoring.</p>
    </div>

    <div class="card">
      <div class="row between">
        <h4>Alerts</h4>
        <span class="pill">{$alerts.length} alerts</span>
      </div>
      {#if $alerts.length === 0}
        <div class="empty">No performance alerts.</div>
      {:else}
        <div class="list">
          {#each $alerts as al}
            <div class="item">
              <div class="meta">
                <span class="time">{new Date(al.timestamp ?? Date.now()).toLocaleString()}</span>
                <span class="type">{al.level ?? al.type ?? 'alert'}</span>
              </div>
              <div class="title">{al.message ?? al.summary ?? 'Alert'}</div>
              {#if al.details}
                <pre class="payload">{JSON.stringify(al.details, null, 2)}</pre>
              {/if}
            </div>
          {/each}
        </div>
      {/if}
    </div>

    <div class="card">
      <div class="row between">
        <h4>Error Statistics</h4>
      </div>
      {#if !$errorStats}
        <div class="empty">No error statistics.</div>
      {:else}
        <pre class="payload">{JSON.stringify($errorStats, null, 2)}</pre>
      {/if}
    </div>

    <div class="card">
      <div class="row between">
        <h4>Recent Errors</h4>
        <div class="row small">
          <label>Limit</label>
          <input type="number" min="1" max="200" bind:value={errorsLimit} />
          <button class="btn" on:click={loadData} disabled={$loading}>Apply</button>
        </div>
      </div>
      {#if $recentErrors.length === 0}
        <div class="empty">No recent errors.</div>
      {:else}
        <div class="list">
          {#each $recentErrors as err}
            <div class="item">
              <div class="meta">
                <span class="time">{new Date(err.timestamp ?? Date.now()).toLocaleString()}</span>
                <span class="type">{err.code ?? 'error'}</span>
              </div>
              <div class="title">{err.message ?? 'Error'}</div>
              {#if err.stack || err.details}
                <pre class="payload">{JSON.stringify(err.stack ? { stack: err.stack } : err.details, null, 2)}</pre>
              {/if}
            </div>
          {/each}
        </div>
      {/if}
    </div>

    <div class="card full">
      <div class="row between">
        <h4>Operation Profiles</h4>
        <span class="pill">{$operationProfiles.length}</span>
      </div>
      {#if $operationProfiles.length === 0}
        <div class="empty">No profiles.</div>
      {:else}
        <div class="profiles">
          {#each $operationProfiles as p}
            <div class="profile">
              <div class="row between">
                <div class="title">{p.name ?? p.id ?? 'operation'}</div>
                {#if p.duration_ms}
                  <span class="pill">{p.duration_ms} ms</span>
                {/if}
              </div>
              <pre class="payload">{JSON.stringify(p, null, 2)}</pre>
            </div>
          {/each}
        </div>
      {/if}
    </div>
  </div>
</div>

<style>
  .perf-alerts-panel { background: #fff; border-radius: 12px; padding: 16px; box-shadow: 0 6px 20px rgba(0,0,0,0.08); }
  .header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px; }
  .actions { display: flex; gap: 8px; }
  .btn { padding: 8px 12px; border-radius: 8px; border: 1px solid rgba(0,0,0,0.08); background: #eef2ff; color: #3730a3; cursor: pointer; }
  .btn.danger { background: #fee2e2; color: #991b1b; border-color: #fecaca; }
  .status { padding: 10px 12px; border-radius: 8px; margin-bottom: 10px; font-weight: 500; }
  .status.success { background: #e7f9ed; color: #126d3b; border: 1px solid #bfe8cc; }
  .status.error { background: #fde8e8; color: #9b1c1c; border: 1px solid #fbd5d5; }

  .grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 12px; }
  .card { background: #f8f9fa; border: 1px solid #edf2f7; border-radius: 10px; padding: 12px; }
  .card.full { grid-column: 1 / -1; }
  .row { display: flex; align-items: center; gap: 8px; }
  .row.between { justify-content: space-between; }
  .row.small { gap: 6px; align-items: center; }
  .muted { color: #6b7280; font-size: 12px; }
  .pill { padding: 2px 8px; border-radius: 999px; font-size: 12px; border: 1px solid #c7d2fe; background: #eef2ff; color: #3730a3; }
  .list { display: grid; gap: 8px; }
  .item { background: #fff; border: 1px solid #e5e7eb; border-radius: 8px; padding: 8px; }
  .meta { display: flex; gap: 8px; font-size: 12px; color: #6b7280; margin-bottom: 6px; }
  .time { font-weight: 500; }
  .type { background: #f3f4f6; border: 1px solid #e5e7eb; border-radius: 999px; padding: 2px 6px; }
  .payload { max-height: 200px; overflow: auto; white-space: pre; font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace; }

  .profiles { display: grid; gap: 8px; }
  .profile { background: #fff; border: 1px solid #e5e7eb; border-radius: 8px; padding: 8px; }
  .title { font-weight: 600; color: #111827; }

  /* Toggle switch */
  .switch { position: relative; display: inline-block; width: 46px; height: 24px; }
  .switch input { opacity: 0; width: 0; height: 0; }
  .slider { position: absolute; cursor: pointer; top: 0; left: 0; right: 0; bottom: 0; background-color: #d1d5db; transition: .2s; border-radius: 999px; }
  .slider:before { position: absolute; content: ""; height: 18px; width: 18px; left: 3px; bottom: 3px; background-color: white; transition: .2s; border-radius: 50%; }
  input:checked + .slider { background-color: #4ade80; }
  input:checked + .slider:before { transform: translateX(22px); }

  @media (max-width: 900px) { .grid { grid-template-columns: 1fr; } }
</style>
