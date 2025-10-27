use std::fs;
use std::path::{Path, PathBuf};

pub fn move_to_quarantine<S: AsRef<Path>, D: AsRef<Path>>(
    src: S,
    quarantine_dir: D,
) -> Result<String, String> {
    let src_path = src.as_ref();
    let q_dir = quarantine_dir.as_ref();

    fs::create_dir_all(q_dir).map_err(|e| format!("Failed to create quarantine dir: {e}"))?;

    let file_name = src_path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "quarantined_file".to_string());

    let mut dest: PathBuf = q_dir.to_path_buf();
    let stamped = format!("{}_{}", chrono::Utc::now().timestamp(), file_name);
    dest.push(stamped);

    fs::rename(src_path, &dest).map_err(|e| format!("Failed to move to quarantine: {e}"))?;

    Ok(dest.to_string_lossy().to_string())
}
