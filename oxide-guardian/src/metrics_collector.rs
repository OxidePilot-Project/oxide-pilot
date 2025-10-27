//! System Metrics Collector for Oxide Guardian
//!
//! This module continuously collects system performance metrics and stores them in SurrealDB
//! for historical analysis, anomaly detection, and incident correlation.
//!
//! # Metrics Collected
//! - CPU usage (per-core and aggregate)
//! - Memory usage (total, used, available, swap)
//! - Disk I/O (read/write throughput, IOPS)
//! - Network statistics (sent/received, active connections)
//! - Process graph (parent-child relationships)
//!
//! # Collection Interval
//! Default: 5 seconds (configurable)
//!
//! # Storage
//! All metrics are stored in SurrealDB with timestamp indexing for efficient time-range queries.

use anyhow::{Context, Result};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use sysinfo::{CpuExt, NetworkExt, ProcessExt, System, SystemExt};
use tokio::sync::RwLock;
use tokio::time::interval;
use tracing::{debug, error, info, warn};

#[cfg(feature = "surrealdb-metrics")]
use oxide_memory::{
    AgentMemory, AgentType, DiskIO, MemorySource, MemoryUsage, NetworkStats, ProcessInfo,
    ProcessStatus, SurrealBackend, SystemMetric,
};

/// Configuration for metrics collector
#[derive(Debug, Clone)]
pub struct MetricsConfig {
    /// Collection interval in seconds
    pub interval_secs: u64,
    /// Enable process tree collection
    pub collect_processes: bool,
    /// CPU threshold for high-usage alerts (percentage)
    pub cpu_alert_threshold: f64,
    /// Memory threshold for high-usage alerts (percentage)
    pub memory_alert_threshold: f64,
    /// Enable disk I/O collection
    pub collect_disk_io: bool,
    /// Enable network statistics collection
    pub collect_network: bool,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            interval_secs: 5,
            collect_processes: true,
            cpu_alert_threshold: 90.0,
            memory_alert_threshold: 90.0,
            collect_disk_io: true,
            collect_network: true,
        }
    }
}

/// System metrics collector for Guardian Agent
///
/// Continuously monitors system performance and stores metrics in SurrealDB.
///
/// # Example
///
/// ```rust,no_run
/// use oxide_guardian::MetricsCollector;
/// use oxide_memory::SurrealBackend;
/// use std::sync::Arc;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let backend = Arc::new(SurrealBackend::new("./data/oxide.db").await?);
///     let mut collector = MetricsCollector::new(backend, Default::default());
///     
///     // Start background collection (runs forever)
///     collector.start().await?;
///     
///     Ok(())
/// }
/// ```
#[cfg(feature = "surrealdb-metrics")]
pub struct MetricsCollector {
    /// SurrealDB backend
    backend: Arc<SurrealBackend>,
    /// System info collector
    system: Arc<RwLock<System>>,
    /// Configuration
    config: MetricsConfig,
    /// Process ID mapping (PID -> last seen timestamp)
    process_map: Arc<RwLock<HashMap<i32, chrono::DateTime<Utc>>>>,
}

#[cfg(feature = "surrealdb-metrics")]
impl MetricsCollector {
    /// Create new metrics collector
    ///
    /// # Arguments
    /// * `backend` - SurrealDB backend for storage
    /// * `config` - Collector configuration
    pub fn new(backend: Arc<SurrealBackend>, config: MetricsConfig) -> Self {
        info!("Initializing metrics collector with interval={}s", config.interval_secs);
        
        let mut system = System::new_all();
        system.refresh_all();
        
        Self {
            backend,
            system: Arc::new(RwLock::new(system)),
            config,
            process_map: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start metrics collection loop (runs forever)
    ///
    /// This is the main entry point for the collector. It will run indefinitely,
    /// collecting and storing metrics at the configured interval.
    ///
    /// # Errors
    /// Returns error if database operations fail. Individual collection errors
    /// are logged but don't stop the loop.
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting metrics collection loop");
        let mut ticker = interval(Duration::from_secs(self.config.interval_secs));
        
        loop {
            ticker.tick().await;
            
            if let Err(e) = self.collect_and_store().await {
                error!("Failed to collect metrics: {:#}", e);
                // Continue loop despite errors
            }
        }
    }

    /// Collect all metrics and store in database
    async fn collect_and_store(&mut self) -> Result<()> {
        let timestamp = Utc::now();
        debug!("Collecting metrics at {}", timestamp);
        
        // Refresh system info
        {
            let mut sys = self.system.write().await;
            sys.refresh_all();
        }
        
        // Collect system-level metrics
        let metric = self.collect_system_metrics(timestamp).await?;
        
        // Store in database
        self.backend
            .insert_system_metric(metric.clone())
            .await
            .context("Failed to store system metric")?;
        
        // Check for alerts
        self.check_alerts(&metric).await;
        
        // Optionally collect process tree
        if self.config.collect_processes {
            if let Err(e) = self.collect_process_tree().await {
                warn!("Failed to collect process tree: {:#}", e);
            }
        }
        
        debug!("Metrics collection completed successfully");
        Ok(())
    }

    /// Collect system-level performance metrics
    async fn collect_system_metrics(&self, timestamp: chrono::DateTime<Utc>) -> Result<SystemMetric> {
        let sys = self.system.read().await;
        
        // CPU usage (global average)
        let cpu_usage = sys.global_cpu_info().cpu_usage() as f64;
        
        // Memory usage
        let total_memory = sys.total_memory();
        let used_memory = sys.used_memory();
        let available_memory = sys.available_memory();
        let memory_percent = (used_memory as f64 / total_memory as f64) * 100.0;
        
        let memory_usage = MemoryUsage {
            total_mb: total_memory as f64 / 1024.0 / 1024.0,
            used_mb: used_memory as f64 / 1024.0 / 1024.0,
            available_mb: available_memory as f64 / 1024.0 / 1024.0,
            percent: memory_percent,
        };
        
        // Disk I/O (aggregate across all disks)
        let disk_io = if self.config.collect_disk_io {
            self.collect_disk_io_stats(&sys).await
        } else {
            DiskIO {
                read_mb_per_sec: 0.0,
                write_mb_per_sec: 0.0,
                iops: 0,
            }
        };
        
        // Network statistics
        let network_stats = if self.config.collect_network {
            self.collect_network_stats(&sys).await
        } else {
            NetworkStats {
                sent_mb_per_sec: 0.0,
                recv_mb_per_sec: 0.0,
                connections_active: 0,
            }
        };
        
        // Metadata
        let metadata = Some(serde_json::json!({
            "hostname": hostname::get()
                .ok()
                .and_then(|s| s.into_string().ok())
                .unwrap_or_else(|| "unknown".to_string()),
            "os": sys.name().unwrap_or_else(|| "unknown".to_string()),
            "os_version": sys.os_version().unwrap_or_else(|| "unknown".to_string()),
            "kernel_version": sys.kernel_version().unwrap_or_else(|| "unknown".to_string()),
            "oxide_version": env!("CARGO_PKG_VERSION"),
        }));
        
        Ok(SystemMetric {
            timestamp,
            cpu_usage,
            memory_usage,
            disk_io,
            network_stats,
            metadata,
        })
    }

    /// Collect disk I/O statistics
    async fn collect_disk_io_stats(&self, sys: &System) -> DiskIO {
        // Note: sysinfo 0.29 doesn't provide real-time I/O rates
        // For production, consider using Windows Performance Counters or Linux /proc/diskstats
        // For now, returning placeholder values
        
        // TODO: Implement actual disk I/O monitoring
        // Windows: Use Performance Data Helper (PDH) API
        // Linux: Parse /proc/diskstats
        
        DiskIO {
            read_mb_per_sec: 0.0,
            write_mb_per_sec: 0.0,
            iops: 0,
        }
    }

    /// Collect network statistics
    async fn collect_network_stats(&self, sys: &System) -> NetworkStats {
        let networks = sys.networks();
        
        let mut total_sent = 0u64;
        let mut total_recv = 0u64;
        
        for (_name, network) in networks.iter() {
            total_sent += network.transmitted();
            total_recv += network.received();
        }
        
        // Convert bytes to MB (divide by sample interval for per-second rate)
        let sent_mb_per_sec = (total_sent as f64) / 1024.0 / 1024.0 / (self.config.interval_secs as f64);
        let recv_mb_per_sec = (total_recv as f64) / 1024.0 / 1024.0 / (self.config.interval_secs as f64);
        
        // Note: Connection count requires OS-specific API
        // TODO: Implement TCP connection counting
        // Windows: GetExtendedTcpTable
        // Linux: Parse /proc/net/tcp
        
        NetworkStats {
            sent_mb_per_sec,
            recv_mb_per_sec,
            connections_active: 0,
        }
    }

    /// Collect process tree and store in graph database
    async fn collect_process_tree(&self) -> Result<()> {
        let sys = self.system.read().await;
        let mut process_map = self.process_map.write().await;
        let now = Utc::now();
        
        debug!("Collecting process tree ({} processes)", sys.processes().len());
        
        for (pid, process) in sys.processes() {
            let pid_i32 = pid.as_u32() as i32;
            
            // Skip if we've seen this process recently (avoid duplicates)
            if let Some(last_seen) = process_map.get(&pid_i32) {
                if now.signed_duration_since(*last_seen).num_seconds() < 60 {
                    continue;
                }
            }
            
            // Create ProcessInfo
            let process_info = ProcessInfo {
                pid: pid_i32,
                name: process.name().to_string(),
                exe_path: process.exe().map(|p| p.display().to_string()),
                cmd: process.cmd().to_vec(),
                start_time: chrono::DateTime::from_timestamp(process.start_time() as i64, 0)
                    .unwrap_or(now),
                end_time: None,
                cpu_percent: process.cpu_usage() as f64,
                memory_mb: (process.memory() as f64) / 1024.0 / 1024.0,
                threads: 1, // sysinfo doesn't expose thread count directly
                status: self.map_process_status(process.status()),
            };
            
            // Store process (TODO: implement process storage in SurrealBackend)
            // For now, just track in memory
            process_map.insert(pid_i32, now);
            
            // TODO: Create graph edges for parent-child relationships
            // if let Some(parent_pid) = process.parent() {
            //     self.backend.create_spawns_relation(parent_pid, pid_i32).await?;
            // }
        }
        
        Ok(())
    }

    /// Map sysinfo ProcessStatus to our enum
    fn map_process_status(&self, status: sysinfo::ProcessStatus) -> ProcessStatus {
        match status {
            sysinfo::ProcessStatus::Run => ProcessStatus::Running,
            sysinfo::ProcessStatus::Sleep => ProcessStatus::Sleeping,
            sysinfo::ProcessStatus::Stop => ProcessStatus::Stopped,
            sysinfo::ProcessStatus::Zombie => ProcessStatus::Zombie,
            _ => ProcessStatus::Sleeping, // Default fallback
        }
    }

    /// Check for alert conditions and create agent memories
    async fn check_alerts(&self, metric: &SystemMetric) {
        // High CPU alert
        if metric.cpu_usage > self.config.cpu_alert_threshold {
            warn!(
                "High CPU usage detected: {:.2}% (threshold: {:.2}%)",
                metric.cpu_usage, self.config.cpu_alert_threshold
            );
            
            // Create agent memory for future analysis
            if let Err(e) = self.create_alert_memory(
                &format!("High CPU usage: {:.2}%", metric.cpu_usage),
                metric.timestamp,
            ).await {
                error!("Failed to create alert memory: {:#}", e);
            }
        }
        
        // High memory alert
        if metric.memory_usage.percent > self.config.memory_alert_threshold {
            warn!(
                "High memory usage detected: {:.2}% (threshold: {:.2}%)",
                metric.memory_usage.percent, self.config.memory_alert_threshold
            );
            
            if let Err(e) = self.create_alert_memory(
                &format!("High memory usage: {:.2}%", metric.memory_usage.percent),
                metric.timestamp,
            ).await {
                error!("Failed to create alert memory: {:#}", e);
            }
        }
    }

    /// Create agent memory for alert
    async fn create_alert_memory(&self, content: &str, timestamp: chrono::DateTime<Utc>) -> Result<()> {
        // TODO: Generate real embeddings
        let embedding = vec![0.0; 1536];
        
        let memory = AgentMemory {
            agent_type: AgentType::Guardian,
            content: content.to_string(),
            embedding,
            timestamp,
            source: MemorySource::PerformanceAnalysis,
            metadata: Some(serde_json::json!({
                "alert_type": "performance",
                "auto_generated": true,
            })),
        };
        
        self.backend.insert_agent_memory(memory).await?;
        Ok(())
    }
}

// Stub implementation when surrealdb feature is disabled
#[cfg(not(feature = "surrealdb-metrics"))]
pub struct MetricsCollector;

#[cfg(not(feature = "surrealdb-metrics"))]
impl MetricsCollector {
    pub fn new(_backend: (), _config: MetricsConfig) -> Self {
        Self
    }
    
    pub async fn start(&mut self) -> Result<()> {
        warn!("MetricsCollector disabled (surrealdb-metrics feature not enabled)");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    #[cfg(feature = "surrealdb-metrics")]
    async fn test_metrics_collection() {
        let temp_dir = TempDir::new().unwrap();
        let backend = Arc::new(
            SurrealBackend::new(temp_dir.path().join("test.db"))
                .await
                .unwrap()
        );
        
        let config = MetricsConfig {
            interval_secs: 1,
            collect_processes: false, // Disable for test
            ..Default::default()
        };
        
        let mut collector = MetricsCollector::new(backend.clone(), config);
        
        // Collect once
        collector.collect_and_store().await.unwrap();
        
        // Query metrics
        let metrics = backend
            .query_metrics_by_time(
                Utc::now() - chrono::Duration::hours(1),
                Utc::now() + chrono::Duration::hours(1),
            )
            .await
            .unwrap();
        
        assert!(metrics.len() > 0);
        assert!(metrics[0].cpu_usage >= 0.0 && metrics[0].cpu_usage <= 100.0);
    }
}
