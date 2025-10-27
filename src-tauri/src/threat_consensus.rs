use chrono::Utc;
use log::{error, info, warn};
use oxide_core::gemini_auth::GeminiAuth;
use oxide_core::openai_client::{self, ChatMessage};
use oxide_core::qwen_auth::QwenAuth;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Indicator {
    pub kind: String,
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
}

async fn analyze_with_openai(snapshot: &Value) -> Result<ModelReport, String> {
    // Build prompt with strict JSON requirement
    let prompt = format!(
        r#"
    You are a security threat analyst. Analyze the JSON system snapshot and return STRICT JSON with keys:
    risk_score (0-100), confidence (0-1), findings[], indicators[], recommendations[], citations[]
    JSON only, no prose.

    Snapshot:
    {}
    "#,
        snapshot
    );

    let model_name = std::env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o".to_string());
    let messages = vec![
        ChatMessage {
            role: "system".into(),
            content: "You are a concise, technical security analyst. JSON output only.".into(),
        },
        ChatMessage {
            role: "user".into(),
            content: prompt,
        },
    ];

    match openai_client::chat_completion(&model_name, messages, Some(0.1), None).await {
        Ok(text) => match serde_json::from_str::<ModelReport>(&text) {
            Ok(mut mr) => {
                mr.provider = "openai".to_string();
                Ok(mr)
            }
            Err(e) => {
                warn!(
                    "OpenAI JSON parse failed, returning low-confidence fallback: {}",
                    e
                );
                Ok(ModelReport {
                    provider: "openai".to_string(),
                    risk_score: 15.0,
                    confidence: 0.3,
                    findings: vec![],
                    indicators: vec![],
                    recommendations: vec![
                        "Manual review recommended; model returned unstructured output".to_string(),
                    ],
                    citations: vec![],
                })
            }
        },
        Err(e) => {
            error!("OpenAI analysis error: {}", e);
            Err(e.to_string())
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Citation {
    pub title: String,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snippet: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatFinding {
    pub id: String,
    pub kind: String,     // process|file|network|config
    pub severity: String, // low|medium|high|critical
    pub summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,
    #[serde(default)]
    pub indicators: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatReport {
    pub risk_score: f32,
    pub confidence: f32,
    pub mode: String,           // dual|single
    pub providers: Vec<String>, // ["gemini", "qwen"] etc.
    #[serde(default)]
    pub findings: Vec<ThreatFinding>,
    #[serde(default)]
    pub indicators: Vec<Indicator>,
    #[serde(default)]
    pub recommendations: Vec<String>,
    #[serde(default)]
    pub citations: Vec<Citation>,
    #[serde(default)]
    pub disagreement_alerts: Vec<String>,
    pub evidence: Value,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ModelReport {
    pub provider: String,
    pub risk_score: f32,
    pub confidence: f32,
    #[serde(default)]
    pub findings: Vec<ThreatFinding>,
    #[serde(default)]
    pub indicators: Vec<Indicator>,
    #[serde(default)]
    pub recommendations: Vec<String>,
    #[serde(default)]
    pub citations: Vec<Citation>,
}

fn normalize_score(v: f32) -> f32 {
    v.clamp(0.0, 100.0)
}
fn normalize_conf(v: f32) -> f32 {
    v.clamp(0.0, 1.0)
}

fn aggregate(reports: Vec<ModelReport>, evidence: Value) -> ThreatReport {
    if reports.is_empty() {
        return ThreatReport {
            risk_score: 0.0,
            confidence: 0.0,
            mode: "single".to_string(),
            providers: vec![],
            findings: vec![],
            indicators: vec![],
            recommendations: vec!["No providers available; unable to analyze".to_string()],
            citations: vec![],
            disagreement_alerts: vec!["No model reports".to_string()],
            evidence,
            timestamp: Utc::now().to_rfc3339(),
        };
    }

    let providers: Vec<String> = reports.iter().map(|r| r.provider.clone()).collect();
    let mode = if providers.len() >= 2 {
        "dual"
    } else {
        "single"
    }
    .to_string();

    // Weighted average by confidence
    let mut num = 0.0f32;
    let mut den = 0.0f32;
    for r in &reports {
        let c = normalize_conf(r.confidence);
        num += normalize_score(r.risk_score) * c.max(0.01);
        den += c.max(0.01);
    }
    let risk_score = if den > 0.0 { num / den } else { 0.0 };
    let confidence = (reports
        .iter()
        .map(|r| normalize_conf(r.confidence))
        .sum::<f32>()
        / reports.len() as f32)
        .clamp(0.0, 1.0);

    // Merge lists with simple concatenation + de-dup for indicators
    let mut findings = vec![];
    let mut indicators: Vec<Indicator> = vec![];
    let mut recommendations = vec![];
    let mut citations = vec![];
    for r in reports.into_iter() {
        findings.extend(r.findings);
        for ind in r.indicators {
            if !indicators
                .iter()
                .any(|x| x.kind == ind.kind && x.value == ind.value)
            {
                indicators.push(ind);
            }
        }
        for rec in r.recommendations {
            if !recommendations.contains(&rec) {
                recommendations.push(rec);
            }
        }
        for cit in r.citations {
            if !citations.iter().any(|c: &Citation| c.url == cit.url) {
                citations.push(cit);
            }
        }
    }

    // Simple disagreement heuristic: if findings > threshold and providers >=2
    let mut disagreement_alerts = vec![];
    if providers.len() >= 2 {
        // naive: if many distinct findings kinds, warn
        let kinds: std::collections::HashSet<_> = findings.iter().map(|f| f.kind.clone()).collect();
        if kinds.len() >= 4 {
            disagreement_alerts
                .push("High diversity of finding kinds; review manually".to_string());
        }
    }

    ThreatReport {
        risk_score,
        confidence,
        mode,
        providers,
        findings,
        indicators,
        recommendations,
        citations,
        disagreement_alerts,
        evidence,
        timestamp: Utc::now().to_rfc3339(),
    }
}

async fn analyze_with_gemini(snapshot: &Value, grounded: bool) -> Result<ModelReport, String> {
    let auth = GeminiAuth::new();

    // Enforce JSON output. If grounding no está realmente disponible, el modelo debe seguir la instrucción.
    let grounding_text = if grounded { "When possible," } else { "" };
    let snapshot_str =
        serde_json::to_string_pretty(snapshot).unwrap_or_else(|_| snapshot.to_string());

    let prompt = format!(
        r#"
    You are a security threat analyst. Analyze the following JSON system snapshot and produce a STRICT JSON object with this shape:
    {{
      "risk_score": number (0-100),
      "confidence": number (0-1),
      "findings": [{{"id": string, "kind": "process|file|network|config", "severity": "low|medium|high|critical", "summary": string, "rationale": string, "indicators": string[]}}],
      "indicators": [{{"kind": "hash|domain|path|proc", "value": string, "context": string}}],
      "recommendations": string[],
      "citations": [{{"title": string, "url": string, "snippet": string}}]
    }}

    Requirements:
    - Output MUST be a single JSON object only, no prose.
    - {} Use Google Search to verify suspicious indicators and include citations to authoritative sources (CVE pages, vendors, security writeups). If not available, still return the JSON.

    Snapshot:
    {}
    "#,
        grounding_text, snapshot_str
    );

    match auth.send_message(&prompt, Some("gemini-1.5-pro")).await {
        Ok(text) => {
            // Try to parse JSON
            match serde_json::from_str::<ModelReport>(&text) {
                Ok(mut mr) => {
                    mr.provider = "gemini".to_string();
                    Ok(mr)
                }
                Err(e) => {
                    warn!(
                        "Gemini JSON parse failed, returning low-confidence fallback: {}",
                        e
                    );
                    Ok(ModelReport {
                        provider: "gemini".to_string(),
                        risk_score: 15.0,
                        confidence: 0.3,
                        findings: vec![],
                        indicators: vec![],
                        recommendations: vec![
                            "Manual review recommended; model returned unstructured output"
                                .to_string(),
                        ],
                        citations: vec![],
                    })
                }
            }
        }
        Err(e) => {
            error!("Gemini analysis error: {}", e);
            Err(e.to_string())
        }
    }
}

async fn analyze_with_qwen(snapshot: &Value) -> Result<ModelReport, String> {
    // Build prompt for JSON-only output
    let snapshot_str =
        serde_json::to_string_pretty(snapshot).unwrap_or_else(|_| snapshot.to_string());
    let prompt = format!(
        r#"
    You are a security threat analyst. Analyze the JSON system snapshot and return STRICT JSON with:
    risk_score, confidence, findings[], indicators[], recommendations[], citations[]
    No prose, JSON only.

    Snapshot:
    {}
    "#,
        snapshot_str
    );

    // Get auth header via QwenAuth helper
    let qauth = QwenAuth::new();
    let auth_header = qauth.get_auth_header().await.map_err(|e| e.to_string())?;
    let base =
        std::env::var("QWEN_API_BASE").map_err(|_| "Missing env QWEN_API_BASE".to_string())?;
    let path = std::env::var("QWEN_CHAT_COMPLETIONS_PATH")
        .unwrap_or_else(|_| "/v1/chat/completions".to_string());
    let url = format!("{}{}", base, path);
    let model_name = std::env::var("QWEN_MODEL").unwrap_or_else(|_| "qwen-plus".to_string());

    let body = serde_json::json!({
      "model": model_name,
      "messages": [
        {"role": "system", "content": "You are a concise, technical security analyst. JSON output only."},
        {"role": "user", "content": prompt}
      ],
      "temperature": 0.1
    });

    let client = Client::new();
    let resp = client
        .post(&url)
        .header("Authorization", auth_header)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Qwen API error: {} - {}", status, text));
    }

    let v: Value = resp.json().await.map_err(|e| e.to_string())?;
    let text = v
        .get("choices")
        .and_then(|c| c.as_array())
        .and_then(|a| a.get(0))
        .and_then(|x| x.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(|t| t.as_str())
        .ok_or_else(|| "Unexpected Qwen response format".to_string())?;

    match serde_json::from_str::<ModelReport>(text) {
        Ok(mut mr) => {
            mr.provider = "qwen".to_string();
            Ok(mr)
        }
        Err(e) => {
            warn!(
                "Qwen JSON parse failed, returning low-confidence fallback: {}",
                e
            );
            Ok(ModelReport {
                provider: "qwen".to_string(),
                risk_score: 15.0,
                confidence: 0.3,
                findings: vec![],
                indicators: vec![],
                recommendations: vec![
                    "Manual review recommended; model returned unstructured output".to_string(),
                ],
                citations: vec![],
            })
        }
    }
}

pub async fn run_consensus(snapshot: Value, _grounded: bool) -> Result<ThreatReport, String> {
    let t0 = std::time::Instant::now();
    // Availability: Gemini, Qwen and OpenAI if authenticated
    let mut providers: Vec<&str> = vec![];

    // Gemini availability (OAuth only)
    let g_available = match oxide_core::google_auth::get_access_token().await {
        Ok(Some(_)) => true,
        _ => false,
    };
    if g_available {
        providers.push("gemini");
    }

    // Qwen availability
    let q_available =
        QwenAuth::new().get_auth_header().await.is_ok() && std::env::var("QWEN_API_BASE").is_ok();
    if q_available {
        providers.push("qwen");
    }

    // OpenAI availability (API Key)
    let o_available = match oxide_core::openai_key::get_api_key().await {
        Ok(Some(_)) => true,
        _ => false,
    };
    if o_available {
        providers.push("openai");
    }

    info!("Consensus starting with providers: {:?}", providers);
    if providers.is_empty() {
        return Err("No LLM providers available (Gemini, Qwen, or OpenAI)".to_string());
    }

    // Launch available analyses in parallel
    let g_fut = if g_available {
        Some(analyze_with_gemini(&snapshot, true))
    } else {
        None
    };
    let q_fut = if q_available {
        Some(analyze_with_qwen(&snapshot))
    } else {
        None
    };
    let o_fut = if o_available {
        Some(analyze_with_openai(&snapshot))
    } else {
        None
    };

    let (g_res, q_res, o_res) = tokio::join!(
        async {
            if let Some(f) = g_fut {
                f.await
            } else {
                Err("gemini_unavailable".into())
            }
        },
        async {
            if let Some(f) = q_fut {
                f.await
            } else {
                Err("qwen_unavailable".into())
            }
        },
        async {
            if let Some(f) = o_fut {
                f.await
            } else {
                Err("openai_unavailable".into())
            }
        },
    );

    let mut reports: Vec<ModelReport> = vec![];
    if let Ok(r) = g_res {
        reports.push(r);
    }
    if let Ok(r) = q_res {
        reports.push(r);
    }
    if let Ok(r) = o_res {
        reports.push(r);
    }

    // Log per-provider confidence and score for debugging
    for r in &reports {
        info!(
            "Provider report: {} -> risk_score={:.1}, confidence={:.2}",
            r.provider, r.risk_score, r.confidence
        );
    }
    let elapsed_ms = t0.elapsed().as_millis();
    info!(
        "Consensus completed in {} ms ({} reports)",
        elapsed_ms,
        reports.len()
    );

    Ok(aggregate(reports, snapshot))
}

pub fn recommendations_from_report(rep: &ThreatReport) -> Vec<String> {
    let mut out = rep.recommendations.clone();
    if rep.risk_score >= 70.0 {
        out.push("High risk detected: enable containment mode and review suspicious processes immediately".to_string());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aggregate_empty_reports() {
        let ev = serde_json::json!({"status":"ok"});
        let rep = aggregate(vec![], ev.clone());
        assert_eq!(rep.risk_score, 0.0);
        assert_eq!(rep.mode, "single");
        assert!(rep.providers.is_empty());
        assert!(rep
            .recommendations
            .iter()
            .any(|r| r.contains("unable to analyze")));
        assert_eq!(rep.evidence["status"], "ok");
    }

    #[test]
    fn aggregate_single() {
        let ev = serde_json::json!({"k":1});
        let r1 = ModelReport {
            provider: "gemini".into(),
            risk_score: 80.0,
            confidence: 0.9,
            findings: vec![ThreatFinding {
                id: "p1".into(),
                kind: "process".into(),
                severity: "high".into(),
                summary: "sus".into(),
                rationale: None,
                indicators: vec!["pid:1".into()],
            }],
            indicators: vec![Indicator {
                kind: "proc".into(),
                value: "pid:1".into(),
                context: None,
            }],
            recommendations: vec!["kill pid 1".into()],
            citations: vec![],
        };
        let rep = aggregate(vec![r1], ev);
        assert_eq!(rep.mode, "single");
        assert_eq!(rep.providers, vec!["gemini"]);
        assert!(rep.risk_score >= 79.0 && rep.risk_score <= 81.0);
        assert!(rep.findings.iter().any(|f| f.id == "p1"));
        let recs = recommendations_from_report(&rep);
        assert!(recs.iter().any(|r| r.contains("containment")));
    }

    #[test]
    fn aggregate_dual_weighted() {
        let ev = serde_json::json!({});
        let r1 = ModelReport {
            provider: "gemini".into(),
            risk_score: 90.0,
            confidence: 1.0,
            findings: vec![],
            indicators: vec![],
            recommendations: vec!["A".into()],
            citations: vec![],
        };
        let r2 = ModelReport {
            provider: "qwen".into(),
            risk_score: 10.0,
            confidence: 0.1,
            findings: vec![],
            indicators: vec![],
            recommendations: vec!["B".into()],
            citations: vec![],
        };
        let rep = aggregate(vec![r1, r2], ev);
        // Weighted towards gemini
        assert!(rep.risk_score > 80.0);
        assert_eq!(rep.mode, "dual");
        assert!(rep.providers.contains(&"gemini".into()) && rep.providers.contains(&"qwen".into()));
    }
}
