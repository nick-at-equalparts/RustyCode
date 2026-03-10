mod api;
mod app;
mod config;
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
    #[arg(short, long)]
    theme: Option<String>,
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
    let (base_url, auto_detected) = match cli.server {
        Some(url) => (url, false),
        None => match auto_detect_server().await {
            Some(url) => (url, true),
            None => ("http://127.0.0.1:4000".to_string(), false),
        },
    };

    tracing::info!("Connecting to OpenCode server at {}", base_url);

    // ── API client + health check ───────────────────────────────────
    let client = ApiClient::new(base_url.clone());

    // Skip redundant health check when auto-detect already verified the server.
    if !auto_detected {
        match client.health().await {
            Ok(resp) => {
                tracing::info!("Server healthy, version: {}", resp.version);
            }
            Err(e) => {
                tracing::warn!("Health check failed: {}. Proceeding anyway.", e);
            }
        }
    }

    // ── Build app ──────────────────────────────────────────────────
    let config = config::Config::load();
    let mut app = App::new(client, base_url);

    // CLI flag > saved config > "default"
    app.theme_name = cli
        .theme
        .or(config.theme)
        .unwrap_or_else(|| "default".to_string());

    // Data loading happens inside the event loop, after the first render,
    // so the TUI appears instantly.

    // ── Run the TUI event loop ──────────────────────────────────────
    app::event_loop::run(&mut app)
        .await
        .context("TUI event loop error")?;

    Ok(())
}

/// Try to find a running OpenCode server on common local ports.
///
/// Probes all ports concurrently and returns the first one that responds.
async fn auto_detect_server() -> Option<String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_millis(500))
        .build()
        .ok()?;

    // Probe all ports concurrently
    let probes: Vec<_> = AUTO_DETECT_PORTS
        .iter()
        .map(|&port| {
            let client = client.clone();
            async move {
                let url = format!("http://127.0.0.1:{}", port);
                let health_url = format!("{}/global/health", url);
                match client.get(&health_url).send().await {
                    Ok(resp) if resp.status().is_success() => Some(url),
                    _ => None,
                }
            }
        })
        .collect();

    let results = futures::future::join_all(probes).await;
    for (i, result) in results.into_iter().enumerate() {
        if let Some(url) = result {
            tracing::info!(
                "Auto-detected OpenCode server on port {}",
                AUTO_DETECT_PORTS[i]
            );
            return Some(url);
        }
    }

    tracing::debug!("No OpenCode server detected on common ports");
    None
}
