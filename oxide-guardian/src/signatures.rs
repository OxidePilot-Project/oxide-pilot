use std::collections::HashSet;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Default)]
pub struct SignatureDb {
    sha256: HashSet<String>,
    blake3: HashSet<String>,
}

impl SignatureDb {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load_from_path<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let content = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read signatures file: {e}"))?;
        // Try JSON first
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            let mut db = SignatureDb::new();
            if let Some(arr) = json.get("sha256").and_then(|v| v.as_array()) {
                for v in arr {
                    if let Some(s) = v.as_str() {
                        db.sha256.insert(s.to_lowercase());
                    }
                }
            }
            if let Some(arr) = json.get("blake3").and_then(|v| v.as_array()) {
                for v in arr {
                    if let Some(s) = v.as_str() {
                        db.blake3.insert(s.to_lowercase());
                    }
                }
            }
            return Ok(db);
        }
        // Fallback: newline-separated hex hashes (mixed types)
        let mut db = SignatureDb::new();
        for line in content.lines() {
            let h = line.trim().to_lowercase();
            if h.len() == 64 && h.chars().all(|c| c.is_ascii_hexdigit()) {
                // Could be sha256 or blake3; accept as sha256 by default
                db.sha256.insert(h);
            }
        }
        Ok(db)
    }

    pub fn contains_sha256(&self, hash: &str) -> bool {
        self.sha256.contains(&hash.to_lowercase())
    }
    pub fn contains_blake3(&self, hash: &str) -> bool {
        self.blake3.contains(&hash.to_lowercase())
    }

    pub fn add_sha256(&mut self, hash: String) {
        self.sha256.insert(hash.to_lowercase());
    }
    pub fn add_blake3(&mut self, hash: String) {
        self.blake3.insert(hash.to_lowercase());
    }
}
