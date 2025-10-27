pub mod monitor;
pub mod guardian;
pub mod optimizer;
pub mod security;
pub mod scanner;
pub mod signatures;
pub mod quarantine;
pub mod external_api;

#[cfg(feature = "surrealdb-metrics")]
pub mod metrics_collector;

// Re-export for convenience
#[cfg(feature = "surrealdb-metrics")]
pub use metrics_collector::{MetricsCollector, MetricsConfig};
