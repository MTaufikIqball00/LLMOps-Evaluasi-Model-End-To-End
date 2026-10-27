pub mod cicd;
pub mod costs;
pub mod deployments;
pub mod evaluations;
pub mod experiments;
pub mod health;
pub mod integrations;
pub mod models;
pub mod monitoring;
pub mod overview;
pub mod traces;

use axum::{
    routing::{get, patch, post, put},
    Router,
};
use std::sync::Arc;
use self::health::AppState;

pub fn build_router(state: Arc<AppState>) -> Router {
    let api_routes = Router::new()
        // Health and Observability
        .route("/health", get(health::health_check))
        .route("/metrics", get(health::prometheus_metrics))
        // Overview
        .route("/overview/stats", get(overview::get_overview_stats))
        // Models
        .route("/models", get(models::list_models).post(models::create_model))
        .route("/models/:id", get(models::get_model))
        .route("/models/:id/stage", patch(models::update_stage))
        // Experiments
        .route(
            "/experiments",
            get(experiments::list_experiments).post(experiments::create_experiment),
        )
        .route("/experiments/compare", get(experiments::compare_experiments))
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
        // Traces
        .route("/traces", get(traces::list_traces).post(traces::create_trace))
        .route("/traces/:id", get(traces::get_trace))
        // Costs & Pricing
        .route("/costs/summary", get(costs::get_cost_summary))
        .route("/costs/pricing", get(costs::list_pricing_catalog))
        .route("/costs/budget", patch(costs::update_budget))
        // Deployments
        .route("/deployments", get(deployments::list_deployments))
        .route("/deployments/:id/restart", post(deployments::restart_deployment))
        // CI/CD Pipelines
        .route("/cicd/workflows", get(cicd::list_workflows))
        .route("/cicd/workflows/trigger", post(cicd::trigger_workflow))
        // Integrations
        .route("/integrations", get(integrations::list_integrations))
        .route("/integrations/:name/test", post(integrations::test_integration))
        .route("/integrations/:name", put(integrations::update_integration))
        .with_state(state.clone());

    Router::new()
        .nest("/api/v1", api_routes)
        // Root fallback health probe
        .route("/health", get(health::health_check).with_state(state))
}
