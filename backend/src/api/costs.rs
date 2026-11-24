use crate::api::health::AppState;
use crate::domain::{BudgetSetting, PricingEntry, UpdateBudgetRequest};
use crate::errors::AppError;
use crate::persistence::adapt_sql;
use axum::{
    extract::{Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::UNIX_EPOCH;

#[derive(Serialize)]
pub struct CostSummaryReport {
    pub budget: BudgetSetting,
    pub is_budget_alert_active: bool,
    pub provider_breakdown: Vec<ProviderCostItem>,
    pub model_breakdown: Vec<ModelCostItem>,
    pub monthly_spend_projection_usd: f64,
    pub pricing_source: String,
}

#[derive(Serialize)]
pub struct ProviderCostItem {
    pub provider: String,
    pub total_tokens: i64,
    pub estimated_cost_usd: f64,
    pub percentage: f64,
}

#[derive(Serialize)]
pub struct ModelCostItem {
    pub model_name: String,
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
    pub total_tokens: i64,
    pub cost_usd: f64,
}

pub async fn get_cost_summary(
    State(state): State<Arc<AppState>>,
) -> Result<Json<CostSummaryReport>, AppError> {
    let budget: BudgetSetting = sqlx::query_as("SELECT * FROM budget_settings LIMIT 1")
        .fetch_one(&state.pool)
        .await
        .unwrap_or(BudgetSetting {
            id: "default-budget".to_string(),
            monthly_budget_usd: 2500.0,
            alert_threshold_pct: 80.0,
            current_spend_usd: 0.0,
            updated_at: chrono::Utc::now().to_rfc3339(),
        });

    let alert_threshold_amount = budget.monthly_budget_usd * (budget.alert_threshold_pct / 100.0);
    let is_alert_active = budget.current_spend_usd >= alert_threshold_amount;

    let provider_rows: Vec<(String, i64, f64)> = sqlx::query_as(
        r#"
        SELECT 
            provider,
            COALESCE(SUM(total_tokens), 0) as tokens,
            COALESCE(SUM(estimated_cost), 0.0) as cost
        FROM request_traces
        GROUP BY provider
        ORDER BY cost DESC
        "#,
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let total_cost: f64 = provider_rows.iter().map(|r| r.2).sum();

    let provider_breakdown: Vec<ProviderCostItem> = provider_rows
        .into_iter()
        .map(|(prov, tokens, cost)| {
            let pct = if total_cost > 0.0 {
                (cost / total_cost) * 100.0
            } else {
                0.0
            };
            ProviderCostItem {
                provider: prov,
                total_tokens: tokens,
                estimated_cost_usd: (cost * 10000.0).round() / 10000.0,
                percentage: (pct * 10.0).round() / 10.0,
            }
        })
        .collect();

    let model_rows: Vec<(String, i64, i64, i64, f64)> = sqlx::query_as(
        r#"
        SELECT 
            model_name,
            COALESCE(SUM(prompt_tokens), 0),
            COALESCE(SUM(completion_tokens), 0),
            COALESCE(SUM(total_tokens), 0),
            COALESCE(SUM(estimated_cost), 0.0)
        FROM request_traces
        GROUP BY model_name
        ORDER BY 5 DESC
        "#,
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let model_breakdown: Vec<ModelCostItem> = model_rows
        .into_iter()
        .map(|(name, p_tok, c_tok, tot_tok, cost)| ModelCostItem {
            model_name: name,
            prompt_tokens: p_tok,
            completion_tokens: c_tok,
            total_tokens: tot_tok,
            cost_usd: (cost * 10000.0).round() / 10000.0,
        })
        .collect();

    let projection = budget.current_spend_usd * 1.35;

    Ok(Json(CostSummaryReport {
        budget,
        is_budget_alert_active: is_alert_active,
        provider_breakdown,
        model_breakdown,
        monthly_spend_projection_usd: (projection * 100.0).round() / 100.0,
        pricing_source: "Configured Standard Pricing Catalog (Active)".to_string(),
    }))
}

pub async fn list_pricing_catalog(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<PricingEntry>>, AppError> {
    let list: Vec<PricingEntry> =
        sqlx::query_as("SELECT * FROM pricing_catalog ORDER BY provider ASC")
            .fetch_all(&state.pool)
            .await?;
    Ok(Json(list))
}

pub async fn update_budget(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<UpdateBudgetRequest>,
) -> Result<Json<BudgetSetting>, AppError> {
    if payload.monthly_budget_usd <= 0.0 {
        return Err(AppError::BadRequest(
            "Monthly budget must be positive".to_string(),
        ));
    }
    if payload.alert_threshold_pct <= 0.0 || payload.alert_threshold_pct > 100.0 {
        return Err(AppError::BadRequest(
            "Alert threshold must be between 1% and 100%".to_string(),
        ));
    }

    let update_sql = adapt_sql(
        r#"
        UPDATE budget_settings
        SET monthly_budget_usd = ?, alert_threshold_pct = ?, updated_at = CURRENT_TIMESTAMP
        WHERE id = 'default-budget'
        "#,
        state.is_postgres,
    );

    sqlx::query(&update_sql)
        .bind(payload.monthly_budget_usd)
        .bind(payload.alert_threshold_pct)
        .execute(&state.pool)
        .await?;

    let updated: BudgetSetting = sqlx::query_as("SELECT * FROM budget_settings LIMIT 1")
        .fetch_one(&state.pool)
        .await?;

    Ok(Json(updated))
}

#[derive(Debug, Deserialize)]
pub struct ComparisonQuery {
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TokenCohortStats {
    pub label: String,
    pub sessions_count: i64,
    pub turns_count: i64,
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
    pub cache_read_tokens: i64,
    pub total_tokens: i64,
    pub avg_tokens_per_turn: f64,
    pub avg_prompt_tokens_per_turn: f64,
    pub avg_output_tokens_per_turn: f64,
    pub estimated_cost_usd: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct TokenComparisonReport {
    pub without_program: TokenCohortStats,
    pub with_program: TokenCohortStats,
    pub total_token_delta_pct: Option<f64>,
    pub output_token_delta_pct: Option<f64>,
    pub avg_tokens_saved_per_turn: f64,
    pub avg_output_tokens_saved_per_turn: f64,
    pub projected_tokens_saved: i64,
    pub projected_cost_saved_usd: f64,
    pub cutoff_timestamp: Option<String>,
    pub data_source: String,
    pub comparison_note: String,
}

#[derive(Default)]
struct CohortAccumulator {
    sessions_count: i64,
    turns_count: i64,
    prompt_tokens: i64,
    completion_tokens: i64,
    cache_read_tokens: i64,
    estimated_cost_usd: f64,
}

impl CohortAccumulator {
    fn into_stats(
        self,
        label: &str,
        prompt_rate_per_1k: f64,
        completion_rate_per_1k: f64,
        use_explicit_cost: bool,
    ) -> TokenCohortStats {
        let total_tokens = self.prompt_tokens + self.completion_tokens;
        let turns = self.turns_count.max(1) as f64;
        let avg_tokens_per_turn = if self.turns_count > 0 {
            ((total_tokens as f64 / turns) * 10.0).round() / 10.0
        } else {
            0.0
        };
        let avg_prompt_tokens_per_turn = if self.turns_count > 0 {
            ((self.prompt_tokens as f64 / turns) * 10.0).round() / 10.0
        } else {
            0.0
        };
        let avg_output_tokens_per_turn = if self.turns_count > 0 {
            ((self.completion_tokens as f64 / turns) * 10.0).round() / 10.0
        } else {
            0.0
        };
        let cost = if use_explicit_cost {
            self.estimated_cost_usd
        } else {
            (self.prompt_tokens as f64 / 1000.0) * prompt_rate_per_1k
                + (self.completion_tokens as f64 / 1000.0) * completion_rate_per_1k
        };

        TokenCohortStats {
            label: label.to_string(),
            sessions_count: self.sessions_count,
            turns_count: self.turns_count,
            prompt_tokens: self.prompt_tokens,
            completion_tokens: self.completion_tokens,
            cache_read_tokens: self.cache_read_tokens,
            total_tokens,
            avg_tokens_per_turn,
            avg_prompt_tokens_per_turn,
            avg_output_tokens_per_turn,
            estimated_cost_usd: (cost * 10000.0).round() / 10000.0,
        }
    }
}

fn build_comparison_report(
    without_program: TokenCohortStats,
    with_program: TokenCohortStats,
    cutoff_timestamp: Option<String>,
    data_source: String,
    prompt_rate_per_1k: f64,
    completion_rate_per_1k: f64,
) -> TokenComparisonReport {
    let total_token_delta_pct = if without_program.turns_count > 0
        && with_program.turns_count > 0
        && without_program.avg_tokens_per_turn > 0.0
    {
        let pct = ((without_program.avg_tokens_per_turn - with_program.avg_tokens_per_turn)
            / without_program.avg_tokens_per_turn)
            * 100.0;
        Some((pct * 10.0).round() / 10.0)
    } else {
        None
    };

    let output_token_delta_pct = if without_program.turns_count > 0
        && with_program.turns_count > 0
        && without_program.avg_output_tokens_per_turn > 0.0
    {
        let pct = ((without_program.avg_output_tokens_per_turn
            - with_program.avg_output_tokens_per_turn)
            / without_program.avg_output_tokens_per_turn)
            * 100.0;
        Some((pct * 10.0).round() / 10.0)
    } else {
        None
    };

    let avg_tokens_saved_per_turn = if without_program.turns_count > 0
        && with_program.turns_count > 0
    {
        ((without_program.avg_tokens_per_turn - with_program.avg_tokens_per_turn) * 10.0).round()
            / 10.0
    } else {
        0.0
    };

    let avg_output_tokens_saved_per_turn =
        if without_program.turns_count > 0 && with_program.turns_count > 0 {
            ((without_program.avg_output_tokens_per_turn - with_program.avg_output_tokens_per_turn)
                * 10.0)
                .round()
                / 10.0
        } else {
            0.0
        };

    let projected_tokens_saved =
        (avg_tokens_saved_per_turn * with_program.turns_count as f64).round() as i64;
    let prompt_saved = (without_program.avg_prompt_tokens_per_turn
        - with_program.avg_prompt_tokens_per_turn)
        * with_program.turns_count as f64;
    let output_saved = avg_output_tokens_saved_per_turn * with_program.turns_count as f64;
    let projected_cost_saved_usd =
        if without_program.turns_count > 0 && with_program.turns_count > 0 {
            let raw = (prompt_saved / 1000.0) * prompt_rate_per_1k
                + (output_saved / 1000.0) * completion_rate_per_1k;
            (raw * 10000.0).round() / 10000.0
        } else {
            0.0
        };

    let comparison_note = if with_program.turns_count == 0 {
        "Belum ada giliran percakapan setelah LLMOps Control Guard aktif. Mulai sesi chat baru di Antigravity agar perbandingan otomatis terisi.".to_string()
    } else if with_program.turns_count < 10 {
        format!(
            "Baru tercatat {} giliran setelah guard aktif ({} sesi). Rata-rata output token langsung terlihat, sedangkan rata-rata input token akan semakin akurat setelah beberapa sesi chat baru dimulai.",
            with_program.turns_count, with_program.sessions_count
        )
    } else {
        format!(
            "Perbandingan dihitung dari {} giliran baseline vs {} giliran dengan LLMOps Control Guard aktif.",
            without_program.turns_count, with_program.turns_count
        )
    };

    TokenComparisonReport {
        without_program,
        with_program,
        total_token_delta_pct,
        output_token_delta_pct,
        avg_tokens_saved_per_turn,
        avg_output_tokens_saved_per_turn,
        projected_tokens_saved,
        projected_cost_saved_usd,
        cutoff_timestamp,
        data_source,
        comparison_note,
    }
}

fn resolve_home_dir() -> Option<PathBuf> {
    std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .ok()
        .filter(|s| !s.trim().is_empty())
        .map(PathBuf::from)
}

fn resolve_antigravity_brain_dir() -> Option<PathBuf> {
    if let Ok(custom) = std::env::var("ANTIGRAVITY_BRAIN_DIR") {
        let trimmed = custom.trim();
        if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("none") {
            return None;
        }
        let path = PathBuf::from(trimmed);
        return path.exists().then_some(path);
    }

    let default_path = resolve_home_dir()?
        .join(".gemini")
        .join("antigravity")
        .join("brain");
    default_path.exists().then_some(default_path)
}

fn resolve_guard_cutoff_ms() -> Option<i64> {
    if let Ok(ts) = std::env::var("LLMOPS_GUARD_INSTALLED_AT") {
        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(ts.trim()) {
            return Some(dt.timestamp_millis());
        }
    }

    let plugin_path = if let Ok(custom) = std::env::var("ANTIGRAVITY_GUARD_PLUGIN_PATH") {
        PathBuf::from(custom)
    } else {
        resolve_home_dir()?
            .join(".gemini")
            .join("config")
            .join("plugins")
            .join("llmops-control-guard")
            .join("plugin.json")
    };

    let modified = fs::metadata(plugin_path).ok()?.modified().ok()?;
    let dur = modified.duration_since(UNIX_EPOCH).ok()?;
    Some(dur.as_millis() as i64)
}

/// Scans Antigravity `transcript.jsonl` files under `brain_dir` and splits real turns
/// into baseline (`< cutoff_epoch_ms`) vs guarded (`>= cutoff_epoch_ms`) cohorts.
pub fn compute_transcript_comparison(
    brain_dir: &Path,
    cutoff_epoch_ms: i64,
    prompt_rate_per_1k: f64,
    completion_rate_per_1k: f64,
) -> Option<TokenComparisonReport> {
    let entries = fs::read_dir(brain_dir).ok()?;
    let mut before = CohortAccumulator::default();
    let mut after = CohortAccumulator::default();

    for entry in entries.flatten() {
        let transcript_path = entry
            .path()
            .join(".system_generated")
            .join("logs")
            .join("transcript.jsonl");
        if !transcript_path.is_file() {
            continue;
        }
        let Ok(file) = fs::File::open(&transcript_path) else {
            continue;
        };

        let mut session_had_before = false;
        let mut session_had_after = false;

        let mut cur_ts: Option<i64> = None;
        let mut cur_in: i64 = 0;
        let mut cur_out: i64 = 0;
        let mut cur_cache: i64 = 0;
        let mut cur_has_resp = false;

        let flush_turn = |ts_opt: Option<i64>,
                          in_tok: i64,
                          out_tok: i64,
                          cache_tok: i64,
                          has_resp: bool,
                          before: &mut CohortAccumulator,
                          after: &mut CohortAccumulator,
                          had_before: &mut bool,
                          had_after: &mut bool| {
            if !has_resp {
                return;
            }
            let turn_ts = ts_opt.unwrap_or(0);
            if turn_ts < cutoff_epoch_ms {
                before.turns_count += 1;
                before.prompt_tokens += in_tok;
                before.completion_tokens += out_tok;
                before.cache_read_tokens += cache_tok;
                *had_before = true;
            } else {
                after.turns_count += 1;
                after.prompt_tokens += in_tok;
                after.completion_tokens += out_tok;
                after.cache_read_tokens += cache_tok;
                *had_after = true;
            }
        };

        let reader = BufReader::new(file);
        for line_res in reader.lines() {
            let Ok(raw_line) = line_res else {
                continue;
            };
            let line = raw_line.trim_start_matches('\u{feff}').trim();
            if line.is_empty() {
                continue;
            }
            let Ok(val) = serde_json::from_str::<serde_json::Value>(line) else {
                continue;
            };
            let step_type = val.get("type").and_then(|v| v.as_str()).unwrap_or("");
            let step_ts = val
                .get("created_at")
                .and_then(|v| v.as_str())
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.timestamp_millis());

            if step_type == "USER_INPUT" {
                flush_turn(
                    cur_ts,
                    cur_in,
                    cur_out,
                    cur_cache,
                    cur_has_resp,
                    &mut before,
                    &mut after,
                    &mut session_had_before,
                    &mut session_had_after,
                );
                cur_ts = step_ts;
                cur_in = 0;
                cur_out = 0;
                cur_cache = 0;
                cur_has_resp = false;
            } else if step_type == "PLANNER_RESPONSE" {
                if cur_ts.is_none() {
                    cur_ts = step_ts;
                }
                cur_in += val
                    .get("input_tokens")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
                cur_out += val
                    .get("output_tokens")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
                cur_cache += val
                    .get("cache_read_tokens")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
                cur_has_resp = true;
            }
        }

        flush_turn(
            cur_ts,
            cur_in,
            cur_out,
            cur_cache,
            cur_has_resp,
            &mut before,
            &mut after,
            &mut session_had_before,
            &mut session_had_after,
        );

        if session_had_before {
            before.sessions_count += 1;
        }
        if session_had_after {
            after.sessions_count += 1;
        }
    }

    if before.turns_count == 0 && after.turns_count == 0 {
        return None;
    }

    let cutoff_iso =
        chrono::DateTime::from_timestamp_millis(cutoff_epoch_ms).map(|dt| dt.to_rfc3339());

    Some(build_comparison_report(
        before.into_stats(
            "Tanpa Program (Baseline)",
            prompt_rate_per_1k,
            completion_rate_per_1k,
            false,
        ),
        after.into_stats(
            "Dengan LLMOps Control Guard",
            prompt_rate_per_1k,
            completion_rate_per_1k,
            false,
        ),
        cutoff_iso,
        "Antigravity Local Transcripts (Live Baseline vs Guard)".to_string(),
        prompt_rate_per_1k,
        completion_rate_per_1k,
    ))
}

fn is_guarded_trace_metadata(metadata: Option<&str>) -> bool {
    let Some(raw) = metadata else {
        return false;
    };
    if let Ok(val) = serde_json::from_str::<serde_json::Value>(raw) {
        if val
            .get("llmops_guard")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            return true;
        }
        if let Some(mode) = val.get("guard_mode").and_then(|v| v.as_str()) {
            if mode.eq_ignore_ascii_case("mode1") || mode.eq_ignore_ascii_case("mode2") {
                return true;
            }
        }
        if val
            .get("source")
            .and_then(|v| v.as_str())
            .is_some_and(|s| s == "antigravity_hook")
        {
            return true;
        }
    }
    false
}

pub async fn get_token_comparison(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ComparisonQuery>,
) -> Result<Json<TokenComparisonReport>, AppError> {
    let source_pref = query
        .source
        .unwrap_or_else(|| "auto".to_string())
        .to_ascii_lowercase();

    // Default Gemini-tier rates ($0.00125/1k prompt, $0.005/1k completion)
    let default_prompt_rate = 0.00125;
    let default_completion_rate = 0.005;

    if source_pref != "database" {
        if let (Some(brain_dir), Some(cutoff_ms)) =
            (resolve_antigravity_brain_dir(), resolve_guard_cutoff_ms())
        {
            if let Some(report) = compute_transcript_comparison(
                &brain_dir,
                cutoff_ms,
                default_prompt_rate,
                default_completion_rate,
            ) {
                return Ok(Json(report));
            }
        }
    }

    // Fallback or explicit database cohort comparison from `request_traces`
    let rows: Vec<(i64, i64, f64, Option<String>)> = sqlx::query_as(
        "SELECT prompt_tokens, completion_tokens, estimated_cost, metadata FROM request_traces",
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let mut before = CohortAccumulator::default();
    let mut after = CohortAccumulator::default();

    for (prompt_tok, comp_tok, est_cost, meta) in rows {
        let cache_tok = meta
            .as_deref()
            .and_then(|m| serde_json::from_str::<serde_json::Value>(m).ok())
            .and_then(|v| v.get("cache_read_tokens").and_then(|c| c.as_i64()))
            .unwrap_or(0);

        if is_guarded_trace_metadata(meta.as_deref()) {
            after.turns_count += 1;
            after.prompt_tokens += prompt_tok;
            after.completion_tokens += comp_tok;
            after.cache_read_tokens += cache_tok;
            after.estimated_cost_usd += est_cost;
        } else {
            before.turns_count += 1;
            before.prompt_tokens += prompt_tok;
            before.completion_tokens += comp_tok;
            before.cache_read_tokens += cache_tok;
            before.estimated_cost_usd += est_cost;
        }
    }

    if before.turns_count > 0 {
        before.sessions_count = 1;
    }
    if after.turns_count > 0 {
        after.sessions_count = 1;
    }

    let report = build_comparison_report(
        before.into_stats(
            "Tanpa Program (Baseline Traces)",
            default_prompt_rate,
            default_completion_rate,
            true,
        ),
        after.into_stats(
            "Dengan LLMOps Control Guard",
            default_prompt_rate,
            default_completion_rate,
            true,
        ),
        None,
        "Database Request Traces Cohort (SQLite/PostgreSQL)".to_string(),
        default_prompt_rate,
        default_completion_rate,
    );

    Ok(Json(report))
}
