use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tokio::signal;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use llmops_backend::{api, config, persistence};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize structured logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(false)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to initialize tracing subscriber");

    let start_time = Instant::now();
    info!("Starting LLMOps Control Center Backend API...");

    // Load configuration
    let config = config::AppConfig::load();

    // Initialize database pool and run migrations
    let pool = persistence::init_db(&config.database_url).await?;

    // Create shared application state
    let state = Arc::new(api::health::AppState {
        pool,
        start_time,
        server_port: config.server_port,
        frontend_origin: config.frontend_origin.clone(),
        mlflow_url: config.mlflow_url,
        prometheus_url: config.prometheus_url,
        grafana_url: config.grafana_url,
        github_repo: config.github_repo,
        github_token: config.github_token,
    });

    // Configure CORS for isolated frontend origin
    let cors = CorsLayer::new()
        .allow_origin(tower_http::cors::AllowOrigin::predicate(|origin, _| {
            let s = origin.to_str().unwrap_or("");
            s.starts_with("http://localhost:3210")
                || s.starts_with("http://127.0.0.1:3210")
                || s.starts_with("http://localhost:3000")
        }))
        .allow_methods(Any)
        .allow_headers(Any);

    // Build Axum router
    let app = api::build_router(state)
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    let addr: SocketAddr = format!("{}:{}", config.server_host, config.server_port).parse()?;
    info!("Binding HTTP server to {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;

    info!("========================================================");
    info!("   LLMOPS CONTROL CENTER — RUST BACKEND OPERATIONAL     ");
    info!("========================================================");
    info!(" API Server:         http://localhost:{}", config.server_port);
    info!(" Health Endpoint:    http://localhost:{}/api/v1/health", config.server_port);
    info!(" Prometheus Metrics: http://localhost:{}/api/v1/metrics", config.server_port);
    info!(" Target Frontend:    {}", config.frontend_origin);
    info!(" Database:           {}", config.database_url);
    info!(" Environment:        {}", config.app_env);
    info!("========================================================");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("LLMOps Control Center Backend shut down cleanly.");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl+C shutdown signal.");
        },
        _ = terminate => {
            info!("Received SIGTERM shutdown signal.");
        },
    }
}
