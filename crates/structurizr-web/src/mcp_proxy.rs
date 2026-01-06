//! MCP (Model Context Protocol) proxy implementation
//!
//! This module handles spawning and managing the MCP server process,
//! and proxying requests from the web server to the MCP server.

use axum::{
    extract::{ws::WebSocketUpgrade, Extension},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use structurizr_config::McpConfig;
use std::{
    net::SocketAddr,
    process::Stdio,
    sync::Arc,
    time::Duration,
};
use tokio::{
    process::{Child, Command},
    sync::RwLock,
    time::sleep,
};
use tracing::{error, info, warn};


/// State for the MCP proxy
pub struct McpProxyState {
    /// Configuration
    config: McpConfig,

    /// Workspaces directory path
    workspaces_dir: String,

    /// The spawned MCP process
    process: Arc<RwLock<Option<Child>>>,

    /// The actual port the MCP server is running on
    actual_port: Arc<RwLock<u16>>,

    /// Whether the MCP server is healthy
    is_healthy: Arc<RwLock<bool>>,
}

impl McpProxyState {
    /// Create new MCP proxy state
    pub async fn new(config: McpConfig, workspaces_dir: String) -> Result<Self, Box<dyn std::error::Error>> {
        let state = Self {
            config: config.clone(),
            workspaces_dir,
            process: Arc::new(RwLock::new(None)),
            actual_port: Arc::new(RwLock::new(config.server.port)),
            is_healthy: Arc::new(RwLock::new(false)),
        };

        if config.enabled && config.auto_start {
            state.start_mcp_server().await?;
        }

        Ok(state)
    }

    /// Start the MCP server process
    async fn start_mcp_server(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting MCP server process");

        // Try to find an available port if configured port is in use
        let port = self.find_available_port().await?;
        *self.actual_port.write().await = port;

        // Build the command - use current executable if available
        let exe_path = std::env::current_exe()
            .unwrap_or_else(|_| std::path::PathBuf::from("structurizr"));
        let mut cmd = Command::new(exe_path);
        cmd.arg("mcp-serve")
            .arg("--workspaces-dir")
            .arg(&self.workspaces_dir)
            .arg("--transport")
            .arg(&self.config.server.transport)
            .arg("--port")
            .arg(port.to_string())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);  // Ensure process is killed when dropped

        // Spawn the process
        let child = cmd.spawn()?;
        *self.process.write().await = Some(child);

        // Wait for the server to be ready
        self.wait_for_health().await?;

        info!("MCP server started on port {}", port);
        Ok(())
    }

    /// Find an available port, starting from the configured port
    /// Uses a simple check without binding to avoid race conditions
    async fn find_available_port(&self) -> Result<u16, Box<dyn std::error::Error>> {
        // Simply use the configured port - the MCP server will handle binding errors
        // If it fails, we'll catch it in the health check
        Ok(self.config.server.port)
    }

    /// Wait for the MCP server to become healthy
    async fn wait_for_health(&self) -> Result<(), Box<dyn std::error::Error>> {
        let max_attempts = 30;
        let mut attempts = 0;

        while attempts < max_attempts {
            if self.check_health().await {
                *self.is_healthy.write().await = true;
                return Ok(());
            }

            attempts += 1;
            sleep(Duration::from_millis(500)).await;
        }

        Err("MCP server failed to become healthy".into())
    }

    /// Check if the MCP server is healthy
    async fn check_health(&self) -> bool {
        let port = *self.actual_port.read().await;
        let url = match self.config.server.transport.as_str() {
            "sse" | "http" => format!("http://127.0.0.1:{}/health", port),
            _ => {
                // For WebSocket, try to connect
                return tokio::net::TcpStream::connect(SocketAddr::from(([127, 0, 0, 1], port)))
                    .await
                    .is_ok();
            }
        };

        // For HTTP/SSE, check the health endpoint
        if let Ok(response) = reqwest::get(&url).await {
            response.status().is_success()
        } else {
            false
        }
    }

    /// Ensure the MCP server is running, restart if necessary
    pub async fn ensure_running(&self) -> Result<(), Box<dyn std::error::Error>> {
        if !*self.is_healthy.read().await {
            warn!("MCP server is not healthy, attempting restart");
            self.restart().await?;
        }
        Ok(())
    }

    /// Restart the MCP server
    async fn restart(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Kill existing process if any
        if let Some(mut child) = self.process.write().await.take() {
            let _ = child.kill().await;
        }

        // Start new process
        self.start_mcp_server().await
    }

    /// Get the WebSocket URL for the MCP server
    pub async fn websocket_url(&self) -> String {
        let port = *self.actual_port.read().await;
        format!("ws://127.0.0.1:{}", port)
    }

    /// Get the SSE URL for the MCP server
    pub async fn sse_url(&self) -> String {
        let port = *self.actual_port.read().await;
        format!("http://127.0.0.1:{}/mcp/sse", port)
    }
}

/// Health check endpoint for MCP proxy
pub async fn mcp_health(
    Extension(proxy): Extension<Arc<McpProxyState>>,
) -> impl IntoResponse {
    if *proxy.is_healthy.read().await {
        (
            StatusCode::OK,
            Json(serde_json::json!({
                "status": "healthy",
                "transport": proxy.config.server.transport,
                "port": *proxy.actual_port.read().await,
            }))
        ).into_response()
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({
                "status": "unhealthy",
                "error": "MCP server is not responding"
            }))
        ).into_response()
    }
}

/// WebSocket proxy handler
pub async fn websocket_proxy_handler(
    ws: WebSocketUpgrade,
    Extension(proxy): Extension<Arc<McpProxyState>>,
) -> Response {
    // Ensure MCP server is running
    if let Err(e) = proxy.ensure_running().await {
        error!("Failed to ensure MCP server is running: {}", e);
        return (StatusCode::SERVICE_UNAVAILABLE, "MCP server unavailable").into_response();
    }

    let url = proxy.websocket_url().await;

    ws.on_upgrade(move |socket| async move {
        if let Err(e) = proxy_websocket(socket, url).await {
            error!("WebSocket proxy error: {}", e);
        }
    })
}

/// Proxy WebSocket connection to MCP server
async fn proxy_websocket(
    client_socket: axum::extract::ws::WebSocket,
    mcp_url: String,
) -> Result<(), Box<dyn std::error::Error>> {
    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::connect_async;

    // Connect to MCP server
    let (mcp_socket, _) = connect_async(&mcp_url).await?;
    let (mut mcp_sink, mut mcp_stream) = mcp_socket.split();
    let (mut client_sink, mut client_stream) = client_socket.split();

    // Proxy messages bidirectionally
    tokio::select! {
        // Client -> MCP
        _ = async {
            while let Some(msg) = client_stream.next().await {
                match msg {
                    Ok(msg) => {
                        let tungstenite_msg = match msg {
                            axum::extract::ws::Message::Text(text) => {
                                tokio_tungstenite::tungstenite::Message::Text(text)
                            }
                            axum::extract::ws::Message::Binary(data) => {
                                tokio_tungstenite::tungstenite::Message::Binary(data)
                            }
                            axum::extract::ws::Message::Close(_) => break,
                            _ => continue,
                        };
                        if mcp_sink.send(tungstenite_msg).await.is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        } => {}

        // MCP -> Client
        _ = async {
            while let Some(msg) = mcp_stream.next().await {
                match msg {
                    Ok(msg) => {
                        let axum_msg = match msg {
                            tokio_tungstenite::tungstenite::Message::Text(text) => {
                                axum::extract::ws::Message::Text(text)
                            }
                            tokio_tungstenite::tungstenite::Message::Binary(data) => {
                                axum::extract::ws::Message::Binary(data)
                            }
                            tokio_tungstenite::tungstenite::Message::Close(_) => break,
                            _ => continue,
                        };
                        if client_sink.send(axum_msg).await.is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        } => {}
    }

    Ok(())
}

/// SSE proxy handler
pub async fn sse_proxy_handler(
    Extension(proxy): Extension<Arc<McpProxyState>>,
    axum::extract::Host(host): axum::extract::Host,
) -> Response {
    use axum::response::sse::{Event, KeepAlive, Sse};
    use futures_util::stream::StreamExt;
    use std::time::Duration;

    // Ensure MCP server is running
    if let Err(e) = proxy.ensure_running().await {
        error!("Failed to ensure MCP server is running: {}", e);
        return (StatusCode::SERVICE_UNAVAILABLE, "MCP server unavailable").into_response();
    }

    let sse_url = proxy.sse_url().await;

    // Build the proxy endpoint URL using the original client's host
    // This ensures clients POST back to the proxy, not the internal MCP server
    let proxy_endpoint = format!("http://{}/mcp", host);

    // Connect to MCP server's SSE endpoint
    let client = reqwest::Client::new();

    match client.get(&sse_url).send().await {
        Ok(response) => {
            // Check if response is successful
            if !response.status().is_success() {
                error!("MCP SSE endpoint returned error: {}", response.status());
                return (StatusCode::BAD_GATEWAY, "MCP SSE endpoint error").into_response();
            }

            // Create event stream from response
            let stream = response.bytes_stream()
                .map(move |chunk_result| {
                    let proxy_endpoint = proxy_endpoint.clone();
                    match chunk_result {
                        Ok(bytes) => {
                            // Parse SSE events from bytes - handle multi-line format
                            let text = String::from_utf8_lossy(&bytes);

                            // Parse SSE fields from the chunk
                            let mut event_type: Option<String> = None;
                            let mut data: Option<String> = None;
                            let mut id: Option<String> = None;

                            for line in text.lines() {
                                if line.starts_with("event:") {
                                    event_type = Some(line.trim_start_matches("event:").trim().to_string());
                                } else if line.starts_with("data:") {
                                    data = Some(line.trim_start_matches("data:").trim().to_string());
                                } else if line.starts_with("id:") {
                                    id = Some(line.trim_start_matches("id:").trim().to_string());
                                } else if line.trim() == ":" {
                                    // Keep-alive comment - ignore
                                }
                            }

                            // Build the event with parsed fields
                            let mut event = Event::default();
                            if let Some(ref et) = event_type {
                                event = event.event(et.clone());
                            }

                            // Rewrite endpoint URLs to point to the proxy
                            if let Some(d) = data {
                                let rewritten_data = if event_type.as_deref() == Some("endpoint") {
                                    // Replace the internal endpoint URL with the proxy URL
                                    proxy_endpoint.clone()
                                } else {
                                    d
                                };
                                event = event.data(rewritten_data);
                            }

                            if let Some(i) = id {
                                event = event.id(i);
                            }

                            Ok::<_, axum::Error>(event)
                        }
                        Err(e) => {
                            error!("Error reading SSE stream: {}", e);
                            Ok(Event::default()
                                .event("error")
                                .data(format!("Stream error: {}", e)))
                        }
                    }
                });

            // Return SSE response with keep-alive
            Sse::new(stream)
                .keep_alive(
                    KeepAlive::new()
                        .interval(Duration::from_secs(30))
                        .text(":")
                )
                .into_response()
        }
        Err(e) => {
            error!("Failed to connect to MCP SSE endpoint: {}", e);
            (StatusCode::BAD_GATEWAY, "Failed to connect to MCP server").into_response()
        }
    }
}

/// POST proxy handler for MCP requests
pub async fn post_proxy_handler(
    Extension(proxy): Extension<Arc<McpProxyState>>,
    Json(request): Json<serde_json::Value>,
) -> Response {
    use axum::Json;

    // Ensure MCP server is running
    if let Err(e) = proxy.ensure_running().await {
        error!("Failed to ensure MCP server is running: {}", e);
        return (StatusCode::SERVICE_UNAVAILABLE, "MCP server unavailable").into_response();
    }

    let port = *proxy.actual_port.read().await;
    let post_url = format!("http://127.0.0.1:{}/mcp", if port != 0 { port } else { proxy.config.server.port });

    // Forward POST to MCP server
    let client = reqwest::Client::new();
    match client.post(&post_url)
        .json(&request)
        .send()
        .await {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<serde_json::Value>().await {
                    Ok(json) => Json(json).into_response(),
                    Err(e) => {
                        error!("Failed to parse MCP response: {}", e);
                        (StatusCode::BAD_GATEWAY, "Invalid MCP response").into_response()
                    }
                }
            } else {
                error!("MCP server returned error: {}", response.status());
                (StatusCode::BAD_GATEWAY, "MCP server error").into_response()
            }
        }
        Err(e) => {
            error!("Failed to forward request to MCP server: {}", e);
            (StatusCode::BAD_GATEWAY, "Failed to connect to MCP server").into_response()
        }
    }
}

/// OAuth authorization server metadata handler
/// Returns 404 with proper JSON to indicate no OAuth is configured
/// This prevents Claude Code from trying OAuth authentication
pub async fn oauth_metadata_handler() -> Response {
    // Return 404 with JSON body indicating no OAuth configuration
    // This tells MCP clients that this server doesn't use OAuth
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({
            "error": "not_found",
            "error_description": "OAuth not supported - this MCP server does not require authentication"
        }))
    ).into_response()
}

/// OAuth protected resource metadata handler (RFC 9470)
/// Also returns a response indicating no OAuth
pub async fn oauth_protected_resource_handler() -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({
            "error": "not_found",
            "error_description": "Protected resource metadata not available"
        }))
    ).into_response()
}