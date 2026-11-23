use crate::config::AppConfig;
use crate::observability::{compute_latency_percentiles, format_prometheus_metrics};
use crate::persistence::{is_postgres_url, DbPool};
use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use std::time::Instant;

#[derive(Serialize)]
pub struct HealthStatus {
    pub status: String,
    pub version: String,
    pub database: String,
    pub uptime_seconds: u64,
    pub allocated_ports: PortAllocations,
    pub timestamp: String,
}

#[derive(Serialize)]
pub struct PortAllocations {
    pub backend_api: u16,
    pub frontend_ui: u16,
    pub prometheus_target: u16,
    pub grafana_target: u16,
    pub mlflow_target: u16,
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct AppState {
    pub pool: DbPool,
    pub is_postgres: bool,
    pub start_time: Instant,
    pub server_port: u16,
    pub frontend_origin: String,
    pub mlflow_url: String,
    pub prometheus_url: String,
    pub grafana_url: String,
    pub github_repo: String,
    pub github_token: Option<String>,
    pub admin_api_key: Option<String>,
    pub request_timeout_secs: u64,
    pub rate_limit_rpm: u32,
    pub llm_gateway_upstream_url: String,
    pub llm_gateway_api_key: Option<String>,
    pub llm_judge_model: String,
    pub alert_webhook_url: Option<String>,
}

impl AppState {
    pub fn from_config(pool: DbPool, config: &AppConfig, start_time: Instant) -> Self {
        Self {
            pool,
            is_postgres: is_postgres_url(&config.database_url),
            start_time,
            server_port: config.server_port,
            frontend_origin: config.frontend_origin.clone(),
            mlflow_url: config.mlflow_url.clone(),
            prometheus_url: config.prometheus_url.clone(),
            grafana_url: config.grafana_url.clone(),
            github_repo: config.github_repo.clone(),
            github_token: config.github_token.clone(),
            admin_api_key: config.admin_api_key.clone(),
            request_timeout_secs: config.request_timeout_secs,
            rate_limit_rpm: config.rate_limit_rpm,
            llm_gateway_upstream_url: config.llm_gateway_upstream_url.clone(),
            llm_gateway_api_key: config.llm_gateway_api_key.clone(),
            llm_judge_model: config.llm_judge_model.clone(),
            alert_webhook_url: config.alert_webhook_url.clone(),
        }
    }

    pub fn for_test(pool: DbPool) -> Self {
        Self {
            pool,
            is_postgres: false,
            start_time: Instant::now(),
            server_port: 0,
            frontend_origin: "http://localhost:3210".to_string(),
            mlflow_url: "http://localhost:5500".to_string(),
            prometheus_url: "http://localhost:9190".to_string(),
            grafana_url: "http://localhost:3310".to_string(),
            github_repo: "enterprise/llmops-platform".to_string(),
            github_token: None,
            admin_api_key: None,
            request_timeout_secs: 30,
            rate_limit_rpm: 120,
            llm_gateway_upstream_url: "https://api.openai.com/v1".to_string(),
            llm_gateway_api_key: None,
            llm_judge_model: "gpt-4o-mini".to_string(),
            alert_webhook_url: None,
        }
    }
}

pub async fn health_check(State(state): State<std::sync::Arc<AppState>>) -> Json<HealthStatus> {
    let engine = if state.is_postgres {
        "PostgreSQL"
    } else {
        "SQLite (WAL)"
    };
    let db_status = match sqlx::query("SELECT 1").execute(&state.pool).await {
        Ok(_) => format!("Connected & Healthy ({engine})"),
        Err(e) => format!("Degraded ({engine}): {e}"),
    };

    Json(HealthStatus {
        status: "Healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        database: db_status,
        uptime_seconds: state.start_time.elapsed().as_secs(),
        allocated_ports: PortAllocations {
            backend_api: state.server_port,
            frontend_ui: 3210,
            prometheus_target: 9190,
            grafana_target: 3310,
            mlflow_target: 5500,
        },
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}

pub async fn prometheus_metrics(State(state): State<std::sync::Arc<AppState>>) -> Response {
    let total_traces: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM request_traces")
        .fetch_one(&state.pool)
        .await
        .unwrap_or((0,));

    let token_sum: (i64,) =
        sqlx::query_as("SELECT COALESCE(SUM(total_tokens), 0) FROM request_traces")
            .fetch_one(&state.pool)
            .await
            .unwrap_or((0,));

    let avg_faith: (f64,) =
        sqlx::query_as("SELECT COALESCE(AVG(faithfulness), 0.0) FROM evaluations")
            .fetch_one(&state.pool)
            .await
            .unwrap_or((0.0,));

    let active_deps: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM deployments WHERE status = 'Healthy'")
            .fetch_one(&state.pool)
            .await
            .unwrap_or((0,));

    let latency_rows: Vec<(f64,)> = sqlx::query_as("SELECT latency_ms FROM request_traces")
        .fetch_all(&state.pool)
        .await
        .unwrap_or_default();
    let pct = compute_latency_percentiles(latency_rows.into_iter().map(|r| r.0).collect());

    let metrics_text = format_prometheus_metrics(
        total_traces.0,
        pct.p50,
        pct.p95,
        pct.p99,
        token_sum.0,
        avg_faith.0,
        active_deps.0,
    );

    (
        [("content-type", "text/plain; version=0.0.4; charset=utf-8")],
        metrics_text,
    )
        .into_response()
}
