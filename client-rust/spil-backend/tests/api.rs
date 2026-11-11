//! API tests with a fake MCP server, so they run without .NET or SQL Server.
//! Run:  cargo test -p spil-backend

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use http_body_util::BodyExt;
use mcp_client::ClientError;
use serde_json::{json, Value};
use spil_backend::config::Settings;
use spil_backend::mcp::{McpBackend, McpService};
use spil_backend::users::UserStore;
use spil_backend::{bootstrap_admin, build_app, AppState};
use tower::ServiceExt;

#[derive(Default)]
struct FakeMcp {
    calls: Mutex<Vec<(String, Value)>>,
}

#[async_trait]
impl McpBackend for FakeMcp {
    async fn connect(&self) -> Result<(), ClientError> { Ok(()) }
    async fn is_connected(&self) -> bool { true }
    async fn server_info(&self) -> Result<Value, ClientError> {
        Ok(json!({ "serverInfo": { "name": "semantic-search" }, "protocolVersion": "2025-11-25" }))
    }
    async fn list_tools(&self) -> Result<Value, ClientError> {
        Ok(json!([
            { "name": "semantic_search", "read_only": true, "input_schema": { "properties": { "query": {} } } },
            { "name": "get_record", "read_only": true, "input_schema": { "properties": {} } },
            { "name": "list_orders", "read_only": true, "input_schema": { "properties": {} } },
            { "name": "create_consultation", "read_only": false, "input_schema": { "properties": { "title": {}, "performedBy": {} } } },
            { "name": "reindex", "read_only": false, "input_schema": { "properties": {} } }
        ]))
    }
    async fn call_tool(&self, name: &str, args: Value) -> Result<Value, ClientError> {
        self.calls.lock().unwrap().push((name.to_string(), args.clone()));
        match name {
            "semantic_search" => Ok(json!({ "query": args["query"], "mode": args["mode"], "method": "regex rules",
                "query_concepts": ["Delivery problem"],
                "results": [{ "source": "articles", "id": 2, "title": "My order has not arrived yet", "score": 0.71 }] })),
            "get_record" if args["id"] == 999 => Err(ClientError::Tool("Error executing tool get_record: No record 999 in 'products'.".into())),
            "get_record" => Ok(json!({ "source": args["source"], "id": args["id"], "fields": {} })),
            "list_orders" => Ok(json!({ "count": 3, "orders": [
                { "order_number": "ORD-00001", "status": "Paid" },
                { "order_number": "ORD-00002", "status": "Invoiced" },
                { "order_number": "ORD-00003", "status": "Confirmed" }] })),
            "index_status" => Ok(json!({ "method": "regex rules", "records": 60, "concepts": 42 })),
            "list_sources" => Ok(json!({ "sources": [{ "name": "products" }] })),
            "reindex" => Ok(json!({ "records": 60 })),
            "create_consultation" if args.get("title").is_none() => Err(ClientError::Tool("title is required (at least 3 characters).".into())),
            "create_consultation" => Ok(json!({ "message": "Consultation 35 created.", "by": args["performedBy"] })),
            _ => Err(ClientError::Tool(format!("unknown tool {name}"))),
        }
    }
    async fn shutdown(&self) {}
}

struct Ctx {
    app: Router,
    fake: Arc<FakeMcp>,
    _dir: tempfile::TempDir,
}

fn settings(db: PathBuf) -> Settings {
    Settings {
        mcp_server_command: "fake.exe".into(),
        mcp_server_args: vec![],
        mcp_request_timeout_secs: 5.0,
        mcp_startup_timeout_secs: 5.0,
        jwt_secret: "test-secret-test-secret-test-secret-123".into(),
        jwt_expire_minutes: 60,
        users_db_path: db,
        admin_username: "admin".into(),
        admin_password: Some("Admin-pass-123".into()),
        login_max_attempts: 3,
        login_window_seconds: 300,
        cors_origins: vec![],
        frontend_dist: PathBuf::from("/nonexistent"),
        host: "127.0.0.1".into(),
        port: 0,
    }
}

fn ctx_with_db(db: Option<PathBuf>) -> Ctx {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("users.db");
    if let Some(src) = db {
        std::fs::copy(src, &path).unwrap();
    }
    let s = settings(path.clone());
    let users = UserStore::open(&path).unwrap();
    bootstrap_admin(&users, "admin", Some("Admin-pass-123")).unwrap();
    let fake = Arc::new(FakeMcp::default());
    let state = AppState::new(s, users, McpService::new(fake.clone()));
    Ctx { app: build_app(state), fake, _dir: dir }
}

fn ctx() -> Ctx {
    ctx_with_db(None)
}

async fn send(app: &Router, method: &str, uri: &str, token: Option<&str>, body: Option<Value>) -> (StatusCode, Value) {
    let mut req = Request::builder().method(method).uri(uri);
    if let Some(t) = token {
        req = req.header("authorization", format!("Bearer {t}"));
    }
    let req = match body {
        Some(b) => req.header("content-type", "application/json").body(Body::from(b.to_string())).unwrap(),
        None => req.body(Body::empty()).unwrap(),
    };
    let res = app.clone().oneshot(req).await.unwrap();
    let status = res.status();
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    (status, serde_json::from_slice(&bytes).unwrap_or(Value::Null))
}

async fn login(app: &Router, user: &str, pw: &str) -> String {
    let (s, body) = send(app, "POST", "/api/auth/login", None, Some(json!({ "username": user, "password": pw }))).await;
    assert_eq!(s, StatusCode::OK, "{body}");
    body["access_token"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn accounts_from_the_old_python_backend_still_work() {
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/python-users.db");
    let c = ctx_with_db(Some(fixture));
    let t = login(&c.app, "alice", "Alice-pass-123").await;
    let (_, me) = send(&c.app, "GET", "/api/auth/me", Some(&t), None).await;
    assert_eq!(me["username"], "alice");
    assert_eq!(me["role"], "user");
    let t = login(&c.app, "admin", "Admin-pass-123").await;
    let (_, h) = send(&c.app, "GET", "/api/search/history", Some(&t), None).await;
    assert_eq!(h[0]["query"], "unhappy clients");
}

#[tokio::test]
async fn requires_login() {
    let c = ctx();
    assert_eq!(send(&c.app, "POST", "/api/search", None, Some(json!({ "query": "x" }))).await.0, StatusCode::UNAUTHORIZED);
    assert_eq!(send(&c.app, "GET", "/api/sources", Some("junk"), None).await.0, StatusCode::UNAUTHORIZED);
    let (s, body) = send(&c.app, "GET", "/api/health", None, None).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(body["ok"], true);
}

#[tokio::test]
async fn search_goes_through_mcp() {
    let c = ctx();
    let t = login(&c.app, "admin", "Admin-pass-123").await;
    let (s, body) = send(&c.app, "POST", "/api/search", Some(&t), Some(json!({ "query": "  parcel   late ", "top_k": 3, "source": "articles" }))).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(body["results"][0]["id"], 2);
    assert_eq!(c.fake.calls.lock().unwrap().last().unwrap().1,
               json!({ "query": "parcel late", "topK": 3, "mode": "semantic", "source": "articles" }));
    let (_, h) = send(&c.app, "GET", "/api/search/history", Some(&t), None).await;
    assert_eq!(h[0]["query"], "parcel late");
}

#[tokio::test]
async fn validation() {
    let c = ctx();
    let t = login(&c.app, "admin", "Admin-pass-123").await;
    for bad in [json!({ "query": "   " }), json!({ "query": "x", "top_k": 500 }), json!({ "query": "x", "source": "a;drop" }),
                json!({ "query": "x", "mode": "hybrid" }), json!({ "nope": 1 })] {
        let (s, body) = send(&c.app, "POST", "/api/search", Some(&t), Some(bad.clone())).await;
        assert_eq!(s, StatusCode::UNPROCESSABLE_ENTITY, "{bad} -> {body}");
        assert!(body["detail"].is_string());
    }
    // pattern mode keeps the regex exactly as typed
    send(&c.app, "POST", "/api/search", Some(&t), Some(json!({ "query": r" E-\d{3}  ", "mode": "pattern" }))).await;
    assert_eq!(c.fake.calls.lock().unwrap().last().unwrap().1["query"], r"E-\d{3}");
}

#[tokio::test]
async fn record_not_found_maps_to_404() {
    let c = ctx();
    let t = login(&c.app, "admin", "Admin-pass-123").await;
    assert_eq!(send(&c.app, "GET", "/api/records/products/5", Some(&t), None).await.0, StatusCode::OK);
    let (s, body) = send(&c.app, "GET", "/api/records/products/999", Some(&t), None).await;
    assert_eq!(s, StatusCode::NOT_FOUND);
    assert_eq!(body["detail"], "No record 999 in 'products'.");
}

#[tokio::test]
async fn roles_and_user_management() {
    let c = ctx();
    let admin = login(&c.app, "admin", "Admin-pass-123").await;
    let (s, _) = send(&c.app, "POST", "/api/admin/users", Some(&admin), Some(json!({ "username": "alice", "password": "short" }))).await;
    assert_eq!(s, StatusCode::UNPROCESSABLE_ENTITY);
    let (s, alice_user) = send(&c.app, "POST", "/api/admin/users", Some(&admin), Some(json!({ "username": "alice", "password": "Alice-pass-123" }))).await;
    assert_eq!(s, StatusCode::CREATED);
    let (s, _) = send(&c.app, "POST", "/api/admin/users", Some(&admin), Some(json!({ "username": "ALICE", "password": "Alice-pass-123" }))).await;
    assert_eq!(s, StatusCode::CONFLICT);

    let alice = login(&c.app, "alice", "Alice-pass-123").await;
    assert_eq!(send(&c.app, "POST", "/api/admin/reindex", Some(&alice), Some(json!({}))).await.0, StatusCode::FORBIDDEN);
    assert_eq!(send(&c.app, "POST", "/api/admin/reindex", Some(&admin), Some(json!({}))).await.0, StatusCode::OK);

    let id = alice_user["id"].as_i64().unwrap();
    assert_eq!(send(&c.app, "PATCH", &format!("/api/admin/users/{id}"), Some(&admin), Some(json!({ "active": false }))).await.0, StatusCode::OK);
    assert_eq!(send(&c.app, "GET", "/api/auth/me", Some(&alice), None).await.0, StatusCode::UNAUTHORIZED);

    let (_, me) = send(&c.app, "GET", "/api/auth/me", Some(&admin), None).await;
    let my_id = me["id"].as_i64().unwrap();
    assert_eq!(send(&c.app, "PATCH", &format!("/api/admin/users/{my_id}"), Some(&admin), Some(json!({ "role": "user" }))).await.0, StatusCode::BAD_REQUEST);
    assert_eq!(send(&c.app, "DELETE", &format!("/api/admin/users/{my_id}"), Some(&admin), None).await.0, StatusCode::BAD_REQUEST);
    assert_eq!(send(&c.app, "DELETE", &format!("/api/admin/users/{id}"), Some(&admin), None).await.0, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn login_rate_limit() {
    let c = ctx();
    for _ in 0..3 {
        let (s, _) = send(&c.app, "POST", "/api/auth/login", None, Some(json!({ "username": "admin", "password": "nope" }))).await;
        assert_eq!(s, StatusCode::UNAUTHORIZED);
    }
    let (s, body) = send(&c.app, "POST", "/api/auth/login", None, Some(json!({ "username": "admin", "password": "nope" }))).await;
    assert_eq!(s, StatusCode::TOO_MANY_REQUESTS);
    assert!(body["detail"].as_str().unwrap().contains("Too many failed attempts"));
}

#[tokio::test]
async fn change_password_logs_out_old_tokens() {
    let c = ctx();
    let t = login(&c.app, "admin", "Admin-pass-123").await;
    let (s, _) = send(&c.app, "POST", "/api/auth/change-password", Some(&t),
                      Some(json!({ "current_password": "wrong", "new_password": "New-pass-4567" }))).await;
    assert_eq!(s, StatusCode::BAD_REQUEST);
    let (s, _) = send(&c.app, "POST", "/api/auth/change-password", Some(&t),
                      Some(json!({ "current_password": "Admin-pass-123", "new_password": "New-pass-4567" }))).await;
    assert_eq!(s, StatusCode::NO_CONTENT);
    assert_eq!(send(&c.app, "GET", "/api/auth/me", Some(&t), None).await.0, StatusCode::UNAUTHORIZED);
    login(&c.app, "admin", "New-pass-4567").await;
}

#[tokio::test]
async fn tools_page() {
    let c = ctx();
    let admin = login(&c.app, "admin", "Admin-pass-123").await;
    send(&c.app, "POST", "/api/admin/users", Some(&admin), Some(json!({ "username": "bob", "password": "Bob-pass-1234" }))).await;
    let bob = login(&c.app, "bob", "Bob-pass-1234").await;

    let (_, tools) = send(&c.app, "GET", "/api/tools", Some(&bob), None).await;
    let find = |n: &str| tools.as_array().unwrap().iter().find(|t| t["name"] == n).unwrap().clone();
    assert_eq!(find("create_consultation")["allowed"], true);
    assert_eq!(find("reindex")["allowed"], false);

    // performedBy is always the signed-in user, even if the browser sends something else
    let (s, body) = send(&c.app, "POST", "/api/tools/create_consultation", Some(&bob),
                         Some(json!({ "arguments": { "title": "Payroll", "performedBy": "mallory" } }))).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(body["by"], "bob");

    let (s, body) = send(&c.app, "POST", "/api/tools/create_consultation", Some(&bob), Some(json!({ "arguments": {} }))).await;
    assert_eq!(s, StatusCode::BAD_REQUEST);
    assert!(body["detail"].as_str().unwrap().contains("title is required"));

    assert_eq!(send(&c.app, "POST", "/api/tools/reindex", Some(&bob), Some(json!({}))).await.0, StatusCode::FORBIDDEN);
    assert_eq!(send(&c.app, "POST", "/api/tools/reindex", Some(&admin), Some(json!({}))).await.0, StatusCode::OK);
    assert_eq!(send(&c.app, "POST", "/api/tools/no_such_tool", Some(&bob), Some(json!({}))).await.0, StatusCode::NOT_FOUND);
    assert_eq!(send(&c.app, "POST", "/api/tools/Bad-Name", Some(&bob), Some(json!({}))).await.0, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(send(&c.app, "POST", "/api/tools/get_record", None, Some(json!({}))).await.0, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn chat_endpoint() {
    let c = ctx();
    let t = login(&c.app, "admin", "Admin-pass-123").await;
    let (s, body) = send(&c.app, "POST", "/api/chat", Some(&t), Some(json!({ "message": "where is my parcel" }))).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(body["tool"], "semantic_search");
    assert!(body["reply"].as_str().unwrap().contains("1 result"));

    // list_orders is offered by the fake server; unpaid filters to Confirmed + Invoiced
    let (_, body) = send(&c.app, "POST", "/api/chat", Some(&t), Some(json!({ "message": "list unpaid orders" }))).await;
    assert_eq!(body["tool"], "list_orders");
    assert_eq!(body["data"]["count"], 2);

    // list_consultations isn't offered by the fake server, so the chat falls back to meaning search
    let (_, body) = send(&c.app, "POST", "/api/chat", Some(&t), Some(json!({ "message": "show completed consultations" }))).await;
    assert_eq!(body["tool"], "semantic_search");
    assert_eq!(send(&c.app, "POST", "/api/chat", Some(&t), Some(json!({ "message": "   " }))).await.0, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn status_and_admin_mcp() {
    let c = ctx();
    let t = login(&c.app, "admin", "Admin-pass-123").await;
    let (_, s) = send(&c.app, "GET", "/api/status", Some(&t), None).await;
    assert_eq!(s["connected"], true);
    assert_eq!(s["index"]["records"], 60);
    assert_eq!(s["transport"], "stdio");
    let (_, m) = send(&c.app, "GET", "/api/admin/mcp", Some(&t), None).await;
    assert_eq!(m["tools"][0]["name"], "semantic_search");
    assert_eq!(send(&c.app, "GET", "/api/nope", Some(&t), None).await.0, StatusCode::NOT_FOUND);
}
