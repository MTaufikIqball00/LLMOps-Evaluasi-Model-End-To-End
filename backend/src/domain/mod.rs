use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Model {
    pub id: String,
    pub name: String,
    pub framework: String,
    pub task: String,
    pub stage: String,
    pub version: String,
    pub parameters_count: Option<String>,
    pub artifact_uri: Option<String>,
    pub description: Option<String>,
    pub tags: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateModelRequest {
    pub name: String,
    pub framework: String,
    pub task: String,
    pub stage: Option<String>,
    pub version: String,
    pub parameters_count: Option<String>,
    pub artifact_uri: Option<String>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateStageRequest {
    pub stage: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Experiment {
    pub id: String,
    pub name: String,
    pub model_id: Option<String>,
    pub framework: String,
    pub status: String,
    pub parameters: String,
    pub metrics: String,
    pub tags: Option<String>,
    pub duration_seconds: i64,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateExperimentRequest {
    pub name: String,
    pub model_id: Option<String>,
    pub framework: String,
    pub parameters: serde_json::Value,
    pub metrics: serde_json::Value,
    pub tags: Option<Vec<String>>,
    pub duration_seconds: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Evaluation {
    pub id: String,
    pub name: String,
    pub model_name: String,
    pub dataset_name: String,
    pub evaluator_framework: String,
    pub status: String,
    pub faithfulness: f64,
    pub answer_relevance: f64,
    pub context_precision: f64,
    pub context_recall: f64,
    pub groundedness: f64,
    pub hallucination_score: f64,
    pub avg_latency_ms: f64,
    pub total_tokens: i64,
    pub test_cases_count: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct EvaluationTestCase {
    pub id: String,
    pub evaluation_id: String,
    pub question: String,
    pub ground_truth: Option<String>,
    pub generated_answer: String,
    pub retrieved_context: Option<String>,
    pub faithfulness_score: f64,
    pub relevance_score: f64,
    pub groundedness_score: f64,
    pub hallucination_detected: i64,
    pub latency_ms: f64,
    pub tokens_used: i64,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateEvaluationRequest {
    pub name: String,
    pub model_name: String,
    pub dataset_name: String,
    pub evaluator_framework: String,
    pub sample_test_cases: Option<Vec<CreateTestCaseInput>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTestCaseInput {
    pub question: String,
    pub ground_truth: Option<String>,
    pub generated_answer: String,
    pub retrieved_context: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RequestTrace {
    pub id: String,
    pub trace_id: String,
    pub span_name: String,
    pub model_name: String,
    pub provider: String,
    pub status_code: i64,
    pub latency_ms: f64,
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
    pub total_tokens: i64,
    pub estimated_cost: f64,
    pub redacted_prompt: Option<String>,
    pub redacted_completion: Option<String>,
    pub metadata: Option<String>,
    pub error_message: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateTraceRequest {
    pub span_name: String,
    pub model_name: String,
    pub provider: String,
    pub status_code: i64,
    pub latency_ms: f64,
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
    pub prompt: Option<String>,
    pub completion: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PricingEntry {
    pub id: String,
    pub provider: String,
    pub model_pattern: String,
    pub prompt_cost_per_1k: f64,
    pub completion_cost_per_1k: f64,
    pub last_updated: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BudgetSetting {
    pub id: String,
    pub monthly_budget_usd: f64,
    pub alert_threshold_pct: f64,
    pub current_spend_usd: f64,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateBudgetRequest {
    pub monthly_budget_usd: f64,
    pub alert_threshold_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Deployment {
    pub id: String,
    pub name: String,
    pub model_name: String,
    pub version: String,
    pub environment: String,
    pub cluster: String,
    pub replicas_desired: i64,
    pub replicas_ready: i64,
    pub status: String,
    pub image_tag: String,
    pub cpu_usage: Option<String>,
    pub memory_usage: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CicdWorkflow {
    pub id: String,
    pub workflow_name: String,
    pub run_id: String,
    pub branch: String,
    pub commit_sha: String,
    pub commit_message: String,
    pub status: String,
    pub trigger_event: String,
    pub duration_seconds: i64,
    pub url: Option<String>,
    pub started_at: String,
}

#[derive(Debug, Deserialize)]
pub struct TriggerWorkflowRequest {
    pub workflow_name: String,
    pub branch: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct IntegrationConfig {
    pub id: String,
    pub service_name: String,
    pub endpoint_url: Option<String>,
    pub auth_type: Option<String>,
    pub auth_token_masked: Option<String>,
    pub is_enabled: i64,
    pub last_status: String,
    pub last_checked_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateIntegrationRequest {
    pub endpoint_url: Option<String>,
    pub auth_token: Option<String>,
    pub is_enabled: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct IntegrationStatusReport {
    pub service_name: String,
    pub display_name: String,
    pub category: String,
    pub status: String,
    pub endpoint_url: Option<String>,
    pub latency_ms: Option<f64>,
    pub message: String,
    pub is_configured: bool,
    pub last_checked: Option<String>,
}
