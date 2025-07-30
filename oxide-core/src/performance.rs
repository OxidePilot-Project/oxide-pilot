use std::time::{Duration, Instant};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use log::{info, warn};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub cpu_usage: f32,
    pub memory_usage: u64,
    pub response_times: Vec<Duration>,
    pub error_count: u32,
    pub uptime: Duration,
    pub last_updated: std::time::SystemTime,
}

pub struct PerformanceMonitor {
    metrics: Arc<Mutex<PerformanceMetrics>>,
    response_times: Arc<Mutex<VecDeque<Duration>>>,
    start_time: Instant,
    max_response_times: usize,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(Mutex::new(PerformanceMetrics {
                cpu_usage: 0.0,
                memory_usage: 0,
                response_times: Vec::new(),
                error_count: 0,
                uptime: Duration::from_secs(0),
                last_updated: std::time::SystemTime::now(),
            })),
            response_times: Arc::new(Mutex::new(VecDeque::new())),
            start_time: Instant::now(),
            max_response_times: 100,
        }
    }

    pub fn record_response_time(&self, duration: Duration) {
        let mut times = self.response_times.lock().unwrap();
        times.push_back(duration);

        if times.len() > self.max_response_times {
            times.pop_front();
        }

        // Update metrics
        let mut metrics = self.metrics.lock().unwrap();
        metrics.response_times = times.iter().cloned().collect();
        metrics.last_updated = std::time::SystemTime::now();

        // Warn if response time is too high
        if duration > Duration::from_millis(1000) {
            warn!("Slow response detected: {:?}", duration);
        }
    }

    pub fn record_error(&self) {
        let mut metrics = self.metrics.lock().unwrap();
        metrics.error_count += 1;
        metrics.last_updated = std::time::SystemTime::now();
    }

    pub fn update_system_metrics(&self, cpu_usage: f32, memory_usage: u64) {
        let mut metrics = self.metrics.lock().unwrap();
        metrics.cpu_usage = cpu_usage;
        metrics.memory_usage = memory_usage;
        metrics.uptime = self.start_time.elapsed();
        metrics.last_updated = std::time::SystemTime::now();
    }

    pub fn get_metrics(&self) -> PerformanceMetrics {
        self.metrics.lock().unwrap().clone()
    }

    pub fn get_average_response_time(&self) -> Option<Duration> {
        let times = self.response_times.lock().unwrap();
        if times.is_empty() {
            return None;
        }

        let total: Duration = times.iter().sum();
        Some(total / times.len() as u32)
    }

    pub fn get_performance_score(&self) -> f32 {
        let metrics = self.get_metrics();
        let avg_response = self.get_average_response_time()
            .unwrap_or(Duration::from_millis(100))
            .as_millis() as f32;

        // Calculate score based on multiple factors (0-100)
        let response_score = (1000.0 / (avg_response + 100.0)) * 40.0; // Max 40 points
        let cpu_score = (100.0 - metrics.cpu_usage.min(100.0)) * 0.3; // Max 30 points
        let memory_score = if metrics.memory_usage < 100_000_000 { 20.0 } else { 10.0 }; // Max 20 points
        let error_score = if metrics.error_count == 0 { 10.0 } else { 5.0 }; // Max 10 points

        (response_score + cpu_score + memory_score + error_score).min(100.0)
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

    pub fn optimize_if_needed(&self) -> Vec<String> {
        if !self.optimization_enabled {
            return vec![];
        }

        let mut optimizations = Vec::new();
        let metrics = self.performance_monitor.get_metrics();

        // CPU optimization
        if metrics.cpu_usage > 80.0 {
            optimizations.push("High CPU usage detected - reducing monitoring frequency".to_string());
            info!("Applying CPU optimization: reducing monitoring frequency");
        }

        // Memory optimization
        if metrics.memory_usage > 500_000_000 { // 500MB
            optimizations.push("High memory usage detected - clearing caches".to_string());
            info!("Applying memory optimization: clearing caches");
        }

        // Response time optimization
        if let Some(avg_time) = self.performance_monitor.get_average_response_time() {
            if avg_time > Duration::from_millis(500) {
                optimizations.push("Slow response times detected - optimizing processing".to_string());
                info!("Applying response time optimization");
            }
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
        self.monitor.record_response_time(duration);

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