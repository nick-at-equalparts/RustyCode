mod api;
mod app;
mod input;
mod types;
mod ui;

use anyhow::{Context, Result};
use clap::Parser;
use tracing_subscriber::EnvFilter;

use api::ApiClient;
use app::App;

/// RustyCode - a terminal user interface for the OpenCode server.
#[derive(Parser, Debug)]
#[command(name = "rustycode", version, about)]
struct Cli {
    /// Server URL to connect to.
    ///
    /// Falls back to the `OPENCODE_SERVER` environment variable, then auto-detects
    /// a running server on common ports, and finally defaults to http://127.0.0.1:4000.
    #[arg(short, long, env = "OPENCODE_SERVER")]
    server: Option<String>,

    /// Enable debug logging (sets RUST_LOG=debug if not already set).
    #[arg(short, long)]
    debug: bool,

    /// Theme name to use for the TUI.
    #[arg(short, long, default_value = "default")]
    theme: String,
}

/// Default ports to probe when auto-detecting a local OpenCode server.
const AUTO_DETECT_PORTS: &[u16] = &[4000, 4001, 4002, 4100];

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // ── Tracing ─────────────────────────────────────────────────────
    let filter = if cli.debug {
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("debug"))
    } else {
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn"))
    };

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .init();

    // ── Resolve server URL ──────────────────────────────────────────
    let base_url = match cli.server {
        Some(url) => url,
        None => auto_detect_server()
            .await
            .unwrap_or_else(|| "http://127.0.0.1:4000".to_string()),
    };

    tracing::info!("Connecting to OpenCode server at {}", base_url);

    // ── API client + health check ───────────────────────────────────
    let client = ApiClient::new(base_url.clone());

    match client.health().await {
        Ok(resp) => {
            tracing::info!("Server healthy, version: {}", resp.version);
        }
        Err(e) => {
            tracing::warn!("Health check failed: {}. Proceeding anyway.", e);
        }
    }

    // ── Build app and load data ─────────────────────────────────────
    let mut app = App::new(client, base_url);
    app.theme_name = cli.theme;

    if let Err(e) = app.load_initial_data().await {
        tracing::warn!("Failed to load initial data: {}", e);
        app.status_message = format!("Partial load: {}", e);
    }

    // ── Run the TUI event loop ──────────────────────────────────────
    app::event_loop::run(&mut app)
        .await
        .context("TUI event loop error")?;

    Ok(())
}

/// Try to find a running OpenCode server on common local ports.
///
/// Returns `Some(url)` for the first port that responds to a health check,
/// or `None` if no server is found.
async fn auto_detect_server() -> Option<String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_millis(500))
        .build()
        .ok()?;

    for port in AUTO_DETECT_PORTS {
        let url = format!("http://127.0.0.1:{}", port);
        let health_url = format!("{}/global/health", url);
        if let Ok(resp) = client.get(&health_url).send().await {
            if resp.status().is_success() {
                tracing::info!("Auto-detected OpenCode server on port {}", port);
                return Some(url);
            }
        }
    }

    tracing::debug!("No OpenCode server detected on common ports");
    None
}
