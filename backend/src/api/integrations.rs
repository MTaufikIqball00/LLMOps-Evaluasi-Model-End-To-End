use axum::{
    extract::{Path, State},
    Json,
};
use std::sync::Arc;
use crate::api::health::AppState;
use crate::domain::{IntegrationConfig, IntegrationStatusReport, UpdateIntegrationRequest};
use crate::errors::AppError;
use crate::integrations::{
    github_actions, grafana, kubernetes, mlflow, prometheus, ragas, trulens, wandb,
};

pub async fn list_integrations(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<IntegrationStatusReport>>, AppError> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build()
        .unwrap_or_default();

    let reports = vec![
        mlflow::check_status(&client, &state.mlflow_url).await,
        wandb::check_status(&client).await,
        prometheus::check_status(&client, &state.prometheus_url).await,
        grafana::check_status(&client, &state.grafana_url).await,
        ragas::check_status().await,
        trulens::check_status().await,
        kubernetes::check_status().await,
        github_actions::check_status(&client, &state.github_repo, state.github_token.as_deref()).await,
    ];

    Ok(Json(reports))
}

pub async fn test_integration(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<IntegrationStatusReport>, AppError> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .build()
        .unwrap_or_default();

    let report = match name.to_lowercase().as_str() {
        "mlflow" => mlflow::check_status(&client, &state.mlflow_url).await,
        "wandb" => wandb::check_status(&client).await,
        "prometheus" => prometheus::check_status(&client, &state.prometheus_url).await,
        "grafana" => grafana::check_status(&client, &state.grafana_url).await,
        "ragas" => ragas::check_status().await,
        "trulens" => trulens::check_status().await,
        "kubernetes" => kubernetes::check_status().await,
        "github_actions" => {
            github_actions::check_status(&client, &state.github_repo, state.github_token.as_deref())
                .await
        }
        _ => return Err(AppError::BadRequest(format!("Unknown integration '{name}'"))),
    };

    Ok(Json(report))
}

pub async fn update_integration(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    Json(payload): Json<UpdateIntegrationRequest>,
) -> Result<Json<IntegrationConfig>, AppError> {
    let is_enabled_int = payload.is_enabled.map(|b| if b { 1 } else { 0 });
    let masked_token = payload.auth_token.as_ref().map(|t| {
        if t.len() > 6 {
            format!("{}***{}", &t[0..3], &t[t.len() - 3..])
        } else {
            "***".to_string()
        }
    });

    let exists: Option<(String,)> = sqlx::query_as("SELECT id FROM integrations_config WHERE service_name = ?")
        .bind(&name)
        .fetch_optional(&state.pool)
        .await?;

    if exists.is_none() {
        return Err(AppError::NotFound(format!("Integration '{name}' not found")));
    }

    if let Some(url) = payload.endpoint_url {
        sqlx::query("UPDATE integrations_config SET endpoint_url = ? WHERE service_name = ?")
            .bind(&url)
            .bind(&name)
            .execute(&state.pool)
            .await?;
    }

    if let Some(masked) = masked_token {
        sqlx::query("UPDATE integrations_config SET auth_token_masked = ? WHERE service_name = ?")
            .bind(&masked)
            .bind(&name)
            .execute(&state.pool)
            .await?;
    }

    if let Some(enabled) = is_enabled_int {
        sqlx::query("UPDATE integrations_config SET is_enabled = ? WHERE service_name = ?")
            .bind(enabled)
            .bind(&name)
            .execute(&state.pool)
            .await?;
    }

    let updated: IntegrationConfig = sqlx::query_as("SELECT * FROM integrations_config WHERE service_name = ?")
        .bind(&name)
        .fetch_one(&state.pool)
        .await?;

    Ok(Json(updated))
}
