pub mod backend;
pub mod memory;

#[cfg(feature = "surrealdb")]
pub mod surreal_backend;

// Re-export key types for convenience
pub use backend::{BackendSearchItem, MemoryBackend};

#[cfg(feature = "surrealdb")]
pub use surreal_backend::{
    AgentMemory, AgentType, DiskIO, IncidentInfo, IncidentSeverity, MemorySource, MemoryUsage,
    MitigationStatus, NetworkStats, ProcessInfo, ProcessStatus, ResolutionStatus, SurrealBackend,
    SystemMetric, ThreatInfo, ThreatSeverity,
};
