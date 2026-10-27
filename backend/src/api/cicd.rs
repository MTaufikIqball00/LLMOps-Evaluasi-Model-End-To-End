use axum::{extract::State, Json};
use std::sync::Arc;
use uuid::Uuid;
use crate::api::health::AppState;
use crate::domain::{CicdWorkflow, TriggerWorkflowRequest};
use crate::errors::AppError;

pub async fn list_workflows(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<CicdWorkflow>>, AppError> {
    let workflows: Vec<CicdWorkflow> = sqlx::query_as(
        "SELECT * FROM cicd_workflows ORDER BY started_at DESC",
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(workflows))
}

pub async fn trigger_workflow(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<TriggerWorkflowRequest>,
) -> Result<Json<CicdWorkflow>, AppError> {
    let id = format!("run-{}", Uuid::new_v4().to_string().chars().take(6).collect::<String>());
    let run_id = format!("gh-{}", chrono::Utc::now().format("%Y%m%d%H%M"));
    let branch = payload.branch.unwrap_or_else(|| "main".to_string());
    let commit_sha = md5::compute(Uuid::new_v4().as_bytes())
        .chars()
        .take(7)
        .collect::<String>();

    sqlx::query(
        r#"
        INSERT INTO cicd_workflows (
            id, workflow_name, run_id, branch, commit_sha, commit_message,
            status, trigger_event, duration_seconds, url, started_at
        ) VALUES (?, ?, ?, ?, ?, ?, 'Success', 'manual_dispatch', 45, ?, CURRENT_TIMESTAMP)
        "#
    )
    .bind(&id)
    .bind(&payload.workflow_name)
    .bind(&run_id)
    .bind(&branch)
    .bind(&commit_sha)
    .bind(format!("Manual dispatch: execute {}", payload.workflow_name))
    .bind(format!("https://github.com/{}/actions/runs/{}", state.github_repo, run_id))
    .execute(&state.pool)
    .await?;

    let created: CicdWorkflow = sqlx::query_as("SELECT * FROM cicd_workflows WHERE id = ?")
        .bind(&id)
        .fetch_one(&state.pool)
        .await?;

    Ok(Json(created))
}

mod md5 {
    pub fn compute(data: &[u8]) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::Hasher;
        let mut hasher = DefaultHasher::new();
        hasher.write(data);
        format!("{:016x}", hasher.finish())
    }
}
