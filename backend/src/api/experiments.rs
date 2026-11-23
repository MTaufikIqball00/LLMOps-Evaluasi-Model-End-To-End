use crate::api::health::AppState;
use crate::domain::{CreateExperimentRequest, Experiment};
use crate::errors::AppError;
use crate::persistence::adapt_sql;
use axum::{
    extract::{Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct ExperimentListQuery {
    pub model_id: Option<String>,
    pub framework: Option<String>,
}

#[derive(Deserialize)]
pub struct CompareQuery {
    pub runs: Option<String>,
}

#[derive(Serialize)]
pub struct RunComparison {
    pub run_ids: Vec<String>,
    pub runs: Vec<Experiment>,
    pub parameter_keys: Vec<String>,
    pub metric_keys: Vec<String>,
}

pub async fn list_experiments(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ExperimentListQuery>,
) -> Result<Json<Vec<Experiment>>, AppError> {
    let mut clauses = Vec::new();
    if query.model_id.is_some() {
        clauses.push("AND model_id = ?");
    }
    if query.framework.is_some() {
        clauses.push("AND LOWER(framework) LIKE LOWER(?)");
    }

    let raw_sql = format!(
        "SELECT * FROM experiments WHERE 1=1 {} ORDER BY created_at DESC",
        clauses.join(" ")
    );
    let sql = adapt_sql(&raw_sql, state.is_postgres);

    let mut q = sqlx::query_as::<_, Experiment>(&sql);
    if let Some(ref mid) = query.model_id {
        q = q.bind(mid);
    }
    if let Some(ref fw) = query.framework {
        q = q.bind(format!("%{fw}%"));
    }

    let runs = q.fetch_all(&state.pool).await?;
    Ok(Json(runs))
}

pub async fn create_experiment(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateExperimentRequest>,
) -> Result<Json<Experiment>, AppError> {
    let id = format!(
        "exp-{}",
        Uuid::new_v4()
            .to_string()
            .chars()
            .take(8)
            .collect::<String>()
    );
    let params_str =
        serde_json::to_string(&payload.parameters).unwrap_or_else(|_| "{}".to_string());
    let metrics_str = serde_json::to_string(&payload.metrics).unwrap_or_else(|_| "{}".to_string());
    let tags_str = payload
        .tags
        .map(|t| serde_json::to_string(&t).unwrap_or_default());
    let duration = payload.duration_seconds.unwrap_or(0);

    let insert_sql = adapt_sql(
        r#"
        INSERT INTO experiments (
            id, name, model_id, framework, status, parameters, metrics, tags, duration_seconds, created_at
        ) VALUES (?, ?, ?, ?, 'Completed', ?, ?, ?, ?, CURRENT_TIMESTAMP)
        "#,
        state.is_postgres,
    );

    sqlx::query(&insert_sql)
        .bind(&id)
        .bind(&payload.name)
        .bind(&payload.model_id)
        .bind(&payload.framework)
        .bind(&params_str)
        .bind(&metrics_str)
        .bind(&tags_str)
        .bind(duration)
        .execute(&state.pool)
        .await?;

    let fetch_sql = adapt_sql("SELECT * FROM experiments WHERE id = ?", state.is_postgres);
    let created: Experiment = sqlx::query_as(&fetch_sql)
        .bind(&id)
        .fetch_one(&state.pool)
        .await?;

    Ok(Json(created))
}

pub async fn compare_experiments(
    State(state): State<Arc<AppState>>,
    Query(query): Query<CompareQuery>,
) -> Result<Json<RunComparison>, AppError> {
    let runs_list: Vec<String> = query
        .runs
        .unwrap_or_default()
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let all_runs: Vec<Experiment> = if runs_list.is_empty() {
        sqlx::query_as("SELECT * FROM experiments ORDER BY created_at DESC LIMIT 3")
            .fetch_all(&state.pool)
            .await?
    } else {
        let placeholders = vec!["?"; runs_list.len()].join(", ");
        let raw_sql = format!("SELECT * FROM experiments WHERE id IN ({placeholders})");
        let sql = adapt_sql(&raw_sql, state.is_postgres);
        let mut q = sqlx::query_as::<_, Experiment>(&sql);
        for id in &runs_list {
            q = q.bind(id);
        }
        q.fetch_all(&state.pool).await?
    };

    let mut param_keys = std::collections::BTreeSet::new();
    let mut metric_keys = std::collections::BTreeSet::new();

    for r in &all_runs {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&r.parameters) {
            if let Some(obj) = val.as_object() {
                for k in obj.keys() {
                    param_keys.insert(k.clone());
                }
            }
        }
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&r.metrics) {
            if let Some(obj) = val.as_object() {
                for k in obj.keys() {
                    metric_keys.insert(k.clone());
                }
            }
        }
    }

    let ids: Vec<String> = all_runs.iter().map(|r| r.id.clone()).collect();

    Ok(Json(RunComparison {
        run_ids: ids,
        runs: all_runs,
        parameter_keys: param_keys.into_iter().collect(),
        metric_keys: metric_keys.into_iter().collect(),
    }))
}
