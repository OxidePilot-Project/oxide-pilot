/*!
Sistema de eventos para comunicación entre agentes
*/

use crate::types::{AgentId, Priority, SystemInfo};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Identificador único de evento
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EventId(pub Uuid);

impl EventId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for EventId {
    fn default() -> Self {
        Self::new()
    }
}

/// Evento base del sistema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemEvent {
    pub id: EventId,
    pub timestamp: DateTime<Utc>,
    pub source: AgentId,
    pub priority: Priority,
    pub event_type: EventType,
}

impl SystemEvent {
    pub fn new(source: AgentId, priority: Priority, event_type: EventType) -> Self {
        Self {
            id: EventId::new(),
            timestamp: Utc::now(),
            source,
            priority,
            event_type,
        }
    }
}

/// Tipos de eventos del sistema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    /// Eventos del sistema operativo
    System(SystemEventData),
    /// Eventos de seguridad
    Security(SecurityEventData),
    /// Eventos de usuario/conversación
    User(UserEventData),
    /// Eventos de agentes
    Agent(AgentEventData),
}

/// Datos de eventos del sistema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemEventData {
    /// Información del sistema actualizada
    SystemInfoUpdate(SystemInfo),
    /// Proceso iniciado
    ProcessStarted { pid: u32, name: String, path: String },
    /// Proceso terminado
    ProcessTerminated { pid: u32, name: String, exit_code: i32 },
    /// Uso alto de CPU
    HighCpuUsage { usage: f32, process: String },
    /// Uso alto de memoria
    HighMemoryUsage { usage: f32, process: String },
    /// Disco lleno
    DiskSpaceLow { usage: f32, drive: String },
}

/// Datos de eventos de seguridad
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityEventData {
    /// Proceso sospechoso detectado
    SuspiciousProcess { pid: u32, name: String, reason: String },
    /// Conexión de red sospechosa
    SuspiciousNetwork { process: String, destination: String },
    /// Modificación de archivo crítico
    CriticalFileModified { path: String, process: String },
    /// Intento de escalación de privilegios
    PrivilegeEscalation { process: String, target: String },
}

/// Datos de eventos de usuario
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserEventData {
    /// Usuario activó el copiloto
    CopilotActivated,
    /// Comando de voz recibido
    VoiceCommand { text: String, confidence: f32 },
    /// Solicitud de ayuda
    HelpRequest { query: String },
    /// Acción RPA solicitada
    RpaRequest { action: String, parameters: serde_json::Value },
}

/// Datos de eventos de agentes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentEventData {
    /// Agente iniciado
    Started,
    /// Agente detenido
    Stopped,
    /// Error en agente
    Error { message: String },
    /// Heartbeat del agente
    Heartbeat,
}