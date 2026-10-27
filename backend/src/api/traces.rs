use crate::api::health::AppState;
use crate::domain::{CreateTraceRequest, RequestTrace};
use crate::errors::AppError;
use crate::observability::redact_secrets;
use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct TraceQuery {
    pub model: Option<String>,
    pub provider: Option<String>,
    pub status: Option<i64>,
    pub limit: Option<i64>,
}

pub async fn list_traces(
    State(state): State<Arc<AppState>>,
    Query(query): Query<TraceQuery>,
) -> Result<Json<Vec<RequestTrace>>, AppError> {
    let mut qb = sqlx::QueryBuilder::new("SELECT * FROM request_traces WHERE 1=1");

    if let Some(ref m) = query.model {
        qb.push(" AND model_name LIKE ");
        qb.push_bind(format!("%{}%", m));
    }
    if let Some(ref p) = query.provider {
        qb.push(" AND provider = ");
        qb.push_bind(p);
    }
    if let Some(status) = query.status {
        qb.push(" AND status_code = ");
        qb.push_bind(status);
    }

    let limit = query.limit.unwrap_or(50).min(200);
    qb.push(" ORDER BY created_at DESC LIMIT ");
    qb.push_bind(limit);

    let traces: Vec<RequestTrace> = qb.build_query_as().fetch_all(&state.pool).await?;
    Ok(Json(traces))
}

pub async fn get_trace(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<RequestTrace>, AppError> {
    let trace: Option<RequestTrace> = sqlx::query_as("SELECT * FROM request_traces WHERE id = ?")
        .bind(&id)
        .fetch_optional(&state.pool)
        .await?;

    trace
        .map(Json)
        .ok_or_else(|| AppError::NotFound(format!("Trace '{id}' not found")))
}

pub async fn create_trace(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateTraceRequest>,
) -> Result<Json<RequestTrace>, AppError> {
    let id = format!(
        "tr-{}",
        Uuid::new_v4()
            .to_string()
            .chars()
            .take(8)
            .collect::<String>()
    );
    let trace_id = format!("tr-{}", chrono::Utc::now().format("%Y%m%d%H%M%S"));

    let total_tokens = payload.prompt_tokens + payload.completion_tokens;

    // Lookup cost per 1k tokens from pricing catalog or use default $0.001
    let prompt_cost = (payload.prompt_tokens as f64 / 1000.0) * 0.0005;
    let comp_cost = (payload.completion_tokens as f64 / 1000.0) * 0.0015;
    let estimated_cost = ((prompt_cost + comp_cost) * 100000.0).round() / 100000.0;

    let redacted_p = payload.prompt.as_deref().map(redact_secrets);
    let redacted_c = payload.completion.as_deref().map(redact_secrets);
    let meta_str = payload
        .metadata
        .map(|m| serde_json::to_string(&m).unwrap_or_default());

    sqlx::query(
        r#"
        INSERT INTO request_traces (
            id, trace_id, span_name, model_name, provider, status_code,
            latency_ms, prompt_tokens, completion_tokens, total_tokens,
            estimated_cost, redacted_prompt, redacted_completion, metadata,
            error_message, created_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
        "#,
    )
    .bind(&id)
    .bind(&trace_id)
    .bind(&payload.span_name)
    .bind(&payload.model_name)
    .bind(&payload.provider)
    .bind(payload.status_code)
    .bind(payload.latency_ms)
    .bind(payload.prompt_tokens)
    .bind(payload.completion_tokens)
    .bind(total_tokens)
    .bind(estimated_cost)
    .bind(&redacted_p)
    .bind(&redacted_c)
    .bind(&meta_str)
    .bind(&payload.error_message)
    .execute(&state.pool)
    .await?;

    let created: RequestTrace = sqlx::query_as("SELECT * FROM request_traces WHERE id = ?")
        .bind(&id)
        .fetch_one(&state.pool)
        .await?;

    Ok(Json(created))
}
