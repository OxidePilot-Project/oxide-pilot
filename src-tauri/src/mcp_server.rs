use axum::{
    body::Body,
    http::{header::AUTHORIZATION, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use std::convert::Infallible;
use std::net::SocketAddr;
use tokio::{sync::oneshot, task::JoinHandle};

#[derive(Clone)]
#[allow(dead_code)] // Reserved for future use
pub struct McpServerConfig {
    pub addr: SocketAddr,
    pub password: Option<String>,
}

pub struct McpServerHandle {
    addr: SocketAddr,
    _shutdown: Option<oneshot::Sender<()>>,
    task: Option<JoinHandle<()>>,
    password_set: bool,
}

#[allow(dead_code)] // Some methods reserved for future use
impl McpServerHandle {
    pub async fn start(
        port: u16,
        password: Option<String>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let addr: SocketAddr = SocketAddr::from(([127, 0, 0, 1], port));
        let (tx, rx) = oneshot::channel::<()>();

        // Build router with simple auth middleware wrapper
        let pwd = password.clone();
        let app = Router::new()
            .route("/health", get(|| async { "ok" }))
            .route("/", get(|| async { "Oxide MCP server running" }))
            .layer(axum::middleware::from_fn(
                move |req: Request<Body>, next: Next| {
                    let pwd = pwd.clone();
                    async move {
                        // If a password is configured, enforce simple Bearer auth
                        if let Some(expected) = pwd.as_ref() {
                            let authorized = req
                                .headers()
                                .get(AUTHORIZATION)
                                .and_then(|h| h.to_str().ok())
                                .map(|v| v.trim())
                                .filter(|v| v.starts_with("Bearer "))
                                .map(|v| v.trim_start_matches("Bearer ").to_string())
                                .map(|token| token == *expected)
                                .unwrap_or(false);
                            if !authorized {
                                return Ok::<Response, Infallible>(
                                    (StatusCode::UNAUTHORIZED, "Unauthorized").into_response(),
                                );
                            }
                        }
                        let res = next.run(req).await;
                        Ok::<Response, Infallible>(res)
                    }
                },
            ));

        let listener = tokio::net::TcpListener::bind(addr).await?;
        let server = axum::serve(listener, app).with_graceful_shutdown(async move {
            let _ = rx.await;
        });

        let handle: JoinHandle<()> = tokio::spawn(async move {
            if let Err(err) = server.await {
                eprintln!("MCP HTTP server error: {}", err);
            }
        });

        Ok(Self {
            addr,
            _shutdown: Some(tx),
            task: Some(handle),
            password_set: password.is_some(),
        })
    }

    pub fn addr(&self) -> SocketAddr {
        self.addr
    }
    pub fn running(&self) -> bool {
        self.task.is_some()
    }
    pub fn password_enabled(&self) -> bool {
        self.password_set
    }

    pub async fn stop(&mut self) {
        if let Some(tx) = self._shutdown.take() {
            let _ = tx.send(());
        }
        if let Some(handle) = self.task.take() {
            let _ = handle.await;
        }
    }
}
