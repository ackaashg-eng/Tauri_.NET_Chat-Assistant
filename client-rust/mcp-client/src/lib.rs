//! # mcp-client
//!
//! A small, robust MCP client that starts an MCP server as a **child process** and
//! talks to it over **STDIO** (JSON-RPC 2.0 on the child's stdin/stdout), using the
//! official Rust MCP SDK (`rmcp`).
//!
//! What it adds on top of `rmcp`:
//! * one long-lived server process shared by many concurrent callers
//! * per-request timeouts
//! * automatic restart of the server if it crashes, with one retry
//! * the server's stderr is forwarded (prefixed) and the last lines are kept, so a
//!   startup failure produces a useful error message
//! * tool results decoded from JSON text into `serde_json::Value`

use std::collections::VecDeque;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use rmcp::model::{CallToolRequestParams, CallToolResult};
use rmcp::service::{Peer, RunningService, ServiceError};
use rmcp::transport::TokioChildProcess;
use rmcp::{RoleClient, ServiceExt};
use serde::Serialize;
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::RwLock;

const STDERR_LINES_KEPT: usize = 40;

/// How to start the MCP server.
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Program to run, e.g. `C:\app\server\SemanticSearch.McpServer.exe` or `dotnet`.
    pub command: String,
    /// Arguments, e.g. `["--db", "data/app.db"]`.
    pub args: Vec<String>,
    /// Extra environment variables for the server process.
    pub env: Vec<(String, String)>,
    /// Working directory for the server process.
    pub cwd: Option<PathBuf>,
    /// Max time for spawning + the MCP initialize handshake (includes index building).
    pub startup_timeout: Duration,
    /// Max time for one tool call.
    pub request_timeout: Duration,
    /// Forward the server's stderr (logs) to this process's stderr.
    pub forward_stderr: bool,
}

impl ServerConfig {
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            args: Vec::new(),
            env: Vec::new(),
            cwd: None,
            startup_timeout: Duration::from_secs(180),
            request_timeout: Duration::from_secs(30),
            forward_stderr: true,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("could not start MCP server '{command}': {source}{stderr}")]
    Spawn {
        command: String,
        source: std::io::Error,
        stderr: String,
    },
    #[error("MCP handshake with the server failed: {message}{stderr}")]
    Handshake { message: String, stderr: String },
    #[error("MCP server did not answer within {0:?}")]
    Timeout(Duration),
    #[error("MCP server connection lost: {0}")]
    Disconnected(String),
    #[error("MCP protocol error: {0}")]
    Protocol(String),
    /// The tool ran but reported an error (e.g. "Unknown source 'x'").
    #[error("{0}")]
    Tool(String),
    #[error("invalid arguments: {0}")]
    InvalidArguments(String),
}

impl ClientError {
    /// A short machine-readable category, handy for mapping to HTTP status codes.
    pub fn kind(&self) -> &'static str {
        match self {
            ClientError::Spawn { .. } | ClientError::Handshake { .. } => "unavailable",
            ClientError::Timeout(_) => "timeout",
            ClientError::Disconnected(_) => "unavailable",
            ClientError::Protocol(_) => "protocol",
            ClientError::Tool(_) => "tool",
            ClientError::InvalidArguments(_) => "invalid_arguments",
        }
    }
}

/// Information about a tool advertised by the server (`tools/list`).
#[derive(Debug, Clone, Serialize)]
pub struct ToolInfo {
    pub name: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub input_schema: Value,
    pub read_only: Option<bool>,
}

struct Connection {
    service: RunningService<RoleClient, ()>,
    generation: u64,
}

/// A connection manager for one MCP server process. Cheap to share: wrap it in an `Arc`.
pub struct McpClient {
    config: ServerConfig,
    conn: RwLock<Option<Connection>>,
    generation: AtomicU64,
    stderr_tail: Arc<Mutex<VecDeque<String>>>,
}

impl McpClient {
    pub fn new(config: ServerConfig) -> Self {
        Self {
            config,
            conn: RwLock::new(None),
            generation: AtomicU64::new(0),
            stderr_tail: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    /// Starts the server (if not running) and performs the MCP handshake.
    pub async fn connect(&self) -> Result<(), ClientError> {
        self.peer().await.map(|_| ())
    }

    pub async fn is_connected(&self) -> bool {
        match self.conn.read().await.as_ref() {
            Some(c) => !c.service.is_transport_closed(),
            None => false,
        }
    }

    /// Name/version/protocol reported by the server during the handshake.
    pub async fn server_info(&self) -> Result<Value, ClientError> {
        let (peer, _) = self.peer().await?;
        let info = peer.peer_info();
        Ok(info
            .map(|i| serde_json::to_value(&*i).unwrap_or(Value::Null))
            .unwrap_or(Value::Null))
    }

    /// `tools/list`: what can this server do?
    pub async fn list_tools(&self) -> Result<Vec<ToolInfo>, ClientError> {
        let tools = self
            .with_retry(|peer| async move { peer.list_all_tools().await })
            .await?;
        Ok(tools
            .into_iter()
            .map(|t| ToolInfo {
                name: t.name.to_string(),
                title: t.title.clone(),
                description: t.description.as_ref().map(|d| d.to_string()),
                input_schema: Value::Object((*t.input_schema).clone()),
                read_only: t.annotations.as_ref().and_then(|a| a.read_only_hint),
            })
            .collect())
    }

    /// `tools/call`: runs a tool and returns its result. Text content that is valid
    /// JSON is parsed; otherwise it is returned as a JSON string.
    pub async fn call_tool(&self, name: &str, arguments: Value) -> Result<Value, ClientError> {
        let args = match arguments {
            Value::Object(map) => Some(map),
            Value::Null => None,
            other => {
                return Err(ClientError::InvalidArguments(format!(
                    "arguments must be a JSON object, got {other}"
                )))
            }
        };
        let name = name.to_string();
        let result = self
            .with_retry(move |peer| {
                let mut params = CallToolRequestParams::new(name.clone());
                if let Some(a) = args.clone() {
                    params = params.with_arguments(a);
                }
                async move { peer.call_tool(params).await }
            })
            .await?;
        decode_tool_result(result)
    }

    /// Stops the server process.
    pub async fn shutdown(&self) {
        if let Some(conn) = self.conn.write().await.take() {
            let _ = conn.service.cancel().await;
        }
    }

    // ------------------------------------------------------------------------

    /// Runs `op`; if the server died, restarts it once and retries.
    async fn with_retry<T, F, Fut>(&self, op: F) -> Result<T, ClientError>
    where
        F: Fn(Peer<RoleClient>) -> Fut,
        Fut: std::future::Future<Output = Result<T, ServiceError>>,
    {
        let mut attempt = 0;
        loop {
            attempt += 1;
            let (peer, generation) = self.peer().await?;
            let timeout = self.config.request_timeout;
            match tokio::time::timeout(timeout, op(peer)).await {
                Err(_) => return Err(ClientError::Timeout(timeout)),
                Ok(Ok(value)) => return Ok(value),
                Ok(Err(err)) => {
                    let lost = matches!(
                        err,
                        ServiceError::TransportClosed | ServiceError::TransportSend(_)
                    );
                    if lost {
                        self.invalidate(generation).await;
                        if attempt < 2 {
                            eprintln!("[mcp-client] server connection lost, restarting server and retrying");
                            continue;
                        }
                        return Err(ClientError::Disconnected(format!(
                            "{err}{}",
                            self.stderr_hint()
                        )));
                    }
                    return Err(map_service_error(err));
                }
            }
        }
    }

    /// Returns a handle to the running server, starting it if needed.
    async fn peer(&self) -> Result<(Peer<RoleClient>, u64), ClientError> {
        {
            let guard = self.conn.read().await;
            if let Some(c) = guard.as_ref() {
                if !c.service.is_transport_closed() {
                    return Ok((c.service.peer().clone(), c.generation));
                }
            }
        }

        let mut guard = self.conn.write().await;
        // Another caller may have reconnected while we waited for the lock.
        if let Some(c) = guard.as_ref() {
            if !c.service.is_transport_closed() {
                return Ok((c.service.peer().clone(), c.generation));
            }
        }
        if let Some(old) = guard.take() {
            let _ = old.service.cancel().await;
        }

        let service = self.spawn().await?;
        let generation = self.generation.fetch_add(1, Ordering::SeqCst) + 1;
        let peer = service.peer().clone();
        *guard = Some(Connection {
            service,
            generation,
        });
        Ok((peer, generation))
    }

    async fn invalidate(&self, generation: u64) {
        let mut guard = self.conn.write().await;
        if guard.as_ref().map(|c| c.generation) == Some(generation) {
            if let Some(old) = guard.take() {
                let _ = old.service.cancel().await;
            }
        }
    }

    async fn spawn(&self) -> Result<RunningService<RoleClient, ()>, ClientError> {
        let cfg = &self.config;
        let mut cmd = Command::new(&cfg.command);
        cmd.args(&cfg.args);
        for (k, v) in &cfg.env {
            cmd.env(k, v);
        }
        if let Some(dir) = &cfg.cwd {
            cmd.current_dir(dir);
        }
        #[cfg(windows)]
        {
            // CREATE_NO_WINDOW: don't flash a console window for the child process.
            cmd.creation_flags(0x0800_0000);
        }

        if let Ok(mut tail) = self.stderr_tail.lock() {
            tail.clear();
        }

        let (transport, stderr) = TokioChildProcess::builder(cmd)
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|source| ClientError::Spawn {
                command: cfg.command.clone(),
                source,
                stderr: String::new(),
            })?;

        if let Some(stderr) = stderr {
            let tail = self.stderr_tail.clone();
            let forward = cfg.forward_stderr;
            tokio::spawn(async move {
                let mut lines = BufReader::new(stderr).lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    if forward {
                        eprintln!("[mcp-server] {line}");
                    }
                    if let Ok(mut t) = tail.lock() {
                        if t.len() == STDERR_LINES_KEPT {
                            t.pop_front();
                        }
                        t.push_back(line);
                    }
                }
            });
        }

        // `()` is the simplest client handler: we don't offer sampling/roots to the server.
        match tokio::time::timeout(cfg.startup_timeout, ().serve(transport)).await {
            Err(_) => Err(ClientError::Handshake {
                message: format!("no initialize response within {:?}", cfg.startup_timeout),
                stderr: self.stderr_hint(),
            }),
            Ok(Err(e)) => {
                // Give the stderr reader a moment to collect the crash message.
                tokio::time::sleep(Duration::from_millis(300)).await;
                Err(ClientError::Handshake {
                    message: e.to_string(),
                    stderr: self.stderr_hint(),
                })
            }
            Ok(Ok(service)) => Ok(service),
        }
    }

    fn stderr_hint(&self) -> String {
        let tail = match self.stderr_tail.lock() {
            Ok(t) => t.iter().rev().take(8).rev().cloned().collect::<Vec<_>>(),
            Err(_) => Vec::new(),
        };
        if tail.is_empty() {
            String::new()
        } else {
            format!("\n--- last server log lines ---\n{}", tail.join("\n"))
        }
    }
}

fn map_service_error(err: ServiceError) -> ClientError {
    match err {
        ServiceError::McpError(e) => ClientError::Protocol(e.message.to_string()),
        ServiceError::Timeout { timeout } => ClientError::Timeout(timeout),
        other => ClientError::Protocol(other.to_string()),
    }
}

/// Turns a `CallToolResult` into JSON (or a tool error).
fn decode_tool_result(result: CallToolResult) -> Result<Value, ClientError> {
    let text: String = result
        .content
        .iter()
        .filter_map(|c| c.as_text().map(|t| t.text.clone()))
        .collect::<Vec<_>>()
        .join("\n");

    if result.is_error.unwrap_or(false) {
        let msg = if text.is_empty() {
            "tool reported an error".to_string()
        } else {
            text
        };
        return Err(ClientError::Tool(msg));
    }
    // Our server returns its JSON as text. Prefer that; fall back to structuredContent
    // (some SDKs only send that, others wrap a text result as {"result": "..."}).
    if let Ok(parsed) = serde_json::from_str::<Value>(&text) {
        return Ok(parsed);
    }
    match result.structured_content {
        Some(structured) if !structured.is_null() => Ok(structured),
        _ => Ok(Value::String(text)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rmcp::model::ContentBlock as Content;

    #[test]
    fn json_text_is_parsed() {
        let r = CallToolResult::success(vec![Content::text(r#"{"results":[1,2]}"#)]);
        assert_eq!(decode_tool_result(r).unwrap()["results"][1], 2);
    }

    #[test]
    fn plain_text_stays_a_string() {
        let r = CallToolResult::success(vec![Content::text("hello")]);
        assert_eq!(
            decode_tool_result(r).unwrap(),
            Value::String("hello".into())
        );
    }

    #[test]
    fn tool_errors_become_client_errors() {
        let r = CallToolResult::error(vec![Content::text("Unknown source 'x'.")]);
        match decode_tool_result(r) {
            Err(ClientError::Tool(m)) => assert!(m.contains("Unknown source")),
            other => panic!("unexpected {other:?}"),
        }
    }

    #[tokio::test]
    async fn missing_server_reports_spawn_error() {
        let client = McpClient::new(ServerConfig::new("definitely-not-a-real-mcp-server-xyz"));
        let err = client.list_tools().await.unwrap_err();
        assert_eq!(err.kind(), "unavailable");
    }
}
