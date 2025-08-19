use crate::signatures::SignatureDb;
use crate::quarantine;
use sha2::{Digest, Sha256};
use blake3;
use std::fs::File;
use std::io::{Read, BufReader};
use std::path::Path;

#[derive(Debug, Clone, serde::Serialize)]
pub struct FileHashes {
    pub sha256: String,
    pub blake3: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct FileScanReport {
    pub path: String,
    pub size: u64,
    pub hashes: FileHashes,
    pub local_match: Option<String>,
    pub external_verdict: Option<ExternalVerdict>,
    pub malicious: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ExternalVerdict {
    pub malicious: bool,
    pub engine_detections: Vec<(String, String)>,
    pub reference: Option<String>,
}

pub struct FileScanner {
    sigdb: Option<SignatureDb>,
    max_file_size_bytes: Option<u64>,
}

impl FileScanner {
    pub fn new(sigdb: Option<SignatureDb>, max_file_size_mb: Option<u64>) -> Self {
        Self {
            sigdb,
            max_file_size_bytes: max_file_size_mb.map(|mb| mb * 1024 * 1024),
        }
    }

    pub fn compute_hashes<P: AsRef<Path>>(path: P) -> Result<(FileHashes, u64), String> {
        let file = File::open(&path).map_err(|e| format!("Failed to open file: {e}"))?;
        let metadata = file.metadata().map_err(|e| format!("Failed to read metadata: {e}"))?;
        let size = metadata.len();
        let mut reader = BufReader::new(file);

        let mut sha = Sha256::new();
        let mut blake3_hasher = blake3::Hasher::new();
        let mut buf = [0u8; 64 * 1024];

        loop {
            let n = reader.read(&mut buf).map_err(|e| format!("Failed to read file: {e}"))?;
            if n == 0 { break; }
            sha.update(&buf[..n]);
            blake3_hasher.update(&buf[..n]);
        }

        let sha256 = format!("{:x}", sha.finalize());
        let blake3 = blake3_hasher.finalize().to_hex().to_string();

        Ok((FileHashes { sha256, blake3 }, size))
    }

    pub fn scan_local<P: AsRef<Path>>(&self, path: P) -> Result<FileScanReport, String> {
        if let Some(limit) = self.max_file_size_bytes {
            let md = std::fs::metadata(&path).map_err(|e| format!("Failed to stat file: {e}"))?;
            if md.len() > limit {
                return Err("File exceeds max_file_size limit".to_string());
            }
        }

        let (hashes, size) = Self::compute_hashes(&path)?;
        let mut local_match = None;
        if let Some(db) = &self.sigdb {
            if db.contains_sha256(&hashes.sha256) {
                local_match = Some("sha256".to_string());
            } else if db.contains_blake3(&hashes.blake3) {
                local_match = Some("blake3".to_string());
            }
        }

        Ok(FileScanReport {
            path: path.as_ref().to_string_lossy().to_string(),
            size,
            hashes,
            local_match: local_match.clone(),
            external_verdict: None,
            malicious: local_match.is_some(),
        })
    }

    pub fn quarantine_if_malicious<P: AsRef<Path>>(
        &self,
        report: &FileScanReport,
        quarantine_dir: Option<P>,
    ) -> Result<Option<String>, String> {
        if report.malicious {
            if let Some(dir) = quarantine_dir {
                let new_path = quarantine::move_to_quarantine(&report.path, dir.as_ref())?;
                return Ok(Some(new_path));
            }
        }
        Ok(None)
    }
}
