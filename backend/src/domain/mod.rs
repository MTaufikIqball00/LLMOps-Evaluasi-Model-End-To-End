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
    pub use_llm_judge: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
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
    pub workflow_file: Option<String>,
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

// ============================================================================
// Prompt Registry & A/B Comparison Domain Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PromptTemplate {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub version: String,
    pub template: String,
    pub variables: Option<String>,
    pub model_target: String,
    pub stage: String,
    pub description: Option<String>,
    pub tags: Option<String>,
    pub avg_faithfulness: f64,
    pub avg_latency_ms: f64,
    pub usage_count: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreatePromptRequest {
    pub name: String,
    pub slug: Option<String>,
    pub version: String,
    pub template: String,
    pub model_target: String,
    pub stage: Option<String>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePromptRequest {
    pub stage: Option<String>,
    pub template: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PromptDiffLine {
    pub line_number: usize,
    pub change_type: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct PromptCompareResponse {
    pub prompt_a: PromptTemplate,
    pub prompt_b: PromptTemplate,
    pub shared_variables: Vec<String>,
    pub added_variables: Vec<String>,
    pub removed_variables: Vec<String>,
    pub faithfulness_delta: f64,
    pub latency_delta_ms: f64,
    pub diff_lines: Vec<PromptDiffLine>,
}

// ============================================================================
// Alerting Rules & Webhook Events Domain Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AlertRule {
    pub id: String,
    pub name: String,
    pub metric_type: String,
    pub operator: String,
    pub threshold: f64,
    pub severity: String,
    pub webhook_url: Option<String>,
    pub channel_type: String,
    pub is_enabled: i64,
    pub last_triggered_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateAlertRuleRequest {
    pub name: String,
    pub metric_type: String,
    pub operator: Option<String>,
    pub threshold: f64,
    pub severity: Option<String>,
    pub webhook_url: Option<String>,
    pub channel_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAlertRuleRequest {
    pub is_enabled: Option<bool>,
    pub threshold: Option<f64>,
    pub webhook_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AlertEvent {
    pub id: String,
    pub rule_id: String,
    pub rule_name: String,
    pub metric_type: String,
    pub observed_value: f64,
    pub threshold: f64,
    pub severity: String,
    pub message: String,
    pub delivery_status: String,
    pub created_at: String,
}

// ============================================================================
// AI Gateway OpenAI-Compatible Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct GatewayChatRequest {
    pub model: String,
    pub messages: Vec<GatewayMessage>,
    pub temperature: Option<f64>,
    pub max_tokens: Option<i64>,
    pub provider: Option<String>,
    pub upstream_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GatewayChoice {
    pub index: i64,
    pub message: GatewayMessage,
    pub finish_reason: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GatewayUsage {
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
    pub total_tokens: i64,
}

#[derive(Debug, Serialize)]
pub struct GatewayChatResponse {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub mode: String,
    pub choices: Vec<GatewayChoice>,
    pub usage: GatewayUsage,
    pub trace_id: String,
    pub estimated_cost_usd: f64,
    pub latency_ms: f64,
}
