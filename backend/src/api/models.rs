use crate::api::health::AppState;
use crate::domain::{CreateModelRequest, Model, UpdateStageRequest};
use crate::errors::AppError;
use crate::persistence::adapt_sql;
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
    let mut clauses = Vec::new();
    if query.stage.is_some() {
        clauses.push("AND stage = ?");
    }
    if query.framework.is_some() {
        clauses.push("AND LOWER(framework) LIKE LOWER(?)");
    }

    let raw_sql = format!(
        "SELECT * FROM models WHERE 1=1 {} ORDER BY created_at DESC",
        clauses.join(" ")
    );
    let sql = adapt_sql(&raw_sql, state.is_postgres);

    let mut q = sqlx::query_as::<_, Model>(&sql);
    if let Some(ref stage) = query.stage {
        q = q.bind(stage);
    }
    if let Some(ref fw) = query.framework {
        q = q.bind(format!("%{fw}%"));
    }

    let models = q.fetch_all(&state.pool).await?;
    Ok(Json(models))
}

pub async fn get_model(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Model>, AppError> {
    let sql = adapt_sql("SELECT * FROM models WHERE id = ?", state.is_postgres);
    let model: Option<Model> = sqlx::query_as(&sql)
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

    let insert_sql = adapt_sql(
        r#"
        INSERT INTO models (
            id, name, framework, task, stage, version, parameters_count,
            artifact_uri, description, tags, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
        "#,
        state.is_postgres,
    );

    sqlx::query(&insert_sql)
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

    let fetch_sql = adapt_sql("SELECT * FROM models WHERE id = ?", state.is_postgres);
    let created: Model = sqlx::query_as(&fetch_sql)
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

    let update_sql = adapt_sql(
        "UPDATE models SET stage = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
        state.is_postgres,
    );
    let rows_affected = sqlx::query(&update_sql)
        .bind(&payload.stage)
        .bind(&id)
        .execute(&state.pool)
        .await?
        .rows_affected();

    if rows_affected == 0 {
        return Err(AppError::NotFound(format!("Model '{id}' not found")));
    }

    let fetch_sql = adapt_sql("SELECT * FROM models WHERE id = ?", state.is_postgres);
    let updated: Model = sqlx::query_as(&fetch_sql)
        .bind(&id)
        .fetch_one(&state.pool)
        .await?;

    Ok(Json(updated))
}
