use crate::scanner::ExternalVerdict;
use std::thread::sleep;
use std::time::Duration;

pub fn virustotal_lookup(sha256: &str, api_key: &str) -> Result<ExternalVerdict, String> {
    let url = format!("https://www.virustotal.com/api/v3/files/{}", sha256);
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {e}"))?;

    let mut attempt: u32 = 0;
    let max_attempts: u32 = 3;
    let mut backoff_ms: u64 = 500; // start with 0.5s

    loop {
        attempt += 1;
        let resp = client
            .get(&url)
            .header("x-apikey", api_key)
            .header(reqwest::header::ACCEPT, "application/json")
            .header(reqwest::header::USER_AGENT, "OxideGuardian/1.0 (+https://github.com/oxide-pilot)")
            .send()
            .map_err(|e| format!("VirusTotal request failed: {e}"))?;

        let status = resp.status();
        if status.is_success() {
            let v: serde_json::Value = resp
                .json()
                .map_err(|e| format!("VirusTotal JSON parse error: {e}"))?;

            let mut malicious = false;
            let mut engines = Vec::new();

            if let Some(results) = v
                .get("data")
                .and_then(|d| d.get("attributes"))
                .and_then(|a| a.get("last_analysis_results"))
                .and_then(|r| r.as_object())
            {
                for (engine, detail) in results {
                    let category = detail.get("category").and_then(|x| x.as_str()).unwrap_or("");
                    let result = detail.get("result").and_then(|x| x.as_str()).unwrap_or("");
                    if category == "malicious" || category == "suspicious" {
                        malicious = true;
                        engines.push((engine.clone(), result.to_string()));
                    }
                }
            }

            return Ok(ExternalVerdict {
                malicious,
                engine_detections: engines,
                reference: v
                    .get("data")
                    .and_then(|d| d.get("links"))
                    .and_then(|l| l.get("self"))
                    .and_then(|s| s.as_str())
                    .map(|s| s.to_string()),
            });
        }

        let code = status.as_u16();
        if code == 404 {
            // Unknown hash in VT database; treat as not malicious with no detections.
            return Ok(ExternalVerdict { malicious: false, engine_detections: Vec::new(), reference: None });
        }

        // Retry on 429 and 5xx with exponential backoff + jitter (bounded attempts)
        if (code == 429 || (500..600).contains(&code)) && attempt < max_attempts {
            let jitter = (attempt as u64 * 37) % 113; // small deterministic jitter
            sleep(Duration::from_millis(backoff_ms + jitter));
            backoff_ms = (backoff_ms * 2).min(5_000);
            continue;
        }

        return Err(format!("VirusTotal returned HTTP {}", status));
    }
}
