import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/tauri";

export interface SystemMetric {
  timestamp: string;
  cpu_usage: number;
  memory_usage: {
    total_mb: number;
    used_mb: number;
    available_mb: number;
    percent: number;
  };
  disk_io: {
    read_mb_per_sec: number;
    write_mb_per_sec: number;
    iops: number;
  };
  network_stats: {
    sent_mb_per_sec: number;
    recv_mb_per_sec: number;
    connections_active: number;
  };
  metadata?: Record<string, unknown>;
}

export interface MetricsSummary {
  avg_cpu: number;
  max_cpu: number;
  avg_memory_percent: number;
  max_memory_percent: number;
  sample_count: number;
  window_start?: string;
  window_end?: string;
}

export interface HourlyMetricsRow {
  avg_cpu: number;
  peak_cpu: number;
  avg_mem_percent: number;
  hour_bucket: string;
  samples: number;
}

export interface ProcessHotspot {
  name: string;
  avg_cpu: number;
  peak_cpu: number;
  avg_memory_mb: number;
  samples: number;
}

export interface ThreatTrainingSample {
  severity: string;
  cpu_usage: number;
  memory_pressure: number;
  network_score: number;
  anomaly_score: number;
  metadata?: Record<string, unknown>;
}

export interface ThreatPrediction {
  provider?: string;
  severity: string;
  score: number;
  confidence?: number;
}

export async function getGuardianStatus() {
  return invoke("get_guardian_status");
}

export async function getMetricsSummary(
  hours?: number,
): Promise<MetricsSummary> {
  return invoke("get_metrics_summary", { hours });
}

export async function getHourlyMetrics(
  hours?: number,
): Promise<HourlyMetricsRow[]> {
  return invoke("get_hourly_metrics", { hours });
}

export async function getRecentMetrics(hours: number) {
  return invoke<{ metrics: SystemMetric[]; count: number }>(
    "get_recent_metrics",
    { hours },
  );
}

export async function getProcessHotspots(
  hours?: number,
): Promise<ProcessHotspot[]> {
  return invoke("get_process_hotspots", { hours });
}

export async function predictThreatRisk(
  features: Record<string, number>,
): Promise<ThreatPrediction> {
  return invoke("predict_threat_risk", { featureVector: features });
}

export async function submitThreatTrainingSample(
  sample: ThreatTrainingSample,
): Promise<void> {
  return invoke("submit_threat_training_sample", { sample });
}

export async function subscribeGuardianMetrics(
  onMetric: (metric: SystemMetric) => void,
): Promise<() => void> {
  await invoke("subscribe_guardian_metrics");
  const unlisten = await listen<SystemMetric>("guardian://metrics", (event) => {
    onMetric(event.payload);
  });
  return () => {
    unlisten();
  };
}
