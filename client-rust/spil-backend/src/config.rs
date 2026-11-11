//! Settings, read from environment variables or `config/server.env`.
//! The variable names are the same as the old FastAPI `backend/.env`, so that file can be reused.

use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Settings {
    // --- MCP server (started by the Rust MCP client as a child process over STDIO) ---
    pub mcp_server_command: String,
    pub mcp_server_args: Vec<String>,
    pub mcp_request_timeout_secs: f64,
    pub mcp_startup_timeout_secs: f64,

    // --- Security ---
    pub jwt_secret: String,
    pub jwt_expire_minutes: i64,
    pub users_db_path: PathBuf,
    pub admin_username: String,
    pub admin_password: Option<String>,
    pub login_max_attempts: usize,
    pub login_window_seconds: u64,
    pub cors_origins: Vec<String>,

    // --- Web ---
    pub frontend_dist: PathBuf,
    pub host: String,
    pub port: u16,
}

#[derive(Debug, thiserror::Error)]
#[error("configuration error: {0}")]
pub struct ConfigError(pub String);

impl Settings {
    /// Reads settings from the process environment. Relative paths are resolved against `base`.
    pub fn from_env(base: &Path) -> Result<Self, ConfigError> {
        let get = |k: &str| std::env::var(k).ok().map(|v| v.trim().to_string()).filter(|v| !v.is_empty());
        let num = |k: &str, default: f64| -> Result<f64, ConfigError> {
            get(k).map_or(Ok(default), |v| v.parse().map_err(|_| ConfigError(format!("{k} must be a number"))))
        };
        let path = |k: &str, default: &str| {
            let p = PathBuf::from(get(k).unwrap_or_else(|| default.to_string()));
            if p.is_absolute() { p } else { base.join(p) }
        };

        let mcp_server_command = get("MCP_SERVER_COMMAND")
            .ok_or_else(|| ConfigError("MCP_SERVER_COMMAND is not set (path to SemanticSearch.McpServer.exe)".into()))?;
        let jwt_secret = get("JWT_SECRET").ok_or_else(|| ConfigError("JWT_SECRET is not set".into()))?;
        if jwt_secret.len() < 32 {
            return Err(ConfigError("JWT_SECRET must be at least 32 characters".into()));
        }

        Ok(Self {
            mcp_server_args: parse_args(&get("MCP_SERVER_ARGS").unwrap_or_default())?,
            mcp_server_command,
            mcp_request_timeout_secs: num("MCP_REQUEST_TIMEOUT", 30.0)?,
            mcp_startup_timeout_secs: num("MCP_STARTUP_TIMEOUT", 180.0)?,
            jwt_secret,
            jwt_expire_minutes: num("JWT_EXPIRE_MINUTES", 480.0)? as i64,
            users_db_path: path("USERS_DB_PATH", "data/users.db"),
            admin_username: get("ADMIN_USERNAME").unwrap_or_else(|| "admin".into()),
            admin_password: get("ADMIN_PASSWORD"),
            login_max_attempts: num("LOGIN_MAX_ATTEMPTS", 5.0)? as usize,
            login_window_seconds: num("LOGIN_WINDOW_SECONDS", 300.0)? as u64,
            cors_origins: get("CORS_ORIGINS")
                .map(|v| v.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect())
                .unwrap_or_default(),
            frontend_dist: path("FRONTEND_DIST", "frontend/dist"),
            host: get("HOST").unwrap_or_else(|| "127.0.0.1".into()),
            port: num("PORT", 8000.0)? as u16,
        })
    }
}

/// `["--connection","Server=..."]` (JSON list) or `--rules "C:\x y\r.json"` (quoted words).
pub fn parse_args(raw: &str) -> Result<Vec<String>, ConfigError> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Ok(Vec::new());
    }
    if raw.starts_with('[') {
        return serde_json::from_str::<Vec<String>>(raw)
            .map_err(|e| ConfigError(format!("MCP_SERVER_ARGS is not a valid JSON list: {e}")));
    }
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut quote: Option<char> = None;
    for ch in raw.chars() {
        match (quote, ch) {
            (Some(q), c) if c == q => quote = None,
            (None, '"' | '\'') => quote = Some(ch),
            (None, c) if c.is_whitespace() => {
                if !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                }
            }
            (_, c) => cur.push(c), // backslashes are kept: Windows paths
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::parse_args;

    #[test]
    fn args_from_json_and_quoted_words() {
        assert_eq!(parse_args(r#"["--connection","Server=a\\b;Database=x"]"#).unwrap(), vec!["--connection", r"Server=a\b;Database=x"]);
        assert_eq!(parse_args(r#"--db "C:\my data\a.db" --model-dir C:\m"#).unwrap(), vec!["--db", r"C:\my data\a.db", "--model-dir", r"C:\m"]);
        assert!(parse_args("").unwrap().is_empty());
    }
}
