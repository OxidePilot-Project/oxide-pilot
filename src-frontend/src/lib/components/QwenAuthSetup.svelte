<script lang="ts">
  import { onDestroy, onMount, tick } from "svelte";
  import { createEventDispatcher } from "svelte";
  import { writable } from "svelte/store";
  import { isTauri } from "$lib/utils/env";

  const dispatch = createEventDispatcher();

  import { tauriInvoke } from "$lib/utils/tauri";

  type StatusType = "success" | "error" | "info" | "warning";
  const status = writable<{ message: string; type: StatusType }>({ message: "", type: "info" });

  let isBusy = false;
  let hasExistingSession = false;

  // Device flow state
  let device_code = "";
  let user_code = "";
  let verification_uri = "";
  let expires_in: number | null = null; // seconds
  let intervalSec = 5; // default seconds; server may override
  let remainingSec = 0; // countdown for UI

  let pollAbort = false;
  let pollTimer: number | null = null;
  let countdownTimer: number | null = null;

  // QR code rendering
  let qrCanvas: HTMLCanvasElement | null = null;
  let qrRenderError: string | null = null;

  async function renderQR() {
    try {
      if (!verification_uri || !qrCanvas) return;
      // Dynamic import to avoid SSR bundle issues
      const mod: any = await import("qrcode");
      // Render compact QR for the verification URL (complete if provided by backend)
      await mod.toCanvas(qrCanvas, verification_uri, {
        width: 180,
        errorCorrectionLevel: "M",
        margin: 1,
        color: { dark: "#111827", light: "#ffffff" }
      });
      qrRenderError = null;
    } catch (e: any) {
      console.error("QR render failed", e);
      qrRenderError = e?.message ?? String(e);
    }
  }

  function clearTimers() {
    if (pollTimer !== null) {
      clearTimeout(pollTimer);
      pollTimer = null;
    }
    if (countdownTimer !== null) {
      clearInterval(countdownTimer);
      countdownTimer = null;
    }
  }

  onDestroy(() => {
    clearTimers();
  });

  async function checkQwenStatus() {
    if (!isTauri) {
      return;
    }
    try {
      const res = await tauriInvoke<string>("qwen_get_auth_status");
      // Consider authenticated if response contains the word 'auth' positively.
      hasExistingSession = /auth/i.test(res) && !/not\s+auth/i.test(res);
      if (hasExistingSession) {
        status.set({ message: "Qwen session is available.", type: "success" });
      }
    } catch (e) {
      // ignore; no session
      hasExistingSession = false;
    }
  }

  onMount(() => {
    checkQwenStatus();
    // In browser mode, support E2E simulation via CustomEvent("qwen_device_flow")
    if (!isTauri && typeof window !== 'undefined') {
      const handler = (ev: Event) => {
        try {
          const detail: any = (ev as CustomEvent).detail || {};
          const action = (detail.action || '').toLowerCase();
          if (action === 'start') {
            // Initialize device flow UI state from payload
            isBusy = true;
            pollAbort = false;
            clearTimers();
            device_code = detail.device_code || 'mock_device_code';
            user_code = detail.user_code || 'MOCK-C0DE';
            verification_uri = detail.verification_uri || 'https://example.com/verify';
            expires_in = typeof detail.expires_in === 'number' ? detail.expires_in : 300;
            intervalSec = typeof detail.interval === 'number' ? detail.interval : 5;
            if (expires_in) startCountdown(expires_in);
            status.set({ message: 'Open the verification URL and enter the code to authorize.', type: 'info' });
            void tick().then(() => void renderQR());
          } else if (action === 'success') {
            clearTimers();
            hasExistingSession = true;
            status.set({ message: 'Authenticated successfully. Tokens stored securely.', type: 'success' });
            isBusy = false;
            dispatch('authSuccess', { provider: 'qwen' });
            dispatch('authComplete');
          } else if (action === 'error') {
            clearTimers();
            const msg = detail.message || 'Authentication failed';
            status.set({ message: msg, type: 'error' });
            isBusy = false;
          } else if (action === 'pending') {
            // no-op; reserved for future simulation of polling
          } else if (action === 'clear') {
            device_code = '';
            user_code = '';
            verification_uri = '';
            expires_in = null;
            isBusy = false;
            clearTimers();
            hasExistingSession = false;
            status.set({ message: 'Session cleared (mock).', type: 'info' });
          }
        } catch (_) { /* ignore */ }
      };
      window.addEventListener('qwen_device_flow', handler as EventListener);
      return () => window.removeEventListener('qwen_device_flow', handler as EventListener);
    }
  });

  function startCountdown(totalSec: number) {
    remainingSec = totalSec;
    countdownTimer = window.setInterval(() => {
      remainingSec = Math.max(0, remainingSec - 1);
      if (remainingSec <= 0) {
        clearTimers();
        status.set({ message: "Device code expired. Please restart authentication.", type: "warning" });
      }
    }, 1000);
  }

  async function startQwenAuth() {
    if (!isTauri) {
      status.set({ message: "Not running in Tauri context. Please use the desktop app.", type: "error" });
      return;
    }

    isBusy = true;
    pollAbort = false;
    clearTimers();

    status.set({ message: "Starting device authorization...", type: "info" });
    try {
      const start = await tauriInvoke<{
        device_code: string;
        user_code: string;
        verification_uri: string;
        expires_in?: number;
        interval?: number;
      }>("qwen_start_device_auth");

      device_code = start.device_code;
      user_code = start.user_code;
      verification_uri = start.verification_uri;
      expires_in = start.expires_in ?? null;
      intervalSec = start.interval ?? 5;

      if (expires_in) {
        startCountdown(expires_in);
      }

      status.set({ message: "Open the verification URL and enter the code to authorize.", type: "info" });

      // Start polling loop respecting interval and slow_down
      void pollUntilDone();
      // Ensure DOM reflects device-info block, then render QR
      await tick();
      void renderQR();
    } catch (err) {
      console.error("Failed to start Qwen device auth:", err);
      status.set({ message: `Failed to start device auth: ${err}`, type: "error" });
      isBusy = false;
    }
  }

  function scheduleNextPoll(delayMs: number) {
    pollTimer = window.setTimeout(() => void pollUntilDone(), delayMs);
  }

  async function pollUntilDone() {
    if (pollAbort || !device_code) return;
    try {
      const res = await tauriInvoke<{ status: string; message?: string }>("qwen_poll_device_auth", { device_code });
      const st = (res.status || "").toLowerCase();
      if (st === "pending") {
        scheduleNextPoll(intervalSec * 1000);
      } else if (st === "slow_down") {
        intervalSec = Math.min(intervalSec + 2, intervalSec + 5);
        scheduleNextPoll(intervalSec * 1000);
      } else if (st === "success") {
        clearTimers();
        hasExistingSession = true;
        status.set({ message: "Authenticated successfully. Tokens stored securely.", type: "success" });
        isBusy = false;
        dispatch("authSuccess", { provider: "qwen" });
        dispatch("authComplete");
      } else {
        clearTimers();
        const msg = res.message ?? "Authentication failed";
        status.set({ message: msg, type: "error" });
        isBusy = false;
      }
    } catch (err) {
      console.error("Error polling Qwen device grant:", err);
      // Network or transient error -> retry with the same interval as a conservative approach
      scheduleNextPoll(intervalSec * 1000);
    }
  }

  async function openVerification() {
    if (!verification_uri) return;
    try {
      window.open(verification_uri, "_blank", "noopener");
    } catch (e) {
      // noop
    }
  }

  async function copyUserCode() {
    try {
      await navigator.clipboard.writeText(user_code);
      status.set({ message: "Code copied to clipboard", type: "success" });
    } catch (e) {
      status.set({ message: "Failed to copy code", type: "error" });
    }
  }

  function cancelPolling() {
    pollAbort = true;
    clearTimers();
    isBusy = false;
    status.set({ message: "Authentication cancelled.", type: "warning" });
  }

  async function clearQwenSession() {
    if (!isTauri) {
      status.set({ message: 'Clear session is a desktop-only operation in this demo mode.', type: 'info' });
      return;
    }
    try {
      await tauriInvoke("qwen_clear_auth");
      hasExistingSession = false;
      status.set({ message: "Qwen session cleared.", type: "info" });
    } catch (e) {
      status.set({ message: `Failed to clear session: ${e}`, type: "error" });
    }
  }
</script>

<div class="qwen-auth-setup" aria-live="polite" aria-atomic="true">
  <div class="header">
    <div class="title">
      <div class="icon">🤖</div>
      <h3>Qwen Device Authorization</h3>
    </div>
    <button class="clear-btn" on:click={clearQwenSession} title="Clear saved Qwen session">Clear Session</button>
  </div>

  {#if hasExistingSession}
    <div class="status success">✅ Qwen is already authenticated.</div>
  {/if}

  <div class="card">
    <div class="actions">
      <button class="primary" on:click={startQwenAuth} disabled={isBusy} aria-busy={isBusy} aria-label="Start device authorization">Start Device Flow</button>
      {#if isBusy}
        <button class="secondary" on:click={cancelPolling} aria-label="Cancel authentication">Cancel</button>
      {/if}
    </div>

    {#if user_code}
      <div class="device-info">
        <div class="row">
          <span class="label">User Code</span>
          <div class="code-box">
            <code>{user_code}</code>
            <button class="copy" on:click={copyUserCode} title="Copy code" aria-label="Copy user code to clipboard">📋</button>
          </div>
        </div>
        <div class="row">
          <span class="label">Verification URL</span>
          <div class="link-box">
            <a href={verification_uri} target="_blank" rel="noopener">{verification_uri}</a>
            <button class="open" on:click={openVerification} title="Open in browser" aria-label="Open verification URL in browser">🌐</button>
          </div>
        </div>
        <div class="row">
          <span class="label">Or Scan QR</span>
          <div class="qr-box">
            <canvas bind:this={qrCanvas} width="200" height="200" aria-label="Qwen verification QR"></canvas>
            {#if qrRenderError}
              <span class="qr-error">QR unavailable: {qrRenderError}</span>
            {/if}
          </div>
        </div>
        {#if expires_in}
          <div class="row">
            <span class="label">Expires In</span>
            <span class="value">{Math.floor(remainingSec / 60)}m {remainingSec % 60}s</span>
          </div>
        {/if}
        <div class="hint">Complete the authorization in your browser, then return here. The app will detect completion automatically.</div>
      </div>
    {/if}

    {#if $status.message}
      <div class="status-area" role="status" aria-live="polite" aria-atomic="true">
        <div class="status {$status.type}">{$status.message}</div>
      </div>
    {/if}
  </div>
</div>
 

<style>
  .qwen-auth-setup { display: flex; flex-direction: column; gap: 12px; }
  .header { display: flex; align-items: center; justify-content: space-between; }
  .title { display: flex; align-items: center; gap: 8px; }
  .icon { font-size: 20px; }
  .clear-btn { background: transparent; border: 1px solid #ddd; padding: 6px 10px; border-radius: 6px; cursor: pointer; }

  .card { background: #fff; border-radius: 10px; padding: 16px; box-shadow: 0 4px 14px rgba(0,0,0,0.08); }
  .actions { display: flex; gap: 10px; margin-bottom: 12px; }
  .primary { background: #4f46e5; color: #fff; border: none; padding: 10px 14px; border-radius: 8px; cursor: pointer; }
  .secondary { background: #f3f4f6; color: #111827; border: 1px solid #e5e7eb; padding: 10px 14px; border-radius: 8px; cursor: pointer; }

  .device-info { display: flex; flex-direction: column; gap: 10px; margin-top: 8px; }
  .row { display: grid; grid-template-columns: 140px 1fr; gap: 10px; align-items: center; }
  .label { color: #6b7280; font-weight: 600; }
  .code-box, .link-box { display: flex; align-items: center; gap: 8px; }
  .code-box code { background: #111827; color: #e5e7eb; padding: 6px 8px; border-radius: 6px; font-weight: 700; letter-spacing: 1px; }
  /* Prevent long values from causing horizontal overflow */
  .row > .code-box, .row > .link-box { min-width: 0; }
  .code-box code, .link-box a { word-break: break-word; overflow-wrap: anywhere; max-width: 100%; }
  .link-box { flex-wrap: wrap; }
  .copy, .open { border: 1px solid #d1d5db; background: #fff; border-radius: 6px; padding: 6px 8px; cursor: pointer; }
  .value { font-weight: 600; color: #111827; }
  .hint { font-size: 12px; color: #6b7280; margin-top: 4px; }
  .qr-box { display: flex; align-items: center; gap: 10px; }
  .qr-box canvas { border: 1px solid #e5e7eb; border-radius: 8px; background: #fff; }
  .qr-error { color: #b91c1c; font-size: 12px; }
  /* Extra safety to ensure no horizontal scroll inside details block */
  .device-info { overflow-x: hidden; }

  .status-area { margin-top: 12px; }
  .status { padding: 10px 12px; border-radius: 8px; }
  .status.info { background: #eff6ff; color: #1d4ed8; }
  .status.success { background: #ecfdf5; color: #065f46; }
  .status.error { background: #fef2f2; color: #b91c1c; }
  .status.warning { background: #fffbeb; color: #92400e; }
</style>
