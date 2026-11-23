use crate::api::health::AppState;
use crate::domain::{AlertEvent, AlertRule, CreateAlertRuleRequest, UpdateAlertRuleRequest};
use crate::errors::AppError;
use crate::observability::alerts::{evaluate_and_dispatch_alerts, send_webhook_notification};
use crate::persistence::adapt_sql;
use axum::{
    extract::{Path, State},
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

pub async fn list_alert_rules(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<AlertRule>>, AppError> {
    let rules: Vec<AlertRule> =
        sqlx::query_as("SELECT * FROM alert_rules ORDER BY created_at DESC")
            .fetch_all(&state.pool)
            .await?;
    Ok(Json(rules))
}

pub async fn create_alert_rule(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateAlertRuleRequest>,
) -> Result<Json<AlertRule>, AppError> {
    let valid_metrics = [
        "hallucination_score",
        "latency_p95_ms",
        "budget_used_pct",
        "error_rate_pct",
    ];
    if !valid_metrics.contains(&payload.metric_type.as_str()) {
        return Err(AppError::BadRequest(format!(
            "Invalid metric_type '{}'. Must be one of: {:?}",
            payload.metric_type, valid_metrics
        )));
    }

    let id = format!(
        "rule-{}",
        Uuid::new_v4()
            .to_string()
            .chars()
            .take(8)
            .collect::<String>()
    );
    let operator = payload.operator.unwrap_or_else(|| "gt".to_string());
    let severity = payload.severity.unwrap_or_else(|| "High".to_string());
    let channel_type = payload
        .channel_type
        .unwrap_or_else(|| "webhook".to_string());
    let webhook_url = payload.webhook_url.filter(|s| !s.trim().is_empty());

    let insert_sql = adapt_sql(
        r#"
        INSERT INTO alert_rules (
            id, name, metric_type, operator, threshold, severity,
            webhook_url, channel_type, is_enabled, last_triggered_at, created_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, 1, NULL, CURRENT_TIMESTAMP)
        "#,
        state.is_postgres,
    );

    sqlx::query(&insert_sql)
        .bind(&id)
        .bind(&payload.name)
        .bind(&payload.metric_type)
        .bind(&operator)
        .bind(payload.threshold)
        .bind(&severity)
        .bind(&webhook_url)
        .bind(&channel_type)
        .execute(&state.pool)
        .await?;

    let fetch_sql = adapt_sql("SELECT * FROM alert_rules WHERE id = ?", state.is_postgres);
    let created: AlertRule = sqlx::query_as(&fetch_sql)
        .bind(&id)
        .fetch_one(&state.pool)
        .await?;

    Ok(Json(created))
}

pub async fn update_alert_rule(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateAlertRuleRequest>,
) -> Result<Json<AlertRule>, AppError> {
    let check_sql = adapt_sql("SELECT * FROM alert_rules WHERE id = ?", state.is_postgres);
    let exists: Option<AlertRule> = sqlx::query_as(&check_sql)
        .bind(&id)
        .fetch_optional(&state.pool)
        .await?;
    if exists.is_none() {
        return Err(AppError::NotFound(format!("Alert rule '{id}' not found")));
    }

    if let Some(enabled) = payload.is_enabled {
        let enabled_int: i64 = if enabled { 1 } else { 0 };
        let sql = adapt_sql(
            "UPDATE alert_rules SET is_enabled = ? WHERE id = ?",
            state.is_postgres,
        );
        sqlx::query(&sql)
            .bind(enabled_int)
            .bind(&id)
            .execute(&state.pool)
            .await?;
    }

    if let Some(threshold) = payload.threshold {
        let sql = adapt_sql(
            "UPDATE alert_rules SET threshold = ? WHERE id = ?",
            state.is_postgres,
        );
        sqlx::query(&sql)
            .bind(threshold)
            .bind(&id)
            .execute(&state.pool)
            .await?;
    }

    if let Some(ref url) = payload.webhook_url {
        let sql = adapt_sql(
            "UPDATE alert_rules SET webhook_url = ? WHERE id = ?",
            state.is_postgres,
        );
        sqlx::query(&sql)
            .bind(url)
            .bind(&id)
            .execute(&state.pool)
            .await?;
    }

    let updated: AlertRule = sqlx::query_as(&check_sql)
        .bind(&id)
        .fetch_one(&state.pool)
        .await?;
    Ok(Json(updated))
}

pub async fn delete_alert_rule(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let del_sql = adapt_sql("DELETE FROM alert_rules WHERE id = ?", state.is_postgres);
    let rows = sqlx::query(&del_sql)
        .bind(&id)
        .execute(&state.pool)
        .await?
        .rows_affected();

    if rows == 0 {
        return Err(AppError::NotFound(format!("Alert rule '{id}' not found")));
    }

    Ok(Json(serde_json::json!({ "deleted": true, "id": id })))
}

pub async fn list_alert_events(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<AlertEvent>>, AppError> {
    let events: Vec<AlertEvent> =
        sqlx::query_as("SELECT * FROM alert_events ORDER BY created_at DESC LIMIT 100")
            .fetch_all(&state.pool)
            .await?;
    Ok(Json(events))
}

pub async fn evaluate_alerts_now(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<AlertEvent>>, AppError> {
    let triggered = evaluate_and_dispatch_alerts(
        &state.pool,
        state.is_postgres,
        state.alert_webhook_url.as_deref(),
    )
    .await?;
    Ok(Json(triggered))
}

pub async fn test_alert_webhook(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let fetch_sql = adapt_sql("SELECT * FROM alert_rules WHERE id = ?", state.is_postgres);
    let rule: AlertRule = sqlx::query_as(&fetch_sql)
        .bind(&id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Alert rule '{id}' not found")))?;

    let target_url = rule
        .webhook_url
        .as_deref()
        .filter(|s| !s.is_empty())
        .or(state.alert_webhook_url.as_deref());

    let delivery_status = if let Some(url) = target_url {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(3))
            .build()
            .unwrap_or_default();
        send_webhook_notification(
            &client,
            url,
            &rule.channel_type,
            &format!("[TEST] {}", rule.name),
            &format!(
                "Verification ping for rule '{}' ({} {} {}).",
                rule.name, rule.metric_type, rule.operator, rule.threshold
            ),
            &rule.severity,
        )
        .await
    } else {
        "Dry-Run Verified (No Webhook URL configured)".to_string()
    };

    Ok(Json(serde_json::json!({
        "rule_id": rule.id,
        "channel_type": rule.channel_type,
        "delivery_status": delivery_status,
        "tested_at": chrono::Utc::now().to_rfc3339()
    })))
}
