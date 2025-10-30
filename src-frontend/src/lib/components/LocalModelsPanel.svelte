<script lang="ts">
import { onMount } from "svelte";
import { isTauri } from "$lib/utils/env";
import { tauriInvoke } from "$lib/utils/tauri";

// Server status
let status: { running: boolean; port?: number; message?: string } | null = null;
let statusError: string | null = null;
let port: number = 1234;
let cors: boolean = true;
let busy: boolean = false;

// Models
let modelsJson: string = "";
let modelListError: string | null = null;

// Download
let modelSpec: string = "ui-tars-1.5";
let useGguf: boolean = true;
let assumeYes: boolean = true;
let downloadMsg: string = "";

// Load
let modelKey: string = "ui-tars-1.5-7b-q4_K_M.gguf";
let identifier: string = "ui-tars-local";
let contextLen: number = 4096;
let gpu: string = "auto";
let ttlSecs: number = 3600;
let loadMsg: string = "";

// Chat
let baseUrl: string = "";
let apiKey: string = "";
let chatModel: string = "";
let systemPrompt: string = "";
let userPrompt: string = "";
let chatResp: string = "";
let chatError: string | null = null;

onMount(async () => {
  await refreshStatus();
});

async function refreshStatus() {
  if (!isTauri) return;
  statusError = null;
  try {
    status = await tauriInvoke("local_llm_server_status");
  } catch (e) {
    status = null;
    statusError = e instanceof Error ? e.message : String(e);
  }
}

async function startServer() {
  if (!isTauri) return;
  busy = true;
  try {
    const msg = await tauriInvoke<string>("local_llm_server_start", {
      port,
      cors,
    });
    await refreshStatus();
    downloadMsg = msg;
  } catch (e) {
    downloadMsg = e instanceof Error ? e.message : String(e);
  } finally {
    busy = false;
  }
}

async function stopServer() {
  if (!isTauri) return;
  busy = true;
  try {
    const msg = await tauriInvoke<string>("local_llm_server_stop");
    await refreshStatus();
    downloadMsg = msg;
  } catch (e) {
    downloadMsg = e instanceof Error ? e.message : String(e);
  } finally {
    busy = false;
  }
}

async function listModels() {
  if (!isTauri) return;
  modelListError = null;
  try {
    const out = await tauriInvoke<string>("local_llm_ls");
    modelsJson = out;
  } catch (e) {
    modelListError = e instanceof Error ? e.message : String(e);
  }
}

async function downloadModel() {
  if (!isTauri) return;
  busy = true;
  try {
    const msg = await tauriInvoke<string>("local_llm_get", {
      modelSpec,
      gguf: useGguf,
      yes: assumeYes,
    });
    downloadMsg = msg;
  } catch (e) {
    downloadMsg = e instanceof Error ? e.message : String(e);
  } finally {
    busy = false;
  }
}

async function loadModel() {
  if (!isTauri) return;
  busy = true;
  try {
    const msg = await tauriInvoke<string>("local_llm_load", {
      modelKey,
      identifier,
      contextLen,
      gpu,
      ttlSecs,
    });
    loadMsg = msg;
  } catch (e) {
    loadMsg = e instanceof Error ? e.message : String(e);
  } finally {
    busy = false;
  }
}

async function sendChat() {
  if (!isTauri) return;
  chatError = null;
  chatResp = "";
  busy = true;
  try {
    const resp = await tauriInvoke<string>("local_llm_chat", {
      baseUrl: baseUrl || undefined,
      apiKey: apiKey || undefined,
      model: chatModel || undefined,
      systemPrompt: systemPrompt || undefined,
      userPrompt,
    });
    chatResp = resp;
  } catch (e) {
    chatError = e instanceof Error ? e.message : String(e);
  } finally {
    busy = false;
  }
}
</script>

{#if !isTauri}
  <p class="warn">Local models are only available in the desktop (Tauri) app.</p>
{:else}
  <section class="panel">
    <h4>LM Studio Server</h4>
    <div class="row">
      <button on:click={refreshStatus}>Refresh Status</button>
      <span class="status" class:ok={status?.running} class:bad={!status?.running}>
        {#if status}
          {status.running ? `Running${status.port ? ` :${status.port}` : ''}` : 'Stopped'}
        {:else}
          Unknown
        {/if}
      </span>
      {#if statusError}<span class="err">{statusError}</span>{/if}
    </div>
    <div class="row">
      <label for="port">Port</label>
      <input id="port" type="number" bind:value={port} min="1" max="65535" />
      <label><input type="checkbox" bind:checked={cors} /> CORS</label>
      <button on:click={startServer} disabled={busy}>Start</button>
      <button on:click={stopServer} disabled={busy}>Stop</button>
    </div>
  </section>

  <section class="panel">
    <h4>Models</h4>
    <div class="row">
      <button on:click={listModels} disabled={busy}>List Models</button>
      {#if modelListError}<span class="err">{modelListError}</span>{/if}
    </div>
    {#if modelsJson}
      <pre class="code">{modelsJson}</pre>
    {/if}
    <div class="row">
      <label for="modelSpec">Model Spec</label>
      <input id="modelSpec" type="text" bind:value={modelSpec} placeholder="e.g., ui-tars-1.5" />
      <label><input type="checkbox" bind:checked={useGguf} /> GGUF</label>
      <label><input type="checkbox" bind:checked={assumeYes} /> Yes</label>
      <button on:click={downloadModel} disabled={busy}>Download</button>
    </div>
    {#if downloadMsg}
      <pre class="code">{downloadMsg}</pre>
    {/if}
    <div class="row">
      <label for="modelKey">Model Key</label>
      <input id="modelKey" type="text" bind:value={modelKey} placeholder="ui-tars-1.5-7b-q4_K_M.gguf" />
      <label for="identifier">Identifier</label>
      <input id="identifier" type="text" bind:value={identifier} placeholder="ui-tars-local" />
    </div>
    <div class="row">
      <label for="contextLen">Context</label>
      <input id="contextLen" type="number" bind:value={contextLen} min="256" max="32768" />
      <label for="gpu">GPU</label>
      <input id="gpu" type="text" bind:value={gpu} placeholder="auto or N" />
      <label for="ttlSecs">TTL (s)</label>
      <input id="ttlSecs" type="number" bind:value={ttlSecs} min="0" />
      <button on:click={loadModel} disabled={busy}>Load</button>
    </div>
    {#if loadMsg}
      <pre class="code">{loadMsg}</pre>
    {/if}
  </section>

  <section class="panel">
    <h4>Quick Chat (Local)</h4>
    <div class="row">
      <label for="baseUrl">Base URL</label>
      <input id="baseUrl" type="text" bind:value={baseUrl} placeholder="http://127.0.0.1:1234/v1" />
      <label for="apiKey">API Key</label>
      <input id="apiKey" type="text" bind:value={apiKey} placeholder="(optional)" />
      <label for="chatModel">Model</label>
      <input id="chatModel" type="text" bind:value={chatModel} placeholder="(optional, uses env or identifier)" />
    </div>
    <div class="row">
      <label for="systemPrompt">System Prompt</label>
      <input id="systemPrompt" type="text" bind:value={systemPrompt} placeholder="(optional)" />
    </div>
    <div class="row">
      <label for="userPrompt" class="sr-only">User Prompt</label>
      <textarea id="userPrompt" bind:value={userPrompt} rows="3" placeholder="Ask something..."></textarea>
    </div>
    <div class="row">
      <button on:click={sendChat} disabled={busy || !userPrompt.trim()}>Send</button>
      {#if chatError}<span class="err">{chatError}</span>{/if}
    </div>
    {#if chatResp}
      <pre class="code">{chatResp}</pre>
    {/if}
  </section>
{/if}

<style>
  .panel { padding: 12px; border: 1px solid rgba(0,0,0,0.06); border-radius: 10px; background: var(--color-surface); margin: 10px 0; }
  .row { display: flex; gap: 8px; align-items: center; flex-wrap: wrap; margin: 8px 0; }
  .status { padding: 4px 8px; border-radius: 6px; font-size: 12px; }
  .status.ok { background: #e6ffed; color: #065f46; }
  .status.bad { background: #fee2e2; color: #991b1b; }
  .err { color: #b91c1c; font-size: 12px; }
  .code { white-space: pre-wrap; background: #0b1020; color: #e3e8f8; padding: 8px; border-radius: 8px; }
  input[type="text"], input[type="number"], textarea { flex: 1 1 220px; padding: 6px 8px; border: 1px solid rgba(0,0,0,0.12); border-radius: 8px; }
  button { padding: 8px 12px; border: 1px solid rgba(0,0,0,0.08); border-radius: 8px; background: var(--color-surface); cursor: pointer; }
  .warn { color: var(--color-muted); }
  /* Visually hidden but accessible */
  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
    border: 0;
  }
</style>
