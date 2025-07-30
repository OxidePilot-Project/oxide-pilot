use log::{info, warn, error};
use sysinfo::{System, SystemExt, ProcessExt};

use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use sysinfo::{System, SystemExt, ProcessExt};

#[derive(Debug, Clone)]
pub struct ResourceUsage {
    pub cpu_percent: f32,
    pub memory_mb: u64,
    pub disk_usage_mb: u64,
    pub network_usage_kb: u64,
}

#[derive(Debug, Clone)]
pub struct OptimizationConfig {
    pub max_cpu_percent: f32,
    pub max_memory_mb: u64,
    pub background_priority: bool,
    pub auto_throttle: bool,
    pub idle_detection: bool,
}

pub struct PerformanceOptimizer {
    sys: System,
    config: OptimizationConfig,
    last_check: Instant,
    is_throttled: Arc<AtomicBool>,
    background_mode: Arc<AtomicBool>,
    idle_threshold: Duration,
}

impl PerformanceOptimizer {
    pub fn new() -> Self {
        let config = OptimizationConfig {
            max_cpu_percent: 5.0, // Max 5% CPU in background
            max_memory_mb: 256,   // Max 256MB RAM
            background_priority: true,
            auto_throttle: true,
            idle_detection: true,
        };

        Self {
            sys: System::new_all(),
            config,
            last_check: Instant::now(),
            is_throttled: Arc::new(AtomicBool::new(false)),
            background_mode: Arc::new(AtomicBool::new(true)),
            idle_threshold: Duration::from_secs(30),
        }
    }

    pub fn update_config(&mut self, config: OptimizationConfig) {
        self.config = config;
    }

    pub fn get_current_usage(&mut self) -> ResourceUsage {
        self.sys.refresh_all();
        
        let current_pid = std::process::id() as i32;
        let process = self.sys.process(current_pid.into());
        
        let (cpu_percent, memory_mb) = match process {
            Some(p) => (p.cpu_usage(), p.memory() / 1024 / 1024),
            None => (0.0, 0),
        };

        ResourceUsage {
            cpu_percent,
            memory_mb,
            disk_usage_mb: 0, // Could be implemented with disk usage tracking
            network_usage_kb: 0, // Could be implemented with network monitoring
        }
    }

    pub fn should_throttle(&mut self) -> bool {
        let usage = self.get_current_usage();
        
        let should_throttle = usage.cpu_percent > self.config.max_cpu_percent ||
                            usage.memory_mb > self.config.max_memory_mb;
        
        self.is_throttled.store(should_throttle, Ordering::Relaxed);
        should_throttle
    }

    pub fn set_background_mode(&self, enabled: bool) {
        self.background_mode.store(enabled, Ordering::Relaxed);
        
        if enabled {
            self.set_process_priority_low();
        } else {
            self.set_process_priority_normal();
        }
    }

    #[cfg(target_os = "windows")]
    fn set_process_priority_low(&self) {
        use winapi::um::processthreadsapi::SetPriorityClass;
        use winapi::um::winbase::{BELOW_NORMAL_PRIORITY_CLASS, IDLE_PRIORITY_CLASS};
        
        unsafe {
            let current_process = winapi::um::processthreadsapi::GetCurrentProcess();
            SetPriorityClass(current_process, BELOW_NORMAL_PRIORITY_CLASS);
        }
    }

    #[cfg(target_os = "windows")]
    fn set_process_priority_normal(&self) {
        use winapi::um::processthreadsapi::SetPriorityClass;
        use winapi::um::winbase::NORMAL_PRIORITY_CLASS;
        
        unsafe {
            let current_process = winapi::um::processthreadsapi::GetCurrentProcess();
            SetPriorityClass(current_process, NORMAL_PRIORITY_CLASS);
        }
    }

    #[cfg(not(target_os = "windows"))]
    fn set_process_priority_low(&self) {
        // Unix-like systems
        unsafe {
            libc::nice(5); // Increase niceness (lower priority)
        }
    }

    #[cfg(not(target_os = "windows"))]
    fn set_process_priority_normal(&self) {
        // Unix-like systems - restore to normal priority
        unsafe {
            libc::nice(0);
        }
    }

    pub fn optimize_memory_usage(&self) {
        // Force garbage collection on supported platforms
        #[cfg(feature = "jemalloc")]
        {
            use jemalloc_ctl::{epoch, stats};
            epoch::advance().unwrap();
        }
    }

    pub fn is_system_idle(&mut self) -> bool {
        self.sys.refresh_cpu();
        
        let cpu_usage: f32 = self.sys.cpus().iter()
            .map(|cpu| cpu.cpu_usage())
            .sum::<f32>() / self.sys.cpus().len() as f32;
        
        cpu_usage < 10.0 // System is idle if CPU usage < 10%
    }

    pub fn get_optimization_recommendations(&mut self) -> Vec<String> {
        let mut recommendations = Vec::new();
        let usage = self.get_current_usage();

        if usage.cpu_percent > self.config.max_cpu_percent {
            recommendations.push(format!(
                "CPU usage ({:.1}%) exceeds limit ({:.1}%). Consider reducing polling frequency.",
                usage.cpu_percent, self.config.max_cpu_percent
            ));
        }

        if usage.memory_mb > self.config.max_memory_mb {
            recommendations.push(format!(
                "Memory usage ({}MB) exceeds limit ({}MB). Consider reducing cache sizes.",
                usage.memory_mb, self.config.max_memory_mb
            ));
        }

        recommendations
    }
}

impl PerformanceOptimizer {
    pub fn new() -> Self {
        Self {
            sys: System::new_all(),
        }
    }

    pub fn analyze_and_optimize(&mut self) {
        self.sys.refresh_all();

        info!("Analyzing system for performance optimization...");

        // Identify high CPU usage processes
        let mut high_cpu_processes = Vec::new();
        for (pid, process) in self.sys.processes() {
            if process.cpu_usage() > 50.0 { // Threshold for high CPU usage
                high_cpu_processes.push((pid, process.name().to_string(), process.cpu_usage()));
            }
        }

        if !high_cpu_processes.is_empty() {
            warn!("High CPU usage detected in the following processes:");
            for (pid, name, cpu) in high_cpu_processes {
                warn!("  PID: {}, Name: {}, CPU: {:.2}%", pid, name, cpu);
                // In a real scenario, you might offer to terminate or reduce priority
                // For now, just logging
            }
        } else {
            info!("No high CPU usage processes detected.");
        }

        // Identify high memory usage processes
        let mut high_mem_processes = Vec::new();
        let total_memory = self.sys.total_memory();
        for (pid, process) in self.sys.processes() {
            let memory_percent = (process.memory() as f64 / total_memory as f64) * 100.0;
            if memory_percent > 10.0 { // Threshold for high memory usage
                high_mem_processes.push((pid, process.name().to_string(), memory_percent));
            }
        }

        if !high_mem_processes.is_empty() {
            warn!("High memory usage detected in the following processes:");
            for (pid, name, mem_percent) in high_mem_processes {
                warn!("  PID: {}, Name: {}, Memory: {:.2}%", pid, name, mem_percent);
                // In a real scenario, you might offer to terminate or suggest actions
            }
        } else {
            info!("No high memory usage processes detected.");
        }

        info!("Performance analysis complete.");
    }
}
