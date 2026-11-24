pub mod alerts;
pub mod cicd;
pub mod costs;
pub mod deployments;
pub mod evaluations;
pub mod experiments;
pub mod gateway;
pub mod health;
pub mod integrations;
pub mod middleware;
pub mod models;
pub mod monitoring;
pub mod openapi;
pub mod overview;
pub mod prompts;
pub mod traces;

pub use self::middleware::build_cors_layer;

use self::health::AppState;
use self::middleware::{api_key_auth_middleware, rate_limit_middleware, RateLimiterState};
use axum::{
    http::StatusCode,
    middleware as axum_mw,
    routing::{get, patch, post, put},
    Router,
};
use std::sync::Arc;
use std::time::Duration;
use tower_http::timeout::TimeoutLayer;

pub fn build_router(state: Arc<AppState>) -> Router {
    let timeout_secs = state.request_timeout_secs.max(1);
    let rate_limiter = RateLimiterState::new(state.rate_limit_rpm);

    let api_routes = Router::new()
        // Health, Metrics, and OpenAPI Docs
        .route("/health", get(health::health_check))
        .route("/metrics", get(health::prometheus_metrics))
        .route("/openapi.json", get(openapi::get_openapi_spec))
        .route("/docs", get(openapi::get_swagger_ui))
        // Overview
        .route("/overview/stats", get(overview::get_overview_stats))
        // Models
        .route(
            "/models",
            get(models::list_models).post(models::create_model),
        )
        .route("/models/:id", get(models::get_model))
        .route("/models/:id/stage", patch(models::update_stage))
        // Experiments
        .route(
            "/experiments",
            get(experiments::list_experiments).post(experiments::create_experiment),
        )
        .route(
            "/experiments/compare",
            get(experiments::compare_experiments),
        )
        // Evaluations
        .route(
            "/evaluations",
            get(evaluations::list_evaluations).post(evaluations::trigger_evaluation),
        )
        .route("/evaluations/:id", get(evaluations::get_evaluation_detail))
        .route(
            "/evaluations/metrics/definitions",
            get(evaluations::get_metric_definitions),
        )
        // Monitoring
        .route(
            "/monitoring/metrics",
            get(monitoring::get_monitoring_metrics),
        )
        .route(
            "/monitoring/latency-distribution",
            get(monitoring::get_latency_distribution),
        )
        // Traces & AI Gateway
        .route(
            "/traces",
            get(traces::list_traces).post(traces::create_trace),
        )
        .route("/traces/:id", get(traces::get_trace))
        .route("/gateway/chat/completions", post(gateway::chat_completions))
        // Prompt Registry & A/B Comparison
        .route(
            "/prompts",
            get(prompts::list_prompts).post(prompts::create_prompt),
        )
        .route("/prompts/compare", get(prompts::compare_prompts))
        .route(
            "/prompts/:id",
            patch(prompts::update_prompt).delete(prompts::delete_prompt),
        )
        // Alerting Rules & Webhook Notifications
        .route(
            "/alerts/rules",
            get(alerts::list_alert_rules).post(alerts::create_alert_rule),
        )
        .route(
            "/alerts/rules/:id",
            patch(alerts::update_alert_rule).delete(alerts::delete_alert_rule),
        )
        .route("/alerts/rules/:id/test", post(alerts::test_alert_webhook))
        .route("/alerts/events", get(alerts::list_alert_events))
        .route("/alerts/evaluate", post(alerts::evaluate_alerts_now))
        // Costs & Pricing
        .route("/costs/summary", get(costs::get_cost_summary))
        .route("/costs/comparison", get(costs::get_token_comparison))
        .route("/costs/pricing", get(costs::list_pricing_catalog))
        .route("/costs/budget", patch(costs::update_budget))
        // Deployments
        .route("/deployments", get(deployments::list_deployments))
        .route(
            "/deployments/:id/restart",
            post(deployments::restart_deployment),
        )
        // CI/CD Pipelines
        .route("/cicd/workflows", get(cicd::list_workflows))
        .route("/cicd/workflows/trigger", post(cicd::trigger_workflow))
        // Integrations
        .route("/integrations", get(integrations::list_integrations))
        .route(
            "/integrations/:name/test",
            post(integrations::test_integration),
        )
        .route("/integrations/:name", put(integrations::update_integration))
        .layer(axum_mw::from_fn_with_state(
            state.clone(),
            api_key_auth_middleware,
        ))
        .layer(axum_mw::from_fn_with_state(
            rate_limiter,
            rate_limit_middleware,
        ))
        .with_state(state.clone());

    Router::new()
        .nest("/api/v1", api_routes)
        // Root fallback health probe
        .route("/health", get(health::health_check).with_state(state))
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(timeout_secs),
        ))
}
