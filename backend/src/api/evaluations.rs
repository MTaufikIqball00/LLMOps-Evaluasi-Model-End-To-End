use crate::api::health::AppState;
use crate::domain::{CreateEvaluationRequest, Evaluation, EvaluationTestCase};
use crate::errors::AppError;
use crate::evaluation::run_evaluation_job;
use axum::{
    extract::{Path, State},
    Json,
};
use serde::Serialize;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Serialize)]
pub struct EvaluationDetail {
    pub evaluation: Evaluation,
    pub test_cases: Vec<EvaluationTestCase>,
}

#[derive(Serialize)]
pub struct MetricDefinition {
    pub metric_name: String,
    pub formula_or_approach: String,
    pub target_range: String,
    pub interpretation: String,
    pub hallucination_relevance: String,
}

pub async fn list_evaluations(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Evaluation>>, AppError> {
    let evals: Vec<Evaluation> =
        sqlx::query_as("SELECT * FROM evaluations ORDER BY created_at DESC")
            .fetch_all(&state.pool)
            .await?;
    Ok(Json(evals))
}

pub async fn get_evaluation_detail(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<EvaluationDetail>, AppError> {
    let eval: Option<Evaluation> = sqlx::query_as("SELECT * FROM evaluations WHERE id = ?")
        .bind(&id)
        .fetch_optional(&state.pool)
        .await?;

    let evaluation =
        eval.ok_or_else(|| AppError::NotFound(format!("Evaluation '{id}' not found")))?;

    let test_cases: Vec<EvaluationTestCase> = sqlx::query_as(
        "SELECT * FROM evaluation_test_cases WHERE evaluation_id = ? ORDER BY created_at ASC",
    )
    .bind(&id)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(EvaluationDetail {
        evaluation,
        test_cases,
    }))
}

pub async fn trigger_evaluation(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateEvaluationRequest>,
) -> Result<Json<Evaluation>, AppError> {
    let id = format!(
        "eval-{}",
        Uuid::new_v4()
            .to_string()
            .chars()
            .take(8)
            .collect::<String>()
    );

    sqlx::query(
        r#"
        INSERT INTO evaluations (
            id, name, model_name, dataset_name, evaluator_framework, status,
            faithfulness, answer_relevance, context_precision, context_recall,
            groundedness, hallucination_score, avg_latency_ms, total_tokens,
            test_cases_count, created_at
        ) VALUES (?, ?, ?, ?, ?, 'Running', 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0, 0, CURRENT_TIMESTAMP)
        "#
    )
    .bind(&id)
    .bind(&payload.name)
    .bind(&payload.model_name)
    .bind(&payload.dataset_name)
    .bind(&payload.evaluator_framework)
    .execute(&state.pool)
    .await?;

    let test_cases = payload.sample_test_cases.unwrap_or_default();

    // Spawn async runner
    run_evaluation_job(state.pool.clone(), id.clone(), test_cases).await;

    let created: Evaluation = sqlx::query_as("SELECT * FROM evaluations WHERE id = ?")
        .bind(&id)
        .fetch_one(&state.pool)
        .await?;

    Ok(Json(created))
}

pub async fn get_metric_definitions() -> Json<Vec<MetricDefinition>> {
    Json(vec![
        MetricDefinition {
            metric_name: "Faithfulness".to_string(),
            formula_or_approach: "Proportion of claims in the generated response that can be mathematically or semantically derived from the retrieved context.".to_string(),
            target_range: "0.85 - 1.00 (Higher is better)".to_string(),
            interpretation: "Measures whether the model sticks strictly to provided factual evidence without adding unverified extrapolations.".to_string(),
            hallucination_relevance: "Primary indicator of factual hallucination. Scores below 0.70 represent high probability of confabulation.".to_string(),
        },
        MetricDefinition {
            metric_name: "Answer Relevance".to_string(),
            formula_or_approach: "Semantic similarity and keyword alignment between the original user prompt and the generated response.".to_string(),
            target_range: "0.80 - 1.00 (Higher is better)".to_string(),
            interpretation: "Evaluates whether the model directly addressed the user question rather than digressing or evasively deflecting.".to_string(),
            hallucination_relevance: "Detects conversational drift and evasion hallucinations.".to_string(),
        },
        MetricDefinition {
            metric_name: "Context Precision".to_string(),
            formula_or_approach: "Signal-to-noise ratio: fraction of relevant context chunks at the top ranks of the vector retrieval list.".to_string(),
            target_range: "0.75 - 1.00 (Higher is better)".to_string(),
            interpretation: "Measures the quality of the vector database retriever / reranker component before LLM generation.".to_string(),
            hallucination_relevance: "Poor precision forces the LLM to search through distractor paragraphs, increasing hallucination probability.".to_string(),
        },
        MetricDefinition {
            metric_name: "Context Recall".to_string(),
            formula_or_approach: "Fraction of ground-truth reference propositions successfully retrieved into the context window.".to_string(),
            target_range: "0.80 - 1.00 (Higher is better)".to_string(),
            interpretation: "Measures if all necessary reference facts were provided to the LLM.".to_string(),
            hallucination_relevance: "Low context recall causes knowledge-gap hallucinations where the model attempts to fill missing facts.".to_string(),
        },
        MetricDefinition {
            metric_name: "Groundedness".to_string(),
            formula_or_approach: "TruLens triad verification: whether each sentence assertion can be verified by citation references.".to_string(),
            target_range: "0.90 - 1.00 (Higher is better)".to_string(),
            interpretation: "Validates statement-by-statement evidence backing.".to_string(),
            hallucination_relevance: "Directly isolates ungrounded assertions as hallucination candidates.".to_string(),
        },
        MetricDefinition {
            metric_name: "Hallucination Score".to_string(),
            formula_or_approach: "1.0 - Faithfulness. Automated estimate of ungrounded claim density.".to_string(),
            target_range: "0.00 - 0.15 (Lower is better)".to_string(),
            interpretation: "Synthesized hallucination index for alert thresholds and deployment gates.".to_string(),
            hallucination_relevance: "Triggers automated CI/CD benchmark failure when exceeding configured tolerance (e.g. > 0.15).".to_string(),
        },
    ])
}
