use crate::api::health::AppState;
use crate::errors::AppError;
use crate::observability::compute_latency_percentiles;
use axum::{extract::State, Json};
use chrono::{DateTime, NaiveDateTime, Utc};
use reqwest::Client;
use serde::Serialize;
use serde_json::Value;
use std::sync::Arc;

#[derive(Serialize)]
pub struct MonitoringTimeseries {
    pub timestamps: Vec<String>,
    pub throughput_rpm: Vec<f64>,
    pub p50_latency_ms: Vec<f64>,
    pub p95_latency_ms: Vec<f64>,
    pub p99_latency_ms: Vec<f64>,
    pub error_rate_pct: Vec<f64>,
    pub active_source: String,
    pub is_prometheus_live: bool,
}

#[derive(Serialize)]
pub struct LatencyBucket {
    pub range_label: String,
    pub count: i64,
    pub percentage: f64,
}

fn parse_trace_timestamp(ts: &str) -> Option<DateTime<Utc>> {
    if let Ok(dt) = DateTime::parse_from_rfc3339(ts) {
        return Some(dt.with_timezone(&Utc));
    }
    if let Ok(naive) = NaiveDateTime::parse_from_str(ts, "%Y-%m-%d %H:%M:%S") {
        return Some(DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc));
    }
    None
}

async fn query_prom_range(
    client: &Client,
    prom_url: &str,
    query: &str,
    start_ts: i64,
    end_ts: i64,
) -> Option<Vec<(i64, f64)>> {
    let base = prom_url.trim_end_matches('/');
    let url = format!("{base}/api/v1/query_range");
    let res = client
        .get(&url)
        .query(&[
            ("query", query),
            ("start", &start_ts.to_string()),
            ("end", &end_ts.to_string()),
            ("step", "300"),
        ])
        .send()
        .await
        .ok()?;

    if !res.status().is_success() {
        return None;
    }

    let body: Value = res.json().await.ok()?;
    let values = body["data"]["result"].get(0)?["values"].as_array()?;
    if values.is_empty() {
        return None;
    }

    let parsed: Vec<(i64, f64)> = values
        .iter()
        .filter_map(|pair| {
            let arr = pair.as_array()?;
            let ts = arr.first()?.as_f64()? as i64;
            let val = arr.get(1)?.as_str()?.parse::<f64>().ok()?;
            if val.is_finite() {
                Some((ts, val))
            } else {
                None
            }
        })
        .collect();

    if parsed.is_empty() {
        None
    } else {
        Some(parsed)
    }
}

async fn try_prometheus_series(state: &AppState) -> Option<MonitoringTimeseries> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_millis(1500))
        .build()
        .ok()?;

    let now = Utc::now().timestamp();
    let start = now - 55 * 60;

    let (p50_res, p95_res, p99_res, rpm_res) = tokio::join!(
        query_prom_range(
            &client,
            &state.prometheus_url,
            "llmops_llm_latency_p50_milliseconds",
            start,
            now
        ),
        query_prom_range(
            &client,
            &state.prometheus_url,
            "llmops_llm_latency_p95_milliseconds",
            start,
            now
        ),
        query_prom_range(
            &client,
            &state.prometheus_url,
            "llmops_llm_latency_p99_milliseconds",
            start,
            now
        ),
        query_prom_range(
            &client,
            &state.prometheus_url,
            "rate(llmops_http_requests_total[5m]) * 60",
            start,
            now
        ),
    );

    let p50_vals = p50_res?;
    let p95_vals = p95_res?;
    let p99_vals = p99_res?;
    let rpm_vals = rpm_res?;

    let len = p50_vals
        .len()
        .min(p95_vals.len())
        .min(p99_vals.len())
        .min(rpm_vals.len());
    if len == 0 {
        return None;
    }

    let timestamps = p50_vals[..len]
        .iter()
        .map(|(ts, _)| {
            DateTime::<Utc>::from_timestamp(*ts, 0)
                .map(|d| d.format("%H:%M").to_string())
                .unwrap_or_else(|| "00:00".to_string())
        })
        .collect();

    Some(MonitoringTimeseries {
        timestamps,
        throughput_rpm: rpm_vals[..len]
            .iter()
            .map(|(_, v)| (v * 10.0).round() / 10.0)
            .collect(),
        p50_latency_ms: p50_vals[..len]
            .iter()
            .map(|(_, v)| (v * 10.0).round() / 10.0)
            .collect(),
        p95_latency_ms: p95_vals[..len]
            .iter()
            .map(|(_, v)| (v * 10.0).round() / 10.0)
            .collect(),
        p99_latency_ms: p99_vals[..len]
            .iter()
            .map(|(_, v)| (v * 10.0).round() / 10.0)
            .collect(),
        error_rate_pct: vec![0.0; len],
        active_source: format!("Prometheus Live PromQL ({})", state.prometheus_url),
        is_prometheus_live: true,
    })
}

pub async fn get_monitoring_metrics(
    State(state): State<Arc<AppState>>,
) -> Result<Json<MonitoringTimeseries>, AppError> {
    if let Some(prom_series) = try_prometheus_series(&state).await {
        return Ok(Json(prom_series));
    }

    // Build real timeseries from persisted request_traces grouped into 12 x 5-minute windows
    let rows: Vec<(f64, i64, String)> = sqlx::query_as(
        "SELECT latency_ms, status_code, created_at FROM request_traces ORDER BY created_at ASC",
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let now = Utc::now();
    let mut timestamps = Vec::with_capacity(12);
    let mut buckets: Vec<Vec<(f64, i64)>> = vec![Vec::new(); 12];

    for idx in 0..12 {
        let minutes_ago = (11 - idx) as i64 * 5;
        let bucket_time = now - chrono::Duration::minutes(minutes_ago);
        timestamps.push(bucket_time.format("%H:%M").to_string());
    }

    for (lat, status, created_at) in &rows {
        let parsed = parse_trace_timestamp(created_at).unwrap_or(now);
        let age_mins = (now - parsed).num_minutes();
        if (0..60).contains(&age_mins) {
            let bucket_from_end = (age_mins / 5).min(11) as usize;
            let idx = 11 - bucket_from_end;
            buckets[idx].push((*lat, *status));
        }
    }

    let mut throughput = Vec::with_capacity(12);
    let mut p50s = Vec::with_capacity(12);
    let mut p95s = Vec::with_capacity(12);
    let mut p99s = Vec::with_capacity(12);
    let mut errors = Vec::with_capacity(12);

    for bucket in buckets {
        if bucket.is_empty() {
            throughput.push(0.0);
            p50s.push(0.0);
            p95s.push(0.0);
            p99s.push(0.0);
            errors.push(0.0);
        } else {
            let count = bucket.len() as f64;
            let err_cnt = bucket.iter().filter(|(_, st)| *st >= 400).count() as f64;
            let pct = compute_latency_percentiles(bucket.iter().map(|(l, _)| *l).collect());

            throughput.push(((count / 5.0) * 100.0).round() / 100.0);
            p50s.push(pct.p50);
            p95s.push(pct.p95);
            p99s.push(pct.p99);
            errors.push(((err_cnt / count) * 1000.0).round() / 10.0);
        }
    }

    Ok(Json(MonitoringTimeseries {
        timestamps,
        throughput_rpm: throughput,
        p50_latency_ms: p50s,
        p95_latency_ms: p95s,
        p99_latency_ms: p99s,
        error_rate_pct: errors,
        active_source: format!(
            "Live Database Telemetry ({} traces in rolling 60m window; Prometheus unreachable)",
            rows.len()
        ),
        is_prometheus_live: false,
    }))
}

pub async fn get_latency_distribution(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<LatencyBucket>>, AppError> {
    let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM request_traces")
        .fetch_one(&state.pool)
        .await
        .unwrap_or((0,));

    let count_total = if total.0 == 0 { 1 } else { total.0 };

    let b1: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM request_traces WHERE latency_ms < 150.0")
        .fetch_one(&state.pool)
        .await
        .unwrap_or((0,));

    let b2: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM request_traces WHERE latency_ms >= 150.0 AND latency_ms < 300.0",
    )
    .fetch_one(&state.pool)
    .await
    .unwrap_or((0,));

    let b3: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM request_traces WHERE latency_ms >= 300.0 AND latency_ms < 500.0",
    )
    .fetch_one(&state.pool)
    .await
    .unwrap_or((0,));

    let b4: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM request_traces WHERE latency_ms >= 500.0")
            .fetch_one(&state.pool)
            .await
            .unwrap_or((0,));

    let make_bucket = |label: &str, count: i64| LatencyBucket {
        range_label: label.to_string(),
        count,
        percentage: if total.0 == 0 {
            0.0
        } else {
            ((count as f64 / count_total as f64) * 1000.0).round() / 10.0
        },
    };

    Ok(Json(vec![
        make_bucket("< 150 ms (Fast)", b1.0),
        make_bucket("150 - 300 ms (Nominal)", b2.0),
        make_bucket("300 - 500 ms (Moderate)", b3.0),
        make_bucket(">= 500 ms (High / Timeout)", b4.0),
    ]))
}
