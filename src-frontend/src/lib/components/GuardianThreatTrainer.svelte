<script lang="ts">
import { createEventDispatcher } from "svelte";
import {
  submitThreatTrainingSample,
  type ThreatTrainingSample,
} from "../utils/guardian";

const dispatch = createEventDispatcher();

let severity: ThreatTrainingSample["severity"] = "medium";
let cpu = 50;
let memory = 50;
let network = 10;
let anomaly = 5;
let submitting = false;
let successMessage: string | null = null;
let errorMessage: string | null = null;

async function handleSubmit() {
  submitting = true;
  successMessage = null;
  errorMessage = null;

  try {
    await submitThreatTrainingSample({
      severity,
      cpu_usage: cpu,
      memory_pressure: memory,
      network_score: network,
      anomaly_score: anomaly,
      metadata: {
        source: "manual_trainer",
        timestamp: new Date().toISOString(),
      },
    });

    successMessage = "Training sample stored successfully.";
    dispatch("submitted");
  } catch (error) {
    console.error(error);
    errorMessage = `Failed to submit sample: ${error}`;
  } finally {
    submitting = false;
  }
}
</script>

<div class="trainer-card">
  <div class="trainer-header">
    <h3>Threat Model Trainer</h3>
    <p>Provide labelled samples to refine the SurrealML risk predictor.</p>
  </div>

  <form on:submit|preventDefault={handleSubmit}>
    <div class="field-group">
      <label for="severity">Severity</label>
      <select id="severity" bind:value={severity}>
        <option value="low">Low</option>
        <option value="medium">Medium</option>
        <option value="high">High</option>
        <option value="critical">Critical</option>
      </select>
    </div>

    <div class="slider-group">
      <label for="cpu">CPU Usage (%): {cpu}</label>
      <input id="cpu" type="range" min="0" max="100" bind:value={cpu} />
    </div>

    <div class="slider-group">
      <label for="memory">Memory Pressure (%): {memory}</label>
      <input id="memory" type="range" min="0" max="100" bind:value={memory} />
    </div>

    <div class="slider-group">
      <label for="network">Network Score: {network}</label>
      <input id="network" type="range" min="0" max="200" bind:value={network} />
    </div>

    <div class="slider-group">
      <label for="anomaly">Anomaly Score: {anomaly}</label>
      <input id="anomaly" type="range" min="0" max="100" bind:value={anomaly} />
    </div>

    {#if successMessage}
      <div class="message success">{successMessage}</div>
    {/if}

    {#if errorMessage}
      <div class="message error">{errorMessage}</div>
    {/if}

    <button type="submit" class="submit-btn" disabled={submitting}>
      {submitting ? 'Submitting…' : 'Submit Training Sample'}
    </button>
  </form>
</div>

<style>
  .trainer-card {
    background: white;
    border: 1px solid #e5e7eb;
    border-radius: 0.75rem;
    padding: 1.5rem;
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
  }

  .trainer-header h3 {
    font-size: 1.25rem;
    font-weight: 600;
    color: #111827;
    margin-bottom: 0.25rem;
  }

  .trainer-header p {
    color: #6b7280;
    font-size: 0.9rem;
  }

  form {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .field-group,
  .slider-group {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  label {
    font-weight: 500;
    color: #374151;
  }

  select {
    padding: 0.5rem;
    border-radius: 0.5rem;
    border: 1px solid #d1d5db;
    font-size: 1rem;
  }

  input[type='range'] {
    width: 100%;
  }

  .message {
    padding: 0.75rem 1rem;
    border-radius: 0.5rem;
    font-size: 0.9rem;
  }

  .message.success {
    background: #ecfdf5;
    color: #047857;
    border: 1px solid #a7f3d0;
  }

  .message.error {
    background: #fef2f2;
    color: #b91c1c;
    border: 1px solid #fecaca;
  }

  .submit-btn {
    align-self: flex-start;
    padding: 0.75rem 1.5rem;
    background: #2563eb;
    color: white;
    border: none;
    border-radius: 0.5rem;
    font-weight: 600;
    cursor: pointer;
    transition: background 0.2s;
  }

  .submit-btn:disabled {
    opacity: 0.7;
    cursor: not-allowed;
  }

  .submit-btn:not(:disabled):hover {
    background: #1d4ed8;
  }
</style>
