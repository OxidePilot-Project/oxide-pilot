use crate::monitor::SystemMonitor;
use oxide_core::config::GuardianConfig;
use oxide_core::types::SystemEvent;
use log::{info, warn, error};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use yara::{Compiler, Rules};

pub struct Guardian {
    monitor: Arc<Mutex<SystemMonitor>>,
    config: Arc<Mutex<GuardianConfig>>,
    yara_rules: Arc<Mutex<Option<Rules>>>,
}

impl Guardian {
    pub fn new(config: GuardianConfig) -> Self {
        let mut guardian = Self {
            monitor: Arc::new(Mutex::new(SystemMonitor::new())),
            config: Arc::new(Mutex::new(config)),
            yara_rules: Arc::new(Mutex::new(None)),
        };
        guardian.load_yara_rules();
        guardian
    }

    pub fn update_config(&self, new_config: GuardianConfig) {
        let mut config = self.config.lock().unwrap();
        *config = new_config;
        info!("Guardian config updated.");
    }

    fn load_yara_rules(&mut self) {
        // In a real scenario, rules would be loaded from a file or remote source
        let rules_str = r#"
rule example_rule {
  strings:
    $a = "malicious_string"
  condition:
    $a
}
"#;
        match Compiler::new().add_rules_str(rules_str) {
            Ok(compiler) => {
                match compiler.compile_rules() {
                    Ok(rules) => {
                        let mut yara_rules = self.yara_rules.lock().unwrap();
                        *yara_rules = Some(rules);
                        info!("YARA rules loaded successfully.");
                    },
                    Err(e) => error!("Failed to compile YARA rules: {}", e),
                }
            },
            Err(e) => error!("Failed to create YARA compiler: {}", e),
        }
    }

    pub fn start_monitoring(&self) {
        let monitor_arc = Arc::clone(&self.monitor);
        let config_arc = Arc::clone(&self.config);
        let yara_rules_arc = Arc::clone(&self.yara_rules);

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
                info!("CPU Usage: {:.2}%", cpu_usage);
                info!("Memory Usage: {}/{} bytes", used_mem, total_mem);

                let processes = monitor.list_processes();
                info!("Monitoring {} processes.", processes.len());

                let yara_rules = yara_rules_arc.lock().unwrap();
                if let Some(rules) = yara_rules.as_ref() {
                    for event in processes.iter() {
                        if event.event_type == "process_info" {
                            if let Some(command) = event.details["command"].as_str() {
                                match rules.scan_mem(command.as_bytes(), 0) {
                                    Ok(matches) => {
                                        if !matches.is_empty() {
                                            warn!("YARA match found in process command: {}", command);
                                            for m in matches {
                                                warn!("  Rule: {}", m.rule_name);
                                            }
                                        }
                                    },
                                    Err(e) => error!("YARA scan error: {}", e),
                                }
                            }
                        }
                    }
                }

                thread::sleep(Duration::from_secs(interval));
            }
        });
    }
}
