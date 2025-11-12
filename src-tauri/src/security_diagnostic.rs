//! Security Diagnostic Commands for Tauri
//!
//! This module provides advanced security diagnostic capabilities
//! exposed to the frontend through Tauri commands.

use oxide_guardian::monitor::SystemMonitor;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Security threat level classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ThreatLevel {
    Clean,
    Low,
    Medium,
    High,
    Critical,
}

impl ThreatLevel {
    /// Convert to a numeric score (0-4)
    pub fn to_score(&self) -> u8 {
        match self {
            ThreatLevel::Clean => 0,
            ThreatLevel::Low => 1,
            ThreatLevel::Medium => 2,
            ThreatLevel::High => 3,
            ThreatLevel::Critical => 4,
        }
    }
}

/// Process information for security analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub name: String,
    pub pid: String,
    pub cpu_usage: f64,
    pub memory_mb: f64,
    pub command: String,
    pub is_suspicious: bool,
    pub suspicion_reasons: Vec<String>,
}

/// System resource information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub cpu_usage: f32,
    pub memory_used_gb: f64,
    pub memory_total_gb: f64,
    pub memory_percent: f64,
}

/// Network activity information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo {
    pub total_bytes_received: u64,
    pub total_bytes_transmitted: u64,
    pub active_interfaces: usize,
}

/// Complete security diagnostic report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityDiagnosticReport {
    pub timestamp: String,
    pub system_info: SystemInfo,
    pub total_processes: usize,
    pub high_cpu_processes: Vec<ProcessInfo>,
    pub suspicious_processes: Vec<ProcessInfo>,
    pub network_info: NetworkInfo,
    pub threat_level: ThreatLevel,
    pub threat_score: u8,
    pub recommendations: Vec<String>,
}

/// State for security diagnostic operations
pub struct SecurityDiagnosticState {
    pub monitor: Arc<RwLock<SystemMonitor>>,
    pub last_scan: Arc<RwLock<Option<SecurityDiagnosticReport>>>,
}

impl SecurityDiagnosticState {
    pub fn new() -> Self {
        Self {
            monitor: Arc::new(RwLock::new(SystemMonitor::new())),
            last_scan: Arc::new(RwLock::new(None)),
        }
    }
}

/// Analyze a process for suspicious behavior
fn analyze_process_suspicion(
    name: &str,
    cpu_usage: f64,
    memory_mb: f64,
    command: &str,
) -> (bool, Vec<String>) {
    let mut is_suspicious = false;
    let mut reasons = Vec::new();

    // Check for suspicious process names
    let suspicious_names = [
        "keylogger", "backdoor", "trojan", "rootkit", "mimikatz",
        "psexec", "metasploit", "netcat", "ncat", "ratserver",
    ];

    let name_lower = name.to_lowercase();
    for suspicious in &suspicious_names {
        if name_lower.contains(suspicious) {
            is_suspicious = true;
            reasons.push(format!("Suspicious name contains '{}'", suspicious));
        }
    }

    // Check for high resource usage without visible window
    if cpu_usage > 50.0 && !command.contains("--type=renderer") && !command.contains("browser") {
        is_suspicious = true;
        reasons.push("High CPU usage without visible interface".to_string());
    }

    // Check for unusual memory usage patterns
    if memory_mb > 1000.0 && name_lower.contains(".tmp") || name_lower.contains("temp") {
        is_suspicious = true;
        reasons.push("High memory usage from temporary location".to_string());
    }

    // Check for processes with obfuscated names (random characters)
    if name.len() > 8 && name.chars().all(|c| c.is_ascii_alphanumeric()) {
        let uppercase_count = name.chars().filter(|c| c.is_uppercase()).count();
        let lowercase_count = name.chars().filter(|c| c.is_lowercase()).count();
        let digit_count = name.chars().filter(|c| c.is_numeric()).count();

        // If name is highly random (mix of upper/lower/digits without clear pattern)
        if digit_count > 2 && uppercase_count > 0 && lowercase_count > 0 {
            is_suspicious = true;
            reasons.push("Obfuscated or random process name".to_string());
        }
    }

    (is_suspicious, reasons)
}

/// Generate recommendations based on threat level and findings
fn generate_recommendations(
    threat_level: &ThreatLevel,
    suspicious_count: usize,
    high_cpu_count: usize,
    system_info: &SystemInfo,
) -> Vec<String> {
    let mut recommendations = Vec::new();

    match threat_level {
        ThreatLevel::Critical => {
            recommendations.push("⚠️ CRITICAL: Highly suspicious processes detected".to_string());
            recommendations.push("1. Disconnect from internet immediately".to_string());
            recommendations.push("2. Run full antivirus scan (Windows Defender or Malwarebytes)".to_string());
            recommendations.push("3. Consider system reinstallation if threats persist".to_string());
            recommendations.push("4. Change all passwords from a different, secure device".to_string());
            recommendations.push("5. Contact a security professional".to_string());
        }
        ThreatLevel::High => {
            recommendations.push("⚠️ HIGH: Suspicious activity detected".to_string());
            recommendations.push("1. Run full antivirus scan immediately".to_string());
            recommendations.push("2. Review all detected suspicious processes manually".to_string());
            recommendations.push("3. Update operating system and all software".to_string());
            recommendations.push("4. Check browser extensions and recently installed programs".to_string());
        }
        ThreatLevel::Medium => {
            recommendations.push("⚠️ MEDIUM: Unusual system behavior detected".to_string());
            recommendations.push("1. Review processes with high resource usage".to_string());
            recommendations.push("2. Run malware scan with Windows Defender".to_string());
            recommendations.push("3. Verify startup programs and scheduled tasks".to_string());
            recommendations.push("4. Monitor system for continued unusual behavior".to_string());
        }
        ThreatLevel::Low => {
            recommendations.push("ℹ️ System appears normal with minor alerts".to_string());
            recommendations.push("1. Monitor system regularly for changes".to_string());
            recommendations.push("2. Keep antivirus definitions updated".to_string());
            recommendations.push("3. Review high CPU processes if system feels slow".to_string());
        }
        ThreatLevel::Clean => {
            recommendations.push("✅ System appears clean - no threats detected".to_string());
            recommendations.push("1. Maintain good security practices".to_string());
            recommendations.push("2. Perform regular security scans".to_string());
            recommendations.push("3. Keep system and software updated".to_string());
            recommendations.push("4. Use strong, unique passwords with 2FA".to_string());
        }
    }

    // Additional system-specific recommendations
    if system_info.cpu_usage > 80.0 {
        recommendations.push(format!(
            "⚠️ CPU usage is high ({:.1}%) - check Task Manager for resource-heavy processes",
            system_info.cpu_usage
        ));
    }

    if system_info.memory_percent > 90.0 {
        recommendations.push(format!(
            "⚠️ Memory usage is high ({:.1}%) - consider closing unused applications",
            system_info.memory_percent
        ));
    }

    if suspicious_count > 0 {
        recommendations.push(format!(
            "Found {} suspicious process(es) - review details carefully",
            suspicious_count
        ));
    }

    if high_cpu_count > 15 {
        recommendations.push(
            "Many high-CPU processes detected - system may be overloaded".to_string()
        );
    }

    recommendations
}

/// Tauri command: Run comprehensive security diagnostic scan
#[tauri::command]
pub async fn run_security_diagnostic(
    state: tauri::State<'_, SecurityDiagnosticState>,
) -> Result<SecurityDiagnosticReport, String> {
    let mut monitor = state.monitor.write().await;
    monitor.refresh_system();

    // Get system information
    let cpu_usage = monitor.get_cpu_usage();
    let (memory_used, memory_total) = monitor.get_memory_usage();
    let memory_used_gb = memory_used as f64 / (1024.0 * 1024.0 * 1024.0);
    let memory_total_gb = memory_total as f64 / (1024.0 * 1024.0 * 1024.0);
    let memory_percent = (memory_used as f64 / memory_total as f64) * 100.0;

    let system_info = SystemInfo {
        cpu_usage,
        memory_used_gb,
        memory_total_gb,
        memory_percent,
    };

    // Analyze processes
    let process_events = monitor.list_processes();
    let mut high_cpu_processes = Vec::new();
    let mut suspicious_processes = Vec::new();

    for event in &process_events {
        if let (Some(name), Some(pid), Some(cpu), Some(memory), Some(command)) = (
            event.details.get("name").and_then(|v| v.as_str()),
            event.details.get("pid").and_then(|v| v.as_str()),
            event.details.get("cpu_usage").and_then(|v| v.as_f64()),
            event.details.get("memory_usage").and_then(|v| v.as_u64()),
            event.details.get("command").and_then(|v| v.as_str()),
        ) {
            let memory_mb = memory as f64 / (1024.0 * 1024.0);

            // Analyze for suspicious behavior
            let (is_suspicious, reasons) = analyze_process_suspicion(
                name,
                cpu,
                memory_mb,
                command,
            );

            let proc_info = ProcessInfo {
                name: name.to_string(),
                pid: pid.to_string(),
                cpu_usage: cpu,
                memory_mb,
                command: command.to_string(),
                is_suspicious,
                suspicion_reasons: reasons.clone(),
            };

            // Track high CPU processes (>20%)
            if cpu > 20.0 {
                high_cpu_processes.push(proc_info.clone());
            }

            // Track suspicious processes
            if is_suspicious {
                suspicious_processes.push(proc_info);
            }
        }
    }

    // Get network information
    let network_events = monitor.get_network_usage();
    let mut total_received = 0u64;
    let mut total_transmitted = 0u64;

    for event in &network_events {
        if let (Some(rx), Some(tx)) = (
            event.details.get("received").and_then(|v| v.as_u64()),
            event.details.get("transmitted").and_then(|v| v.as_u64()),
        ) {
            total_received += rx;
            total_transmitted += tx;
        }
    }

    let network_info = NetworkInfo {
        total_bytes_received: total_received,
        total_bytes_transmitted: total_transmitted,
        active_interfaces: network_events.len(),
    };

    // Determine threat level
    let threat_level = if !suspicious_processes.is_empty() {
        if suspicious_processes.len() >= 3 {
            ThreatLevel::Critical
        } else {
            ThreatLevel::High
        }
    } else if high_cpu_processes.len() > 15 {
        ThreatLevel::Medium
    } else if cpu_usage > 80.0 || memory_percent > 90.0 {
        ThreatLevel::Medium
    } else if high_cpu_processes.len() > 10 {
        ThreatLevel::Low
    } else {
        ThreatLevel::Clean
    };

    let threat_score = threat_level.to_score();

    // Generate recommendations
    let recommendations = generate_recommendations(
        &threat_level,
        suspicious_processes.len(),
        high_cpu_processes.len(),
        &system_info,
    );

    let report = SecurityDiagnosticReport {
        timestamp: chrono::Utc::now().to_rfc3339(),
        system_info,
        total_processes: process_events.len(),
        high_cpu_processes,
        suspicious_processes,
        network_info,
        threat_level,
        threat_score,
        recommendations,
    };

    // Store the last scan
    let mut last_scan = state.last_scan.write().await;
    *last_scan = Some(report.clone());

    Ok(report)
}

/// Tauri command: Get the last security diagnostic report
#[tauri::command]
pub async fn get_last_security_scan(
    state: tauri::State<'_, SecurityDiagnosticState>,
) -> Result<Option<SecurityDiagnosticReport>, String> {
    let last_scan = state.last_scan.read().await;
    Ok(last_scan.clone())
}

/// Tauri command: Get quick system health status
#[tauri::command]
pub async fn get_system_health(
    state: tauri::State<'_, SecurityDiagnosticState>,
) -> Result<serde_json::Value, String> {
    let mut monitor = state.monitor.write().await;
    monitor.refresh_system();

    let cpu_usage = monitor.get_cpu_usage();
    let (memory_used, memory_total) = monitor.get_memory_usage();
    let memory_percent = (memory_used as f64 / memory_total as f64) * 100.0;

    let health_status = if cpu_usage > 90.0 || memory_percent > 95.0 {
        "critical"
    } else if cpu_usage > 70.0 || memory_percent > 80.0 {
        "warning"
    } else {
        "healthy"
    };

    Ok(serde_json::json!({
        "status": health_status,
        "cpu_usage": cpu_usage,
        "memory_percent": memory_percent,
        "memory_used_gb": memory_used as f64 / (1024.0 * 1024.0 * 1024.0),
        "memory_total_gb": memory_total as f64 / (1024.0 * 1024.0 * 1024.0),
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_threat_level_scores() {
        assert_eq!(ThreatLevel::Clean.to_score(), 0);
        assert_eq!(ThreatLevel::Low.to_score(), 1);
        assert_eq!(ThreatLevel::Medium.to_score(), 2);
        assert_eq!(ThreatLevel::High.to_score(), 3);
        assert_eq!(ThreatLevel::Critical.to_score(), 4);
    }

    #[test]
    fn test_analyze_process_suspicion() {
        // Test legitimate process
        let (is_sus, reasons) = analyze_process_suspicion("chrome.exe", 10.0, 200.0, "C:\\Program Files\\Chrome\\chrome.exe");
        assert!(!is_sus);
        assert!(reasons.is_empty());

        // Test suspicious name
        let (is_sus, reasons) = analyze_process_suspicion("keylogger.exe", 5.0, 50.0, "C:\\Temp\\keylogger.exe");
        assert!(is_sus);
        assert!(!reasons.is_empty());

        // Test high CPU without browser
        let (is_sus, reasons) = analyze_process_suspicion("unknown.exe", 60.0, 100.0, "C:\\Windows\\unknown.exe");
        assert!(is_sus);
        assert!(reasons.iter().any(|r| r.contains("High CPU")));
    }

    #[test]
    fn test_recommendations_generation() {
        let system_info = SystemInfo {
            cpu_usage: 50.0,
            memory_used_gb: 8.0,
            memory_total_gb: 16.0,
            memory_percent: 50.0,
        };

        let recs = generate_recommendations(&ThreatLevel::Clean, 0, 5, &system_info);
        assert!(!recs.is_empty());
        assert!(recs[0].contains("clean"));

        let recs_critical = generate_recommendations(&ThreatLevel::Critical, 3, 5, &system_info);
        assert!(recs_critical[0].contains("CRITICAL"));
    }
}
