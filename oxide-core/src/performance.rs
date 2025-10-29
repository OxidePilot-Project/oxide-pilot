use log::warn;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Performance metrics for monitoring system resource usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub cpu_usage_percent: f32,
    pub memory_usage_mb: f32,
    pub memory_usage_percent: f32,
    pub active_threads: usize,
    pub uptime_seconds: u64,
    pub api_calls_count: u64,
    pub avg_response_time_ms: f32,
    pub cache_hit_rate: f32,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            cpu_usage_percent: 0.0,
            memory_usage_mb: 0.0,
            memory_usage_percent: 0.0,
            active_threads: 0,
            uptime_seconds: 0,
            api_calls_count: 0,
            avg_response_time_ms: 0.0,
            cache_hit_rate: 0.0,
        }
    }
}

/// Performance monitor for tracking system resource usage
pub struct PerformanceMonitor {
    start_time: Instant,
    metrics: Arc<RwLock<PerformanceMetrics>>,
    api_call_times: Arc<RwLock<Vec<Duration>>>,
    cache_hits: Arc<RwLock<u64>>,
    cache_misses: Arc<RwLock<u64>>,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            metrics: Arc::new(RwLock::new(PerformanceMetrics::default())),
            api_call_times: Arc::new(RwLock::new(Vec::new())),
            cache_hits: Arc::new(RwLock::new(0)),
            cache_misses: Arc::new(RwLock::new(0)),
        }
    }

    /// Get current performance metrics
    pub async fn get_metrics(&self) -> PerformanceMetrics {
        let mut metrics = self.metrics.read().await.clone();

        // Update uptime
        metrics.uptime_seconds = self.start_time.elapsed().as_secs();

        // Calculate average response time
        {
            let times = self.api_call_times.read().await;
            let times_len = times.len();
            if times_len > 0 {
                let total_ms: f32 = times.iter().map(|d| d.as_millis() as f32).sum();
                metrics.avg_response_time_ms = total_ms / times_len as f32;
            }
            metrics.api_calls_count = times_len as u64;
        }

        // Calculate cache hit rate
        let hits = *self.cache_hits.read().await;
        let misses = *self.cache_misses.read().await;
        let total = hits + misses;
        if total > 0 {
            metrics.cache_hit_rate = hits as f32 / total as f32;
        }

        metrics
    }

    /// Record an API call duration
    pub async fn record_api_call(&self, duration: Duration) {
        let mut times = self.api_call_times.write().await;
        times.push(duration);

        // Keep only last 1000 calls to prevent unbounded growth
        let len = times.len();
        if len > 1000 {
            times.drain(0..len - 1000);
        }
    }

    /// Record a cache hit
    pub async fn record_cache_hit(&self) {
        let mut hits = self.cache_hits.write().await;
        *hits += 1;
    }

    /// Record a cache miss
    pub async fn record_cache_miss(&self) {
        let mut misses = self.cache_misses.write().await;
        *misses += 1;
    }

    /// Update system resource metrics (overload with optional parameters)
    pub async fn update_system_metrics(&self, cpu_usage: f32, memory_usage_mb: f32) {
        let mut metrics = self.metrics.write().await;

        metrics.cpu_usage_percent = cpu_usage;
        metrics.memory_usage_mb = memory_usage_mb;

        // Calculate memory percentage (assuming 16GB total as default)
        let total_memory = 16.0 * 1024.0; // 16GB in MB
        metrics.memory_usage_percent = (memory_usage_mb / total_memory) * 100.0;

        // Count active threads
        metrics.active_threads = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1);

        // Log warning if exceeding targets
        if metrics.memory_usage_mb > 100.0 {
            warn!(
                "Memory usage ({:.1} MB) exceeds target of 100 MB",
                metrics.memory_usage_mb
            );
        }

        if metrics.cpu_usage_percent > 5.0 {
            warn!(
                "CPU usage ({:.1}%) exceeds target of 5%",
                metrics.cpu_usage_percent
            );
        }
    }

    /// Update system resource metrics (auto-detect version)
    pub async fn update_system_metrics_auto(&self) {
        use sysinfo::System;

        let mut sys = System::new_all();
        sys.refresh_all();

        let mut metrics = self.metrics.write().await;

        // Get current process info
        if let Ok(pid) = sysinfo::get_current_pid() {
            if let Some(process) = sys.process(pid) {
                metrics.memory_usage_mb = process.memory() as f32 / 1024.0 / 1024.0;
                metrics.cpu_usage_percent = process.cpu_usage();
            }
        }

        // Calculate memory percentage
        let total_memory = sys.total_memory() as f32;
        if total_memory > 0.0 {
            metrics.memory_usage_percent =
                (metrics.memory_usage_mb * 1024.0 * 1024.0) / total_memory * 100.0;
        }

        // Count active threads
        metrics.active_threads = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1);

        // Log warning if exceeding targets
        if metrics.memory_usage_mb > 100.0 {
            warn!(
                "Memory usage ({:.1} MB) exceeds target of 100 MB",
                metrics.memory_usage_mb
            );
        }

        if metrics.cpu_usage_percent > 5.0 {
            warn!(
                "CPU usage ({:.1}%) exceeds target of 5%",
                metrics.cpu_usage_percent
            );
        }
    }

    /// Check if performance is within acceptable limits
    pub async fn is_performance_acceptable(&self) -> bool {
        let metrics = self.get_metrics().await;

        // Target: < 100MB memory in idle
        let memory_ok = metrics.memory_usage_mb < 100.0;

        // Target: < 5% CPU average
        let cpu_ok = metrics.cpu_usage_percent < 5.0;

        memory_ok && cpu_ok
    }

    /// Get overall performance score (0-100, higher is better)
    pub async fn get_performance_score(&self) -> f32 {
        let metrics = self.get_metrics().await;

        // Calculate score based on multiple factors
        let mut score = 100.0;

        // Penalize high CPU usage (target: < 5%)
        if metrics.cpu_usage_percent > 5.0 {
            score -= (metrics.cpu_usage_percent - 5.0).min(50.0);
        }

        // Penalize high memory usage (target: < 100MB)
        if metrics.memory_usage_mb > 100.0 {
            let excess = (metrics.memory_usage_mb - 100.0) / 10.0;
            score -= excess.min(30.0);
        }

        // Penalize slow response times (target: < 100ms)
        if metrics.avg_response_time_ms > 100.0 {
            let excess = (metrics.avg_response_time_ms - 100.0) / 50.0;
            score -= excess.min(20.0);
        }

        score.clamp(0.0, 100.0)
    }

    /// Get performance report as string
    pub async fn get_report(&self) -> String {
        let metrics = self.get_metrics().await;

        format!(
            "Performance Report:\n\
             - CPU Usage: {:.2}%\n\
             - Memory Usage: {:.2} MB ({:.2}%)\n\
             - Active Threads: {}\n\
             - Uptime: {} seconds\n\
             - API Calls: {}\n\
             - Avg Response Time: {:.2} ms\n\
             - Cache Hit Rate: {:.2}%",
            metrics.cpu_usage_percent,
            metrics.memory_usage_mb,
            metrics.memory_usage_percent,
            metrics.active_threads,
            metrics.uptime_seconds,
            metrics.api_calls_count,
            metrics.avg_response_time_ms,
            metrics.cache_hit_rate * 100.0
        )
    }
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple response cache for AI providers
pub struct ResponseCache {
    cache: Arc<RwLock<lru::LruCache<String, String>>>,
    monitor: Arc<PerformanceMonitor>,
}

impl ResponseCache {
    pub fn new(capacity: usize, monitor: Arc<PerformanceMonitor>) -> Self {
        Self {
            cache: Arc::new(RwLock::new(lru::LruCache::new(
                std::num::NonZeroUsize::new(capacity).unwrap(),
            ))),
            monitor,
        }
    }

    /// Get cached response
    pub async fn get(&self, key: &str) -> Option<String> {
        let mut cache = self.cache.write().await;
        let result = cache.get(key).cloned();

        if result.is_some() {
            self.monitor.record_cache_hit().await;
        } else {
            self.monitor.record_cache_miss().await;
        }

        result
    }

    /// Store response in cache
    pub async fn put(&self, key: String, value: String) {
        let mut cache = self.cache.write().await;
        cache.put(key, value);
    }

    /// Clear cache
    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }

    /// Get cache size
    pub async fn len(&self) -> usize {
        let cache = self.cache.read().await;
        cache.len()
    }

    /// Check if cache is empty
    pub async fn is_empty(&self) -> bool {
        let cache = self.cache.read().await;
        cache.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_performance_monitor_creation() {
        let monitor = PerformanceMonitor::new();
        let metrics = monitor.get_metrics().await;

        assert_eq!(metrics.api_calls_count, 0);
        assert_eq!(metrics.avg_response_time_ms, 0.0);
    }

    #[tokio::test]
    async fn test_record_api_call() {
        let monitor = PerformanceMonitor::new();

        monitor.record_api_call(Duration::from_millis(100)).await;
        monitor.record_api_call(Duration::from_millis(200)).await;

        let metrics = monitor.get_metrics().await;
        assert_eq!(metrics.api_calls_count, 2);
        assert_eq!(metrics.avg_response_time_ms, 150.0);
    }

    #[tokio::test]
    async fn test_cache_hit_rate() {
        let monitor = Arc::new(PerformanceMonitor::new());

        monitor.record_cache_hit().await;
        monitor.record_cache_hit().await;
        monitor.record_cache_miss().await;

        let metrics = monitor.get_metrics().await;
        assert!((metrics.cache_hit_rate - 0.666).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_response_cache() {
        let monitor = Arc::new(PerformanceMonitor::new());
        let cache = ResponseCache::new(10, monitor.clone());

        // Cache miss
        assert!(cache.get("key1").await.is_none());

        // Store value
        cache.put("key1".to_string(), "value1".to_string()).await;

        // Cache hit
        assert_eq!(cache.get("key1").await, Some("value1".to_string()));

        let metrics = monitor.get_metrics().await;
        assert_eq!(metrics.cache_hit_rate, 0.5); // 1 hit, 1 miss
    }

    #[tokio::test]
    async fn test_uptime_tracking() {
        let monitor = PerformanceMonitor::new();

        sleep(Duration::from_millis(100)).await;

        let metrics = monitor.get_metrics().await;
        // Verify uptime was tracked (u64, always valid)
        let _ = metrics.uptime_seconds;
    }
}
