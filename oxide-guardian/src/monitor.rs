use chrono::Utc;
use oxide_core::types::SystemEvent;
use sysinfo::{CpuExt, DiskExt, NetworkExt, ProcessExt, System, SystemExt};
use uuid::Uuid;

pub struct SystemMonitor {
    sys: System,
}

impl Default for SystemMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemMonitor {
    pub fn new() -> Self {
        Self {
            sys: System::new_all(),
        }
    }

    pub fn refresh_system(&mut self) {
        self.sys.refresh_all();
    }

    pub fn get_cpu_usage(&self) -> f32 {
        self.sys.global_cpu_info().cpu_usage()
    }

    pub fn get_memory_usage(&self) -> (u64, u64) {
        (self.sys.used_memory(), self.sys.total_memory())
    }

    pub fn list_processes(&self) -> Vec<SystemEvent> {
        let mut events = Vec::new();
        for (pid, process) in self.sys.processes() {
            let event = SystemEvent {
                id: Uuid::new_v4(),
                timestamp: Utc::now(),
                event_type: "process_info".to_string(),
                details: serde_json::json!({
                    "pid": pid.to_string(),
                    "name": process.name(),
                    "cpu_usage": process.cpu_usage(),
                    "memory_usage": process.memory(),
                    "status": process.status().to_string(),
                    "command": process.cmd().join(" "),
                }),
            };
            events.push(event);
        }
        events
    }

    pub fn get_disk_usage(&self) -> Vec<SystemEvent> {
        let mut events = Vec::new();
        for disk in self.sys.disks() {
            let event = SystemEvent {
                id: Uuid::new_v4(),
                timestamp: Utc::now(),
                event_type: "disk_info".to_string(),
                details: serde_json::json!({
                    "name": disk.name().to_string_lossy(),
                    "total_space": disk.total_space(),
                    "available_space": disk.available_space(),
                    "file_system": String::from_utf8_lossy(disk.file_system()),
                    "mount_point": disk.mount_point().to_string_lossy(),
                }),
            };
            events.push(event);
        }
        events
    }

    pub fn get_network_usage(&self) -> Vec<SystemEvent> {
        let mut events = Vec::new();
        for (interface_name, data) in self.sys.networks() {
            let event = SystemEvent {
                id: Uuid::new_v4(),
                timestamp: Utc::now(),
                event_type: "network_info".to_string(),
                details: serde_json::json!({
                    "interface": interface_name,
                    "received": data.received(),
                    "transmitted": data.transmitted(),
                    "packets_received": data.packets_received(),
                    "packets_transmitted": data.packets_transmitted(),
                    "errors_on_received": data.errors_on_received(),
                    "errors_on_transmitted": data.errors_on_transmitted(),
                }),
            };
            events.push(event);
        }
        events
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_monitor_creation() {
        let _monitor = SystemMonitor::new();
        // Test that the monitor can be created successfully
        // Creation itself is the test - if it panics, test fails
    }

    #[test]
    fn test_get_memory_usage() {
        let monitor = SystemMonitor::new();
        let (used, total) = monitor.get_memory_usage();
        // Memory values should be reasonable
        assert!(total > 0);
        assert!(used <= total);
    }
}
