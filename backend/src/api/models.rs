use crate::api::health::AppState;
use crate::domain::{CreateModelRequest, Model, UpdateStageRequest};
use crate::errors::AppError;
use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct ModelListQuery {
    pub stage: Option<String>,
    pub framework: Option<String>,
}

pub async fn list_models(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ModelListQuery>,
) -> Result<Json<Vec<Model>>, AppError> {
    let mut qb = sqlx::QueryBuilder::new("SELECT * FROM models WHERE 1=1");
    if let Some(ref stage) = query.stage {
        qb.push(" AND stage = ");
        qb.push_bind(stage);
    }
    if let Some(ref fw) = query.framework {
        qb.push(" AND framework LIKE ");
        qb.push_bind(format!("%{}%", fw));
    }
    qb.push(" ORDER BY created_at DESC");

    let models: Vec<Model> = qb.build_query_as().fetch_all(&state.pool).await?;
    Ok(Json(models))
}

pub async fn get_model(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Model>, AppError> {
    let model: Option<Model> = sqlx::query_as("SELECT * FROM models WHERE id = ?")
        .bind(&id)
        .fetch_optional(&state.pool)
        .await?;

    model
        .map(Json)
        .ok_or_else(|| AppError::NotFound(format!("Model with id '{id}' not found")))
}

pub async fn create_model(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateModelRequest>,
) -> Result<Json<Model>, AppError> {
    let id = format!(
        "m-{}",
        Uuid::new_v4()
            .to_string()
            .chars()
            .take(8)
            .collect::<String>()
    );
    let stage = payload.stage.unwrap_or_else(|| "Development".to_string());
    let tags_json = payload
        .tags
        .map(|t| serde_json::to_string(&t).unwrap_or_default());

    sqlx::query(
        r#"
        INSERT INTO models (
            id, name, framework, task, stage, version, parameters_count,
            artifact_uri, description, tags, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
        "#,
    )
    .bind(&id)
    .bind(&payload.name)
    .bind(&payload.framework)
    .bind(&payload.task)
    .bind(&stage)
    .bind(&payload.version)
    .bind(&payload.parameters_count)
    .bind(&payload.artifact_uri)
    .bind(&payload.description)
    .bind(&tags_json)
    .execute(&state.pool)
    .await?;

    let created: Model = sqlx::query_as("SELECT * FROM models WHERE id = ?")
        .bind(&id)
        .fetch_one(&state.pool)
        .await?;

    Ok(Json(created))
}

pub async fn update_stage(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateStageRequest>,
) -> Result<Json<Model>, AppError> {
    let valid_stages = ["Development", "Staging", "Production", "Archived"];
    if !valid_stages.contains(&payload.stage.as_str()) {
        return Err(AppError::BadRequest(format!(
            "Invalid stage '{}'. Must be one of: {:?}",
            payload.stage, valid_stages
        )));
    }

    let rows_affected =
        sqlx::query("UPDATE models SET stage = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?")
            .bind(&payload.stage)
            .bind(&id)
            .execute(&state.pool)
            .await?
            .rows_affected();

    if rows_affected == 0 {
        return Err(AppError::NotFound(format!("Model '{id}' not found")));
    }

    let updated: Model = sqlx::query_as("SELECT * FROM models WHERE id = ?")
        .bind(&id)
        .fetch_one(&state.pool)
        .await?;

    Ok(Json(updated))
}
