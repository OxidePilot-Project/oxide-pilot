use crate::monitor::SystemMonitor;
use oxide_core::config::GuardianConfig;
use oxide_core::types::SystemEvent;
use log::{info, warn, error};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
#[cfg(feature = "yara-detection")]
use yara::{Compiler, Rules};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, serde::Serialize)]
pub struct ThreatEvent {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub threat_type: ThreatType,
    pub severity: ThreatSeverity,
    pub description: String,
    pub process_name: Option<String>,
    pub process_id: Option<u32>,
    pub details: HashMap<String, String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub enum ThreatType {
    MalwareSignature,
    SuspiciousProcess,
    HighResourceUsage,
    UnauthorizedNetworkAccess,
    FileSystemAnomaly,
}

#[derive(Debug, Clone, serde::Serialize)]
pub enum ThreatSeverity {
    Low,
    Medium,
    High,
    Critical,
}

pub struct ThreatDetector {
    #[cfg(feature = "yara-detection")]
    yara_rules: Arc<Mutex<Option<Rules>>>,
    process_baseline: Arc<Mutex<HashMap<String, ProcessBaseline>>>,
    threat_history: Arc<Mutex<Vec<ThreatEvent>>>,
}

#[derive(Debug, Clone)]
struct ProcessBaseline {
    average_cpu: f32,
    average_memory: u64,
    #[allow(dead_code)]
    first_seen: DateTime<Utc>,
    execution_count: u32,
}

impl Default for ThreatDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl ThreatDetector {
    pub fn new() -> Self {
        let detector = Self {
            #[cfg(feature = "yara-detection")]
            yara_rules: Arc::new(Mutex::new(None)),
            process_baseline: Arc::new(Mutex::new(HashMap::new())),
            threat_history: Arc::new(Mutex::new(Vec::new())),
        };
        #[cfg(feature = "yara-detection")]
        detector.load_yara_rules();
        detector
    }

    #[cfg(feature = "yara-detection")]
    fn load_yara_rules(&mut self) {
        let rules_str = r#"
rule suspicious_powershell {
  strings:
    $a = "powershell" nocase
    $b = "-encodedcommand" nocase
    $c = "-windowstyle hidden" nocase
  condition:
    $a and ($b or $c)
}

rule potential_ransomware {
  strings:
    $a = ".encrypt" nocase
    $b = ".locked" nocase
    $c = "ransom" nocase
    $d = "bitcoin" nocase
  condition:
    any of them
}

rule suspicious_network_tool {
  strings:
    $a = "netcat" nocase
    $b = "nmap" nocase
    $c = "wireshark" nocase
  condition:
    any of them
}
"#;
        match Compiler::new().add_rules_str(rules_str) {
            Ok(compiler) => {
                match compiler.compile_rules() {
                    Ok(rules) => {
                        let mut yara_rules = self.yara_rules.lock().unwrap();
                        *yara_rules = Some(rules);
                        info!("Enhanced YARA rules loaded successfully.");
                    },
                    Err(e) => error!("Failed to compile YARA rules: {}", e),
                }
            },
            Err(e) => error!("Failed to create YARA compiler: {}", e),
        }
    }

    pub fn analyze_processes(&self, processes: &[SystemEvent]) -> Vec<ThreatEvent> {
        let mut threats = Vec::new();
        #[cfg(feature = "yara-detection")]
        let yara_rules = self.yara_rules.lock().unwrap();
        let mut baseline = self.process_baseline.lock().unwrap();

        for event in processes {
            if event.event_type == "process_info" {
                let process_name = event.details.get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();
                let process_id = event.details.get("pid")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<u32>().ok());
                let cpu_usage = event.details.get("cpu_usage")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0) as f32;
                let memory_usage = event.details.get("memory")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);

                // Update baseline
                let entry = baseline.entry(process_name.clone()).or_insert(ProcessBaseline {
                    average_cpu: cpu_usage,
                    average_memory: memory_usage,
                    first_seen: Utc::now(),
                    execution_count: 0,
                });
                entry.execution_count += 1;
                entry.average_cpu = (entry.average_cpu + cpu_usage) / 2.0;
                entry.average_memory = (entry.average_memory + memory_usage) / 2;

                // Check for suspicious resource usage
                if cpu_usage > 80.0 {
                    threats.push(ThreatEvent {
                        id: uuid::Uuid::new_v4().to_string(),
                        timestamp: Utc::now(),
                        threat_type: ThreatType::HighResourceUsage,
                        severity: ThreatSeverity::Medium,
                        description: format!("High CPU usage detected: {cpu_usage:.2}%"),
                        process_name: Some(process_name.clone()),
                        process_id,
                        details: HashMap::from([
                            ("cpu_usage".to_string(), cpu_usage.to_string()),
                            ("memory_usage".to_string(), memory_usage.to_string()),
                        ]),
                    });
                }

                // YARA rule scanning
                #[cfg(feature = "yara-detection")]
                if let Some(rules) = yara_rules.as_ref() {
                    if let Some(command) = event.details.get("command") {
                        match rules.scan_mem(command.as_bytes(), 0) {
                            Ok(matches) => {
                                if !matches.is_empty() {
                                    for m in matches {
                                        threats.push(ThreatEvent {
                                            id: uuid::Uuid::new_v4().to_string(),
                                            timestamp: Utc::now(),
                                            threat_type: ThreatType::MalwareSignature,
                                            severity: ThreatSeverity::High,
                                            description: format!("YARA rule match: {}", m.rule_name),
                                            process_name: Some(process_name.clone()),
                                            process_id,
                                            details: HashMap::from([
                                                ("rule_name".to_string(), m.rule_name.to_string()),
                                                ("command".to_string(), command.clone()),
                                            ]),
                                        });
                                    }
                                }
                            },
                            Err(e) => error!("YARA scan error: {}", e),
                        }
                    }
                }

                // Heuristic analysis for suspicious processes
                if self.is_suspicious_process(&process_name, &event.details) {
                    let mut details_map = HashMap::new();
                    if let Some(obj) = event.details.as_object() {
                        for (k, v) in obj {
                            details_map.insert(k.clone(), v.to_string());
                        }
                    }

                    threats.push(ThreatEvent {
                        id: uuid::Uuid::new_v4().to_string(),
                        timestamp: Utc::now(),
                        threat_type: ThreatType::SuspiciousProcess,
                        severity: ThreatSeverity::Medium,
                        description: format!("Suspicious process behavior detected: {process_name}"),
                        process_name: Some(process_name.clone()),
                        process_id,
                        details: details_map,
                    });
                }
            }
        }

        // Store threats in history
        let mut history = self.threat_history.lock().unwrap();
        history.extend(threats.clone());

        // Keep only last 1000 threats to prevent memory bloat
        if history.len() > 1000 {
            let len = history.len();
            history.drain(0..len - 1000);
        }

        threats
    }

    fn is_suspicious_process(&self, process_name: &str, details: &serde_json::Value) -> bool {
        // Check for suspicious process names
        let suspicious_names = [
            "cmd.exe", "powershell.exe", "wscript.exe", "cscript.exe",
            "regsvr32.exe", "rundll32.exe", "mshta.exe"
        ];

        if suspicious_names.iter().any(|&name| process_name.to_lowercase().contains(name)) {
            // Check for suspicious command line arguments
            if let Some(command) = details.get("command").and_then(|v| v.as_str()) {
                let suspicious_args = [
                    "-encodedcommand", "-windowstyle hidden", "-noprofile",
                    "invoke-expression", "downloadstring", "bypass"
                ];

                return suspicious_args.iter().any(|&arg|
                    command.to_lowercase().contains(arg)
                );
            }
        }

        // Check for processes running from suspicious locations
        if let Some(path) = details.get("exe").and_then(|v| v.as_str()) {
            let suspicious_paths = [
                "\\temp\\", "\\appdata\\local\\temp\\", "\\users\\public\\",
                "\\programdata\\", "\\windows\\temp\\"
            ];

            return suspicious_paths.iter().any(|&path_part|
                path.to_lowercase().contains(path_part)
            );
        }

        false
    }

    pub fn get_threat_history(&self) -> Vec<ThreatEvent> {
        self.threat_history.lock().unwrap().clone()
    }
}

pub struct Guardian {
    monitor: Arc<Mutex<SystemMonitor>>,
    config: Arc<Mutex<GuardianConfig>>,
    threat_detector: Arc<ThreatDetector>,
}

impl Guardian {
    pub fn new(config: GuardianConfig) -> Self {
        Self {
            monitor: Arc::new(Mutex::new(SystemMonitor::new())),
            config: Arc::new(Mutex::new(config)),
            threat_detector: Arc::new(ThreatDetector::new()),
        }
    }

    pub fn update_config(&self, new_config: GuardianConfig) {
        let mut config = self.config.lock().unwrap();
        *config = new_config;
        info!("Guardian config updated.");
    }

    pub fn start_monitoring(&self) {
        let monitor_arc = Arc::clone(&self.monitor);
        let config_arc = Arc::clone(&self.config);
        let threat_detector_arc = Arc::clone(&self.threat_detector);

        thread::spawn(move || {
            loop {
                let config = config_arc.lock().unwrap();
                if !config.enabled {
                    info!("Guardian monitoring is disabled. Sleeping...");
                    thread::sleep(Duration::from_secs(5));
                    continue;
                }

                let interval = config.monitor_interval_secs;
                drop(config); // Release lock

                let mut monitor = monitor_arc.lock().unwrap();
                monitor.refresh_system();

                let cpu_usage = monitor.get_cpu_usage();
                let (used_mem, total_mem) = monitor.get_memory_usage();
                info!("System Status - CPU: {cpu_usage:.2}%, Memory: {used_mem}/{total_mem} bytes");

                let processes = monitor.list_processes();
                info!("Monitoring {} processes.", processes.len());

                // Analyze processes for threats
                let threats = threat_detector_arc.analyze_processes(&processes);

                for threat in threats {
                    match threat.severity {
                        ThreatSeverity::Critical => error!("CRITICAL THREAT: {}", threat.description),
                        ThreatSeverity::High => error!("HIGH THREAT: {}", threat.description),
                        ThreatSeverity::Medium => warn!("MEDIUM THREAT: {}", threat.description),
                        ThreatSeverity::Low => info!("LOW THREAT: {}", threat.description),
                    }

                    if let Some(process_name) = &threat.process_name {
                        info!("  Process: {} (PID: {:?})", process_name, threat.process_id);
                    }
                }

                thread::sleep(Duration::from_secs(interval));
            }
        });
    }

    pub fn get_threat_history(&self) -> Vec<ThreatEvent> {
        self.threat_detector.get_threat_history()
    }

    pub fn get_system_status(&self) -> SystemStatus {
        let monitor = self.monitor.lock().unwrap();
        SystemStatus {
            cpu_usage: monitor.get_cpu_usage(),
            memory_usage: monitor.get_memory_usage(),
            process_count: monitor.list_processes().len(),
            threat_count: self.threat_detector.get_threat_history().len(),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SystemStatus {
    pub cpu_usage: f32,
    pub memory_usage: (u64, u64), // (used, total)
    pub process_count: usize,
    pub threat_count: usize,
}
