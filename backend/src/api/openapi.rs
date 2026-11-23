use axum::{
    response::{Html, IntoResponse, Response},
    Json,
};
use serde_json::{json, Map, Value};

pub async fn get_openapi_spec() -> Json<Value> {
    let mut paths = Map::new();

    paths.insert(
        "/health".to_string(),
        json!({ "get": { "summary": "System Health & DB Engine Probe", "responses": { "200": { "description": "Healthy status" } } } }),
    );
    paths.insert(
        "/metrics".to_string(),
        json!({ "get": { "summary": "Prometheus OpenMetrics Exposition", "responses": { "200": { "description": "text/plain Prometheus metrics" } } } }),
    );
    paths.insert(
        "/overview/stats".to_string(),
        json!({ "get": { "summary": "Aggregated Dashboard KPIs & Real Percentiles", "responses": { "200": { "description": "Overview statistics" } } } }),
    );
    paths.insert(
        "/models".to_string(),
        json!({
            "get": { "summary": "List Registered Models", "parameters": [{ "name": "stage", "in": "query" }, { "name": "framework", "in": "query" }], "responses": { "200": { "description": "Model list" } } },
            "post": { "summary": "Register New Model", "responses": { "200": { "description": "Created model" } } }
        }),
    );
    paths.insert(
        "/models/{id}/stage".to_string(),
        json!({ "patch": { "summary": "Promote or Archive Model Stage", "responses": { "200": { "description": "Updated model" } } } }),
    );
    paths.insert(
        "/experiments".to_string(),
        json!({
            "get": { "summary": "List Experiment Runs", "responses": { "200": { "description": "Experiment list" } } },
            "post": { "summary": "Log Experiment Run", "responses": { "200": { "description": "Created experiment" } } }
        }),
    );
    paths.insert(
        "/experiments/compare".to_string(),
        json!({ "get": { "summary": "Side-by-Side Run Comparison", "responses": { "200": { "description": "Diff matrix" } } } }),
    );
    paths.insert(
        "/evaluations".to_string(),
        json!({
            "get": { "summary": "List Evaluation Benchmarks", "responses": { "200": { "description": "Evaluations list" } } },
            "post": { "summary": "Trigger Async Evaluation (Native Ragas or LLM Judge)", "responses": { "200": { "description": "Queued job" } } }
        }),
    );
    paths.insert(
        "/evaluations/{id}".to_string(),
        json!({ "get": { "summary": "Get Evaluation & Test Cases", "responses": { "200": { "description": "Evaluation details" } } } }),
    );
    paths.insert(
        "/evaluations/metrics/definitions".to_string(),
        json!({ "get": { "summary": "Evaluation Metric Formulas", "responses": { "200": { "description": "Metric definitions" } } } }),
    );
    paths.insert(
        "/monitoring/metrics".to_string(),
        json!({ "get": { "summary": "Real Rolling Telemetry & PromQL Series", "responses": { "200": { "description": "Timeseries metrics" } } } }),
    );
    paths.insert(
        "/monitoring/latency-distribution".to_string(),
        json!({ "get": { "summary": "Latency Histogram Buckets", "responses": { "200": { "description": "Buckets" } } } }),
    );
    paths.insert(
        "/traces".to_string(),
        json!({
            "get": { "summary": "List Scrubbed Request Traces", "responses": { "200": { "description": "Trace list" } } },
            "post": { "summary": "Ingest Trace with Scrubbing & Atomic Budget Spend", "responses": { "200": { "description": "Persisted trace" } } }
        }),
    );
    paths.insert(
        "/gateway/chat/completions".to_string(),
        json!({ "post": { "summary": "OpenAI-Compatible AI Gateway with Auto Tracing & Costing", "responses": { "200": { "description": "Chat completion + trace_id" } } } }),
    );
    paths.insert(
        "/costs/summary".to_string(),
        json!({ "get": { "summary": "Spend, Pricing Breakdown & Budget Projection", "responses": { "200": { "description": "Cost summary" } } } }),
    );
    paths.insert(
        "/costs/pricing".to_string(),
        json!({ "get": { "summary": "Provider Pricing Catalog", "responses": { "200": { "description": "Pricing catalog" } } } }),
    );
    paths.insert(
        "/costs/budget".to_string(),
        json!({ "patch": { "summary": "Update Budget Limit & Alert Threshold", "responses": { "200": { "description": "Updated budget" } } } }),
    );
    paths.insert(
        "/prompts".to_string(),
        json!({
            "get": { "summary": "List Prompt Templates", "responses": { "200": { "description": "Prompts list" } } },
            "post": { "summary": "Create Prompt Version", "responses": { "200": { "description": "Created prompt" } } }
        }),
    );
    paths.insert(
        "/prompts/{id}".to_string(),
        json!({
            "patch": { "summary": "Update Prompt Template or Stage", "responses": { "200": { "description": "Updated prompt" } } },
            "delete": { "summary": "Delete Prompt Version", "responses": { "200": { "description": "Deleted" } } }
        }),
    );
    paths.insert(
        "/prompts/compare".to_string(),
        json!({ "get": { "summary": "A/B Prompt Diff & Variable Comparison", "responses": { "200": { "description": "Comparison diff" } } } }),
    );
    paths.insert(
        "/alerts/rules".to_string(),
        json!({
            "get": { "summary": "List Alert Rules", "responses": { "200": { "description": "Alert rules" } } },
            "post": { "summary": "Create Alert Rule", "responses": { "200": { "description": "Created rule" } } }
        }),
    );
    paths.insert(
        "/alerts/rules/{id}".to_string(),
        json!({
            "patch": { "summary": "Update or Toggle Alert Rule", "responses": { "200": { "description": "Updated rule" } } },
            "delete": { "summary": "Delete Alert Rule", "responses": { "200": { "description": "Deleted" } } }
        }),
    );
    paths.insert(
        "/alerts/rules/{id}/test".to_string(),
        json!({ "post": { "summary": "Send Test Webhook Notification", "responses": { "200": { "description": "Delivery result" } } } }),
    );
    paths.insert(
        "/alerts/evaluate".to_string(),
        json!({ "post": { "summary": "Evaluate All Alert Rules Now", "responses": { "200": { "description": "Triggered events" } } } }),
    );
    paths.insert(
        "/alerts/events".to_string(),
        json!({ "get": { "summary": "List Triggered Alert Events", "responses": { "200": { "description": "Events history" } } } }),
    );
    paths.insert(
        "/deployments".to_string(),
        json!({ "get": { "summary": "List Kubernetes Serving Deployments", "responses": { "200": { "description": "Deployment list" } } } }),
    );
    paths.insert(
        "/deployments/{id}/restart".to_string(),
        json!({ "post": { "summary": "Trigger Rolling Restart", "responses": { "200": { "description": "Updated deployment" } } } }),
    );
    paths.insert(
        "/cicd/workflows".to_string(),
        json!({ "get": { "summary": "List CI/CD Runs", "responses": { "200": { "description": "Workflows list" } } } }),
    );
    paths.insert(
        "/cicd/workflows/trigger".to_string(),
        json!({ "post": { "summary": "Dispatch GitHub Actions Workflow or Local Gate", "responses": { "200": { "description": "Dispatched run" } } } }),
    );
    paths.insert(
        "/integrations".to_string(),
        json!({ "get": { "summary": "List & Probe All 8 External Integrations", "responses": { "200": { "description": "Reports" } } } }),
    );
    paths.insert(
        "/openapi.json".to_string(),
        json!({ "get": { "summary": "OpenAPI 3.1.0 JSON Specification", "responses": { "200": { "description": "OpenAPI spec" } } } }),
    );
    paths.insert(
        "/docs".to_string(),
        json!({ "get": { "summary": "Interactive Swagger UI Documentation", "responses": { "200": { "description": "HTML Swagger UI" } } } }),
    );

    Json(json!({
        "openapi": "3.1.0",
        "info": {
            "title": "LLMOps Control Center & Model Evaluation API",
            "version": env!("CARGO_PKG_VERSION"),
            "description": "Enterprise REST API for Model Registry, Experiments, Ragas/TruLens Evaluations, Real-Time Observability, AI Gateway, Prompt Registry, Alerting, Deployments, and CI/CD Gates.",
            "license": { "name": "Apache 2.0", "url": "https://www.apache.org/licenses/LICENSE-2.0" }
        },
        "servers": [
            { "url": "/api/v1", "description": "Primary Versioned API" }
        ],
        "components": {
            "securitySchemes": {
                "ApiKeyHeader": { "type": "apiKey", "in": "header", "name": "X-API-Key" },
                "BearerAuth": { "type": "http", "scheme": "bearer" }
            }
        },
        "paths": Value::Object(paths)
    }))
}

pub async fn get_swagger_ui() -> Response {
    let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>LLMOps Control Center — API Docs (OpenAPI)</title>
  <link rel="stylesheet" href="https://unpkg.com/swagger-ui-dist@5/swagger-ui.css" />
  <style>body { margin: 0; background: #0f172a; } .swagger-ui { background: #fff; }</style>
</head>
<body>
  <div id="swagger-ui"></div>
  <script src="https://unpkg.com/swagger-ui-dist@5/swagger-ui-bundle.js"></script>
  <script>
    window.onload = () => {
      window.ui = SwaggerUIBundle({
        url: '/api/v1/openapi.json',
        dom_id: '#swagger-ui',
        presets: [SwaggerUIBundle.presets.apis]
      });
    };
  </script>
</body>
</html>"#;
    Html(html).into_response()
}
