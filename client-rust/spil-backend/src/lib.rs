//! # spil-backend
//!
//! The SPIL Labs web backend, written in Rust (it replaces the old Python/FastAPI backend).
//!
//! ```text
//! Tauri app / browser ──HTTP──► spil-backend (this) + Rust MCP client ──STDIO──► C# MCP server ──SQL──► SQL Server
//! ```
//!
//! * serves the React screens and the `/api` endpoints
//! * sign-in (bcrypt + JWT), roles, login rate limiting   — accounts in `users.db` (SQLite)
//! * chat rules: regex decides which MCP tool answers a message
//! * tool runner for the Tools page
//! * never touches SQL Server itself: every data request is an MCP tool call

pub mod api;
pub mod auth;
pub mod chat;
pub mod config;
pub mod error;
pub mod mcp;
pub mod users;

use std::path::Path;
use std::sync::Arc;

use axum::extract::Request;
use axum::http::{header, HeaderName, HeaderValue, Method};
use axum::middleware::{self, Next};
use axum::response::Response;
use axum::Router;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};

use crate::auth::LoginRateLimiter;
use crate::config::Settings;
use crate::mcp::McpService;
use crate::users::UserStore;

/// Everything the request handlers share.
#[derive(Clone)]
pub struct AppState {
    pub settings: Arc<Settings>,
    pub users: Arc<UserStore>,
    pub mcp: Arc<McpService>,
    pub limiter: Arc<LoginRateLimiter>,
}

impl AppState {
    pub fn new(settings: Settings, users: UserStore, mcp: McpService) -> Self {
        let limiter = LoginRateLimiter::new(settings.login_max_attempts, settings.login_window_seconds);
        Self { settings: Arc::new(settings), users: Arc::new(users), mcp: Arc::new(mcp), limiter: Arc::new(limiter) }
    }
}

/// Creates the first administrator when there are no accounts yet.
pub fn bootstrap_admin(users: &UserStore, username: &str, password: Option<&str>) -> Result<(), users::StoreError> {
    if users.count()? > 0 {
        return Ok(());
    }
    match password {
        None => tracing::warn!("No users exist yet. Set ADMIN_PASSWORD in config/server.env and restart to create the first admin."),
        Some(pw) => {
            users.create(username, pw, "admin")?;
            tracing::info!("Created first administrator account '{username}'. You can now remove ADMIN_PASSWORD from the config.");
        }
    }
    Ok(())
}

/// The whole web app: `/api/*` plus the React build (with single-page-app fallback).
pub fn build_app(state: AppState) -> Router {
    let dist = state.settings.frontend_dist.clone();
    let mut app = Router::new().nest("/api", api::router()).with_state(state.clone());

    if dist.join("index.html").exists() {
        let spa = ServeDir::new(&dist).fallback(ServeFile::new(dist.join("index.html")));
        app = app.fallback_service(spa);
    } else {
        tracing::warn!("React build not found at {} - run 'npm run build' in frontend (the API still works).", dist.display());
    }

    if !state.settings.cors_origins.is_empty() {
        let origins: Vec<HeaderValue> = state.settings.cors_origins.iter().filter_map(|o| o.parse().ok()).collect();
        app = app.layer(
            CorsLayer::new()
                .allow_origin(AllowOrigin::list(origins))
                .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
                .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE]),
        );
    }
    app.layer(middleware::from_fn(security_headers))
}

async fn security_headers(req: Request, next: Next) -> Response {
    let mut res = next.run(req).await;
    let h = res.headers_mut();
    let mut set = |name: HeaderName, value: &'static str| {
        h.entry(name).or_insert(HeaderValue::from_static(value));
    };
    set(header::X_CONTENT_TYPE_OPTIONS, "nosniff");
    set(header::X_FRAME_OPTIONS, "DENY");
    set(header::REFERRER_POLICY, "no-referrer");
    set(
        header::CONTENT_SECURITY_POLICY,
        "default-src 'self'; img-src 'self' data:; style-src 'self' 'unsafe-inline' https://fonts.googleapis.com; \
         font-src 'self' https://fonts.gstatic.com; frame-ancestors 'none'",
    );
    res
}

/// Loads `config/server.env` (if present) into the environment; real environment variables win.
pub fn load_env_file(path: &Path) {
    if !path.exists() {
        return;
    }
    match std::fs::read_to_string(path) {
        Ok(text) => {
            for (key, value) in parse_env(&text) {
                if std::env::var_os(&key).is_none() {
                    std::env::set_var(key, value);
                }
            }
            tracing::info!("Settings loaded from {}", path.display());
        }
        Err(e) => tracing::warn!("Could not read {}: {e}", path.display()),
    }
}

/// Reads `KEY=VALUE` lines. Values are taken literally: a backslash is just a backslash, so
/// Windows paths like `C:\Users\...` and JSON like `localhost\\SQLEXPRESS` work as written
/// (the same as the old Python backend). Blank lines and `#` comments are skipped, a UTF-8 BOM
/// (PowerShell adds one) is ignored, and one pair of surrounding quotes is removed.
pub fn parse_env(text: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for line in text.trim_start_matches('\u{feff}').lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let line = line.strip_prefix("export ").unwrap_or(line);
        let Some((key, value)) = line.split_once('=') else { continue };
        let key = key.trim();
        if key.is_empty() {
            continue;
        }
        let mut value = value.trim();
        let quoted = value.len() >= 2
            && ((value.starts_with('"') && value.ends_with('"')) || (value.starts_with('\'') && value.ends_with('\'')));
        if quoted {
            value = &value[1..value.len() - 1];
        } else if let Some(i) = value.find(" #") {
            value = value[..i].trim_end(); // inline comment
        }
        out.push((key.to_string(), value.to_string()));
    }
    out
}

#[cfg(test)]
mod env_tests {
    use super::parse_env;

    #[test]
    fn windows_paths_json_and_bom() {
        let text = "\u{feff}MCP_REQUEST_TIMEOUT=30\r\n# comment\r\n\r\n\
            MCP_SERVER_COMMAND=C:\\Users\\ASUS\\Music\\SpilLabs\\dist\\server\\SemanticSearch.McpServer.exe\r\n\
            MCP_SERVER_ARGS=[\"--connection\",\"Server=localhost\\\\SQLEXPRESS;Database=LumenSearch\"]\r\n\
            JWT_SECRET=abc+/def==\r\nADMIN_PASSWORD=\r\nNAME=\"quoted value\" \r\n";
        let v = parse_env(text);
        let get = |k: &str| v.iter().find(|(a, _)| a == k).map(|(_, b)| b.as_str());
        assert_eq!(get("MCP_REQUEST_TIMEOUT"), Some("30"));
        assert_eq!(get("MCP_SERVER_COMMAND"), Some(r"C:\Users\ASUS\Music\SpilLabs\dist\server\SemanticSearch.McpServer.exe"));
        assert_eq!(get("MCP_SERVER_ARGS"), Some(r#"["--connection","Server=localhost\\SQLEXPRESS;Database=LumenSearch"]"#));
        assert_eq!(get("JWT_SECRET"), Some("abc+/def=="));
        assert_eq!(get("ADMIN_PASSWORD"), Some(""));
        assert_eq!(get("NAME"), Some("quoted value"));
    }
}
