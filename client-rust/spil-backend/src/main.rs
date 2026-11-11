//! `spil-backend` — starts the SPIL Labs web backend.
//!
//! ```text
//! spil-backend.exe                       settings from config\server.env, http://127.0.0.1:8000
//! spil-backend.exe --lan                 listen on all network cards (other PCs can connect)
//! spil-backend.exe --config other.env --port 9000
//! ```
//! Run it from the project folder: relative paths in the settings are relative to the current folder.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use mcp_client::{McpClient, ServerConfig};
use spil_backend::config::Settings;
use spil_backend::mcp::McpService;
use spil_backend::users::UserStore;
use spil_backend::{bootstrap_admin, build_app, load_env_file, AppState};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info,tower_http=warn,rmcp=warn".into()),
        )
        .with_target(false)
        .init();

    if let Err(e) = run().await {
        tracing::error!("{e}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    // --- command line ---
    let mut config = PathBuf::from("config/server.env");
    let (mut lan, mut port): (bool, Option<u16>) = (false, None);
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--config" => config = args.next().ok_or("--config needs a file")?.into(),
            "--port" => port = Some(args.next().ok_or("--port needs a number")?.parse()?),
            "--lan" => lan = true,
            "-h" | "--help" => {
                println!("usage: spil-backend [--config config/server.env] [--port 8000] [--lan]");
                return Ok(());
            }
            other => return Err(format!("unknown option '{other}' (try --help)").into()),
        }
    }

    load_env_file(&config);
    let base = std::env::current_dir()?;
    let mut settings = Settings::from_env(&base)?;
    if lan {
        settings.host = "0.0.0.0".into();
    }
    if let Some(p) = port {
        settings.port = p;
    }

    // --- accounts (same users.db as the old backend) ---
    let users = UserStore::open(&settings.users_db_path)?;
    bootstrap_admin(&users, &settings.admin_username, settings.admin_password.as_deref())?;
    tracing::info!("Accounts: {} ({} users)", settings.users_db_path.display(), users.count()?);

    // --- MCP client: starts the C# server as a child process and talks over STDIO ---
    let mut server = ServerConfig::new(settings.mcp_server_command.clone());
    server.args = settings.mcp_server_args.clone();
    server.request_timeout = Duration::from_secs_f64(settings.mcp_request_timeout_secs);
    server.startup_timeout = Duration::from_secs_f64(settings.mcp_startup_timeout_secs);
    let mcp = McpService::new(Arc::new(McpClient::new(server)));

    let addr: SocketAddr = format!("{}:{}", settings.host, settings.port).parse()?;
    let state = AppState::new(settings, users, mcp);

    // Start the MCP server in the background: the website is usable at once and
    // /api/status reports "starting" until the index is ready.
    let mcp_bg = state.mcp.clone();
    tokio::spawn(async move { mcp_bg.connect().await });

    let app = build_app(state.clone());
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("SPIL Labs assistant on http://{addr}  (Ctrl+C to stop)");

    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
            tracing::info!("Stopping...");
        })
        .await?;

    state.mcp.shutdown().await;
    Ok(())
}
