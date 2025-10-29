<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { writable } from "svelte/store";
  import { isTauri } from "$lib/utils/env";
  import { tauriInvoke } from "$lib/utils/tauri";

  const loading = writable(false);
  const status = writable<string | null>(null);
  const error = writable<string | null>(null);

  // Create session form state
  let userId = "";
  let permissionsCsv = "read,write";
  let ipAddress = "";
  let userAgent = "";
  const createdSessionId = writable<string | null>(null);

  // Validation / permission check
  let checkSessionId = "";
  let permissionToCheck = "admin";
  const validateResult = writable<null | boolean>(null);
  const permissionResult = writable<null | boolean>(null);

  // Security events
  const events = writable<any[]>([]);
  let eventsLimit = 25;
  let refreshTimer: number | null = null;

  // File scan state
  let filePath: string = "";
  let useCloud = true;
  let quarantine = true;
  let scanResult: any = null;
  let vtConfigured: boolean | null = null;
  onMount(async () => {
    if (isTauri) {
      try {
        vtConfigured = await tauriInvoke("is_virustotal_configured");
      } catch (e) {
        vtConfigured = null;
      }
    }
  });
  async function pickFile() {
    if (!isTauri) {
      error.set("Desktop runtime required for file picker.");
      return;
    }
    try {
      const { open } = await import("@tauri-apps/api/dialog");
      const selected = await open({ multiple: false });
      if (typeof selected === "string") {
        filePath = selected;
      }
    } catch (e: any) {
      error.set(e?.message ?? String(e));
    }
  }

  // Folder scan state and controls
  let folderPath: string = "";
  let folderScanId: string | null = null;
  let folderProgress: any = null; // { discovered, scanned, total, malicious, errors, current_file, local_match, external_verdict, cancelled, completed, duration_ms }
  const folderUnsubs: Array<() => void> = [];

  // Autonomous Threat Consensus state
  let threatReport: any = null;
  let threatRecs: string[] = [];
  let consensusLoading = false;
  let consensusError: string | null = null;

  async function runThreatConsensus() {
    if (!isTauri) {
      error.set("Desktop runtime required.");
      return;
    }
    consensusLoading = true;
    consensusError = null;
    threatReport = null;
    threatRecs = [];
    try {
      const jsonStr = await tauriInvoke<string>("run_threat_consensus");
      try {
        threatReport = JSON.parse(jsonStr || "null");
      } catch (e) {
        consensusError = "Failed to parse threat report JSON.";
      }
      // fetch recommendations
      try {
        threatRecs = await tauriInvoke<string[]>("get_threat_recommendations");
      } catch (e) {
        // non-fatal
      }
      status.set("Threat consensus completed.");
    } catch (e: any) {
      consensusError = e?.message ?? String(e);
    } finally {
      consensusLoading = false;
    }
  }

  async function pickFolder() {
    if (!isTauri) {
      error.set("Desktop runtime required for folder picker.");
      return;
    }
    try {
      const { open } = await import("@tauri-apps/api/dialog");
      const selected = await open({ directory: true, multiple: false });
      if (typeof selected === "string") {
        folderPath = selected;
      }
    } catch (e: any) {
      error.set(e?.message ?? String(e));
    }
  }

  function detachFolderScanListeners() {
    while (folderUnsubs.length) {
      const u = folderUnsubs.pop();
      try { u && u(); } catch {}
    }
  }

  async function attachFolderScanListeners() {
    if (!isTauri) return;
    const { listen } = await import("@tauri-apps/api/event");
    const u1 = await listen("folder_scan_started", (ev: any) => {
      const p = ev?.payload;
      if (!folderScanId || p?.scan_id !== folderScanId) return;
      folderProgress = { discovered: 0, scanned: 0, total: 0, malicious: 0, errors: 0 };
    });
    const u2 = await listen("folder_scan_progress", (ev: any) => {
      const p = ev?.payload;
      if (!folderScanId || p?.scan_id !== folderScanId) return;
      folderProgress = { ...(folderProgress || {}), ...(p || {}) };
    });
    const u3 = await listen("folder_scan_cancelled", (ev: any) => {
      const p = ev?.payload;
      if (!folderScanId || p?.scan_id !== folderScanId) return;
      folderProgress = { ...(folderProgress || {}), ...(p || {}), cancelled: true };
      detachFolderScanListeners();
    });
    const u4 = await listen("folder_scan_completed", (ev: any) => {
      const p = ev?.payload;
      if (!folderScanId || p?.scan_id !== folderScanId) return;
      folderProgress = { ...(folderProgress || {}), ...(p || {}), completed: true };
      detachFolderScanListeners();
      // Refresh security events after completion
      loadEvents();
    });
    folderUnsubs.push(u1 as any, u2 as any, u3 as any, u4 as any);
  }

  async function startFolderScan() {
    if (!isTauri) {
      error.set("Desktop runtime required.");
      return;
    }
    if (!folderPath) {
      error.set("Please select a folder to scan.");
      return;
    }
    loading.set(true);
    status.set(null);
    error.set(null);
    folderProgress = null;
    try {
      const id = await tauriInvoke<string>("start_folder_scan", {
        root: folderPath,
        use_cloud: useCloud,
        quarantine,
      });
      folderScanId = id;
      await attachFolderScanListeners();
      status.set("Folder scan started.");
    } catch (e: any) {
      error.set(e?.message ?? String(e));
    } finally {
      loading.set(false);
    }
  }

  async function cancelFolderScan() {
    if (!isTauri || !folderScanId) return;
    try {
      await tauriInvoke("cancel_folder_scan", { scan_id: folderScanId });
      status.set("Folder scan cancellation requested.");
    } catch (e: any) {
      error.set(e?.message ?? String(e));
    }
  }
  async function runScan() {
    if (!isTauri) {
      error.set("Desktop runtime required.");
      return;
    }
    if (!filePath) {
      error.set("Please select a file to scan.");
      return;
    }
    loading.set(true);
    status.set(null);
    error.set(null);
    scanResult = null;
    try {
      scanResult = await tauriInvoke("scan_file_command", {
        path: filePath,
        use_cloud: useCloud,
        quarantine,
      });
      status.set(scanResult?.malicious ? "Malicious file detected." : "No threats detected.");
      // refresh events after scan
      await loadEvents();
    } catch (e: any) {
      error.set(e?.message ?? String(e));
    } finally {
      loading.set(false);
    }
  }

  async function createSession() {
    if (!isTauri) {
      error.set("Desktop runtime required.");
      return;
    }
    loading.set(true);
    status.set(null);
    error.set(null);
    try {
      const permissions = permissionsCsv
        .split(",")
        .map((s) => s.trim())
        .filter(Boolean);
      const sessionId = await tauriInvoke<string>("create_security_session", {
        user_id: userId || "user-1",
        permissions,
        ip_address: ipAddress || null,
        user_agent: userAgent || null,
      });
      createdSessionId.set(sessionId);
      checkSessionId = sessionId;
      status.set("Session created.");
      await loadEvents();
    } catch (e: any) {
      error.set(e?.message ?? String(e));
    } finally {
      loading.set(false);
    }
  }

  async function validateSession() {
    if (!isTauri) return;
    loading.set(true);
    status.set(null);
    error.set(null);
    try {
      const ok = await tauriInvoke<boolean>("validate_security_session", {
        session_id: checkSessionId,
      });
      validateResult.set(ok);
      status.set(ok ? "Session is valid." : "Session is NOT valid.");
    } catch (e: any) {
      error.set(e?.message ?? String(e));
    } finally {
      loading.set(false);
    }
  }

  async function checkPermission() {
    if (!isTauri) return;
    loading.set(true);
    status.set(null);
    error.set(null);
    try {
      const ok = await tauriInvoke<boolean>("check_security_permission", {
        session_id: checkSessionId,
        permission: permissionToCheck,
      });
      permissionResult.set(ok);
      status.set(ok ? `Permission \"${permissionToCheck}\" granted.` : `Permission \"${permissionToCheck}\" denied.`);
    } catch (e: any) {
      error.set(e?.message ?? String(e));
    } finally {
      loading.set(false);
    }
  }

  async function loadEvents() {
    if (!isTauri) return;
    try {
      const list = await tauriInvoke<any[]>("get_security_events", { limit: eventsLimit });
      events.set(list ?? []);
    } catch (e) {
      // non-fatal
      console.warn("get_security_events failed", e);
    }
  }

  onMount(async () => {
    await loadEvents();
    // Auto-refresh
    refreshTimer = setInterval(loadEvents, 5000);
    // In browser mode (non-Tauri), support E2E simulation via CustomEvent('folder_scan')
    if (!isTauri && typeof window !== 'undefined') {
      const handler = (ev: Event) => {
        try {
          const detail: any = (ev as CustomEvent).detail || {};
          const action = (detail.action || '').toLowerCase();
          if (action === 'set') {
            // Initialize or switch the active scan id/path for simulation
            folderScanId = detail.scan_id || detail.id || folderScanId;
            if (detail.folder) folderPath = detail.folder;
            folderProgress = null;
          } else if (action === 'event') {
            const type = (detail.type || '').toLowerCase();
            const p = detail.payload || {};
            const id = p.scan_id || detail.scan_id || folderScanId;
            if (!folderScanId || id !== folderScanId) return;
            if (type === 'started') {
              folderProgress = { discovered: 0, scanned: 0, total: 0, malicious: 0, errors: 0 };
            } else if (type === 'progress') {
              folderProgress = { ...(folderProgress || {}), ...(p || {}) };
            } else if (type === 'cancelled') {
              folderProgress = { ...(folderProgress || {}), ...(p || {}), cancelled: true };
            } else if (type === 'completed') {
              folderProgress = { ...(folderProgress || {}), ...(p || {}), completed: true };
            }
          }
        } catch {
          // ignore
        }
      };
      window.addEventListener('folder_scan', handler as EventListener);
      // ensure cleanup on destroy
      folderUnsubs.push(() => window.removeEventListener('folder_scan', handler as EventListener));
    }
  });

  onDestroy(() => {
    if (refreshTimer) clearInterval(refreshTimer);
    detachFolderScanListeners();
  });
</script>

<div class="security-center">
  <div class="header">
    <h2>🔒 Security Center</h2>
    <div class="actions">
      <button class="btn" on:click={loadEvents} disabled={$loading}>⟳ Refresh</button>
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
      <h3>Create Security Session</h3>
      <div class="field">
        <label for="user-id">User ID</label>
        <input id="user-id" type="text" bind:value={userId} placeholder="user-123" />
      </div>
      <div class="field">
        <label for="permissions-csv">Permissions (CSV)</label>
        <input id="permissions-csv" type="text" bind:value={permissionsCsv} placeholder="read,write,admin" />
      </div>
      <div class="field inline">
        <div>
          <label for="ip-address">IP Address (optional)</label>
          <input id="ip-address" type="text" bind:value={ipAddress} placeholder="127.0.0.1" />
        </div>
        <div>
          <label for="user-agent">User Agent (optional)</label>
          <input id="user-agent" type="text" bind:value={userAgent} placeholder="OxidePilot/1.0" />
        </div>
      </div>
      <button class="btn primary" on:click={createSession} disabled={$loading || !isTauri}>Create Session</button>
      {#if $createdSessionId}
        <div class="note">Session ID: <code>{$createdSessionId}</code></div>
      {/if}
    </div>

    <div class="card">
      <h3>Autonomous Threat Analysis (No External VT)</h3>
      <div class="row">
        <button class="btn primary" on:click={runThreatConsensus} disabled={consensusLoading || !isTauri}>
          {#if consensusLoading}Running…{/if}{#if !consensusLoading}Run Threat Consensus{/if}
        </button>
      </div>
      {#if consensusError}
        <div class="note warn">{consensusError}</div>
      {/if}
      {#if threatReport}
        <div class="note">Mode: <code>{threatReport.mode}</code> | Risk: <strong>{threatReport.risk_score?.toFixed?.(1) ?? threatReport.risk_score}</strong> | Confidence: {threatReport.confidence}</div>
        {#if threatRecs?.length}
          <div class="note"><strong>Recommendations</strong></div>
          <ul>
            {#each threatRecs as r}
              <li>{r}</li>
            {/each}
          </ul>
        {/if}
        <details>
          <summary>Show full report JSON</summary>
          <pre class="payload">{JSON.stringify(threatReport, null, 2)}</pre>
        </details>
      {/if}
    </div>

    <div class="card">
      <h3>Validate / Check Permission</h3>
      <div class="field">
        <label for="check-session-id">Session ID</label>
        <input id="check-session-id" type="text" bind:value={checkSessionId} placeholder="paste session id" />
      </div>
      <div class="row">
        <button class="btn" on:click={validateSession} disabled={$loading || !isTauri}>Validate Session</button>
        {#if $validateResult !== null}
          <span class="pill { $validateResult ? 'ok' : 'bad' }">{$validateResult ? 'Valid' : 'Invalid'}</span>
        {/if}
      </div>

      <div class="field">
        <label for="permission-check">Permission</label>
        <input id="permission-check" type="text" bind:value={permissionToCheck} placeholder="admin" />
      </div>
      <div class="row">
        <button class="btn" on:click={checkPermission} disabled={$loading || !isTauri}>Check Permission</button>
        {#if $permissionResult !== null}
          <span class="pill { $permissionResult ? 'ok' : 'bad' }">{$permissionResult ? 'Granted' : 'Denied'}</span>
        {/if}
      </div>
    </div>

    <div class="card">
      <h3>Antivirus: File Scan</h3>
      <div class="field">
        <label for="file-path">File</label>
        <div class="row">
          <input id="file-path" type="text" bind:value={filePath} placeholder="Select a file..." readonly />
          <button class="btn" on:click={pickFile} disabled={!isTauri}>Browse…</button>
        </div>
      </div>
      <div class="row">
        <label><input type="checkbox" bind:checked={useCloud} /> Use VirusTotal (cloud)</label>
        <label><input type="checkbox" bind:checked={quarantine} /> Quarantine if malicious</label>
      </div>
      {#if useCloud && vtConfigured === false}
        <div class="note warn">VirusTotal key not configured. Set <code>VIRUSTOTAL_API_KEY</code> in <code>src-tauri/.env</code> or configure an encrypted key in settings.</div>
      {/if}
      <div class="row">
        <button class="btn primary" on:click={runScan} disabled={$loading || !isTauri}>Scan File</button>
      </div>
      {#if scanResult}
        <div class="note">Result for <code>{scanResult?.path}</code></div>
        <pre class="payload">{JSON.stringify(scanResult, null, 2)}</pre>
      {/if}
    </div>

    <div class="card">
      <h3>Antivirus: Folder Scan</h3>
      <div class="field">
        <label for="folder-path">Folder</label>
        <div class="row">
          <input id="folder-path" type="text" bind:value={folderPath} placeholder="Select a folder..." readonly />
          <button class="btn" on:click={pickFolder} disabled={!isTauri}>Browse…</button>
        </div>
      </div>
      <div class="row">
        <label><input type="checkbox" bind:checked={useCloud} /> Use VirusTotal (cloud)</label>
        <label><input type="checkbox" bind:checked={quarantine} /> Quarantine if malicious</label>
      </div>
      <div class="row">
        <button class="btn primary" on:click={startFolderScan} disabled={$loading || !isTauri || !folderPath}>Start Scan</button>
        <button class="btn" on:click={cancelFolderScan} disabled={!folderScanId}>Cancel</button>
      </div>
      {#if folderProgress}
        <div class="note">Scan ID: <code>{folderScanId}</code></div>
        <div class="progress">
          <div>Discovered: {folderProgress.discovered ?? 0}</div>
          <div>Scanned: {folderProgress.scanned ?? 0} / {folderProgress.total ?? 0}</div>
          <div>Malicious: {folderProgress.malicious ?? 0}</div>
          <div>Errors: {folderProgress.errors ?? 0}</div>
          {#if folderProgress.current_file}
            <div>Current: <code>{folderProgress.current_file}</code></div>
          {/if}
          {#if folderProgress.local_match !== undefined}
            <div>Local match: {String(folderProgress.local_match)}</div>
          {/if}
          {#if folderProgress.external_verdict}
            <div>External verdict: {folderProgress.external_verdict}</div>
          {/if}
          {#if folderProgress.completed}
            <div class="note">Completed in {folderProgress.duration_ms ?? 0} ms.</div>
          {/if}
          {#if folderProgress.cancelled}
            <div class="note warn">Scan cancelled.</div>
          {/if}
        </div>
      {/if}
    </div>

    <div class="card full">
      <div class="events-header">
        <h3>Security Events</h3>
        <div class="controls">
          <label for="events-limit">Limit</label>
          <input id="events-limit" type="number" min="1" max="200" bind:value={eventsLimit} />
        </div>
      </div>
      {#if $events.length === 0}
        <div class="empty">No security events yet.</div>
      {:else}
        <div class="events">
          {#each $events as ev}
            <div class="event">
              <div class="meta">
                <span class="time">{new Date(ev.timestamp ?? Date.now()).toLocaleString()}</span>
                <span class="type">{ev.event_type ?? ev.kind ?? 'event'}</span>
              </div>
              <pre class="payload">{JSON.stringify(ev, null, 2)}</pre>
            </div>
          {/each}
        </div>
      {/if}
    </div>
  </div>
</div>

<style>
  .security-center { background: #fff; border-radius: 12px; padding: 16px; box-shadow: 0 6px 20px rgba(0,0,0,0.08); }
  .header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px; }
  .actions { display: flex; gap: 8px; }
  .status { padding: 10px 12px; border-radius: 8px; margin-bottom: 10px; font-weight: 500; }
  .status.success { background: #e7f9ed; color: #126d3b; border: 1px solid #bfe8cc; }
  .status.error { background: #fde8e8; color: #9b1c1c; border: 1px solid #fbd5d5; }

  .grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 12px; }
  .card { background: #f8f9fa; border: 1px solid #edf2f7; border-radius: 10px; padding: 12px; }
  .card.full { grid-column: 1 / -1; }
  .field { display: flex; flex-direction: column; gap: 6px; margin-bottom: 10px; }
  .field.inline { display: grid; grid-template-columns: 1fr 1fr; gap: 10px; }
  label { font-size: 12px; color: #6b7280; }
  input[type="text"], input[type="number"] { padding: 8px 10px; border-radius: 8px; border: 1px solid #e5e7eb; background: #fff; color: #111827; }
  .row { display: flex; align-items: center; gap: 8px; }
  .btn { padding: 8px 12px; border-radius: 8px; border: 1px solid rgba(0,0,0,0.08); background: #eef2ff; color: #3730a3; cursor: pointer; }
  .btn.primary { background: #1f2937; color: #fff; border-color: #1f2937; }
  .note { margin-top: 8px; color: #374151; font-size: 12px; }
  .note.warn { background: #fff7ed; color: #92400e; border: 1px solid #fed7aa; padding: 6px 8px; border-radius: 6px; }
  code { background: #111827; color: #e5e7eb; padding: 2px 6px; border-radius: 6px; }

  .progress { display: grid; grid-template-columns: repeat(2, minmax(0,1fr)); gap: 6px; margin-top: 6px; }

  .events-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px; }
  .controls { display: flex; align-items: center; gap: 8px; }
  .empty { color: #6b7280; padding: 10px; }
  .events { display: grid; gap: 8px; }
  .event { background: #fff; border: 1px solid #e5e7eb; border-radius: 8px; padding: 8px; }
  .meta { display: flex; gap: 8px; font-size: 12px; color: #6b7280; margin-bottom: 6px; }
  .time { font-weight: 500; }
  .type { background: #eef2ff; color: #3730a3; border: 1px solid #c7d2fe; padding: 2px 6px; border-radius: 999px; }
  .payload { max-height: 200px; overflow: auto; white-space: pre; font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace; }
  .pill { padding: 2px 8px; border-radius: 999px; font-size: 12px; border: 1px solid; }
  .pill.ok { background: #e7f9ed; color: #126d3b; border-color: #bfe8cc; }
  .pill.bad { background: #fde8e8; color: #9b1c1c; border-color: #fbd5d5; }

  @media (max-width: 900px) { .grid { grid-template-columns: 1fr; } .field.inline { grid-template-columns: 1fr; } }
</style>
