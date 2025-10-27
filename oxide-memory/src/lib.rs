pub mod memory;
pub mod backend;

#[cfg(feature = "surrealdb")]
pub mod surreal_backend;

// Re-export key types for convenience
pub use backend::{BackendSearchItem, MemoryBackend};

#[cfg(feature = "surrealdb")]
pub use surreal_backend::{
    SurrealBackend, SystemMetric, MemoryUsage, DiskIO, NetworkStats,
    ProcessInfo, ProcessStatus, ThreatInfo, ThreatSeverity, MitigationStatus,
    IncidentInfo, IncidentSeverity, ResolutionStatus,
    AgentMemory, AgentType, MemorySource,
};
