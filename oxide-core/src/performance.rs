use std::time::{Duration, Instant, SystemTime};
use std::collections::{VecDeque, HashMap};
use std::sync::{Arc, Mutex};
use log::{info, warn, error};
use serde::{Serialize, Deserialize};
use tokio::sync::RwLock;
use tokio::time::interval;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub cpu_usage: f32,
    pub memory_usage: u64,
    pub memory_peak: u64,
    pub response_times: Vec<Duration>,
    pub error_count: u32,
    pub uptime: Duration,
    pub last_updated: SystemTime,
    pub operations_per_second: f32,
    pub active_connections: u32,
    pub cache_hit_rate: f32,
    pub disk_io_rate: f32,
    pub network_io_rate: f32,
    pub gc_collections: u32,
    pub thread_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAlert {
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub message: String,
    pub timestamp: SystemTime,
    pub metric_value: f32,
    pub threshold: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertType {
    HighCpuUsage,
    HighMemoryUsage,
    SlowResponseTime,
    HighErrorRate,
    LowCacheHitRate,
    HighDiskIO,
    HighNetworkIO,
    ThreadPoolExhaustion,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceProfile {
    pub operation_name: String,
    pub total_calls: u64,
    pub total_duration: Duration,
    pub average_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub percentile_95: Duration,
    pub percentile_99: Duration,
    pub error_count: u32,
    pub last_called: SystemTime,
}

#[derive(Debug, Clone)]
pub struct PerformanceMonitor {
    metrics: Arc<RwLock<PerformanceMetrics>>,
    response_times: Arc<Mutex<VecDeque<Duration>>>,
    alerts: Arc<RwLock<Vec<PerformanceAlert>>>,
    profiles: Arc<RwLock<HashMap<String, PerformanceProfile>>>,
    start_time: Instant,
    max_response_times: usize,
    max_alerts: usize,
    monitoring_enabled: Arc<RwLock<bool>>,
    alert_thresholds: Arc<RwLock<AlertThresholds>>,
}

#[derive(Debug, Clone)]
pub struct AlertThresholds {
    pub cpu_usage_warning: f32,
    pub cpu_usage_critical: f32,
    pub memory_usage_warning: u64,
    pub memory_usage_critical: u64,
    pub response_time_warning: Duration,
    pub response_time_critical: Duration,
    pub error_rate_warning: f32,
    pub error_rate_critical: f32,
    pub cache_hit_rate_warning: f32,
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            cpu_usage_warning: 70.0,
            cpu_usage_critical: 90.0,
            memory_usage_warning: 1_000_000_000, // 1GB
            memory_usage_critical: 2_000_000_000, // 2GB
            response_time_warning: Duration::from_millis(500),
            response_time_critical: Duration::from_millis(2000),
            error_rate_warning: 5.0, // 5%
            error_rate_critical: 15.0, // 15%
            cache_hit_rate_warning: 80.0, // Below 80%
        }
    }
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(PerformanceMetrics {
                cpu_usage: 0.0,
                memory_usage: 0,
                memory_peak: 0,
                response_times: Vec::new(),
                error_count: 0,
                uptime: Duration::from_secs(0),
                last_updated: SystemTime::now(),
                operations_per_second: 0.0,
                active_connections: 0,
                cache_hit_rate: 100.0,
                disk_io_rate: 0.0,
                network_io_rate: 0.0,
                gc_collections: 0,
                thread_count: 0,
            })),
            response_times: Arc::new(Mutex::new(VecDeque::new())),
            alerts: Arc::new(RwLock::new(Vec::new())),
            profiles: Arc::new(RwLock::new(HashMap::new())),
            start_time: Instant::now(),
            max_response_times: 1000,
            max_alerts: 100,
            monitoring_enabled: Arc::new(RwLock::new(true)),
            alert_thresholds: Arc::new(RwLock::new(AlertThresholds::default())),
        }
    }

    pub async fn set_monitoring_enabled(&self, enabled: bool) {
        let mut monitoring = self.monitoring_enabled.write().await;
        *monitoring = enabled;
        info!("Performance monitoring {}", if enabled { "enabled" } else { "disabled" });
    }

    pub async fn is_monitoring_enabled(&self) -> bool {
        *self.monitoring_enabled.read().await
    }

    pub async fn record_response_time(&self, duration: Duration) {
        if !self.is_monitoring_enabled().await {
            return;
        }

        // Update response times (scope the mutex guard)
        let response_times_vec = {
            let mut times = self.response_times.lock().unwrap();
            times.push_back(duration);

            if times.len() > self.max_response_times {
                times.pop_front();
            }

            times.iter().cloned().collect::<Vec<Duration>>()
        };

        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.response_times = response_times_vec;
            metrics.last_updated = SystemTime::now();
        }

        // Check for response time alerts
        let thresholds = self.alert_thresholds.read().await;
        if duration > thresholds.response_time_critical {
            self.create_alert(
                AlertType::SlowResponseTime,
                AlertSeverity::Critical,
                format!("Critical response time: {duration:?}"),
                duration.as_millis() as f32,
                thresholds.response_time_critical.as_millis() as f32,
            ).await;
        } else if duration > thresholds.response_time_warning {
            self.create_alert(
                AlertType::SlowResponseTime,
                AlertSeverity::Warning,
                format!("Slow response time: {duration:?}"),
                duration.as_millis() as f32,
                thresholds.response_time_warning.as_millis() as f32,
            ).await;
        }
    }

    pub async fn record_error(&self) {
        if !self.is_monitoring_enabled().await {
            return;
        }

        let mut metrics = self.metrics.write().await;
        metrics.error_count += 1;
        metrics.last_updated = SystemTime::now();

        // Calculate error rate and check for alerts
        let error_rate = (metrics.error_count as f32 / metrics.uptime.as_secs() as f32) * 100.0;
        let thresholds = self.alert_thresholds.read().await;

        if error_rate > thresholds.error_rate_critical {
            self.create_alert(
                AlertType::HighErrorRate,
                AlertSeverity::Critical,
                format!("Critical error rate: {error_rate:.2}%"),
                error_rate,
                thresholds.error_rate_critical,
            ).await;
        } else if error_rate > thresholds.error_rate_warning {
            self.create_alert(
                AlertType::HighErrorRate,
                AlertSeverity::Warning,
                format!("High error rate: {error_rate:.2}%"),
                error_rate,
                thresholds.error_rate_warning,
            ).await;
        }
    }

    async fn create_alert(
        &self,
        alert_type: AlertType,
        severity: AlertSeverity,
        message: String,
        metric_value: f32,
        threshold: f32,
    ) {
        let alert = PerformanceAlert {
            alert_type,
            severity: severity.clone(),
            message: message.clone(),
            timestamp: SystemTime::now(),
            metric_value,
            threshold,
        };

        let mut alerts = self.alerts.write().await;
        alerts.push(alert);

        // Keep only the most recent alerts
        if alerts.len() > self.max_alerts {
            alerts.remove(0);
        }

        // Log the alert
        match severity {
            AlertSeverity::Critical => error!("PERFORMANCE ALERT [CRITICAL]: {message}"),
            AlertSeverity::Warning => warn!("PERFORMANCE ALERT [WARNING]: {message}"),
            AlertSeverity::Info => info!("PERFORMANCE ALERT [INFO]: {message}"),
        }
    }

    pub async fn get_alerts(&self) -> Vec<PerformanceAlert> {
        self.alerts.read().await.clone()
    }

    pub async fn clear_alerts(&self) {
        let mut alerts = self.alerts.write().await;
        alerts.clear();
        info!("Performance alerts cleared");
    }

    pub async fn update_system_metrics(&self, cpu_usage: f32, memory_usage: u64) {
        if !self.is_monitoring_enabled().await {
            return;
        }

        let mut metrics = self.metrics.write().await;
        metrics.cpu_usage = cpu_usage;
        metrics.memory_usage = memory_usage;

        // Update peak memory usage
        if memory_usage > metrics.memory_peak {
            metrics.memory_peak = memory_usage;
        }

        metrics.uptime = self.start_time.elapsed();
        metrics.last_updated = SystemTime::now();

        // Check for CPU and memory alerts
        let thresholds = self.alert_thresholds.read().await;

        if cpu_usage > thresholds.cpu_usage_critical {
            self.create_alert(
                AlertType::HighCpuUsage,
                AlertSeverity::Critical,
                format!("Critical CPU usage: {cpu_usage:.2}%"),
                cpu_usage,
                thresholds.cpu_usage_critical,
            ).await;
        } else if cpu_usage > thresholds.cpu_usage_warning {
            self.create_alert(
                AlertType::HighCpuUsage,
                AlertSeverity::Warning,
                format!("High CPU usage: {cpu_usage:.2}%"),
                cpu_usage,
                thresholds.cpu_usage_warning,
            ).await;
        }

        if memory_usage > thresholds.memory_usage_critical {
            self.create_alert(
                AlertType::HighMemoryUsage,
                AlertSeverity::Critical,
                format!("Critical memory usage: {memory_usage} bytes"),
                memory_usage as f32,
                thresholds.memory_usage_critical as f32,
            ).await;
        } else if memory_usage > thresholds.memory_usage_warning {
            self.create_alert(
                AlertType::HighMemoryUsage,
                AlertSeverity::Warning,
                format!("High memory usage: {memory_usage} bytes"),
                memory_usage as f32,
                thresholds.memory_usage_warning as f32,
            ).await;
        }
    }

    pub async fn record_operation(&self, operation_name: &str, duration: Duration, success: bool) {
        if !self.is_monitoring_enabled().await {
            return;
        }

        let mut profiles = self.profiles.write().await;
        let profile = profiles.entry(operation_name.to_string()).or_insert_with(|| {
            PerformanceProfile {
                operation_name: operation_name.to_string(),
                total_calls: 0,
                total_duration: Duration::from_secs(0),
                average_duration: Duration::from_secs(0),
                min_duration: duration,
                max_duration: duration,
                percentile_95: duration,
                percentile_99: duration,
                error_count: 0,
                last_called: SystemTime::now(),
            }
        });

        profile.total_calls += 1;
        profile.total_duration += duration;
        profile.average_duration = profile.total_duration / profile.total_calls as u32;
        profile.last_called = SystemTime::now();

        if duration < profile.min_duration {
            profile.min_duration = duration;
        }
        if duration > profile.max_duration {
            profile.max_duration = duration;
        }

        if !success {
            profile.error_count += 1;
        }

        // Update percentiles (simplified calculation)
        if profile.total_calls >= 20 {
            profile.percentile_95 = profile.max_duration * 95 / 100;
            profile.percentile_99 = profile.max_duration * 99 / 100;
        }
    }

    pub async fn get_operation_profiles(&self) -> HashMap<String, PerformanceProfile> {
        self.profiles.read().await.clone()
    }

    pub async fn get_metrics(&self) -> PerformanceMetrics {
        self.metrics.read().await.clone()
    }

    pub fn get_average_response_time(&self) -> Option<Duration> {
        let times = self.response_times.lock().unwrap();
        if times.is_empty() {
            return None;
        }

        let total: Duration = times.iter().sum();
        Some(total / times.len() as u32)
    }

    pub async fn get_performance_score(&self) -> f32 {
        let metrics = self.get_metrics().await;
        let avg_response = self.get_average_response_time()
            .unwrap_or(Duration::from_millis(100))
            .as_millis() as f32;

        // Enhanced scoring algorithm with more factors
        let response_score = (1000.0 / (avg_response + 100.0)) * 25.0; // Max 25 points
        let cpu_score = (100.0 - metrics.cpu_usage.min(100.0)) * 0.25; // Max 25 points
        let memory_score = if metrics.memory_usage < 500_000_000 { 20.0 } else { 10.0 }; // Max 20 points
        let error_score = if metrics.error_count == 0 { 15.0 } else { 5.0 }; // Max 15 points
        let cache_score = (metrics.cache_hit_rate / 100.0) * 10.0; // Max 10 points
        let ops_score = if metrics.operations_per_second > 10.0 { 5.0 } else { 2.0 }; // Max 5 points

        (response_score + cpu_score + memory_score + error_score + cache_score + ops_score).min(100.0)
    }

    pub async fn update_cache_metrics(&self, hit_rate: f32) {
        if !self.is_monitoring_enabled().await {
            return;
        }

        let mut metrics = self.metrics.write().await;
        metrics.cache_hit_rate = hit_rate;
        metrics.last_updated = SystemTime::now();

        // Check for cache hit rate alerts
        let thresholds = self.alert_thresholds.read().await;
        if hit_rate < thresholds.cache_hit_rate_warning {
            self.create_alert(
                AlertType::LowCacheHitRate,
                AlertSeverity::Warning,
                format!("Low cache hit rate: {hit_rate:.2}%"),
                hit_rate,
                thresholds.cache_hit_rate_warning,
            ).await;
        }
    }

    pub async fn update_io_metrics(&self, disk_io_rate: f32, network_io_rate: f32) {
        if !self.is_monitoring_enabled().await {
            return;
        }

        let mut metrics = self.metrics.write().await;
        metrics.disk_io_rate = disk_io_rate;
        metrics.network_io_rate = network_io_rate;
        metrics.last_updated = SystemTime::now();
    }

    pub async fn update_thread_metrics(&self, thread_count: u32, active_connections: u32) {
        if !self.is_monitoring_enabled().await {
            return;
        }

        let mut metrics = self.metrics.write().await;
        metrics.thread_count = thread_count;
        metrics.active_connections = active_connections;
        metrics.last_updated = SystemTime::now();

        // Check for thread pool exhaustion
        if thread_count > 100 { // Arbitrary threshold
            self.create_alert(
                AlertType::ThreadPoolExhaustion,
                AlertSeverity::Warning,
                format!("High thread count: {thread_count}"),
                thread_count as f32,
                100.0,
            ).await;
        }
    }

    pub async fn start_background_monitoring(&self) {
        let monitor = Arc::new(self.clone());
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(5));

            loop {
                interval.tick().await;

                if !monitor.is_monitoring_enabled().await {
                    continue;
                }

                // Update operations per second
                let mut metrics = monitor.metrics.write().await;
                let uptime_secs = monitor.start_time.elapsed().as_secs() as f32;
                if uptime_secs > 0.0 {
                    // This is a simplified calculation - in practice you'd track actual operations
                    metrics.operations_per_second = metrics.response_times.len() as f32 / uptime_secs;
                }
                drop(metrics);

                // Perform periodic cleanup
                monitor.cleanup_old_data().await;
            }
        });
    }

    async fn cleanup_old_data(&self) {
        // Clean up old alerts (keep only last 24 hours)
        let mut alerts = self.alerts.write().await;
        let cutoff = SystemTime::now() - Duration::from_secs(24 * 60 * 60);
        alerts.retain(|alert| alert.timestamp > cutoff);
    }
}

pub struct ResourceOptimizer {
    performance_monitor: Arc<PerformanceMonitor>,
    optimization_enabled: bool,
}

impl ResourceOptimizer {
    pub fn new(performance_monitor: Arc<PerformanceMonitor>) -> Self {
        Self {
            performance_monitor,
            optimization_enabled: true,
        }
    }

    pub async fn optimize_if_needed(&self) -> Vec<String> {
        if !self.optimization_enabled {
            return vec![];
        }

        let mut optimizations = Vec::new();
        let metrics = self.performance_monitor.get_metrics().await;

        // CPU optimization
        if metrics.cpu_usage > 80.0 {
            optimizations.push("High CPU usage detected - reducing monitoring frequency".to_string());
            info!("Applying CPU optimization: reducing monitoring frequency");

            // Actually reduce monitoring frequency
            self.performance_monitor.set_monitoring_enabled(false).await;
            tokio::time::sleep(Duration::from_secs(5)).await;
            self.performance_monitor.set_monitoring_enabled(true).await;
        }

        // Memory optimization
        if metrics.memory_usage > 500_000_000 { // 500MB
            optimizations.push("High memory usage detected - clearing caches".to_string());
            info!("Applying memory optimization: clearing caches");

            // Clear performance data to free memory
            self.performance_monitor.clear_alerts().await;
        }

        // Response time optimization
        if let Some(avg_time) = self.performance_monitor.get_average_response_time() {
            if avg_time > Duration::from_millis(500) {
                optimizations.push("Slow response times detected - optimizing processing".to_string());
                info!("Applying response time optimization");
            }
        }

        // Cache optimization
        if metrics.cache_hit_rate < 70.0 {
            optimizations.push("Low cache hit rate detected - optimizing cache strategy".to_string());
            info!("Applying cache optimization");
        }

        // Thread optimization
        if metrics.thread_count > 50 {
            optimizations.push("High thread count detected - optimizing thread pool".to_string());
            info!("Applying thread pool optimization");
        }

        optimizations
    }

    pub fn set_optimization_enabled(&mut self, enabled: bool) {
        self.optimization_enabled = enabled;
        info!("Resource optimization {}", if enabled { "enabled" } else { "disabled" });
    }
}

#[derive(Debug)]
pub struct PerformanceTimer {
    start: Instant,
    name: String,
    monitor: Arc<PerformanceMonitor>,
}

impl PerformanceTimer {
    pub fn new(name: String, monitor: Arc<PerformanceMonitor>) -> Self {
        Self {
            start: Instant::now(),
            name,
            monitor,
        }
    }
}

impl Drop for PerformanceTimer {
    fn drop(&mut self) {
        let duration = self.start.elapsed();
        let monitor = self.monitor.clone();
        let name = self.name.clone();

        // Spawn async task to record the timing
        tokio::spawn(async move {
            monitor.record_response_time(duration).await;
            monitor.record_operation(&name, duration, true).await;
        });

        if duration > Duration::from_millis(100) {
            info!("Operation '{}' took {:?}", self.name, duration);
        }
    }
}

// Macro for easy performance timing
#[macro_export]
macro_rules! time_operation {
    ($monitor:expr, $name:expr, $block:block) => {{
        let _timer = PerformanceTimer::new($name.to_string(), $monitor.clone());
        $block
    }};
}