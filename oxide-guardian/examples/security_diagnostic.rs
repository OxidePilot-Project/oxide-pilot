// Oxide Guardian - Advanced Security Diagnostic Tool
// Utiliza las capacidades nativas de Rust del Guardian Agent

use chrono::Utc;
use oxide_guardian::monitor::SystemMonitor;
use serde_json;

#[derive(serde::Serialize)]
struct SecurityReport {
    timestamp: String,
    system_info: SystemInfo,
    process_analysis: ProcessAnalysis,
    network_analysis: NetworkAnalysis,
    threat_level: ThreatLevel,
    recommendations: Vec<String>,
}

#[derive(serde::Serialize)]
struct SystemInfo {
    cpu_usage: f32,
    memory_used_gb: f64,
    memory_total_gb: f64,
    memory_percent: f64,
}

#[derive(serde::Serialize)]
struct ProcessAnalysis {
    total_processes: usize,
    high_cpu_processes: Vec<ProcessInfo>,
    suspicious_processes: Vec<ProcessInfo>,
}

#[derive(serde::Serialize, Clone)]
struct ProcessInfo {
    name: String,
    pid: String,
    cpu_usage: f64,
    memory_mb: f64,
    command: String,
}

#[derive(serde::Serialize)]
struct NetworkAnalysis {
    total_bytes_received: u64,
    total_bytes_transmitted: u64,
    active_interfaces: usize,
}

#[derive(serde::Serialize, Debug, Clone)]
enum ThreatLevel {
    Clean,
    Low,
    Medium,
    High,
    Critical,
}

fn main() {
    println!("╔════════════════════════════════════════════════════════╗");
    println!("║   OXIDE GUARDIAN - ADVANCED SECURITY DIAGNOSTIC        ║");
    println!("║   Análisis Profundo del Sistema                        ║");
    println!("╚════════════════════════════════════════════════════════╝");
    println!();

    let mut monitor = SystemMonitor::new();
    monitor.refresh_system();

    // 1. Análisis de Sistema
    println!("🔍 [1/4] Analizando recursos del sistema...");
    let cpu_usage = monitor.get_cpu_usage();
    let (memory_used, memory_total) = monitor.get_memory_usage();
    let memory_used_gb = memory_used as f64 / (1024.0 * 1024.0 * 1024.0);
    let memory_total_gb = memory_total as f64 / (1024.0 * 1024.0 * 1024.0);
    let memory_percent = (memory_used as f64 / memory_total as f64) * 100.0;

    println!("   CPU: {:.2}%", cpu_usage);
    println!(
        "   RAM: {:.2} GB / {:.2} GB ({:.1}%)",
        memory_used_gb, memory_total_gb, memory_percent
    );

    let system_info = SystemInfo {
        cpu_usage,
        memory_used_gb,
        memory_total_gb,
        memory_percent,
    };

    // 2. Análisis de Procesos
    println!("\n🔍 [2/4] Analizando procesos en ejecución...");
    let process_events = monitor.list_processes();

    let mut high_cpu_processes = Vec::new();
    let mut suspicious_processes = Vec::new();

    for event in &process_events {
        if let Some(cpu) = event.details.get("cpu_usage") {
            if let Some(cpu_val) = cpu.as_f64() {
                if cpu_val > 20.0 {
                    let proc_info = ProcessInfo {
                        name: event
                            .details
                            .get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown")
                            .to_string(),
                        pid: event
                            .details
                            .get("pid")
                            .and_then(|v| v.as_str())
                            .unwrap_or("0")
                            .to_string(),
                        cpu_usage: cpu_val,
                        memory_mb: event
                            .details
                            .get("memory_usage")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0) as f64
                            / (1024.0 * 1024.0),
                        command: event
                            .details
                            .get("command")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                    };

                    high_cpu_processes.push(proc_info.clone());

                    // Detectar nombres sospechosos
                    let suspicious_names = [
                        "keylogger",
                        "backdoor",
                        "trojan",
                        "rootkit",
                        "mimikatz",
                        "psexec",
                        "metasploit",
                        "netcat",
                    ];
                    let name_lower = proc_info.name.to_lowercase();

                    if suspicious_names.iter().any(|&s| name_lower.contains(s)) {
                        suspicious_processes.push(proc_info);
                    }
                }
            }
        }
    }

    println!("   Total de procesos: {}", process_events.len());
    println!(
        "   Procesos con alto CPU (>20%): {}",
        high_cpu_processes.len()
    );
    println!(
        "   Procesos sospechosos detectados: {}",
        suspicious_processes.len()
    );

    if !suspicious_processes.is_empty() {
        println!("\n   ⚠️  ALERTA: Procesos sospechosos encontrados:");
        for proc in &suspicious_processes {
            println!(
                "      - {} (PID: {}, CPU: {:.1}%)",
                proc.name, proc.pid, proc.cpu_usage
            );
        }
    }

    let process_analysis = ProcessAnalysis {
        total_processes: process_events.len(),
        high_cpu_processes: high_cpu_processes.clone(),
        suspicious_processes: suspicious_processes.clone(),
    };

    // 3. Análisis de Red
    println!("\n🔍 [3/4] Analizando actividad de red...");
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

    println!("   Interfaces activas: {}", network_events.len());
    println!(
        "   Datos recibidos: {:.2} MB",
        total_received as f64 / (1024.0 * 1024.0)
    );
    println!(
        "   Datos transmitidos: {:.2} MB",
        total_transmitted as f64 / (1024.0 * 1024.0)
    );

    let network_analysis = NetworkAnalysis {
        total_bytes_received: total_received,
        total_bytes_transmitted: total_transmitted,
        active_interfaces: network_events.len(),
    };

    // 4. Evaluación del Nivel de Amenaza
    println!("\n🔍 [4/4] Evaluando nivel de amenaza...");

    let threat_level = if !suspicious_processes.is_empty() {
        ThreatLevel::Critical
    } else if high_cpu_processes.len() > 10 {
        ThreatLevel::Medium
    } else if cpu_usage > 80.0 || memory_percent > 90.0 {
        ThreatLevel::Medium
    } else if high_cpu_processes.len() > 5 {
        ThreatLevel::Low
    } else {
        ThreatLevel::Clean
    };

    println!("   Nivel de amenaza: {:?}", threat_level);

    // 5. Generar Recomendaciones
    let mut recommendations = Vec::new();

    match threat_level {
        ThreatLevel::Critical => {
            recommendations
                .push("⚠️  CRÍTICO: Se detectaron procesos altamente sospechosos".to_string());
            recommendations.push("1. Desconectar inmediatamente de internet".to_string());
            recommendations.push("2. Ejecutar análisis completo de antivirus".to_string());
            recommendations.push("3. Considerar reinstalación del sistema operativo".to_string());
            recommendations
                .push("4. Cambiar todas las contraseñas desde otro dispositivo".to_string());
            recommendations.push("5. Contactar con un profesional de seguridad".to_string());
        }
        ThreatLevel::High => {
            recommendations.push("⚠️  ALTO: Actividad sospechosa detectada".to_string());
            recommendations.push("1. Ejecutar análisis completo de antivirus".to_string());
            recommendations.push("2. Revisar procesos y conexiones manualmente".to_string());
            recommendations.push("3. Actualizar sistema operativo y software".to_string());
        }
        ThreatLevel::Medium => {
            recommendations.push("Rendimiento inusual detectado".to_string());
            recommendations.push("1. Revisar procesos con alto consumo de recursos".to_string());
            recommendations.push("2. Ejecutar análisis de malware".to_string());
            recommendations.push("3. Verificar programas de inicio automático".to_string());
        }
        ThreatLevel::Low => {
            recommendations.push("Sistema aparentemente normal con alertas menores".to_string());
            recommendations.push("1. Monitorear el sistema regularmente".to_string());
            recommendations.push("2. Mantener el antivirus actualizado".to_string());
        }
        ThreatLevel::Clean => {
            recommendations
                .push("✓ Sistema limpio - No se detectaron amenazas evidentes".to_string());
            recommendations.push("1. Mantener buenas prácticas de seguridad".to_string());
            recommendations.push("2. Realizar análisis periódicos".to_string());
            recommendations.push("3. Mantener el sistema actualizado".to_string());
        }
    }

    // 6. Generar Reporte
    let report = SecurityReport {
        timestamp: Utc::now().to_rfc3339(),
        system_info,
        process_analysis,
        network_analysis,
        threat_level: threat_level.clone(),
        recommendations: recommendations.clone(),
    };

    // Guardar reporte JSON
    let report_path = "oxide-guardian-report.json";
    if let Ok(json) = serde_json::to_string_pretty(&report) {
        if std::fs::write(report_path, json).is_ok() {
            println!("\n📄 Reporte guardado en: {}", report_path);
        }
    }

    // Mostrar resumen
    println!("\n╔════════════════════════════════════════════════════════╗");
    println!("║                  RESUMEN DEL ANÁLISIS                  ║");
    println!("╚════════════════════════════════════════════════════════╝");
    println!();

    for rec in &recommendations {
        println!("  {}", rec);
    }

    println!("\n═══════════════════════════════════════════════════════════");
    println!(
        "Análisis completado: {}",
        Utc::now().format("%Y-%m-%d %H:%M:%S")
    );
    println!("═══════════════════════════════════════════════════════════\n");

    // Código de salida según nivel de amenaza
    let exit_code = match threat_level {
        ThreatLevel::Clean => 0,
        ThreatLevel::Low => 1,
        ThreatLevel::Medium => 2,
        ThreatLevel::High => 3,
        ThreatLevel::Critical => 4,
    };

    std::process::exit(exit_code);
}
