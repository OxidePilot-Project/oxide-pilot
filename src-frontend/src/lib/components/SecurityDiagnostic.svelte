<script lang="ts">
  import { invoke } from '@tauri-apps/api/tauri';
  import { onMount } from 'svelte';

  interface ProcessInfo {
    name: string;
    pid: string;
    cpu_usage: number;
    memory_mb: number;
    command: string;
    is_suspicious: boolean;
    suspicion_reasons: string[];
  }

  interface SystemInfo {
    cpu_usage: number;
    memory_used_gb: number;
    memory_total_gb: number;
    memory_percent: number;
  }

  interface NetworkInfo {
    total_bytes_received: number;
    total_bytes_transmitted: number;
    active_interfaces: number;
  }

  interface SecurityDiagnosticReport {
    timestamp: string;
    system_info: SystemInfo;
    total_processes: number;
    high_cpu_processes: ProcessInfo[];
    suspicious_processes: ProcessInfo[];
    network_info: NetworkInfo;
    threat_level: 'Clean' | 'Low' | 'Medium' | 'High' | 'Critical';
    threat_score: number;
    recommendations: string[];
  }

  let scanning = false;
  let report: SecurityDiagnosticReport | null = null;
  let error: string | null = null;
  let lastScanTime: string | null = null;

  // Threat level colors and icons
  const threatLevelConfig = {
    Clean: { color: 'text-green-600', bgColor: 'bg-green-100', icon: '✅', label: 'Limpio' },
    Low: { color: 'text-blue-600', bgColor: 'bg-blue-100', icon: 'ℹ️', label: 'Bajo' },
    Medium: { color: 'text-yellow-600', bgColor: 'bg-yellow-100', icon: '⚠️', label: 'Medio' },
    High: { color: 'text-orange-600', bgColor: 'bg-orange-100', icon: '🔶', label: 'Alto' },
    Critical: { color: 'text-red-600', bgColor: 'bg-red-100', icon: '🚨', label: 'Crítico' },
  };

  async function runDiagnostic() {
    scanning = true;
    error = null;

    try {
      report = await invoke<SecurityDiagnosticReport>('run_security_diagnostic');
      lastScanTime = new Date(report.timestamp).toLocaleString('es-ES', {
        dateStyle: 'medium',
        timeStyle: 'medium'
      });
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
      console.error('Error running diagnostic:', err);
    } finally {
      scanning = false;
    }
  }

  async function loadLastScan() {
    try {
      const lastReport = await invoke<SecurityDiagnosticReport | null>('get_last_security_scan');
      if (lastReport) {
        report = lastReport;
        lastScanTime = new Date(lastReport.timestamp).toLocaleString('es-ES', {
          dateStyle: 'medium',
          timeStyle: 'medium'
        });
      }
    } catch (err) {
      console.error('Error loading last scan:', err);
    }
  }

  onMount(() => {
    loadLastScan();
  });

  function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  }
</script>

<div class="security-diagnostic p-6 max-w-7xl mx-auto">
  <div class="header mb-6">
    <h1 class="text-3xl font-bold text-gray-800 mb-2">
      🛡️ Diagnóstico de Seguridad del Sistema
    </h1>
    <p class="text-gray-600">
      Análisis exhaustivo para detectar amenazas y actividades sospechosas
    </p>
  </div>

  <div class="actions mb-6">
    <button
      on:click={runDiagnostic}
      disabled={scanning}
      class="px-6 py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:bg-gray-400 disabled:cursor-not-allowed transition-colors font-semibold shadow-md"
    >
      {scanning ? '🔄 Analizando...' : '🔍 Iniciar Análisis'}
    </button>
    {#if lastScanTime}
      <span class="ml-4 text-sm text-gray-600">
        Último análisis: {lastScanTime}
      </span>
    {/if}
  </div>

  {#if error}
    <div class="error bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded-lg mb-6">
      <strong>Error:</strong> {error}
    </div>
  {/if}

  {#if report}
    <div class="report space-y-6">
      <!-- Threat Level Card -->
      <div class="threat-level-card bg-white rounded-lg shadow-lg p-6 border-l-4 {threatLevelConfig[report.threat_level].bgColor.replace('bg-', 'border-')}">
        <div class="flex items-center justify-between">
          <div>
            <h2 class="text-2xl font-bold {threatLevelConfig[report.threat_level].color}">
              {threatLevelConfig[report.threat_level].icon} Nivel de Amenaza: {threatLevelConfig[report.threat_level].label}
            </h2>
            <p class="text-gray-600 mt-1">Puntuación: {report.threat_score}/4</p>
          </div>
          <div class="text-6xl">
            {threatLevelConfig[report.threat_level].icon}
          </div>
        </div>
      </div>

      <!-- System Info Grid -->
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <div class="stat-card bg-white rounded-lg shadow p-4">
          <div class="text-gray-600 text-sm font-medium mb-1">Uso de CPU</div>
          <div class="text-2xl font-bold text-blue-600">{report.system_info.cpu_usage.toFixed(1)}%</div>
        </div>
        <div class="stat-card bg-white rounded-lg shadow p-4">
          <div class="text-gray-600 text-sm font-medium mb-1">Memoria RAM</div>
          <div class="text-2xl font-bold text-purple-600">
            {report.system_info.memory_used_gb.toFixed(1)} / {report.system_info.memory_total_gb.toFixed(1)} GB
          </div>
          <div class="text-sm text-gray-500">{report.system_info.memory_percent.toFixed(1)}%</div>
        </div>
        <div class="stat-card bg-white rounded-lg shadow p-4">
          <div class="text-gray-600 text-sm font-medium mb-1">Procesos Totales</div>
          <div class="text-2xl font-bold text-green-600">{report.total_processes}</div>
        </div>
        <div class="stat-card bg-white rounded-lg shadow p-4">
          <div class="text-gray-600 text-sm font-medium mb-1">Interfaces de Red</div>
          <div class="text-2xl font-bold text-teal-600">{report.network_info.active_interfaces}</div>
        </div>
      </div>

      <!-- Suspicious Processes Alert -->
      {#if report.suspicious_processes.length > 0}
        <div class="suspicious-alert bg-red-50 border-2 border-red-400 rounded-lg p-6">
          <h3 class="text-xl font-bold text-red-700 mb-4">
            🚨 Procesos Sospechosos Detectados ({report.suspicious_processes.length})
          </h3>
          <div class="space-y-3">
            {#each report.suspicious_processes as proc}
              <div class="bg-white rounded p-4 border border-red-300">
                <div class="flex justify-between items-start mb-2">
                  <div>
                    <span class="font-bold text-red-700">{proc.name}</span>
                    <span class="text-gray-600 text-sm ml-2">PID: {proc.pid}</span>
                  </div>
                  <div class="text-right">
                    <div class="text-sm text-gray-600">CPU: {proc.cpu_usage.toFixed(1)}%</div>
                    <div class="text-sm text-gray-600">RAM: {proc.memory_mb.toFixed(0)} MB</div>
                  </div>
                </div>
                <div class="text-sm text-gray-700 mb-2 font-mono break-all">
                  {proc.command.substring(0, 100)}{proc.command.length > 100 ? '...' : ''}
                </div>
                <div class="text-sm text-red-600">
                  <strong>Razones:</strong>
                  <ul class="list-disc list-inside mt-1">
                    {#each proc.suspicion_reasons as reason}
                      <li>{reason}</li>
                    {/each}
                  </ul>
                </div>
              </div>
            {/each}
          </div>
        </div>
      {/if}

      <!-- High CPU Processes -->
      {#if report.high_cpu_processes.length > 0}
        <div class="high-cpu-card bg-white rounded-lg shadow-lg p-6">
          <h3 class="text-xl font-bold text-gray-800 mb-4">
            ⚡ Procesos con Alto Uso de CPU ({report.high_cpu_processes.length})
          </h3>
          <div class="overflow-x-auto">
            <table class="w-full text-sm">
              <thead class="bg-gray-100">
                <tr>
                  <th class="px-4 py-2 text-left">Proceso</th>
                  <th class="px-4 py-2 text-left">PID</th>
                  <th class="px-4 py-2 text-right">CPU %</th>
                  <th class="px-4 py-2 text-right">RAM (MB)</th>
                  <th class="px-4 py-2 text-left">Estado</th>
                </tr>
              </thead>
              <tbody>
                {#each report.high_cpu_processes.slice(0, 10) as proc}
                  <tr class="border-b hover:bg-gray-50">
                    <td class="px-4 py-2 font-medium">{proc.name}</td>
                    <td class="px-4 py-2 text-gray-600">{proc.pid}</td>
                    <td class="px-4 py-2 text-right font-bold text-orange-600">{proc.cpu_usage.toFixed(1)}%</td>
                    <td class="px-4 py-2 text-right">{proc.memory_mb.toFixed(0)}</td>
                    <td class="px-4 py-2">
                      {#if proc.is_suspicious}
                        <span class="text-red-600 font-bold">⚠️ Sospechoso</span>
                      {:else}
                        <span class="text-green-600">✓ Normal</span>
                      {/if}
                    </td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        </div>
      {/if}

      <!-- Network Activity -->
      <div class="network-card bg-white rounded-lg shadow-lg p-6">
        <h3 class="text-xl font-bold text-gray-800 mb-4">🌐 Actividad de Red</h3>
        <div class="grid grid-cols-2 gap-4">
          <div>
            <div class="text-gray-600 text-sm mb-1">Datos Recibidos</div>
            <div class="text-lg font-bold text-blue-600">
              {formatBytes(report.network_info.total_bytes_received)}
            </div>
          </div>
          <div>
            <div class="text-gray-600 text-sm mb-1">Datos Transmitidos</div>
            <div class="text-lg font-bold text-green-600">
              {formatBytes(report.network_info.total_bytes_transmitted)}
            </div>
          </div>
        </div>
      </div>

      <!-- Recommendations -->
      <div class="recommendations-card bg-white rounded-lg shadow-lg p-6">
        <h3 class="text-xl font-bold text-gray-800 mb-4">💡 Recomendaciones</h3>
        <ul class="space-y-2">
          {#each report.recommendations as rec}
            <li class="flex items-start">
              <span class="text-blue-500 mr-2">•</span>
              <span class="text-gray-700">{rec}</span>
            </li>
          {/each}
        </ul>
      </div>
    </div>
  {/if}

  {#if !report && !scanning && !error}
    <div class="empty-state bg-gray-50 rounded-lg p-12 text-center">
      <div class="text-6xl mb-4">🔒</div>
      <h3 class="text-xl font-bold text-gray-700 mb-2">No hay análisis disponibles</h3>
      <p class="text-gray-600 mb-6">Haz clic en "Iniciar Análisis" para ejecutar un diagnóstico de seguridad completo</p>
    </div>
  {/if}
</div>

<style>
  .security-diagnostic {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
  }

  .stat-card {
    transition: transform 0.2s, box-shadow 0.2s;
  }

  .stat-card:hover {
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  }

  table {
    border-collapse: collapse;
  }

  thead {
    position: sticky;
    top: 0;
  }
</style>
