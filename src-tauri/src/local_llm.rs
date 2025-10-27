use serde::{Deserialize, Serialize};
use std::env;
use std::process::Stdio;
use tokio::process::Command;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LocalLlmServerStatus {
    pub running: bool,
    pub port: Option<u16>,
    pub message: Option<String>,
}

async fn run_lms<I, S>(args: I) -> Result<String, String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    // Resolve LM Studio CLI binary from env or fallback to `lms`
    let bin = env::var("LMSTUDIO_CLI")
        .or_else(|_| env::var("LMS_CLI"))
        .unwrap_or_else(|_| "lms".to_string());
    let mut cmd = Command::new(bin);
    for a in args {
        cmd.arg(a.as_ref());
    }
    let output = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| format!("Failed to start lms: {e}"))?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    if !output.status.success() {
        return Err(format!(
            "lms exited with code {:?}: {}",
            output.status.code(),
            if stderr.trim().is_empty() {
                stdout
            } else {
                stderr
            }
        ));
    }
    Ok(stdout)
}

pub async fn server_start(port: Option<u16>, cors: bool) -> Result<String, String> {
    let mut args: Vec<String> = vec!["server".into(), "start".into()];
    if let Some(p) = port {
        args.push("--port".into());
        args.push(p.to_string());
    }
    if cors {
        args.push("--cors".into());
    }
    run_lms(args.iter().map(|s| s.as_str())).await
}

pub async fn server_stop() -> Result<String, String> {
    let args = ["server", "stop"];
    run_lms(args).await
}

pub async fn server_status() -> Result<LocalLlmServerStatus, String> {
    let args = ["server", "status"];
    let out = run_lms(args).await?;
    // Heuristic parse: look for "running" and a port number
    let lower = out.to_lowercase();
    let running = lower.contains("running") && !lower.contains("stopped");
    // Try extract a port like :1234 or on port 1234
    let port = lower
        .split([' ', '\n', '\r', '\t'])
        .filter_map(|tok| tok.trim_start_matches(':').parse::<u16>().ok())
        .next();
    Ok(LocalLlmServerStatus {
        running,
        port,
        message: Some(out),
    })
}

pub async fn ls_json() -> Result<String, String> {
    let args = ["ls", "--json"];
    run_lms(args).await
}

pub async fn get_model(model_spec: &str, gguf: bool, yes: bool) -> Result<String, String> {
    let mut args: Vec<String> = vec!["get".into(), model_spec.to_string()];
    if gguf {
        args.push("--gguf".into());
    }
    if yes {
        args.push("--yes".into());
    }
    run_lms(args.iter().map(|s| s.as_str())).await
}

pub async fn load_model(
    model_key: &str,
    identifier: Option<&str>,
    context_len: Option<u32>,
    gpu: Option<&str>,
    ttl_secs: Option<u32>,
) -> Result<String, String> {
    let mut args: Vec<String> = vec!["load".into(), model_key.to_string()];
    if let Some(id) = identifier {
        args.push("--identifier".into());
        args.push(id.to_string());
    }
    if let Some(cl) = context_len {
        args.push("--context-length".into());
        args.push(cl.to_string());
    }
    if let Some(g) = gpu {
        args.push("--gpu".into());
        args.push(g.to_string());
    }
    if let Some(ttl) = ttl_secs {
        args.push("--ttl".into());
        args.push(ttl.to_string());
    }
    run_lms(args.iter().map(|s| s.as_str())).await
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
}

pub async fn chat_completion(
    base_url: Option<String>,
    api_key: Option<String>,
    model: String,
    system_prompt: Option<String>,
    user_prompt: String,
) -> Result<String, String> {
    let base = base_url.unwrap_or_else(|| "http://127.0.0.1:1234/v1".to_string());
    let url = format!("{}/chat/completions", base.trim_end_matches('/'));

    let mut messages = Vec::new();
    if let Some(sys) = system_prompt {
        messages.push(ChatMessage {
            role: "system".into(),
            content: sys,
        });
    }
    messages.push(ChatMessage {
        role: "user".into(),
        content: user_prompt,
    });

    let body = ChatRequest {
        model,
        messages,
        temperature: Some(0.2),
    };

    let client = reqwest::Client::new();
    let mut req = client.post(url).header("Content-Type", "application/json");
    if let Some(key) = api_key {
        req = req.header("Authorization", format!("Bearer {key}"));
    }

    let resp = req.json(&body).send().await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Local LLM API error: {status} - {text}"));
    }
    let v: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    if let Some(content) = v
        .get("choices")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|first| first.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
    {
        return Ok(content.to_string());
    }
    if let Some(text) = v.get("text").and_then(|t| t.as_str()) {
        return Ok(text.to_string());
    }
    Err("Unexpected local LLM response format".to_string())
}
