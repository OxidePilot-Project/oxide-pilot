use log::{info, warn};

#[cfg(feature = "cognee")]
pub struct CogneeSupervisor {
    client: oxide_cognee_bridge::CogneeClient,
    base_url: String,
    token: Option<String>,
    child: std::sync::Arc<tokio::sync::Mutex<Option<tokio::process::Child>>>,
}

#[cfg(feature = "cognee")]
impl CogneeSupervisor {
    pub fn new(base_url: String, token: Option<String>) -> Result<Self, String> {
        let client = oxide_cognee_bridge::CogneeClient::new(base_url.clone(), token.clone())
            .map_err(|e| e.to_string())?;
        Ok(Self {
            client,
            base_url,
            token,
            child: std::sync::Arc::new(tokio::sync::Mutex::new(None)),
        })
    }

    pub async fn health_check(&self) -> Result<(), String> {
        self.client.health().await.map_err(|e| e.to_string())
    }

    /// Attempts to start the sidecar if not healthy. Returns Ok when healthy.
    pub async fn ensure_running(
        &self,
        python_cmd: Option<String>,
        host: Option<String>,
        port: Option<u16>,
        working_dir: Option<std::path::PathBuf>,
    ) -> Result<(), String> {
        if self.health_check().await.is_ok() {
            return Ok(());
        }
        self.start(python_cmd, host, port, working_dir).await?;
        // Give it a brief moment and re-check
        let mut attempts = 0;
        loop {
            attempts += 1;
            match self.health_check().await {
                Ok(()) => {
                    info!("Cognee sidecar is healthy after start");
                    return Ok(());
                }
                Err(e) if attempts < 5 => {
                    warn!("Waiting for Cognee sidecar to become healthy: {}", e);
                    tokio::time::sleep(std::time::Duration::from_millis(250)).await;
                }
                Err(e) => return Err(format!("Cognee sidecar failed to become healthy: {e}")),
            }
        }
    }

    /// Start the sidecar using `python -m uvicorn cognee_sidecar.app:app`.
    pub async fn start(
        &self,
        python_cmd: Option<String>,
        host: Option<String>,
        port: Option<u16>,
        working_dir: Option<std::path::PathBuf>,
    ) -> Result<(), String> {
        let mut child_guard = self.child.lock().await;
        if child_guard.as_ref().map(|c| c.id().is_some()).unwrap_or(false) {
            warn!("Cognee sidecar process already started");
            return Ok(());
        }

        let python = python_cmd.unwrap_or_else(|| "python".to_string());
        let host = host.unwrap_or_else(|| "127.0.0.1".to_string());
        let port = port.unwrap_or(8765);

        let mut cmd = tokio::process::Command::new(python);
        cmd.arg("-m")
            .arg("uvicorn")
            .arg("cognee_sidecar.app:app")
            .arg("--host")
            .arg(&host)
            .arg("--port")
            .arg(port.to_string());

        if let Some(dir) = working_dir {
            cmd.current_dir(dir);
        }

        // Pass token to the sidecar as env, if provided
        if let Some(t) = &self.token {
            cmd.env("COGNEE_SIDECAR_TOKEN", t);
        }

        // Bind only to loopback by default
        cmd.env("HOST", &host);

        info!("Starting Cognee sidecar via uvicorn on {}:{}", host, port);
        match cmd.spawn() {
            Ok(child) => {
                *child_guard = Some(child);
                Ok(())
            }
            Err(e) => Err(format!("Failed to spawn Cognee sidecar: {e}")),
        }
    }

    /// Stop the sidecar process if started by this supervisor.
    pub async fn stop(&self) -> Result<(), String> {
        let mut child_guard = self.child.lock().await;
        if let Some(child) = child_guard.as_mut() {
            match child.kill().await {
                Ok(_) => {
                    *child_guard = None;
                    info!("Cognee sidecar process stopped");
                    Ok(())
                }
                Err(e) => Err(format!("Failed to stop Cognee sidecar: {e}")),
            }
        } else {
            Ok(())
        }
    }
}

#[cfg(not(feature = "cognee"))]
pub struct CogneeSupervisor;

#[cfg(not(feature = "cognee"))]
impl CogneeSupervisor {
    pub fn new(_base_url: String, _token: Option<String>) -> Result<Self, String> { Ok(Self) }
    pub async fn health_check(&self) -> Result<(), String> { Ok(()) }
    pub async fn ensure_running(
        &self,
        _python_cmd: Option<String>,
        _host: Option<String>,
        _port: Option<u16>,
        _working_dir: Option<std::path::PathBuf>,
    ) -> Result<(), String> { Ok(()) }
    pub async fn start(
        &self,
        _python_cmd: Option<String>,
        _host: Option<String>,
        _port: Option<u16>,
        _working_dir: Option<std::path::PathBuf>,
    ) -> Result<(), String> { Ok(()) }
    pub async fn stop(&self) -> Result<(), String> { Ok(()) }
}
