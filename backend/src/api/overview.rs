use crate::api::health::AppState;
use crate::errors::AppError;
use crate::observability::compute_latency_percentiles;
use axum::{extract::State, Json};
use serde::Serialize;
use std::sync::Arc;

#[derive(Serialize)]
pub struct OverviewStats {
    pub total_requests: i64,
    pub successful_requests: i64,
    pub failed_requests: i64,
    pub success_rate_pct: f64,
    pub latency: LatencyStats,
    pub tokens: TokenStats,
    pub cost: CostStats,
    pub models_count: i64,
    pub model_distribution: Vec<ModelUsageItem>,
    pub evaluations: EvaluationSummaryStats,
    pub deployments: DeploymentSummaryStats,
    pub active_environment: String,
    pub data_freshness: String,
}

#[derive(Serialize)]
pub struct LatencyStats {
    pub p50_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
    pub avg_ms: f64,
}

#[derive(Serialize)]
pub struct TokenStats {
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
    pub total_tokens: i64,
}

#[derive(Serialize)]
pub struct CostStats {
    pub current_spend_usd: f64,
    pub monthly_budget_usd: f64,
    pub budget_used_pct: f64,
}

#[derive(Serialize)]
pub struct ModelUsageItem {
    pub model_name: String,
    pub requests_count: i64,
    pub percentage: f64,
}

#[derive(Serialize)]
pub struct EvaluationSummaryStats {
    pub total_evaluations: i64,
    pub avg_faithfulness: f64,
    pub avg_relevance: f64,
    pub avg_groundedness: f64,
    pub avg_hallucination_score: f64,
}

#[derive(Serialize)]
pub struct DeploymentSummaryStats {
    pub total: i64,
    pub healthy: i64,
    pub ready_replicas: i64,
    pub desired_replicas: i64,
}

pub async fn get_overview_stats(
    State(state): State<Arc<AppState>>,
) -> Result<Json<OverviewStats>, AppError> {
    let trace_counts: (i64, i64, i64) = sqlx::query_as(
        r#"
        SELECT 
            COUNT(*),
            COALESCE(SUM(CASE WHEN status_code = 200 THEN 1 ELSE 0 END), 0),
            COALESCE(SUM(CASE WHEN status_code != 200 THEN 1 ELSE 0 END), 0)
        FROM request_traces
        "#,
    )
    .fetch_one(&state.pool)
    .await?;

    let total = trace_counts.0;
    let successful = trace_counts.1;
    let failed = trace_counts.2;
    let success_rate = if total > 0 {
        (successful as f64 / total as f64) * 100.0
    } else {
        100.0
    };

    let token_counts: (i64, i64, i64) = sqlx::query_as(
        r#"
        SELECT 
            COALESCE(SUM(prompt_tokens), 0),
            COALESCE(SUM(completion_tokens), 0),
            COALESCE(SUM(total_tokens), 0)
        FROM request_traces
        "#,
    )
    .fetch_one(&state.pool)
    .await?;

    // Compute real percentiles directly from persisted request_traces latencies
    let latency_rows: Vec<(f64,)> = sqlx::query_as("SELECT latency_ms FROM request_traces")
        .fetch_all(&state.pool)
        .await
        .unwrap_or_default();
    let pct = compute_latency_percentiles(latency_rows.into_iter().map(|r| r.0).collect());

    let budget_row: (f64, f64) =
        sqlx::query_as("SELECT monthly_budget_usd, current_spend_usd FROM budget_settings LIMIT 1")
            .fetch_one(&state.pool)
            .await
            .unwrap_or((2500.0, 0.0));

    let budget_used_pct = if budget_row.0 > 0.0 {
        (budget_row.1 / budget_row.0) * 100.0
    } else {
        0.0
    };

    let models_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM models")
        .fetch_one(&state.pool)
        .await?;

    let model_rows: Vec<(String, i64)> = sqlx::query_as(
        r#"
        SELECT model_name, COUNT(*) as cnt
        FROM request_traces
        GROUP BY model_name
        ORDER BY cnt DESC
        "#,
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let model_distribution: Vec<ModelUsageItem> = model_rows
        .into_iter()
        .map(|(name, count)| {
            let pct_share = if total > 0 {
                (count as f64 / total as f64) * 100.0
            } else {
                0.0
            };
            ModelUsageItem {
                model_name: name,
                requests_count: count,
                percentage: (pct_share * 10.0).round() / 10.0,
            }
        })
        .collect();

    let eval_summary: (i64, f64, f64, f64, f64) = sqlx::query_as(
        r#"
        SELECT 
            COUNT(*),
            COALESCE(AVG(faithfulness), 0.0),
            COALESCE(AVG(answer_relevance), 0.0),
            COALESCE(AVG(groundedness), 0.0),
            COALESCE(AVG(hallucination_score), 0.0)
        FROM evaluations
        "#,
    )
    .fetch_one(&state.pool)
    .await
    .unwrap_or((0, 0.0, 0.0, 0.0, 0.0));

    let dep_summary: (i64, i64, i64, i64) = sqlx::query_as(
        r#"
        SELECT 
            COUNT(*),
            COALESCE(SUM(CASE WHEN status = 'Healthy' THEN 1 ELSE 0 END), 0),
            COALESCE(SUM(replicas_ready), 0),
            COALESCE(SUM(replicas_desired), 0)
        FROM deployments
        "#,
    )
    .fetch_one(&state.pool)
    .await
    .unwrap_or((0, 0, 0, 0));

    let engine_label = if state.is_postgres {
        "PostgreSQL"
    } else {
        "SQLite"
    };

    Ok(Json(OverviewStats {
        total_requests: total,
        successful_requests: successful,
        failed_requests: failed,
        success_rate_pct: (success_rate * 10.0).round() / 10.0,
        latency: LatencyStats {
            p50_ms: pct.p50,
            p95_ms: pct.p95,
            p99_ms: pct.p99,
            avg_ms: pct.avg,
        },
        tokens: TokenStats {
            prompt_tokens: token_counts.0,
            completion_tokens: token_counts.1,
            total_tokens: token_counts.2,
        },
        cost: CostStats {
            current_spend_usd: (budget_row.1 * 100.0).round() / 100.0,
            monthly_budget_usd: budget_row.0,
            budget_used_pct: (budget_used_pct * 10.0).round() / 10.0,
        },
        models_count: models_count.0,
        model_distribution,
        evaluations: EvaluationSummaryStats {
            total_evaluations: eval_summary.0,
            avg_faithfulness: (eval_summary.1 * 1000.0).round() / 1000.0,
            avg_relevance: (eval_summary.2 * 1000.0).round() / 1000.0,
            avg_groundedness: (eval_summary.3 * 1000.0).round() / 1000.0,
            avg_hallucination_score: (eval_summary.4 * 1000.0).round() / 1000.0,
        },
        deployments: DeploymentSummaryStats {
            total: dep_summary.0,
            healthy: dep_summary.1,
            ready_replicas: dep_summary.2,
            desired_replicas: dep_summary.3,
        },
        active_environment: format!("Production ({engine_label})"),
        data_freshness: format!("Live {engine_label} Telemetry"),
    }))
}
