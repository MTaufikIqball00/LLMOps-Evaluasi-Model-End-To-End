use crate::api::health::AppState;
use crate::domain::Deployment;
use crate::errors::AppError;
use crate::persistence::adapt_sql;
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
    let update_sql = adapt_sql(
        "UPDATE deployments SET updated_at = CURRENT_TIMESTAMP WHERE id = ?",
        state.is_postgres,
    );
    let rows = sqlx::query(&update_sql)
        .bind(&id)
        .execute(&state.pool)
        .await?
        .rows_affected();

    if rows == 0 {
        return Err(AppError::NotFound(format!("Deployment '{id}' not found")));
    }

    let fetch_sql = adapt_sql("SELECT * FROM deployments WHERE id = ?", state.is_postgres);
    let dep: Deployment = sqlx::query_as(&fetch_sql)
        .bind(&id)
        .fetch_one(&state.pool)
        .await?;

    Ok(Json(dep))
}
