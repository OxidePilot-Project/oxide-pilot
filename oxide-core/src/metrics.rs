use std::time::Instant;
use std::collections::HashMap;
use log::{info, debug};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    pub name: String,
    pub value: f64,
    pub unit: String,
    pub timestamp: u64,
}

pub struct MetricsCollector {
    start_time: Instant,
    metrics: HashMap<String, Vec<Metric>>,
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            metrics: HashMap::new(),
        }
    }

    pub fn record_metric(&mut self, name: &str, value: f64, unit: &str) {
        let timestamp = self.start_time.elapsed().as_millis() as u64;
        let metric = Metric {
            name: name.to_string(),
            value,
            unit: unit.to_string(),
            timestamp,
        };
        self.metrics.entry(name.to_string()).or_default().push(metric);
        debug!("Recorded metric: {name} = {value} {unit}");
    }

    pub fn get_metrics(&self) -> &HashMap<String, Vec<Metric>> {
        &self.metrics
    }

    pub fn get_metric_history(&self, name: &str) -> Option<&Vec<Metric>> {
        self.metrics.get(name)
    }

    pub fn reset(&mut self) {
        self.metrics.clear();
        self.start_time = Instant::now();
        info!("Metrics collector reset.");
    }
}
