use oxide_guardian::monitor::SystemMonitor;

#[test]
fn test_system_monitor_cpu_usage() {
    let mut monitor = SystemMonitor::new();
    monitor.refresh_system();
    let cpu_usage = monitor.get_cpu_usage();
    // CPU usage can be 0 if the system is idle or during initial refresh
    // We just check if it doesn't panic and is within a reasonable range [0, 100]
    assert!(cpu_usage >= 0.0 && cpu_usage <= 100.0);
}

#[test]
fn test_system_monitor_memory_usage() {
    let mut monitor = SystemMonitor::new();
    monitor.refresh_system();
    let (used_mem, total_mem) = monitor.get_memory_usage();
    assert!(total_mem > 0);
    assert!(used_mem <= total_mem);
}

#[test]
fn test_system_monitor_list_processes() {
    let mut monitor = SystemMonitor::new();
    monitor.refresh_system();
    let processes = monitor.list_processes();
    assert!(!processes.is_empty());
    // Check if at least one process has a name and PID
    assert!(processes.iter().any(|p| p.details["name"].is_string() && p.details["pid"].is_string()));
}
