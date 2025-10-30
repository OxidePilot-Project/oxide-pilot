<script lang="ts">
import { onMount } from "svelte";

export const fallback: string = "Something went wrong. Please try again.";

let error: Error | null = null;
let errorInfo: string = "";

onMount(() => {
  const handleError = (event: ErrorEvent) => {
    error = event.error;
    errorInfo = event.message;
    console.error("Error caught by boundary:", event.error);
  };

  window.addEventListener("error", handleError);

  return () => {
    window.removeEventListener("error", handleError);
  };
});

function retry() {
  error = null;
  errorInfo = "";
  window.location.reload();
}
</script>

{#if error}
  <div class="error-boundary" role="alert" aria-live="assertive">
    <div class="error-content">
      <h2>⚠️ Error</h2>
      <p class="error-message">{fallback}</p>
      {#if errorInfo}
        <details>
          <summary>Technical Details</summary>
          <pre>{errorInfo}</pre>
        </details>
      {/if}
      <button on:click={retry} class="retry-button">
        🔄 Retry
      </button>
    </div>
  </div>
{:else}
  <slot />
{/if}

<style>
  .error-boundary {
    display: flex;
    justify-content: center;
    align-items: center;
    min-height: 400px;
    padding: 2rem;
    background: #fee;
    border: 2px solid #fcc;
    border-radius: 8px;
    margin: 1rem;
  }

  .error-content {
    text-align: center;
    max-width: 600px;
  }

  h2 {
    color: #c00;
    margin-bottom: 1rem;
  }

  .error-message {
    color: #600;
    margin-bottom: 1rem;
  }

  details {
    text-align: left;
    margin: 1rem 0;
    padding: 1rem;
    background: #fff;
    border-radius: 4px;
  }

  pre {
    overflow-x: auto;
    font-size: 0.875rem;
    color: #333;
  }

  .retry-button {
    padding: 0.5rem 1rem;
    background: #3b82f6;
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 1rem;
  }

  .retry-button:hover {
    background: #2563eb;
  }
</style>
