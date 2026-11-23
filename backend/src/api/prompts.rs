use crate::api::health::AppState;
use crate::domain::{
    CreatePromptRequest, PromptCompareResponse, PromptDiffLine, PromptTemplate, UpdatePromptRequest,
};
use crate::errors::AppError;
use crate::persistence::adapt_sql;
use axum::{
    extract::{Path, Query, State},
    Json,
};
use regex::Regex;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::sync::{Arc, OnceLock};
use uuid::Uuid;

static VAR_REGEX: OnceLock<Regex> = OnceLock::new();

fn extract_variables(template: &str) -> Vec<String> {
    let re = VAR_REGEX.get_or_init(|| Regex::new(r"\{\{\s*([a-zA-Z0-9_.-]+)\s*\}\}").unwrap());
    let mut vars = BTreeSet::new();
    for cap in re.captures_iter(template) {
        if let Some(m) = cap.get(1) {
            vars.insert(m.as_str().to_string());
        }
    }
    vars.into_iter().collect()
}

fn slugify(input: &str) -> String {
    let mut out = String::new();
    let mut prev_dash = false;
    for ch in input.to_ascii_lowercase().chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
            prev_dash = false;
        } else if !prev_dash && !out.is_empty() {
            out.push('-');
            prev_dash = true;
        }
    }
    out.trim_end_matches('-').to_string()
}

#[derive(Deserialize)]
pub struct PromptListQuery {
    pub stage: Option<String>,
    pub slug: Option<String>,
}

#[derive(Deserialize)]
pub struct PromptCompareQuery {
    pub a: Option<String>,
    pub b: Option<String>,
}

pub async fn list_prompts(
    State(state): State<Arc<AppState>>,
    Query(query): Query<PromptListQuery>,
) -> Result<Json<Vec<PromptTemplate>>, AppError> {
    let mut clauses = Vec::new();
    if query.stage.is_some() {
        clauses.push("AND stage = ?");
    }
    if query.slug.is_some() {
        clauses.push("AND slug = ?");
    }

    let raw_sql = format!(
        "SELECT * FROM prompts WHERE 1=1 {} ORDER BY updated_at DESC",
        clauses.join(" ")
    );
    let sql = adapt_sql(&raw_sql, state.is_postgres);

    let mut q = sqlx::query_as::<_, PromptTemplate>(&sql);
    if let Some(ref st) = query.stage {
        q = q.bind(st);
    }
    if let Some(ref sl) = query.slug {
        q = q.bind(sl);
    }

    let list = q.fetch_all(&state.pool).await?;
    Ok(Json(list))
}

pub async fn create_prompt(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreatePromptRequest>,
) -> Result<Json<PromptTemplate>, AppError> {
    if payload.name.trim().is_empty() || payload.template.trim().is_empty() {
        return Err(AppError::BadRequest(
            "Prompt name and template must not be empty".to_string(),
        ));
    }

    let id = format!(
        "prm-{}",
        Uuid::new_v4()
            .to_string()
            .chars()
            .take(8)
            .collect::<String>()
    );
    let slug = payload
        .slug
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| slugify(&payload.name));
    let stage = payload.stage.unwrap_or_else(|| "Draft".to_string());
    let vars_json = serde_json::to_string(&extract_variables(&payload.template))
        .unwrap_or_else(|_| "[]".to_string());
    let tags_json = payload
        .tags
        .map(|t| serde_json::to_string(&t).unwrap_or_default());

    let insert_sql = adapt_sql(
        r#"
        INSERT INTO prompts (
            id, name, slug, version, template, variables, model_target, stage,
            description, tags, avg_faithfulness, avg_latency_ms, usage_count,
            created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 0.90, 250.0, 0, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
        "#,
        state.is_postgres,
    );

    sqlx::query(&insert_sql)
        .bind(&id)
        .bind(&payload.name)
        .bind(&slug)
        .bind(&payload.version)
        .bind(&payload.template)
        .bind(&vars_json)
        .bind(&payload.model_target)
        .bind(&stage)
        .bind(&payload.description)
        .bind(&tags_json)
        .execute(&state.pool)
        .await?;

    let fetch_sql = adapt_sql("SELECT * FROM prompts WHERE id = ?", state.is_postgres);
    let created: PromptTemplate = sqlx::query_as(&fetch_sql)
        .bind(&id)
        .fetch_one(&state.pool)
        .await?;

    Ok(Json(created))
}

pub async fn update_prompt(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(payload): Json<UpdatePromptRequest>,
) -> Result<Json<PromptTemplate>, AppError> {
    let check_sql = adapt_sql("SELECT * FROM prompts WHERE id = ?", state.is_postgres);
    let existing: Option<PromptTemplate> = sqlx::query_as(&check_sql)
        .bind(&id)
        .fetch_optional(&state.pool)
        .await?;
    let current = existing.ok_or_else(|| AppError::NotFound(format!("Prompt '{id}' not found")))?;

    if let Some(ref st) = payload.stage {
        let valid = ["Draft", "Staging", "Production", "Archived"];
        if !valid.contains(&st.as_str()) {
            return Err(AppError::BadRequest(format!(
                "Invalid prompt stage '{}'. Must be one of: {:?}",
                st, valid
            )));
        }
        let sql = adapt_sql(
            "UPDATE prompts SET stage = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
            state.is_postgres,
        );
        sqlx::query(&sql)
            .bind(st)
            .bind(&id)
            .execute(&state.pool)
            .await?;
    }

    if let Some(ref tmpl) = payload.template {
        let vars_json =
            serde_json::to_string(&extract_variables(tmpl)).unwrap_or_else(|_| "[]".to_string());
        let sql = adapt_sql(
            "UPDATE prompts SET template = ?, variables = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
            state.is_postgres,
        );
        sqlx::query(&sql)
            .bind(tmpl)
            .bind(&vars_json)
            .bind(&id)
            .execute(&state.pool)
            .await?;
    }

    if let Some(ref desc) = payload.description {
        let sql = adapt_sql(
            "UPDATE prompts SET description = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
            state.is_postgres,
        );
        sqlx::query(&sql)
            .bind(desc)
            .bind(&id)
            .execute(&state.pool)
            .await?;
    } else if payload.stage.is_none() && payload.template.is_none() {
        return Ok(Json(current));
    }

    let updated: PromptTemplate = sqlx::query_as(&check_sql)
        .bind(&id)
        .fetch_one(&state.pool)
        .await?;
    Ok(Json(updated))
}

pub async fn delete_prompt(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let del_sql = adapt_sql("DELETE FROM prompts WHERE id = ?", state.is_postgres);
    let rows = sqlx::query(&del_sql)
        .bind(&id)
        .execute(&state.pool)
        .await?
        .rows_affected();

    if rows == 0 {
        return Err(AppError::NotFound(format!("Prompt '{id}' not found")));
    }

    Ok(Json(serde_json::json!({ "deleted": true, "id": id })))
}

pub async fn compare_prompts(
    State(state): State<Arc<AppState>>,
    Query(query): Query<PromptCompareQuery>,
) -> Result<Json<PromptCompareResponse>, AppError> {
    let all_prompts: Vec<PromptTemplate> =
        sqlx::query_as("SELECT * FROM prompts ORDER BY created_at ASC")
            .fetch_all(&state.pool)
            .await?;

    if all_prompts.len() < 2 {
        return Err(AppError::BadRequest(
            "At least two prompt versions are required for A/B comparison".to_string(),
        ));
    }

    let prompt_a = match query.a {
        Some(ref id_a) => all_prompts
            .iter()
            .find(|p| &p.id == id_a)
            .cloned()
            .ok_or_else(|| AppError::NotFound(format!("Prompt A '{id_a}' not found")))?,
        None => all_prompts[0].clone(),
    };

    let prompt_b = match query.b {
        Some(ref id_b) => all_prompts
            .iter()
            .find(|p| &p.id == id_b)
            .cloned()
            .ok_or_else(|| AppError::NotFound(format!("Prompt B '{id_b}' not found")))?,
        None => all_prompts[1].clone(),
    };

    let vars_a: BTreeSet<String> = extract_variables(&prompt_a.template).into_iter().collect();
    let vars_b: BTreeSet<String> = extract_variables(&prompt_b.template).into_iter().collect();

    let shared_variables: Vec<String> = vars_a.intersection(&vars_b).cloned().collect();
    let added_variables: Vec<String> = vars_b.difference(&vars_a).cloned().collect();
    let removed_variables: Vec<String> = vars_a.difference(&vars_b).cloned().collect();

    let lines_a: Vec<&str> = prompt_a.template.lines().collect();
    let lines_b: Vec<&str> = prompt_b.template.lines().collect();
    let set_a: BTreeSet<&str> = lines_a.iter().copied().collect();
    let set_b: BTreeSet<&str> = lines_b.iter().copied().collect();

    let mut diff_lines = Vec::new();
    let mut line_num = 1usize;

    for l in &lines_a {
        if set_b.contains(l) {
            diff_lines.push(PromptDiffLine {
                line_number: line_num,
                change_type: "unchanged".to_string(),
                content: (*l).to_string(),
            });
        } else {
            diff_lines.push(PromptDiffLine {
                line_number: line_num,
                change_type: "removed".to_string(),
                content: (*l).to_string(),
            });
        }
        line_num += 1;
    }
    for l in &lines_b {
        if !set_a.contains(l) {
            diff_lines.push(PromptDiffLine {
                line_number: line_num,
                change_type: "added".to_string(),
                content: (*l).to_string(),
            });
            line_num += 1;
        }
    }

    let faithfulness_delta =
        ((prompt_b.avg_faithfulness - prompt_a.avg_faithfulness) * 1000.0).round() / 1000.0;
    let latency_delta_ms =
        ((prompt_b.avg_latency_ms - prompt_a.avg_latency_ms) * 10.0).round() / 10.0;

    Ok(Json(PromptCompareResponse {
        prompt_a,
        prompt_b,
        shared_variables,
        added_variables,
        removed_variables,
        faithfulness_delta,
        latency_delta_ms,
        diff_lines,
    }))
}
