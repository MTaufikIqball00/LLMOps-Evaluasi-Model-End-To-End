use axum::http::StatusCode;
use llmops_backend::{api, domain::*, persistence};
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Instant;
use tokio::net::TcpListener;
use uuid::Uuid;

#[allow(dead_code)]
struct TestApp {
    pub base_url: String,
    pub pool: persistence::DbPool,
    pub db_file: String,
}

impl TestApp {
    async fn spawn() -> Self {
        let db_id = Uuid::new_v4().to_string();
        let db_dir = std::path::Path::new("target/tmp/test_dbs");
        tokio::fs::create_dir_all(&db_dir).await.ok();
        let db_file_path = format!("target/tmp/test_dbs/{}.db", db_id);
        let database_url = format!("sqlite://{}?mode=rwc", db_file_path);

        let pool = persistence::init_db(&database_url)
            .await
            .expect("Failed to initialize test SQLite database");

        let state = Arc::new(api::health::AppState {
            pool: pool.clone(),
            start_time: Instant::now(),
            server_port: 0,
            frontend_origin: "http://localhost:3210".to_string(),
            mlflow_url: "http://localhost:5500".to_string(),
            prometheus_url: "http://localhost:9190".to_string(),
            grafana_url: "http://localhost:3310".to_string(),
            github_repo: "enterprise/llmops-platform".to_string(),
            github_token: None,
        });

        let app = api::build_router(state);

        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("Failed to bind random TCP port");
        let local_addr = listener.local_addr().expect("Failed to get local address");
        let base_url = format!("http://{}", local_addr);

        tokio::spawn(async move {
            axum::serve(listener, app)
                .await
                .expect("Failed to run test axum server");
        });

        TestApp {
            base_url,
            pool,
            db_file: db_file_path,
        }
    }
}

impl Drop for TestApp {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.db_file);
    }
}

#[tokio::test]
async fn test_health_and_root_endpoints() {
    let app = TestApp::spawn().await;
    let client = reqwest::Client::new();

    // 1. Root /health fallback probe
    let res = client
        .get(format!("{}/health", app.base_url))
        .send()
        .await
        .expect("Failed to send request");
    assert_eq!(res.status(), StatusCode::OK);
    let health: Value = res.json().await.expect("Failed to parse health json");
    assert_eq!(health["status"], "Healthy");
    assert!(health["database"].as_str().unwrap().contains("Connected"));

    // 2. Standard /api/v1/health probe
    let res_v1 = client
        .get(format!("{}/api/v1/health", app.base_url))
        .send()
        .await
        .expect("Failed to send request");
    assert_eq!(res_v1.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_prometheus_metrics_exposition() {
    let app = TestApp::spawn().await;
    let client = reqwest::Client::new();

    let res = client
        .get(format!("{}/api/v1/metrics", app.base_url))
        .send()
        .await
        .expect("Failed to send request");
    assert_eq!(res.status(), StatusCode::OK);
    let body = res.text().await.expect("Failed to read body");
    assert!(body.contains("llmops_http_requests_total"));
    assert!(body.contains("llmops_llm_latency_p50_milliseconds"));
    assert!(body.contains("llmops_llm_tokens_total"));
    assert!(body.contains("llmops_evaluation_faithfulness_avg"));
}

#[tokio::test]
async fn test_overview_stats_endpoint() {
    let app = TestApp::spawn().await;
    let client = reqwest::Client::new();

    let res = client
        .get(format!("{}/api/v1/overview/stats", app.base_url))
        .send()
        .await
        .expect("Failed to send request");
    assert_eq!(res.status(), StatusCode::OK);
    let stats: Value = res.json().await.expect("Failed to parse overview");
    assert!(stats["total_requests"].as_i64().unwrap() >= 0);
    assert!(stats["latency"]["p50_ms"].as_f64().unwrap() >= 0.0);
    assert!(stats["models_count"].as_i64().unwrap() >= 4); // Seeded with 4 models
}

#[tokio::test]
async fn test_models_lifecycle_and_validation() {
    let app = TestApp::spawn().await;
    let client = reqwest::Client::new();

    // 1. List models
    let res = client
        .get(format!("{}/api/v1/models", app.base_url))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let models: Vec<Model> = res.json().await.unwrap();
    assert!(!models.is_empty());

    // 2. Filter by stage
    let res_prod = client
        .get(format!("{}/api/v1/models?stage=Production", app.base_url))
        .send()
        .await
        .unwrap();
    assert_eq!(res_prod.status(), StatusCode::OK);
    let prod_models: Vec<Model> = res_prod.json().await.unwrap();
    for m in prod_models {
        assert_eq!(m.stage, "Production");
    }

    // 3. Create a new model
    let new_model = json!({
        "name": "Audit-Test-Model-7B",
        "framework": "vLLM / PyTorch",
        "task": "Text Generation",
        "stage": "Development",
        "version": "v1.0.0",
        "parameters_count": "7B",
        "artifact_uri": "s3://test-artifacts/model-7b",
        "description": "Integration test model",
        "tags": ["test", "audit"]
    });
    let res_create = client
        .post(format!("{}/api/v1/models", app.base_url))
        .json(&new_model)
        .send()
        .await
        .unwrap();
    assert_eq!(res_create.status(), StatusCode::OK);
    let created: Model = res_create.json().await.unwrap();
    assert_eq!(created.name, "Audit-Test-Model-7B");
    assert_eq!(created.stage, "Development");

    // 4. Get by ID
    let res_get = client
        .get(format!("{}/api/v1/models/{}", app.base_url, created.id))
        .send()
        .await
        .unwrap();
    assert_eq!(res_get.status(), StatusCode::OK);
    let fetched: Model = res_get.json().await.unwrap();
    assert_eq!(fetched.id, created.id);

    // 5. Promote stage to Staging
    let res_patch = client
        .patch(format!(
            "{}/api/v1/models/{}/stage",
            app.base_url, created.id
        ))
        .json(&json!({ "stage": "Staging" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res_patch.status(), StatusCode::OK);
    let updated: Model = res_patch.json().await.unwrap();
    assert_eq!(updated.stage, "Staging");

    // 6. Attempt invalid stage transition (Rejection with 400 Bad Request)
    let res_invalid = client
        .patch(format!(
            "{}/api/v1/models/{}/stage",
            app.base_url, created.id
        ))
        .json(&json!({ "stage": "InvalidStageName" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res_invalid.status(), StatusCode::BAD_REQUEST);

    // 7. Non-existent model lookup (404 Not Found)
    let res_404 = client
        .get(format!(
            "{}/api/v1/models/non-existent-id-9999",
            app.base_url
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(res_404.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_experiments_tracking_and_comparison() {
    let app = TestApp::spawn().await;
    let client = reqwest::Client::new();

    // 1. List experiments
    let res = client
        .get(format!("{}/api/v1/experiments", app.base_url))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let exps: Vec<Experiment> = res.json().await.unwrap();
    assert!(!exps.is_empty());

    // 2. Create experiment
    let new_exp = json!({
        "name": "audit-test-exp-run-01",
        "model_id": "m-llama31-70b",
        "framework": "MLflow",
        "parameters": { "lr": 0.0001, "batch_size": 16 },
        "metrics": { "eval_loss": 0.42, "accuracy": 0.95 },
        "tags": ["test"],
        "duration_seconds": 120
    });
    let res_create = client
        .post(format!("{}/api/v1/experiments", app.base_url))
        .json(&new_exp)
        .send()
        .await
        .unwrap();
    assert_eq!(res_create.status(), StatusCode::OK);
    let created: Experiment = res_create.json().await.unwrap();
    assert_eq!(created.name, "audit-test-exp-run-01");

    // 3. Compare runs
    let compare_url = format!(
        "{}/api/v1/experiments/compare?runs={},exp-lora-run-84",
        app.base_url, created.id
    );
    let res_compare = client.get(&compare_url).send().await.unwrap();
    assert_eq!(res_compare.status(), StatusCode::OK);
    let comparison: Value = res_compare.json().await.unwrap();
    let runs = comparison["runs"].as_array().unwrap();
    assert!(runs.len() >= 2);
    assert!(!comparison["parameter_keys"].as_array().unwrap().is_empty());
    assert!(!comparison["metric_keys"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_evaluations_and_metric_definitions() {
    let app = TestApp::spawn().await;
    let client = reqwest::Client::new();

    // 1. Get metric definitions
    let res_defs = client
        .get(format!(
            "{}/api/v1/evaluations/metrics/definitions",
            app.base_url
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(res_defs.status(), StatusCode::OK);
    let defs: Vec<Value> = res_defs.json().await.unwrap();
    assert_eq!(defs.len(), 6);
    let names: Vec<&str> = defs
        .iter()
        .filter_map(|d| d["metric_name"].as_str())
        .collect();
    assert!(names.contains(&"Faithfulness"));
    assert!(names.contains(&"Hallucination Score"));

    // 2. Trigger asynchronous evaluation run
    let trigger_payload = json!({
        "name": "Audit Autonomous Eval Run",
        "model_name": "Llama-3.1-70B-Instruct",
        "dataset_name": "audit-gold-eval",
        "evaluator_framework": "Ragas & TruLens Suite",
        "sample_test_cases": [
            {
                "question": "What is the capital of France?",
                "ground_truth": "The capital of France is Paris.",
                "generated_answer": "Paris is the capital of France.",
                "retrieved_context": "France is a country in Western Europe. Its capital and largest city is Paris."
            }
        ]
    });
    let res_trigger = client
        .post(format!("{}/api/v1/evaluations", app.base_url))
        .json(&trigger_payload)
        .send()
        .await
        .unwrap();
    assert_eq!(res_trigger.status(), StatusCode::OK);
    let eval: Evaluation = res_trigger.json().await.unwrap();
    assert_eq!(eval.name, "Audit Autonomous Eval Run");

    // Wait briefly for background Tokio task to complete
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // 3. Inspect evaluation detail & test cases
    let res_detail = client
        .get(format!("{}/api/v1/evaluations/{}", app.base_url, eval.id))
        .send()
        .await
        .unwrap();
    assert_eq!(res_detail.status(), StatusCode::OK);
    let detail: Value = res_detail.json().await.unwrap();
    assert_eq!(detail["evaluation"]["status"], "Completed");
    let tc_array = detail["test_cases"].as_array().unwrap();
    assert!(!tc_array.is_empty());
    assert!(tc_array[0]["faithfulness_score"].as_f64().unwrap() > 0.80);
    assert_eq!(tc_array[0]["hallucination_detected"].as_i64().unwrap(), 0);
}

#[tokio::test]
async fn test_request_traces_and_secret_scrubbing() {
    let app = TestApp::spawn().await;
    let client = reqwest::Client::new();

    // 1. Create trace with sensitive credentials
    let raw_prompt = "Execute user query with sk-abcdef1234567890abcdef123456 and token ghp_11223344556677889900aa.";
    let raw_completion = "Authorization approved with Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9. Successful response.";
    let trace_payload = json!({
        "span_name": "audit_trace_span",
        "model_name": "Llama-3.1-70B-Instruct",
        "provider": "vllm_local",
        "status_code": 200,
        "latency_ms": 142.5,
        "prompt_tokens": 120,
        "completion_tokens": 45,
        "prompt": raw_prompt,
        "completion": raw_completion,
        "metadata": { "env": "audit-test" },
        "error_message": null
    });

    let res = client
        .post(format!("{}/api/v1/traces", app.base_url))
        .json(&trace_payload)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let created: RequestTrace = res.json().await.unwrap();

    // Verify secret scrubbing occurred before DB persistence
    assert!(!created
        .redacted_prompt
        .as_ref()
        .unwrap()
        .contains("sk-abcdef1234567890abcdef123456"));
    assert!(!created
        .redacted_prompt
        .as_ref()
        .unwrap()
        .contains("ghp_11223344556677889900aa"));
    assert!(created
        .redacted_prompt
        .as_ref()
        .unwrap()
        .contains("[REDACTED_SECRET]"));
    assert!(!created
        .redacted_completion
        .as_ref()
        .unwrap()
        .contains("Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9"));
    assert!(created
        .redacted_completion
        .as_ref()
        .unwrap()
        .contains("[REDACTED_SECRET]"));

    // 2. Query trace by ID
    let res_get = client
        .get(format!("{}/api/v1/traces/{}", app.base_url, created.id))
        .send()
        .await
        .unwrap();
    assert_eq!(res_get.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_costs_and_budget_management() {
    let app = TestApp::spawn().await;
    let client = reqwest::Client::new();

    // 1. Get cost summary
    let res = client
        .get(format!("{}/api/v1/costs/summary", app.base_url))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let summary: Value = res.json().await.unwrap();
    assert!(summary["budget"]["monthly_budget_usd"].as_f64().unwrap() > 0.0);
    assert!(!summary["pricing_source"].as_str().unwrap().is_empty());

    // 2. List pricing catalog
    let res_price = client
        .get(format!("{}/api/v1/costs/pricing", app.base_url))
        .send()
        .await
        .unwrap();
    assert_eq!(res_price.status(), StatusCode::OK);
    let catalog: Vec<PricingEntry> = res_price.json().await.unwrap();
    assert!(catalog.len() >= 5);

    // 3. Update budget settings
    let update_payload = json!({
        "monthly_budget_usd": 3200.0,
        "alert_threshold_pct": 75.0
    });
    let res_update = client
        .patch(format!("{}/api/v1/costs/budget", app.base_url))
        .json(&update_payload)
        .send()
        .await
        .unwrap();
    assert_eq!(res_update.status(), StatusCode::OK);
    let updated: BudgetSetting = res_update.json().await.unwrap();
    assert_eq!(updated.monthly_budget_usd, 3200.0);
    assert_eq!(updated.alert_threshold_pct, 75.0);

    // 4. Validate negative budget rejection (400)
    let res_bad = client
        .patch(format!("{}/api/v1/costs/budget", app.base_url))
        .json(&json!({ "monthly_budget_usd": -100.0, "alert_threshold_pct": 80.0 }))
        .send()
        .await
        .unwrap();
    assert_eq!(res_bad.status(), StatusCode::BAD_REQUEST);

    // 5. Validate threshold > 100% rejection (400)
    let res_bad_pct = client
        .patch(format!("{}/api/v1/costs/budget", app.base_url))
        .json(&json!({ "monthly_budget_usd": 1000.0, "alert_threshold_pct": 120.0 }))
        .send()
        .await
        .unwrap();
    assert_eq!(res_bad_pct.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_deployments_and_restart_simulation() {
    let app = TestApp::spawn().await;
    let client = reqwest::Client::new();

    // 1. List deployments
    let res = client
        .get(format!("{}/api/v1/deployments", app.base_url))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let deps: Vec<Deployment> = res.json().await.unwrap();
    assert!(!deps.is_empty());
    let target_id = deps[0].id.clone();

    // 2. Rolling restart deployment
    let res_restart = client
        .post(format!(
            "{}/api/v1/deployments/{}/restart",
            app.base_url, target_id
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(res_restart.status(), StatusCode::OK);
    let restarted: Deployment = res_restart.json().await.unwrap();
    assert_eq!(restarted.id, target_id);

    // 3. Restart non-existent deployment (404)
    let res_404 = client
        .post(format!(
            "{}/api/v1/deployments/dep-fake-9999/restart",
            app.base_url
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(res_404.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_cicd_workflow_automation() {
    let app = TestApp::spawn().await;
    let client = reqwest::Client::new();

    // 1. List CI/CD workflows
    let res = client
        .get(format!("{}/api/v1/cicd/workflows", app.base_url))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let wfs: Vec<CicdWorkflow> = res.json().await.unwrap();
    assert!(!wfs.is_empty());

    // 2. Trigger workflow manual dispatch
    let trigger_payload = json!({
        "workflow_name": "Audit Benchmark Quality Gate",
        "branch": "feature/autonomous-qa"
    });
    let res_trigger = client
        .post(format!("{}/api/v1/cicd/workflows/trigger", app.base_url))
        .json(&trigger_payload)
        .send()
        .await
        .unwrap();
    assert_eq!(res_trigger.status(), StatusCode::OK);
    let triggered: CicdWorkflow = res_trigger.json().await.unwrap();
    assert_eq!(triggered.workflow_name, "Audit Benchmark Quality Gate");
    assert_eq!(triggered.branch, "feature/autonomous-qa");
    assert_eq!(triggered.trigger_event, "manual_dispatch");
    assert_eq!(triggered.status, "Success");
}

#[tokio::test]
async fn test_integrations_probe_hub() {
    let app = TestApp::spawn().await;
    let client = reqwest::Client::new();

    // 1. List all 8 integration reports
    let res = client
        .get(format!("{}/api/v1/integrations", app.base_url))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let reports: Vec<Value> = res.json().await.unwrap();
    assert_eq!(reports.len(), 8);

    // 2. Test individual integration probe (ragas)
    let res_test = client
        .post(format!("{}/api/v1/integrations/ragas/test", app.base_url))
        .send()
        .await
        .unwrap();
    assert_eq!(res_test.status(), StatusCode::OK);
    let ragas_rep: Value = res_test.json().await.unwrap();
    assert_eq!(ragas_rep["service_name"], "ragas");
    assert!(ragas_rep["status"].as_str().unwrap().contains("Connected"));

    // 3. Test unknown integration probe rejection (400)
    let res_bad = client
        .post(format!(
            "{}/api/v1/integrations/unknown_service/test",
            app.base_url
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(res_bad.status(), StatusCode::BAD_REQUEST);

    // 4. Update integration endpoint
    let res_update = client
        .put(format!("{}/api/v1/integrations/prometheus", app.base_url))
        .json(&json!({
            "endpoint_url": "http://localhost:9199",
            "is_enabled": true
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res_update.status(), StatusCode::OK);
}
