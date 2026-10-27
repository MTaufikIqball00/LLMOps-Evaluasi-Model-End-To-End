use crate::api::health::AppState;
use crate::domain::Deployment;
use crate::errors::AppError;
use axum::{
    extract::{Path, State},
    Json,
};
use std::sync::Arc;

pub async fn list_deployments(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Deployment>>, AppError> {
    let deps: Vec<Deployment> =
        sqlx::query_as("SELECT * FROM deployments ORDER BY environment ASC")
            .fetch_all(&state.pool)
            .await?;
    Ok(Json(deps))
}

pub async fn restart_deployment(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Deployment>, AppError> {
    let rows = sqlx::query("UPDATE deployments SET updated_at = CURRENT_TIMESTAMP WHERE id = ?")
        .bind(&id)
        .execute(&state.pool)
        .await?
        .rows_affected();

    if rows == 0 {
        return Err(AppError::NotFound(format!("Deployment '{id}' not found")));
    }

    let dep: Deployment = sqlx::query_as("SELECT * FROM deployments WHERE id = ?")
        .bind(&id)
        .fetch_one(&state.pool)
        .await?;

    Ok(Json(dep))
}
