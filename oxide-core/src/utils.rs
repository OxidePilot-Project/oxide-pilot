/*!
Utilidades comunes del sistema
*/

use crate::types::OxideResult;
use std::path::Path;
use tracing::{error, info};

/// Inicializar el sistema de logging
pub fn init_logging(level: &str, log_dir: &Path) -> OxideResult<()> {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

    // Crear directorio de logs si no existe
    if !log_dir.exists() {
        std::fs::create_dir_all(log_dir)?;
    }

    let level = match level.to_lowercase().as_str() {
        "trace" => tracing::Level::TRACE,
        "debug" => tracing::Level::DEBUG,
        "info" => tracing::Level::INFO,
        "warn" => tracing::Level::WARN,
        "error" => tracing::Level::ERROR,
        _ => tracing::Level::INFO,
    };

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(false)
                .with_thread_ids(true)
                .with_level(true)
        )
        .with(tracing_subscriber::filter::LevelFilter::from_level(level))
        .init();

    info!("Sistema de logging inicializado en nivel: {}", level);
    Ok(())
}

/// Formatear bytes en formato legible
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.2} {}", size, UNITS[unit_index])
}

/// Formatear porcentaje
pub fn format_percentage(value: f32) -> String {
    format!("{:.1}%", value)
}

/// Validar que un proceso es seguro para interactuar
pub fn is_safe_process(process_name: &str) -> bool {
    const SYSTEM_PROCESSES: &[&str] = &[
        "System",
        "Registry",
        "smss.exe",
        "csrss.exe",
        "wininit.exe",
        "winlogon.exe",
        "services.exe",
        "lsass.exe",
        "svchost.exe",
        "dwm.exe",
    ];

    !SYSTEM_PROCESSES.iter().any(|&sys_proc| {
        process_name.to_lowercase().contains(&sys_proc.to_lowercase())
    })
}

/// Sanitizar entrada de usuario
pub fn sanitize_input(input: &str) -> String {
    input
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace() || ".,!?-_".contains(*c))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1048576), "1.00 MB");
        assert_eq!(format_bytes(1073741824), "1.00 GB");
    }

    #[test]
    fn test_format_percentage() {
        assert_eq!(format_percentage(75.5), "75.5%");
        assert_eq!(format_percentage(100.0), "100.0%");
    }

    #[test]
    fn test_is_safe_process() {
        assert!(!is_safe_process("System"));
        assert!(!is_safe_process("lsass.exe"));
        assert!(is_safe_process("notepad.exe"));
        assert!(is_safe_process("chrome.exe"));
    }

    #[test]
    fn test_sanitize_input() {
        assert_eq!(sanitize_input("Hello, World!"), "Hello, World!");
        assert_eq!(sanitize_input("Test<script>"), "Testscript");
        assert_eq!(sanitize_input("Normal text 123"), "Normal text 123");
    }
}