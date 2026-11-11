//! Talks to the C# MCP server through the Rust MCP client (child process over STDIO).
//! The backend never touches SQL Server itself: every data request is an MCP tool call.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use mcp_client::{ClientError, McpClient};
use serde_json::Value;

use crate::error::ApiError;

/// What the backend needs from an MCP connection. The real one is `McpClient`;
/// tests use a fake so they run without .NET or SQL Server.
#[async_trait]
pub trait McpBackend: Send + Sync {
    async fn connect(&self) -> Result<(), ClientError>;
    async fn is_connected(&self) -> bool;
    async fn server_info(&self) -> Result<Value, ClientError>;
    async fn list_tools(&self) -> Result<Value, ClientError>;
    async fn call_tool(&self, name: &str, args: Value) -> Result<Value, ClientError>;
    async fn shutdown(&self);
}

#[async_trait]
impl McpBackend for McpClient {
    async fn connect(&self) -> Result<(), ClientError> {
        McpClient::connect(self).await
    }
    async fn is_connected(&self) -> bool {
        McpClient::is_connected(self).await
    }
    async fn server_info(&self) -> Result<Value, ClientError> {
        McpClient::server_info(self).await
    }
    async fn list_tools(&self) -> Result<Value, ClientError> {
        Ok(serde_json::to_value(McpClient::list_tools(self).await?).unwrap_or(Value::Array(vec![])))
    }
    async fn call_tool(&self, name: &str, args: Value) -> Result<Value, ClientError> {
        McpClient::call_tool(self, name, args).await
    }
    async fn shutdown(&self) {
        McpClient::shutdown(self).await
    }
}

pub struct McpService {
    backend: Arc<dyn McpBackend>,
    cache: Mutex<HashMap<String, (Instant, Value)>>,
    last_error: Mutex<Option<String>>,
}

impl McpService {
    pub fn new(backend: Arc<dyn McpBackend>) -> Self {
        Self { backend, cache: Mutex::new(HashMap::new()), last_error: Mutex::new(None) }
    }

    /// Starts the server and does the MCP handshake. Failure is remembered for /api/status.
    pub async fn connect(&self) {
        match self.backend.connect().await {
            Ok(()) => {
                self.set_error(None);
                let info = self.backend.server_info().await.unwrap_or(Value::Null);
                // (computed outside the macro: tracing's macros have their own `Value` type)
                let server = info.get("serverInfo").cloned().unwrap_or(Value::Null).to_string();
                let protocol = info.get("protocolVersion").and_then(Value::as_str).unwrap_or("?").to_string();
                tracing::info!("Connected to MCP server {server} (protocol {protocol})");
            }
            Err(e) => {
                tracing::error!("Could not start the MCP server: {e}");
                self.set_error(Some(e.to_string()));
            }
        }
    }

    pub async fn shutdown(&self) {
        self.backend.shutdown().await
    }

    pub async fn is_connected(&self) -> bool {
        self.backend.is_connected().await
    }

    pub fn last_error(&self) -> Option<String> {
        self.last_error.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }

    fn set_error(&self, e: Option<String>) {
        *self.last_error.lock().unwrap_or_else(|e| e.into_inner()) = e;
    }

    /// `tools/call`.
    pub async fn call(&self, tool: &str, args: Value) -> Result<Value, ApiError> {
        let started = Instant::now();
        let result = self.backend.call_tool(tool, args).await.map_err(to_api_error)?;
        self.set_error(None);
        tracing::debug!("tools/call {tool} took {} ms", started.elapsed().as_millis());
        Ok(result)
    }

    /// `tools/list`.
    pub async fn list_tools(&self) -> Result<Value, ApiError> {
        self.backend.list_tools().await.map_err(to_api_error)
    }

    pub async fn server_info(&self) -> Result<Value, ApiError> {
        self.backend.server_info().await.map_err(to_api_error)
    }

    /// Calls a tool (or `tools/list` when `tool` is None) and remembers the answer for `ttl`.
    pub async fn cached(&self, key: &str, ttl: Duration, tool: Option<&str>) -> Result<Value, ApiError> {
        if let Some((at, v)) = self.cache.lock().unwrap_or_else(|e| e.into_inner()).get(key) {
            if at.elapsed() < ttl {
                return Ok(v.clone());
            }
        }
        let value = match tool {
            None => self.list_tools().await?,
            Some(t) => self.call(t, Value::Object(Default::default())).await?,
        };
        self.cache.lock().unwrap_or_else(|e| e.into_inner()).insert(key.to_string(), (Instant::now(), value.clone()));
        Ok(value)
    }

    pub fn invalidate_cache(&self) {
        self.cache.lock().unwrap_or_else(|e| e.into_inner()).clear();
    }
}

fn to_api_error(e: ClientError) -> ApiError {
    match e {
        ClientError::Tool(message) | ClientError::InvalidArguments(message) => {
            // e.g. "Error executing tool get_record: No record 99 in 'products'."
            let detail = match message.strip_prefix("Error executing tool ") {
                Some(rest) => rest.split_once(": ").map(|(_, m)| m.to_string()).unwrap_or(message.clone()),
                None => message,
            };
            if detail.starts_with("No record") || detail.starts_with("There is no") {
                ApiError::not_found(detail)
            } else {
                ApiError::bad_request(detail)
            }
        }
        ClientError::Timeout(_) => ApiError::new(axum::http::StatusCode::GATEWAY_TIMEOUT, "The search server took too long to answer."),
        other => {
            tracing::error!("MCP failure: {other}");
            ApiError::new(axum::http::StatusCode::SERVICE_UNAVAILABLE, "The search server is not available right now.")
        }
    }
}
