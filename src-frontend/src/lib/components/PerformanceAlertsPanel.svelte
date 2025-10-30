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
      tauriInvoke<any[]>("get_recent_errors", { limit: errorsLimit }).catch(
        () => [],
      ),
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
    await tauriInvoke("set_performance_monitoring", {
      enabled: monitoringEnabled,
    });
    status.set(
      monitoringEnabled
        ? "Performance monitoring enabled."
        : "Performance monitoring disabled.",
    );
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
          <label for="errors-limit">Limit</label>
          <input id="errors-limit" type="number" min="1" max="200" bind:value={errorsLimit} />
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
  .header h3 { color: #1f2937; font-size: 18px; font-weight: 700; }
  .actions { display: flex; gap: 8px; }
  .btn { padding: 8px 12px; border-radius: 8px; border: 1px solid rgba(0,0,0,0.12); background: #eef2ff; color: #3730a3; cursor: pointer; font-weight: 600; transition: all 0.2s; }
  .btn:hover:not(:disabled) { background: #ddd6fe; border-color: #a78bfa; }
  .btn.danger { background: #fee2e2; color: #991b1b; border-color: #fecaca; font-weight: 600; }
  .btn.danger:hover:not(:disabled) { background: #fecaca; border-color: #dc2626; }
  .status { padding: 10px 12px; border-radius: 8px; margin-bottom: 10px; font-weight: 600; }
  .status.success { background: #d1fae5; color: #065f46; border: 1px solid #10b981; }
  .status.error { background: #fee2e2; color: #7f1d1d; border: 1px solid #dc2626; }

  .grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 12px; }
  .card { background: #fafafa; border: 1px solid #d1d5db; border-radius: 10px; padding: 14px; box-shadow: 0 1px 3px rgba(0,0,0,0.05); }
  .card.full { grid-column: 1 / -1; }
  .card h4 { color: #111827; font-weight: 700; font-size: 15px; }
  .row { display: flex; align-items: center; gap: 8px; }
  .row.between { justify-content: space-between; }
  .row.small { gap: 6px; align-items: center; }
  .row.small label { color: #374151; font-weight: 600; font-size: 13px; }
  .row.small input { border: 1px solid #9ca3af; padding: 4px 8px; border-radius: 6px; font-weight: 600; }
  .muted { color: #4b5563; font-size: 13px; font-weight: 500; }
  .empty { color: #6b7280; font-size: 14px; padding: 16px; text-align: center; font-weight: 500; background: #f3f4f6; border-radius: 8px; }
  .pill { padding: 3px 10px; border-radius: 999px; font-size: 13px; border: 1px solid #6366f1; background: #eef2ff; color: #4338ca; font-weight: 700; }
  .list { display: grid; gap: 8px; }
  .item { background: #fff; border: 1px solid #d1d5db; border-radius: 8px; padding: 10px; box-shadow: 0 1px 2px rgba(0,0,0,0.05); }
  .meta { display: flex; gap: 8px; font-size: 12px; color: #4b5563; margin-bottom: 6px; font-weight: 600; }
  .time { font-weight: 700; color: #374151; }
  .type { background: #e5e7eb; border: 1px solid #9ca3af; border-radius: 999px; padding: 2px 8px; color: #1f2937; font-weight: 600; }
  .payload { max-height: 200px; overflow: auto; white-space: pre; font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace; background: #f9fafb; border: 1px solid #e5e7eb; padding: 8px; border-radius: 6px; color: #1f2937; font-size: 12px; }

  .profiles { display: grid; gap: 8px; }
  .profile { background: #fff; border: 1px solid #d1d5db; border-radius: 8px; padding: 10px; box-shadow: 0 1px 2px rgba(0,0,0,0.05); }
  .title { font-weight: 700; color: #111827; font-size: 14px; }

  /* Toggle switch */
  .switch { position: relative; display: inline-block; width: 46px; height: 24px; }
  .switch input { opacity: 0; width: 0; height: 0; }
  .slider { position: absolute; cursor: pointer; top: 0; left: 0; right: 0; bottom: 0; background-color: #9ca3af; transition: .2s; border-radius: 999px; }
  .slider:before { position: absolute; content: ""; height: 18px; width: 18px; left: 3px; bottom: 3px; background-color: white; transition: .2s; border-radius: 50%; box-shadow: 0 1px 3px rgba(0,0,0,0.2); }
  input:checked + .slider { background-color: #22c55e; }
  input:checked + .slider:before { transform: translateX(22px); }
  input:disabled + .slider { opacity: 0.5; cursor: not-allowed; }

  @media (max-width: 900px) { .grid { grid-template-columns: 1fr; } }
</style>
