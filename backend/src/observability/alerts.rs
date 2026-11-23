use crate::domain::{AlertEvent, AlertRule};
use crate::errors::AppError;
use crate::observability::compute_latency_percentiles;
use crate::persistence::{adapt_sql, DbPool};
use reqwest::Client;
use serde_json::json;
use tracing::{info, warn};
use uuid::Uuid;

fn compare_metric(observed: f64, operator: &str, threshold: f64) -> bool {
    match operator {
        "gt" | ">" => observed > threshold,
        "gte" | ">=" => observed >= threshold,
        "lt" | "<" => observed < threshold,
        "lte" | "<=" => observed <= threshold,
        _ => observed > threshold,
    }
}

pub async fn send_webhook_notification(
    client: &Client,
    webhook_url: &str,
    channel_type: &str,
    title: &str,
    message: &str,
    severity: &str,
) -> String {
    let payload = match channel_type.to_ascii_lowercase().as_str() {
        "slack" => json!({
            "text": format!("[{}] *{}*\n{}", severity.to_uppercase(), title, message)
        }),
        "discord" => json!({
            "content": format!("[{}] **{}** — {}", severity.to_uppercase(), title, message)
        }),
        _ => json!({
            "source": "llmops-control-center",
            "severity": severity,
            "title": title,
            "message": message,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }),
    };

    match client.post(webhook_url).json(&payload).send().await {
        Ok(res) if res.status().is_success() => "Delivered".to_string(),
        Ok(res) => format!("Failed: HTTP {}", res.status()),
        Err(e) => format!("Failed: {}", e),
    }
}

/// Evaluates all enabled alert rules against real DB metrics and logs/dispatches any breaches.
pub async fn evaluate_and_dispatch_alerts(
    pool: &DbPool,
    is_pg: bool,
    default_webhook: Option<&str>,
) -> Result<Vec<AlertEvent>, AppError> {
    let rules_sql = adapt_sql(
        "SELECT * FROM alert_rules WHERE is_enabled = 1 ORDER BY created_at ASC",
        is_pg,
    );
    let rules: Vec<AlertRule> = sqlx::query_as(&rules_sql).fetch_all(pool).await?;
    if rules.is_empty() {
        return Ok(vec![]);
    }

    // Collect live observed metrics from DB
    let max_hallucination: (f64,) =
        sqlx::query_as("SELECT COALESCE(MAX(hallucination_score), 0.0) FROM evaluations")
            .fetch_one(pool)
            .await
            .unwrap_or((0.0,));

    let latencies: Vec<(f64,)> = sqlx::query_as("SELECT latency_ms FROM request_traces")
        .fetch_all(pool)
        .await
        .unwrap_or_default();
    let pct = compute_latency_percentiles(latencies.into_iter().map(|r| r.0).collect());

    let budget_row: (f64, f64) =
        sqlx::query_as("SELECT monthly_budget_usd, current_spend_usd FROM budget_settings LIMIT 1")
            .fetch_one(pool)
            .await
            .unwrap_or((2500.0, 0.0));
    let budget_used_pct = if budget_row.0 > 0.0 {
        ((budget_row.1 / budget_row.0) * 1000.0).round() / 10.0
    } else {
        0.0
    };

    let err_counts: (i64, i64) = sqlx::query_as(
        "SELECT COUNT(*), COALESCE(SUM(CASE WHEN status_code >= 400 THEN 1 ELSE 0 END), 0) FROM request_traces",
    )
    .fetch_one(pool)
    .await
    .unwrap_or((0, 0));
    let error_rate_pct = if err_counts.0 > 0 {
        ((err_counts.1 as f64 / err_counts.0 as f64) * 1000.0).round() / 10.0
    } else {
        0.0
    };

    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .build()
        .unwrap_or_default();

    let mut triggered_events = Vec::new();

    for rule in rules {
        let observed = match rule.metric_type.as_str() {
            "hallucination_score" => max_hallucination.0,
            "latency_p95_ms" => pct.p95,
            "budget_used_pct" => budget_used_pct,
            "error_rate_pct" => error_rate_pct,
            _ => continue,
        };

        if !compare_metric(observed, &rule.operator, rule.threshold) {
            continue;
        }

        let event_id = format!(
            "alev-{}",
            Uuid::new_v4()
                .to_string()
                .chars()
                .take(8)
                .collect::<String>()
        );
        let msg = format!(
            "Rule '{}' breached: {} observed at {:.3} (threshold {} {:.3})",
            rule.name, rule.metric_type, observed, rule.operator, rule.threshold
        );

        let target_webhook = rule
            .webhook_url
            .as_deref()
            .filter(|s| !s.is_empty())
            .or(default_webhook);

        let delivery_status = if let Some(url) = target_webhook {
            let status = send_webhook_notification(
                &client,
                url,
                &rule.channel_type,
                &rule.name,
                &msg,
                &rule.severity,
            )
            .await;
            if status.starts_with("Failed") {
                warn!("Alert webhook dispatch failure for {}: {}", rule.id, status);
            } else {
                info!("Alert webhook dispatched for rule {}", rule.name);
            }
            status
        } else {
            "Logged (No Webhook Configured)".to_string()
        };

        let insert_sql = adapt_sql(
            r#"
            INSERT INTO alert_events (
                id, rule_id, rule_name, metric_type, observed_value, threshold,
                severity, message, delivery_status, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
            "#,
            is_pg,
        );

        sqlx::query(&insert_sql)
            .bind(&event_id)
            .bind(&rule.id)
            .bind(&rule.name)
            .bind(&rule.metric_type)
            .bind(observed)
            .bind(rule.threshold)
            .bind(&rule.severity)
            .bind(&msg)
            .bind(&delivery_status)
            .execute(pool)
            .await?;

        let upd_sql = adapt_sql(
            "UPDATE alert_rules SET last_triggered_at = CURRENT_TIMESTAMP WHERE id = ?",
            is_pg,
        );
        sqlx::query(&upd_sql).bind(&rule.id).execute(pool).await?;

        let fetch_sql = adapt_sql("SELECT * FROM alert_events WHERE id = ?", is_pg);
        if let Ok(ev) = sqlx::query_as::<_, AlertEvent>(&fetch_sql)
            .bind(&event_id)
            .fetch_one(pool)
            .await
        {
            triggered_events.push(ev);
        }
    }

    Ok(triggered_events)
}
