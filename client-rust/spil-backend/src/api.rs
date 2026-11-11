//! HTTP API used by the React app (browser and Tauri). Same addresses and JSON as the old
//! FastAPI backend. Every data request becomes an MCP tool call to the C# server.

use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::extract::State;
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, patch, post};
use axum::{Json, Router};
use regex::Regex;
use serde::Deserialize;
use serde_json::{json, Map, Value};
use std::sync::LazyLock;

use crate::auth::{create_token, AdminUser, ClientIp, CurrentUser};
use crate::chat;
use crate::error::{ApiError, ApiJson, ApiPath, ApiQuery};
use crate::users::User;
use crate::AppState;

type ApiResult<T> = Result<T, ApiError>;

/// Tools that change things for everyone; only admins may run them from the Tools page.
const ADMIN_ONLY_TOOLS: &[&str] = &["reindex"];
const MAX_ARGUMENT_BYTES: usize = 20_000;
const MODES: &[&str] = &["semantic", "keyword", "pattern"];

static SOURCE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[A-Za-z0-9_]+$").unwrap());
static SOURCES: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[A-Za-z0-9_,]+$").unwrap());
static USERNAME: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[A-Za-z0-9_.\-]{3,32}$").unwrap());
static TOOL_NAME: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[a-z][a-z0-9_]{1,63}$").unwrap());

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/auth/login", post(login))
        .route("/auth/me", get(me))
        .route("/auth/change-password", post(change_password))
        .route("/search", post(search))
        .route("/search/history", get(history))
        .route("/search/explain", get(explain))
        .route("/chat", post(chat_message))
        .route("/sources", get(sources))
        .route("/records/{source}/{id}", get(record))
        .route("/tools", get(list_tools))
        .route("/tools/{name}", post(run_tool))
        .route("/status", get(status))
        .route("/health", get(health))
        .route("/admin/mcp", get(admin_mcp))
        .route("/admin/reindex", post(admin_reindex))
        .route("/admin/users", get(admin_users).post(admin_create_user))
        .route("/admin/users/{id}", patch(admin_update_user).delete(admin_delete_user))
        .route("/admin/users/{id}/password", post(admin_reset_password))
        .fallback(|| async { ApiError::not_found("Not found") })
}

// ------------------------------------------------------------------------------- helpers

fn len_between(value: &str, field: &str, min: usize, max: usize) -> ApiResult<()> {
    let n = value.chars().count();
    if n < min || n > max {
        return Err(ApiError::invalid(if n < min {
            format!("{field}: must not be empty")
        } else {
            format!("{field}: at most {max} characters")
        }));
    }
    Ok(())
}

fn check_password(v: &str, field: &str) -> ApiResult<()> {
    if v.chars().count() < 10 {
        return Err(ApiError::invalid(format!("{field}: password must be at least 10 characters")));
    }
    if v.len() > 72 {
        return Err(ApiError::invalid(format!("{field}: at most 72 characters")));
    }
    if v.chars().all(char::is_alphabetic) || v.chars().all(|c| c.is_ascii_digit()) {
        return Err(ApiError::invalid(format!("{field}: password must mix letters with numbers or symbols")));
    }
    Ok(())
}

fn check_mode(mode: &str) -> ApiResult<()> {
    if MODES.contains(&mode) {
        Ok(())
    } else {
        Err(ApiError::invalid("mode: use semantic, keyword or pattern"))
    }
}

fn check_score(v: Option<f64>) -> ApiResult<()> {
    match v {
        Some(s) if !(0.0..=1.0).contains(&s) => Err(ApiError::invalid("min_score: must be between 0 and 1")),
        _ => Ok(()),
    }
}

fn check_top_k(v: i64) -> ApiResult<()> {
    if (1..=50).contains(&v) { Ok(()) } else { Err(ApiError::invalid("top_k: must be between 1 and 50")) }
}

async fn blocking<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> ApiResult<T> {
    tokio::task::spawn_blocking(f).await.map_err(|_| ApiError::internal())
}

fn default_top_k_5() -> i64 { 5 }
fn default_top_k_8() -> i64 { 8 }
fn default_mode() -> String { "semantic".into() }
fn default_role() -> String { "user".into() }

// ---------------------------------------------------------------------------------- auth

#[derive(Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

async fn login(State(st): State<AppState>, ClientIp(ip): ClientIp, ApiJson(body): ApiJson<LoginRequest>) -> ApiResult<Json<Value>> {
    len_between(&body.username, "username", 1, 64)?;
    len_between(&body.password, "password", 1, 128)?;
    let key = format!("{ip}|{}", body.username.to_lowercase());
    let wait = st.limiter.retry_after(&key);
    if wait > 0 {
        return Err(ApiError::too_many(wait));
    }

    let users = st.users.clone();
    let (name, pw) = (body.username.clone(), body.password.clone());
    let user = blocking(move || users.authenticate(&name, &pw)).await??;
    let Some(user) = user else {
        st.limiter.failed(&key);
        tracing::warn!("Failed login for '{}' from {ip}", body.username);
        return Err(ApiError::new(StatusCode::UNAUTHORIZED, "Wrong username or password"));
    };

    st.limiter.reset(&key);
    let (token, expires_in) = create_token(&user, &st.settings.jwt_secret, st.settings.jwt_expire_minutes)?;
    tracing::info!("User '{}' signed in", user.username);
    Ok(Json(json!({ "access_token": token, "token_type": "bearer", "expires_in": expires_in, "user": user })))
}

async fn me(CurrentUser(user): CurrentUser) -> Json<User> {
    Json(user)
}

#[derive(Deserialize)]
struct ChangePasswordRequest {
    current_password: String,
    new_password: String,
}

async fn change_password(State(st): State<AppState>, CurrentUser(user): CurrentUser,
                         ApiJson(body): ApiJson<ChangePasswordRequest>) -> ApiResult<StatusCode> {
    len_between(&body.current_password, "current_password", 0, 128)?;
    check_password(&body.new_password, "new_password")?;
    let users = st.users.clone();
    let id = user.id;
    let ok = blocking(move || -> Result<bool, crate::users::StoreError> {
        if !users.check_password(id, &body.current_password)? {
            return Ok(false);
        }
        users.set_password(id, &body.new_password)?;
        Ok(true)
    })
    .await??;
    if !ok {
        return Err(ApiError::bad_request("Current password is wrong"));
    }
    Ok(StatusCode::NO_CONTENT)
}

// -------------------------------------------------------------------------------- search

#[derive(Deserialize)]
struct SearchRequest {
    query: String,
    #[serde(default = "default_top_k_5")]
    top_k: i64,
    min_score: Option<f64>,
    source: Option<String>,
    #[serde(default = "default_mode")]
    mode: String,
}

async fn search(State(st): State<AppState>, CurrentUser(user): CurrentUser, ApiJson(body): ApiJson<SearchRequest>) -> ApiResult<Json<Value>> {
    check_mode(&body.mode)?;
    check_top_k(body.top_k)?;
    check_score(body.min_score)?;
    len_between(&body.query, "query", 1, 500)?;
    // Tidy spaces in normal questions; leave regex patterns exactly as typed.
    let query = if body.mode == "pattern" { body.query.trim().to_string() } else { body.query.split_whitespace().collect::<Vec<_>>().join(" ") };
    if query.is_empty() {
        return Err(ApiError::invalid("query: must not be blank"));
    }
    if body.mode == "pattern" && query.chars().count() > 200 {
        return Err(ApiError::invalid("query: patterns can be at most 200 characters"));
    }
    if let Some(s) = &body.source {
        if s.len() > 64 || !SOURCE.is_match(s) {
            return Err(ApiError::invalid("source: letters, digits and _ only"));
        }
    }

    let mut args = json!({ "query": query, "topK": body.top_k, "mode": body.mode });
    if let Some(m) = body.min_score {
        args["minScore"] = json!(m);
    }
    if let Some(s) = &body.source {
        args["source"] = json!(s);
    }
    let result = st.mcp.call("semantic_search", args).await?;
    let count = result.get("results").and_then(Value::as_array).map_or(0, Vec::len) as i64;
    tracing::info!("search user={} mode={} results={count}", user.username, body.mode);
    st.users.log_search(user.id, &query, &body.mode, count)?;
    Ok(Json(result))
}

#[derive(Deserialize)]
struct ChatRequest {
    message: String,
    #[serde(default = "default_top_k_8")]
    top_k: i64,
    min_score: Option<f64>,
    source: Option<String>,
    #[serde(default = "default_mode")]
    mode: String,
}

/// Answers a chat message: regex rules pick the MCP tool, the tool's result becomes the reply.
async fn chat_message(State(st): State<AppState>, CurrentUser(user): CurrentUser, ApiJson(body): ApiJson<ChatRequest>) -> ApiResult<Json<Value>> {
    let message = body.message.trim().to_string();
    if message.is_empty() {
        return Err(ApiError::invalid("message: must not be blank"));
    }
    len_between(&message, "message", 1, 500)?;
    check_mode(&body.mode)?;
    check_top_k(body.top_k)?;
    check_score(body.min_score)?;
    if let Some(s) = &body.source {
        if s.len() > 200 || !SOURCES.is_match(s) {
            return Err(ApiError::invalid("source: letters, digits, _ and commas only"));
        }
    }

    let options = chat::Options { top_k: body.top_k, min_score: body.min_score, source: body.source.clone(), mode: body.mode.clone() };
    let mut plan = chat::route(&message, &options);
    let tools = st.mcp.cached("tools", Duration::from_secs(60), None).await?;
    let available = tools.as_array().is_some_and(|a| a.iter().any(|t| t["name"] == plan.tool.as_str()));
    if !available {
        plan = chat::search_plan(&message, &options); // e.g. a server without the ERP tools
    }

    let started = Instant::now();
    let result = st.mcp.call(&plan.tool, Value::Object(plan.arguments.clone())).await?;
    let (reply, data) = chat::summarise(&plan, result);
    let took = started.elapsed().as_millis() as u64;

    let count = if plan.kind == "search" {
        data.get("results").and_then(Value::as_array).map_or(0, Vec::len) as i64
    } else {
        data.get("count").and_then(Value::as_i64).unwrap_or(1)
    };
    tracing::info!("chat user={} tool={} results={count}", user.username, plan.tool);
    st.users.log_search(user.id, &message, "chat", count)?;
    Ok(Json(json!({
        "reply": reply, "tool": plan.tool, "arguments": plan.arguments, "kind": plan.kind,
        "rows_key": plan.rows_key, "data": data, "took_ms": took,
    })))
}

async fn history(State(st): State<AppState>, CurrentUser(user): CurrentUser) -> ApiResult<Json<Value>> {
    Ok(Json(json!(st.users.history(user.id, 10)?)))
}

#[derive(Deserialize)]
struct ExplainQuery {
    q: String,
}

/// How the regex rules read a question: compared words and recognised concepts.
async fn explain(State(st): State<AppState>, _u: CurrentUser, ApiQuery(p): ApiQuery<ExplainQuery>) -> ApiResult<Json<Value>> {
    len_between(&p.q, "q", 1, 500)?;
    Ok(Json(st.mcp.call("explain_query", json!({ "query": p.q })).await?))
}

async fn sources(State(st): State<AppState>, _u: CurrentUser) -> ApiResult<Json<Value>> {
    Ok(Json(st.mcp.cached("sources", Duration::from_secs(60), Some("list_sources")).await?))
}

async fn record(State(st): State<AppState>, _u: CurrentUser, ApiPath((source, id)): ApiPath<(String, i64)>) -> ApiResult<Json<Value>> {
    if source.len() > 64 || !SOURCE.is_match(&source) || id < 0 {
        return Err(ApiError::invalid("Invalid source or id"));
    }
    Ok(Json(st.mcp.call("get_record", json!({ "source": source, "id": id })).await?))
}

// --------------------------------------------------------------------------------- tools

fn tool_allowed(tool: &Value, user: &User) -> bool {
    !(ADMIN_ONLY_TOOLS.contains(&tool["name"].as_str().unwrap_or("")) && !user.is_admin())
}

/// The tools the MCP server advertises (tools/list), and whether you may run each one.
async fn list_tools(State(st): State<AppState>, CurrentUser(user): CurrentUser) -> ApiResult<Json<Value>> {
    let tools = st.mcp.cached("tools", Duration::from_secs(60), None).await?;
    let out: Vec<Value> = tools
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(|mut t| {
            let allowed = tool_allowed(&t, &user);
            t["allowed"] = json!(allowed);
            t
        })
        .collect();
    Ok(Json(Value::Array(out)))
}

#[derive(Deserialize)]
struct ToolCallRequest {
    #[serde(default)]
    arguments: Map<String, Value>,
}

/// Runs one MCP tool (tools/call). Tools that change data record who ran them.
async fn run_tool(State(st): State<AppState>, CurrentUser(user): CurrentUser, ApiPath(name): ApiPath<String>,
                  ApiJson(body): ApiJson<ToolCallRequest>) -> ApiResult<Json<Value>> {
    if !TOOL_NAME.is_match(&name) {
        return Err(ApiError::invalid("Invalid tool name"));
    }
    let tools = st.mcp.cached("tools", Duration::from_secs(60), None).await?;
    let tool = tools
        .as_array()
        .and_then(|a| a.iter().find(|t| t["name"] == name.as_str()))
        .cloned()
        .ok_or_else(|| ApiError::not_found(format!("There is no tool called '{name}'.")))?;
    if !tool_allowed(&tool, &user) {
        return Err(ApiError::forbidden("Only administrators can run this tool."));
    }

    let mut args = body.arguments;
    if serde_json::to_string(&args).map_or(0, |s| s.len()) > MAX_ARGUMENT_BYTES {
        return Err(ApiError::new(StatusCode::PAYLOAD_TOO_LARGE, "The arguments are too large."));
    }
    // The signed-in user is always recorded as the person making a change (can't be faked from the browser).
    if tool.pointer("/input_schema/properties/performedBy").is_some() {
        args.insert("performedBy".into(), json!(user.username));
    }

    let writes = tool.get("read_only") != Some(&Value::Bool(true));
    tracing::info!("tool {name} by {}{}", user.username, if writes { " (changes data)" } else { "" });
    let result = st.mcp.call(&name, Value::Object(args)).await?;
    if writes {
        st.mcp.invalidate_cache();
    }
    Ok(Json(result))
}

// -------------------------------------------------------------------------------- system

async fn status(State(st): State<AppState>, _u: CurrentUser) -> Json<Value> {
    let mut connected = st.mcp.is_connected().await;
    let mut error = st.mcp.last_error();
    let index = match st.mcp.cached("index_status", Duration::from_secs(10), Some("index_status")).await {
        Ok(v) => {
            connected = true;
            v
        }
        Err(e) => {
            error = error.or(Some(e.detail));
            Value::Null
        }
    };
    Json(json!({ "connected": connected, "transport": "stdio", "index": index, "error": error }))
}

/// Public and cross-origin, so the Tauri app can check a server address before opening it.
async fn health() -> impl IntoResponse {
    ([(header::ACCESS_CONTROL_ALLOW_ORIGIN, HeaderValue::from_static("*"))], Json(json!({ "ok": true, "app": "spil-labs-assistant" })))
}

// --------------------------------------------------------------------------------- admin

/// What the MCP handshake and tools/list returned - handy for learning MCP.
async fn admin_mcp(State(st): State<AppState>, _a: AdminUser) -> ApiResult<Json<Value>> {
    Ok(Json(json!({
        "server": st.mcp.server_info().await?,
        "tools": st.mcp.list_tools().await?,
        "command": st.settings.mcp_server_command,
        "args": st.settings.mcp_server_args,
        "transport": "stdio",
    })))
}

/// Reloads the regex rules file and re-reads the SQL Server tables.
async fn admin_reindex(State(st): State<AppState>, AdminUser(user): AdminUser) -> ApiResult<Json<Value>> {
    tracing::info!("Reindex requested by {}", user.username);
    let result = st.mcp.call("reindex", json!({})).await?;
    st.mcp.invalidate_cache();
    Ok(Json(result))
}

async fn admin_users(State(st): State<AppState>, _a: AdminUser) -> ApiResult<Json<Vec<User>>> {
    Ok(Json(st.users.list()?))
}

#[derive(Deserialize)]
struct CreateUserRequest {
    username: String,
    password: String,
    #[serde(default = "default_role")]
    role: String,
}

async fn admin_create_user(State(st): State<AppState>, AdminUser(admin): AdminUser,
                           ApiJson(body): ApiJson<CreateUserRequest>) -> ApiResult<(StatusCode, Json<User>)> {
    if !USERNAME.is_match(&body.username) {
        return Err(ApiError::invalid("username: 3-32 letters, digits, '.', '_' or '-'"));
    }
    check_password(&body.password, "password")?;
    if !crate::users::ROLES.contains(&body.role.as_str()) {
        return Err(ApiError::invalid("role: use admin or user"));
    }
    let users = st.users.clone();
    let created = blocking(move || users.create(&body.username, &body.password, &body.role)).await??;
    tracing::info!("{} created user {} ({})", admin.username, created.username, created.role);
    Ok((StatusCode::CREATED, Json(created)))
}

#[derive(Deserialize)]
struct UpdateUserRequest {
    role: Option<String>,
    active: Option<bool>,
}

fn guard_last_admin(st: &AppState, target: &User, removing_admin: bool) -> ApiResult<()> {
    if removing_admin && target.is_admin() && target.active && st.users.count_active_admins()? <= 1 {
        return Err(ApiError::bad_request("You cannot remove the last active administrator."));
    }
    Ok(())
}

async fn admin_update_user(State(st): State<AppState>, _a: AdminUser, ApiPath(id): ApiPath<i64>,
                           ApiJson(body): ApiJson<UpdateUserRequest>) -> ApiResult<Json<Value>> {
    if let Some(r) = &body.role {
        if !crate::users::ROLES.contains(&r.as_str()) {
            return Err(ApiError::invalid("role: use admin or user"));
        }
    }
    let target = st.users.get(id)?.ok_or_else(|| ApiError::not_found("User not found"))?;
    guard_last_admin(&st, &target, body.role.as_deref() == Some("user") || body.active == Some(false))?;
    let updated = st.users.update(id, body.role.as_deref(), body.active)?;
    Ok(Json(json!(updated)))
}

#[derive(Deserialize)]
struct SetPasswordRequest {
    password: String,
}

async fn admin_reset_password(State(st): State<AppState>, _a: AdminUser, ApiPath(id): ApiPath<i64>,
                              ApiJson(body): ApiJson<SetPasswordRequest>) -> ApiResult<StatusCode> {
    check_password(&body.password, "password")?;
    st.users.get(id)?.ok_or_else(|| ApiError::not_found("User not found"))?;
    let users = st.users.clone();
    blocking(move || users.set_password(id, &body.password)).await??;
    Ok(StatusCode::NO_CONTENT)
}

async fn admin_delete_user(State(st): State<AppState>, AdminUser(admin): AdminUser, ApiPath(id): ApiPath<i64>) -> ApiResult<StatusCode> {
    if id == admin.id {
        return Err(ApiError::bad_request("You cannot delete your own account."));
    }
    let target = st.users.get(id)?.ok_or_else(|| ApiError::not_found("User not found"))?;
    guard_last_admin(&st, &target, true)?;
    st.users.delete(id)?;
    Ok(StatusCode::NO_CONTENT)
}

#[allow(dead_code)]
fn _assert_send(_: Arc<AppState>) {}
