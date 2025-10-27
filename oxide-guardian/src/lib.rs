pub mod external_api;
pub mod guardian;
pub mod monitor;
pub mod optimizer;
pub mod quarantine;
pub mod scanner;
pub mod security;
pub mod signatures;

#[cfg(feature = "surrealdb-metrics")]
pub mod metrics_collector;

// Re-export for convenience
#[cfg(feature = "surrealdb-metrics")]
pub use metrics_collector::{MetricsCollector, MetricsConfig};
