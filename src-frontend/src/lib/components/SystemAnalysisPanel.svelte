<script lang="ts">
  import { onMount } from "svelte";
  import { writable } from "svelte/store";
  import { isTauri } from "$lib/utils/env";

  import { tauriInvoke } from "$lib/utils/tauri";

  // Optional model overrides
  let geminiModel: string = "";
  let qwenModel: string = "";

  const loading = writable(false);
  const error = writable<string | null>(null);
  const geminiSummary = writable<string | null>(null);
  const qwenDeepAnalysis = writable<string | null>(null);
  const snapshot = writable<any | null>(null);

  async function runAnalysis() {
    if (!isTauri) {
      error.set("Not running in Tauri context. Please use the desktop app.");
      return;
    }
    loading.set(true);
    error.set(null);
    geminiSummary.set(null);
    qwenDeepAnalysis.set(null);
    snapshot.set(null);

    try {
      const args: Record<string, unknown> = {};
      if (geminiModel.trim()) args.gemini_model = geminiModel.trim();
      if (qwenModel.trim()) args.qwen_model = qwenModel.trim();

      const res = await tauriInvoke<string>("run_multi_agent_analysis", args);
      // Backend returns a JSON string
      let parsed: any = null;
      try {
        parsed = JSON.parse(res);
      } catch {
        // Fallback: show raw text in Gemini panel
        geminiSummary.set(res);
        qwenDeepAnalysis.set("(No structured response)");
        snapshot.set(null);
        return;
      }

      geminiSummary.set(parsed?.gemini_summary ?? null);
      qwenDeepAnalysis.set(parsed?.qwen_deep_analysis ?? null);
      snapshot.set(parsed?.snapshot ?? null);
    } catch (e: any) {
      console.error("run_multi_agent_analysis failed", e);
      error.set(e?.toString?.() ?? String(e));
    } finally {
      loading.set(false);
    }
  }

  function copy(text: string) {
    try {
      navigator.clipboard.writeText(text);
    } catch (_) {
      // ignore
    }
  }

  onMount(() => {
    // no-op for now
  });
</script>

<div class="analysis-panel">
  <h2>🧠 Multi-Agent System Analysis</h2>
  <p class="subtitle">Gemini planning + Qwen deep analysis on current system snapshot.</p>

  <div class="controls">
    <div class="models">
      <div class="model-field">
        <label for="gemini-model">Gemini model (optional)</label>
        <input id="gemini-model" bind:value={geminiModel} placeholder="e.g. gemini-1.5-pro" />
      </div>
      <div class="model-field">
        <label for="qwen-model">Qwen model (optional)</label>
        <input id="qwen-model" bind:value={qwenModel} placeholder="e.g. qwen-plus" />
      </div>
    </div>
    <button class="run-btn" on:click={runAnalysis} disabled={$loading || !isTauri}>
      {#if $loading}
        <span class="spinner"></span> Running...
      {:else}
        ▶️ Run Analysis
      {/if}
    </button>
  </div>

  {#if $error}
    <div class="error-box">{$error}</div>
  {/if}

  <div class="results-grid">
    <div class="result-card">
      <div class="card-header">
        <h3>Gemini Summary</h3>
        {#if $geminiSummary}
          <button class="copy" on:click={() => copy($geminiSummary!)}>Copy</button>
        {/if}
      </div>
      {#if $geminiSummary}
        <pre class="text">{$geminiSummary}</pre>
      {:else}
        <div class="placeholder">No result yet. Click "Run Analysis".</div>
      {/if}
    </div>

    <div class="result-card">
      <div class="card-header">
        <h3>Qwen Deep Analysis</h3>
        {#if $qwenDeepAnalysis}
          <button class="copy" on:click={() => copy($qwenDeepAnalysis!)}>Copy</button>
        {/if}
      </div>
      {#if $qwenDeepAnalysis}
        <pre class="text">{$qwenDeepAnalysis}</pre>
      {:else}
        <div class="placeholder">Awaiting analysis...</div>
      {/if}
    </div>

    <div class="result-card full">
      <div class="card-header">
        <h3>System Snapshot</h3>
        {#if $snapshot}
          <button class="copy" on:click={() => copy(JSON.stringify($snapshot, null, 2))}>Copy JSON</button>
        {/if}
      </div>
      {#if $snapshot}
        <pre class="json">{JSON.stringify($snapshot, null, 2)}</pre>
      {:else}
        <div class="placeholder">Snapshot will appear after analysis.</div>
      {/if}
    </div>
  </div>
</div>

<style>
  .analysis-panel {
    padding: 20px;
  }
  .subtitle { color: var(--color-muted); margin-top: -6px; margin-bottom: 14px; }

  .controls { display: flex; align-items: flex-end; gap: 12px; margin-bottom: 16px; flex-wrap: wrap; }
  .models { display: flex; gap: 12px; flex-wrap: wrap; }
  .model-field { display: flex; flex-direction: column; gap: 6px; min-width: 240px; }
  .model-field input { padding: 8px 10px; border-radius: 8px; border: 1px solid rgba(0,0,0,0.1); background: var(--color-surface); color: var(--color-text); }

  .run-btn { padding: 10px 16px; border-radius: var(--radius-pill); border: 1px solid rgba(0,0,0,0.06); background: var(--color-primary); color: #fff; cursor: pointer; box-shadow: 0 4px 12px rgba(79,70,229,0.25); }
  .run-btn[disabled] { opacity: 0.6; cursor: not-allowed; }
  .spinner { display: inline-block; width: 14px; height: 14px; border: 2px solid rgba(255,255,255,0.6); border-top-color: #fff; border-radius: 50%; margin-right: 8px; animation: spin 1s linear infinite; }
  @keyframes spin { to { transform: rotate(360deg); } }

  .error-box { background: #fee2e2; color: #991b1b; padding: 10px 12px; border-radius: 8px; border: 1px solid #fecaca; margin-bottom: 12px; }

  .results-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 16px; }
  .result-card { background: var(--color-surface); border-radius: 12px; padding: 12px; box-shadow: var(--shadow-md); display: flex; flex-direction: column; }
  .result-card.full { grid-column: 1 / -1; }
  .card-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px; }
  .copy { padding: 6px 10px; border-radius: 8px; background: #eef2ff; color: #3730a3; border: 1px solid #c7d2fe; cursor: pointer; }
  .text { white-space: pre-wrap; line-height: 1.35; font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace; }
  .json { white-space: pre; overflow: auto; max-height: 420px; font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace; }
  .placeholder { color: var(--color-muted); }

  @media (max-width: 900px) {
    .results-grid { grid-template-columns: 1fr; }
  }
</style>
