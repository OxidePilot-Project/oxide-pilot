<script lang="ts">
import { onDestroy, onMount } from "svelte";
import { writable } from "svelte/store";
import { isTauri } from "$lib/utils/env";

// Lazy-load Tauri invoke to avoid SSR importing '@tauri-apps/api/tauri'
type InvokeFn = <T = any>(cmd: string, args?: Record<string, unknown>) => Promise<T>;
let invokeFn: InvokeFn | null = null;
async function tauriInvoke<T = any>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (!isTauri) throw new Error("Not running in Tauri context");
  if (!invokeFn) {
    const mod = await import("@tauri-apps/api/tauri");
    invokeFn = mod.invoke as InvokeFn;
  }
  return invokeFn<T>(cmd, args);
}

const isRecording = writable(false);
const inputVolume = writable(0);
const audioDevices = writable<{ input: string[]; output: string[] }>({
  input: [],
  output: [],
});
const recordedAudio = writable<Uint8Array | null>(null);

let volumeInterval: number;
const recordingDuration = 3.0;

onMount(async () => {
  if (!isTauri) {
    // In web/SSR, don't attempt to access Tauri audio APIs
    return;
  }
  await loadAudioDevices();
  startVolumeMonitoring();
});

onDestroy(() => {
  if (volumeInterval) {
    clearInterval(volumeInterval);
  }
});

async function loadAudioDevices() {
  try {
    const devices = (await tauriInvoke("get_audio_devices")) as [string[], string[]];
    audioDevices.set({
      input: devices[0],
      output: devices[1],
    });
  } catch (error) {
    console.error("Failed to load audio devices:", error);
  }
}

function startVolumeMonitoring() {
  volumeInterval = setInterval(async () => {
    try {
      const volume = (await tauriInvoke("get_input_volume")) as number;
      inputVolume.set(volume);
    } catch (_error) {
      // Silently handle errors to avoid spam
    }
  }, 100);
}

async function startRecording() {
  try {
    isRecording.set(true);
    console.log(`Recording for ${recordingDuration} seconds...`);

    const audioData = (await tauriInvoke("record_audio", {
      duration_secs: recordingDuration,
    })) as number[];

    recordedAudio.set(new Uint8Array(audioData));
    console.log(`Recorded ${audioData.length} bytes of audio`);
  } catch (error) {
    console.error("Recording failed:", error);
    alert(`Recording failed: ${error}`);
  } finally {
    isRecording.set(false);
  }
}

async function playRecording() {
  const audio = $recordedAudio;
  if (!audio) {
    alert("No recording to play");
    return;
  }

  try {
    await tauriInvoke("play_audio", { audio_data: Array.from(audio) });
    console.log("Audio playback completed");
  } catch (error) {
    console.error("Playback failed:", error);
    alert(`Playback failed: ${error}`);
  }
}

function clearRecording() {
  recordedAudio.set(null);
}

function getVolumeBarWidth(volume: number): number {
  return Math.min(volume * 1000, 100); // Scale and cap at 100%
}

function getVolumeColor(volume: number): string {
  if (volume > 0.1) return "#e74c3c"; // Red for loud
  if (volume > 0.05) return "#f39c12"; // Orange for medium
  if (volume > 0.01) return "#27ae60"; // Green for quiet
  return "#95a5a6"; // Gray for silence
}
</script>

<div class="audio-controls">
  <h3>🎤 Audio Controls</h3>

  <!-- Volume Monitor -->
  <div class="volume-monitor">
    <label>Input Volume:</label>
    <div class="volume-bar">
      <div
        class="volume-level"
        style="width: {getVolumeBarWidth($inputVolume)}%; background-color: {getVolumeColor($inputVolume)}"
      ></div>
    </div>
    <span class="volume-text">{($inputVolume * 100).toFixed(1)}%</span>
  </div>

  <!-- Recording Controls -->
  <div class="recording-section">
    <div class="duration-control">
      <label for="duration">Recording Duration:</label>
      <input
        type="range"
        id="duration"
        min="1"
        max="10"
        step="0.5"
        bind:value={recordingDuration}
        disabled={$isRecording}
      />
      <span>{recordingDuration}s</span>
    </div>

    <div class="recording-buttons">
      <button
        class="record-button"
        class:recording={$isRecording}
        on:click={startRecording}
        disabled={$isRecording}
      >
        {#if $isRecording}
          🔴 Recording...
        {:else}
          🎤 Record Audio
        {/if}
      </button>

      {#if $recordedAudio}
        <button class="play-button" on:click={playRecording}>
          🔊 Play Recording
        </button>
        <button class="clear-button" on:click={clearRecording}>
          🗑️ Clear
        </button>
      {/if}
    </div>

    {#if $recordedAudio}
      <div class="recording-info">
        ✅ Recording ready ({$recordedAudio.length} bytes)
      </div>
    {/if}
  </div>

  <!-- Audio Devices -->
  <div class="devices-section">
    <h4>Audio Devices</h4>

    <div class="device-list">
      <div class="device-category">
        <strong>Input Devices ({$audioDevices.input.length}):</strong>
        {#if $audioDevices.input.length === 0}
          <p class="no-devices">No input devices found</p>
        {:else}
          <ul>
            {#each $audioDevices.input as device}
              <li>🎤 {device}</li>
            {/each}
          </ul>
        {/if}
      </div>

      <div class="device-category">
        <strong>Output Devices ({$audioDevices.output.length}):</strong>
        {#if $audioDevices.output.length === 0}
          <p class="no-devices">No output devices found</p>
        {:else}
          <ul>
            {#each $audioDevices.output as device}
              <li>🔊 {device}</li>
            {/each}
          </ul>
        {/if}
      </div>
    </div>
  </div>
</div>

<style>
  .audio-controls {
    background: white;
    border-radius: 12px;
    padding: 20px;
    margin: 20px 0;
    box-shadow: 0 4px 15px rgba(0, 0, 0, 0.1);
  }

  h3 {
    color: #2c3e50;
    margin: 0 0 20px 0;
    font-size: 20px;
  }

  h4 {
    color: #34495e;
    margin: 15px 0 10px 0;
    font-size: 16px;
  }

  .volume-monitor {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-bottom: 20px;
    padding: 15px;
    background: #f8f9fa;
    border-radius: 8px;
  }

  .volume-bar {
    flex: 1;
    height: 20px;
    background: #ecf0f1;
    border-radius: 10px;
    overflow: hidden;
    position: relative;
  }

  .volume-level {
    height: 100%;
    transition: width 0.1s ease, background-color 0.3s ease;
    border-radius: 10px;
  }

  .volume-text {
    font-weight: 600;
    min-width: 50px;
    text-align: right;
    color: #2c3e50;
  }

  .recording-section {
    margin-bottom: 20px;
  }

  .duration-control {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-bottom: 15px;
  }

  .duration-control input[type="range"] {
    flex: 1;
  }

  .recording-buttons {
    display: flex;
    gap: 10px;
    flex-wrap: wrap;
  }

  .record-button, .play-button, .clear-button {
    padding: 12px 20px;
    border: none;
    border-radius: 8px;
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .record-button {
    background: #e74c3c;
    color: white;
  }

  .record-button:hover:not(:disabled) {
    background: #c0392b;
    transform: translateY(-1px);
  }

  .record-button.recording {
    background: #e74c3c;
    animation: pulse 1s infinite;
  }

  .play-button {
    background: #27ae60;
    color: white;
  }

  .play-button:hover {
    background: #229954;
    transform: translateY(-1px);
  }

  .clear-button {
    background: #95a5a6;
    color: white;
  }

  .clear-button:hover {
    background: #7f8c8d;
    transform: translateY(-1px);
  }

  .record-button:disabled {
    background: #bdc3c7;
    cursor: not-allowed;
    transform: none;
  }

  .recording-info {
    margin-top: 10px;
    padding: 10px;
    background: #d5f4e6;
    color: #27ae60;
    border-radius: 6px;
    font-weight: 500;
  }

  .devices-section {
    border-top: 1px solid #ecf0f1;
    padding-top: 20px;
  }

  .device-list {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 20px;
  }

  .device-category ul {
    margin: 5px 0;
    padding-left: 20px;
  }

  .device-category li {
    margin: 5px 0;
    color: #2c3e50;
  }

  .no-devices {
    color: #7f8c8d;
    font-style: italic;
    margin: 5px 0;
  }

  @keyframes pulse {
    0% { opacity: 1; }
    50% { opacity: 0.7; }
    100% { opacity: 1; }
  }

  @media (max-width: 600px) {
    .device-list {
      grid-template-columns: 1fr;
    }

    .volume-monitor {
      flex-direction: column;
      align-items: stretch;
    }

    .recording-buttons {
      flex-direction: column;
    }
  }
</style>