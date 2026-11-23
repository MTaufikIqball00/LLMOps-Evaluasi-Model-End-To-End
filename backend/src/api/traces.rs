use crate::api::health::AppState;
use crate::domain::{CreateTraceRequest, PricingEntry, RequestTrace};
use crate::errors::AppError;
use crate::observability::{alerts, redact_secrets};
use crate::persistence::adapt_sql;
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
    let mut clauses = Vec::new();
    if query.model.is_some() {
        clauses.push("AND LOWER(model_name) LIKE LOWER(?)");
    }
    if query.provider.is_some() {
        clauses.push("AND provider = ?");
    }
    if query.status.is_some() {
        clauses.push("AND status_code = ?");
    }

    let raw_sql = format!(
        "SELECT * FROM request_traces WHERE 1=1 {} ORDER BY created_at DESC LIMIT ?",
        clauses.join(" ")
    );
    let sql = adapt_sql(&raw_sql, state.is_postgres);

    let mut q = sqlx::query_as::<_, RequestTrace>(&sql);
    if let Some(ref m) = query.model {
        q = q.bind(format!("%{m}%"));
    }
    if let Some(ref p) = query.provider {
        q = q.bind(p);
    }
    if let Some(status) = query.status {
        q = q.bind(status);
    }
    let limit = query.limit.unwrap_or(50).min(200);
    q = q.bind(limit);

    let traces = q.fetch_all(&state.pool).await?;
    Ok(Json(traces))
}

pub async fn get_trace(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<RequestTrace>, AppError> {
    let sql = adapt_sql(
        "SELECT * FROM request_traces WHERE id = ?",
        state.is_postgres,
    );
    let trace: Option<RequestTrace> = sqlx::query_as(&sql)
        .bind(&id)
        .fetch_optional(&state.pool)
        .await?;

    trace
        .map(Json)
        .ok_or_else(|| AppError::NotFound(format!("Trace '{id}' not found")))
}

async fn resolve_pricing_rates(state: &AppState, model_name: &str) -> (f64, f64) {
    let pricing_rows: Vec<PricingEntry> =
        sqlx::query_as("SELECT * FROM pricing_catalog ORDER BY last_updated DESC")
            .fetch_all(&state.pool)
            .await
            .unwrap_or_default();

    let target = model_name.to_ascii_lowercase();

    if let Some(exact) = pricing_rows
        .iter()
        .find(|p| p.model_pattern.to_ascii_lowercase() == target)
    {
        return (exact.prompt_cost_per_1k, exact.completion_cost_per_1k);
    }

    if let Some(partial) = pricing_rows.iter().find(|p| {
        let pat = p.model_pattern.to_ascii_lowercase();
        target.contains(&pat) || pat.contains(&target)
    }) {
        return (partial.prompt_cost_per_1k, partial.completion_cost_per_1k);
    }

    (0.0005, 0.0015)
}

pub async fn persist_trace_with_cost(
    state: &AppState,
    payload: CreateTraceRequest,
) -> Result<RequestTrace, AppError> {
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

    let (prompt_rate, comp_rate) = resolve_pricing_rates(state, &payload.model_name).await;
    let prompt_cost = (payload.prompt_tokens as f64 / 1000.0) * prompt_rate;
    let comp_cost = (payload.completion_tokens as f64 / 1000.0) * comp_rate;
    let estimated_cost = ((prompt_cost + comp_cost) * 1_000_000.0).round() / 1_000_000.0;

    // Apply secret scrubber across prompt, completion, metadata, and error_message
    let redacted_p = payload.prompt.as_deref().map(redact_secrets);
    let redacted_c = payload.completion.as_deref().map(redact_secrets);
    let redacted_meta = payload
        .metadata
        .map(|m| redact_secrets(&serde_json::to_string(&m).unwrap_or_default()));
    let redacted_err = payload.error_message.as_deref().map(redact_secrets);

    let mut tx = state.pool.begin().await?;

    let insert_sql = adapt_sql(
        r#"
        INSERT INTO request_traces (
            id, trace_id, span_name, model_name, provider, status_code,
            latency_ms, prompt_tokens, completion_tokens, total_tokens,
            estimated_cost, redacted_prompt, redacted_completion, metadata,
            error_message, created_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
        "#,
        state.is_postgres,
    );

    sqlx::query(&insert_sql)
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
        .bind(&redacted_meta)
        .bind(&redacted_err)
        .execute(&mut *tx)
        .await?;

    if estimated_cost > 0.0 {
        let budget_sql = adapt_sql(
            "UPDATE budget_settings SET current_spend_usd = current_spend_usd + ?, updated_at = CURRENT_TIMESTAMP WHERE id = 'default-budget'",
            state.is_postgres,
        );
        sqlx::query(&budget_sql)
            .bind(estimated_cost)
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await?;

    let fetch_sql = adapt_sql(
        "SELECT * FROM request_traces WHERE id = ?",
        state.is_postgres,
    );
    let created: RequestTrace = sqlx::query_as(&fetch_sql)
        .bind(&id)
        .fetch_one(&state.pool)
        .await?;

    // Check alert thresholds asynchronously without slowing trace ingestion
    let pool_clone = state.pool.clone();
    let is_pg = state.is_postgres;
    let webhook = state.alert_webhook_url.clone();
    tokio::spawn(async move {
        let _ = alerts::evaluate_and_dispatch_alerts(&pool_clone, is_pg, webhook.as_deref()).await;
    });

    Ok(created)
}

pub async fn create_trace(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateTraceRequest>,
) -> Result<Json<RequestTrace>, AppError> {
    let created = persist_trace_with_cost(&state, payload).await?;
    Ok(Json(created))
}
