<script lang="ts">
import { onMount } from "svelte";
import { fade, fly } from "svelte/transition";

export let message: string;
export const type: "success" | "error" | "warning" | "info" = "info";
export const duration: number = 3000;
export const onClose: () => void = () => {};

let visible = true;

const icons = {
  success: "✅",
  error: "❌",
  warning: "⚠️",
  info: "ℹ️",
};

const colors = {
  success: "#10b981",
  error: "#ef4444",
  warning: "#f59e0b",
  info: "#3b82f6",
};

onMount(() => {
  if (duration > 0) {
    const timer = setTimeout(() => {
      close();
    }, duration);

    return () => clearTimeout(timer);
  }
});

function close() {
  visible = false;
  setTimeout(onClose, 300);
}
</script>

{#if visible}
  <div
    class="toast"
    style="border-left-color: {colors[type]}"
    role="alert"
    aria-live="polite"
    transition:fly={{ y: -20, duration: 300 }}
  >
    <span class="icon">{icons[type]}</span>
    <span class="message">{message}</span>
    <button
      class="close-button"
      on:click={close}
      aria-label="Close notification"
    >
      ×
    </button>
  </div>
{/if}

<style>
  .toast {
    position: fixed;
    top: 20px;
    right: 20px;
    min-width: 300px;
    max-width: 500px;
    padding: 1rem;
    background: white;
    border-left: 4px solid;
    border-radius: 4px;
    box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
    display: flex;
    align-items: center;
    gap: 0.75rem;
    z-index: 9999;
  }

  .icon {
    font-size: 1.25rem;
    flex-shrink: 0;
  }

  .message {
    flex: 1;
    color: #333;
  }

  .close-button {
    background: none;
    border: none;
    font-size: 1.5rem;
    color: #999;
    cursor: pointer;
    padding: 0;
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .close-button:hover {
    color: #333;
  }
</style>
