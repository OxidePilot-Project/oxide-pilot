<script lang="ts">
import { onMount } from "svelte";
import { systemConfig } from "$lib/stores/systemConfig";
import { isTauri } from "$lib/utils/env";

import { tauriInvoke } from "$lib/utils/tauri";

let googleApiKey = "";
let isLoading = false;
let error = "";
let showSetup = false;

onMount(async () => {
  if (!isTauri) {
    // In browser/SSR, show setup UI but skip invoking backend
    showSetup = true;
    return;
  }
  try {
    const config = await tauriInvoke<any>("get_system_config");
    if (!config.ai_providers?.google?.api_key) {
      showSetup = true;
    }
  } catch (err) {
    console.error("Error checking config:", err);
    showSetup = true;
  }
});

async function saveGoogleApiKey() {
  if (!googleApiKey.trim()) {
    error = "Por favor ingresa tu clave API de Google Gemini";
    return;
  }

  isLoading = true;
  error = "";

  try {
    const currentConfig = await tauriInvoke<any>("get_system_config");
    const updatedConfig = {
      ...currentConfig,
      ai_providers: {
        ...currentConfig.ai_providers,
        google: {
          api_key: googleApiKey.trim(),
          model: "gemini-1.5-pro",
          max_tokens: 8192,
          temperature: 0.7,
        },
      },
    };

    await tauriInvoke("update_system_config", { config: updatedConfig });
    systemConfig.set(updatedConfig);
    showSetup = false;
  } catch (err) {
    error = `Error al guardar la configuración: ${err}`;
  } finally {
    isLoading = false;
  }
}
</script>

{#if showSetup}
  <div class="fixed inset-0 bg-gray-900 bg-opacity-50 flex items-center justify-center z-50">
    <div class="bg-white rounded-lg p-8 max-w-md w-full mx-4">
      <h2 class="text-2xl font-bold mb-4 text-gray-800">Configuración Inicial de Oxide Pilot</h2>
      <p class="text-gray-600 mb-6">
        Para comenzar, necesitas configurar tu clave API de Google Gemini. Esta clave se usará para todas las funciones de IA del sistema.
      </p>

      <div class="mb-4">
        <label for="google-api-key" class="block text-sm font-medium text-gray-700 mb-2">
          Clave API de Google Gemini
        </label>
        <input
          id="google-api-key"
          type="password"
          bind:value={googleApiKey}
          placeholder="Ingresa tu clave API aquí..."
          class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
        />
      </div>

      {#if error}
        <div class="mb-4 p-3 bg-red-100 border border-red-400 text-red-700 rounded">
          {error}
        </div>
      {/if}

      <button
        on:click={saveGoogleApiKey}
        disabled={isLoading}
        class="w-full bg-blue-600 text-white py-2 px-4 rounded-md hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed"
      >
        {isLoading ? 'Guardando...' : 'Guardar y Continuar'}
      </button>

      <p class="text-xs text-gray-500 mt-4">
        Puedes obtener tu clave API desde: <a href="https://makersuite.google.com/app/apikey" target="_blank" class="text-blue-600 hover:underline">Google AI Studio</a>
      </p>
    </div>
  </div>
{/if}
