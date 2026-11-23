use crate::errors::AppError;
use regex::Regex;
use sqlx::{any::AnyPoolOptions, AnyPool};
use std::fs;
use std::path::Path;
use tracing::info;

pub type DbPool = AnyPool;

pub fn is_postgres_url(database_url: &str) -> bool {
    let trimmed = database_url.trim().to_ascii_lowercase();
    trimmed.starts_with("postgres://") || trimmed.starts_with("postgresql://")
}

/// Adapts SQL query placeholders (`?` -> `$1, $2, ...`) and `CURRENT_TIMESTAMP`
/// for PostgreSQL when `is_pg` is true, leaving SQLite queries untouched.
pub fn adapt_sql(sql: &str, is_pg: bool) -> String {
    if !is_pg {
        return sql.to_string();
    }

    let mut out = String::with_capacity(sql.len() + 32);
    let bytes = sql.as_bytes();
    let len = bytes.len();
    let mut i = 0usize;
    let mut param_idx = 0usize;
    let mut in_single_quote = false;
    let kw = b"CURRENT_TIMESTAMP";

    while i < len {
        let ch = bytes[i];
        if in_single_quote {
            out.push(ch as char);
            if ch == b'\'' {
                if i + 1 < len && bytes[i + 1] == b'\'' {
                    out.push('\'');
                    i += 2;
                    continue;
                }
                in_single_quote = false;
            }
            i += 1;
            continue;
        }

        if ch == b'\'' {
            in_single_quote = true;
            out.push('\'');
            i += 1;
            continue;
        }

        if ch == b'?' {
            param_idx += 1;
            out.push('$');
            out.push_str(&param_idx.to_string());
            i += 1;
            continue;
        }

        if i + kw.len() <= len && bytes[i..i + kw.len()].eq_ignore_ascii_case(kw) {
            let prev_ok = i == 0 || !(bytes[i - 1].is_ascii_alphanumeric() || bytes[i - 1] == b'_');
            let next_ok = i + kw.len() == len
                || !(bytes[i + kw.len()].is_ascii_alphanumeric() || bytes[i + kw.len()] == b'_');
            if prev_ok && next_ok {
                out.push_str(
                    "to_char(CURRENT_TIMESTAMP AT TIME ZONE 'UTC', 'YYYY-MM-DD HH24:MI:SS')",
                );
                i += kw.len();
                continue;
            }
        }

        // Preserve valid UTF-8 characters
        let s = &sql[i..];
        if let Some(c) = s.chars().next() {
            out.push(c);
            i += c.len_utf8();
        } else {
            i += 1;
        }
    }

    out
}

/// Converts SQLite-compatible DDL migrations into PostgreSQL DDL so text timestamps,
/// 64-bit integers, and 64-bit floats decode uniformly through `sqlx::AnyPool`.
pub fn adapt_migration_for_postgres(sql: &str) -> String {
    let ts_default_re =
        Regex::new(r"(?i)\b(?:TEXT|TIMESTAMP)\s+NOT\s+NULL\s+DEFAULT\s+CURRENT_TIMESTAMP\b")
            .unwrap();
    let ts_re = Regex::new(r"(?i)\bTIMESTAMP\b").unwrap();
    let real_re = Regex::new(r"(?i)\bREAL\b").unwrap();
    let int_re = Regex::new(r"(?i)\bINTEGER\b").unwrap();

    let s1 = ts_default_re.replace_all(
        sql,
        "TEXT NOT NULL DEFAULT to_char(CURRENT_TIMESTAMP AT TIME ZONE 'UTC', 'YYYY-MM-DD HH24:MI:SS')",
    );
    let s2 = ts_re.replace_all(&s1, "TEXT");
    let s3 = real_re.replace_all(&s2, "DOUBLE PRECISION");
    int_re.replace_all(&s3, "BIGINT").to_string()
}

/// Rebuilds pre-existing SQLite tables that declared columns as `TIMESTAMP` into
/// `TEXT` columns so `sqlx::AnyPool` (which rejects `SqliteTypeInfo(Datetime)`) can
/// decode them uniformly across fresh and pre-existing databases.
async fn migrate_legacy_sqlite_timestamps(pool: &DbPool) -> Result<(), AppError> {
    let tables: Vec<(String, String)> = sqlx::query_as(
        "SELECT name, sql FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' AND sql LIKE '%TIMESTAMP%'",
    )
    .fetch_all(pool)
    .await?;

    let ts_re = Regex::new(r"(?i)\bTIMESTAMP\b")
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Invalid timestamp regex: {e}")))?;

    let legacy_tables: Vec<(String, String)> = tables
        .into_iter()
        .filter(|(_, sql)| ts_re.is_match(sql))
        .collect();

    if legacy_tables.is_empty() {
        return Ok(());
    }

    info!(
        "Migrating {} legacy SQLite table(s) from TIMESTAMP to TEXT for AnyPool compatibility...",
        legacy_tables.len()
    );

    let mut conn = pool.acquire().await?;
    sqlx::query("PRAGMA foreign_keys = OFF;")
        .execute(&mut *conn)
        .await?;
    sqlx::query("BEGIN TRANSACTION;")
        .execute(&mut *conn)
        .await?;

    let migration_res: Result<(), sqlx::Error> = async {
        for (table_name, create_sql) in legacy_tables {
            let temp_name = format!("__mig_{table_name}");
            let safe_create_sql = ts_re.replace_all(&create_sql, "TEXT").to_string();
            let header_pattern = format!(
                r#"(?i)^\s*CREATE\s+TABLE\s+(?:IF\s+NOT\s+EXISTS\s+)?(?:["`\[]?{}["`\]]?)(?P<delim>[\s(])"#,
                regex::escape(&table_name)
            );
            let header_re = Regex::new(&header_pattern).map_err(|e| {
                sqlx::Error::Protocol(format!(
                    "Failed to compile DDL regex for {table_name}: {e}"
                ))
            })?;
            let temp_create_sql = header_re
                .replace(&safe_create_sql, format!("CREATE TABLE {temp_name}${{delim}}"))
                .to_string();

            sqlx::query(&temp_create_sql).execute(&mut *conn).await?;
            let copy_sql = format!("INSERT INTO {temp_name} SELECT * FROM {table_name}");
            sqlx::query(&copy_sql).execute(&mut *conn).await?;
            let drop_sql = format!("DROP TABLE {table_name}");
            sqlx::query(&drop_sql).execute(&mut *conn).await?;
            let rename_sql = format!("ALTER TABLE {temp_name} RENAME TO {table_name}");
            sqlx::query(&rename_sql).execute(&mut *conn).await?;
        }
        Ok(())
    }
    .await;

    match migration_res {
        Ok(()) => {
            sqlx::query("COMMIT;").execute(&mut *conn).await?;
            sqlx::query("PRAGMA foreign_keys = ON;")
                .execute(&mut *conn)
                .await?;
            Ok(())
        }
        Err(err) => {
            let _ = sqlx::query("ROLLBACK;").execute(&mut *conn).await;
            let _ = sqlx::query("PRAGMA foreign_keys = ON;")
                .execute(&mut *conn)
                .await;
            Err(AppError::Database(err))
        }
    }
}

pub async fn init_db(database_url: &str) -> Result<DbPool, AppError> {
    sqlx::any::install_default_drivers();
    let is_pg = is_postgres_url(database_url);

    // Ensure parent directory exists for file-based sqlite
    if let Some(file_path) = database_url.strip_prefix("sqlite://") {
        let clean_path = file_path.split('?').next().unwrap_or(file_path);
        if let Some(parent) = Path::new(clean_path).parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).map_err(|e| {
                    AppError::BadRequest(format!("Failed to create db directory: {e}"))
                })?;
            }
        }
    }

    let max_connections = if !is_pg && database_url.contains(":memory:") {
        1
    } else {
        10
    };

    let pool = AnyPoolOptions::new()
        .max_connections(max_connections)
        .after_connect(move |conn, _meta| {
            Box::pin(async move {
                if !is_pg {
                    sqlx::query("PRAGMA foreign_keys = ON;")
                        .execute(&mut *conn)
                        .await?;
                    sqlx::query("PRAGMA journal_mode = WAL;")
                        .execute(&mut *conn)
                        .await?;
                    sqlx::query("PRAGMA busy_timeout = 5000;")
                        .execute(&mut *conn)
                        .await?;
                }
                Ok(())
            })
        })
        .connect(database_url)
        .await?;

    if !is_pg {
        migrate_legacy_sqlite_timestamps(&pool).await?;
    }

    // Run migrations (001 & 002)
    let migrations = [
        include_str!("../../migrations/001_initial_schema.sql"),
        include_str!("../../migrations/002_prompts_and_alerts.sql"),
    ];

    for raw_sql in migrations {
        let migration_sql = if is_pg {
            adapt_migration_for_postgres(raw_sql)
        } else {
            raw_sql.to_string()
        };

        for statement in migration_sql.split(';') {
            let trimmed = statement.trim();
            if !trimmed.is_empty() {
                sqlx::query(trimmed).execute(&pool).await?;
            }
        }
    }

    // Seed baseline data if empty
    seed_if_empty(&pool, is_pg).await?;

    info!(
        "Database initialized and verified successfully ({}) at {}",
        if is_pg { "PostgreSQL" } else { "SQLite WAL" },
        if is_pg {
            "<redacted postgres uri>"
        } else {
            database_url
        }
    );
    Ok(pool)
}

async fn seed_if_empty(pool: &DbPool, is_pg: bool) -> Result<(), AppError> {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM models")
        .fetch_one(pool)
        .await?;

    if count.0 == 0 {
        info!("Seeding baseline operational data for LLMOps Control Center...");

        let statements = [
            // Seed models
            r#"
            INSERT INTO models (id, name, framework, task, stage, version, parameters_count, artifact_uri, description, tags, created_at, updated_at)
            VALUES 
            ('m-llama31-70b', 'Llama-3.1-70B-Instruct', 'vLLM / PyTorch', 'Text Generation / RAG', 'Production', 'v2.1.0', '70B', 's3://ml-artifacts/llama-3.1-70b-instruct/v2.1.0', 'Meta primary enterprise reasoning and customer support agent.', '["production", "reasoning", "fp8"]', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
            ('m-mistral-large', 'Mistral-Large-2', 'TGI / HF', 'Multi-turn Dialogue', 'Staging', 'v1.4.0', '123B', 's3://ml-artifacts/mistral-large-2/v1.4.0', 'High precision multilingual instruction-following model in staging candidate testing.', '["staging", "multilingual", "candidate"]', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
            ('m-qwen25-coder', 'Qwen-2.5-Coder-32B', 'vLLM', 'Code Generation', 'Development', 'v0.9.3', '32.5B', 's3://ml-artifacts/qwen-coder/v0.9.3', 'Fine-tuned developer code completion engine.', '["dev", "code", "lora"]', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
            ('m-deepseek-v2', 'DeepSeek-Coder-V2-Lite', 'vLLM', 'Code Generation', 'Archived', 'v1.0.0', '16B', 's3://ml-artifacts/deepseek-v2-lite/v1.0.0', 'Superseded by Qwen-2.5-Coder.', '["archived", "legacy"]', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
            "#,
            // Seed experiments
            r#"
            INSERT INTO experiments (id, name, model_id, framework, status, parameters, metrics, tags, duration_seconds, created_at)
            VALUES
            ('exp-lora-run-84', 'llama31-customer-support-lora-r16', 'm-llama31-70b', 'MLflow', 'Completed', 
             '{"learning_rate": 0.0002, "batch_size": 16, "lora_r": 16, "lora_alpha": 32, "max_seq_len": 4096, "optimizer": "adamw_8bit"}',
             '{"train_loss": 0.842, "eval_loss": 0.912, "perplexity": 2.48, "rouge_1": 0.642, "bleu": 42.8}',
             '["fine-tuning", "customer-support", "production-candidate"]', 4320, CURRENT_TIMESTAMP),
            ('exp-dpo-run-12', 'mistral-large-dpo-alignment-v3', 'm-mistral-large', 'Weights & Biases', 'Completed',
             '{"beta": 0.1, "learning_rate": 0.00005, "batch_size": 8, "loss_type": "sigmoid", "max_prompt_length": 2048}',
             '{"chosen_reward": 1.84, "rejected_reward": -0.62, "accuracy": 0.884, "margin": 2.46}',
             '["alignment", "dpo", "safety"]', 7200, CURRENT_TIMESTAMP),
            ('exp-code-qlora-05', 'qwen-coder-python-specialized', 'm-qwen25-coder', 'Native', 'Running',
             '{"learning_rate": 0.0001, "batch_size": 32, "lora_r": 32, "target_modules": "q_proj,v_proj,k_proj"}',
             '{"step": 1250, "loss": 1.140, "tokens_per_sec": 4820}',
             '["code", "python", "active-job"]', 2800, CURRENT_TIMESTAMP)
            "#,
            // Seed pricing catalog
            r#"
            INSERT INTO pricing_catalog (id, provider, model_pattern, prompt_cost_per_1k, completion_cost_per_1k, last_updated)
            VALUES
            ('p-gpt4o', 'OpenAI', 'gpt-4o', 0.0025, 0.0100, CURRENT_TIMESTAMP),
            ('p-gpt4o-mini', 'OpenAI', 'gpt-4o-mini', 0.00015, 0.0006, CURRENT_TIMESTAMP),
            ('p-claude-sonnet', 'Anthropic', 'claude-3-5-sonnet', 0.0030, 0.0150, CURRENT_TIMESTAMP),
            ('p-llama-70b', 'Meta / Self-Hosted', 'Llama-3.1-70B-Instruct', 0.0005, 0.0009, CURRENT_TIMESTAMP),
            ('p-gemini-pro', 'Google', 'gemini-1.5-pro', 0.00125, 0.0050, CURRENT_TIMESTAMP),
            ('p-mistral-large', 'Mistral AI', 'mistral-large-2', 0.0020, 0.0060, CURRENT_TIMESTAMP)
            "#,
            // Seed budget settings
            r#"
            INSERT INTO budget_settings (id, monthly_budget_usd, alert_threshold_pct, current_spend_usd, updated_at)
            VALUES ('default-budget', 2500.0, 80.0, 1428.50, CURRENT_TIMESTAMP)
            "#,
            // Seed evaluations
            r#"
            INSERT INTO evaluations (id, name, model_name, dataset_name, evaluator_framework, status, faithfulness, answer_relevance, context_precision, context_recall, groundedness, hallucination_score, avg_latency_ms, total_tokens, test_cases_count, created_at)
            VALUES
            ('eval-golden-rag-suite-01', 'Q3 Enterprise Policy RAG Benchmark', 'Llama-3.1-70B-Instruct', 'enterprise-kb-v3-eval', 'Ragas & TruLens Suite', 'Completed', 0.942, 0.918, 0.885, 0.902, 0.954, 0.068, 312.4, 48200, 25, CURRENT_TIMESTAMP),
            ('eval-mistral-safety-02', 'Multilingual Financial Compliance Audit', 'Mistral-Large-2', 'fin-compliance-100', 'TruLens Triad', 'Completed', 0.961, 0.892, 0.920, 0.884, 0.948, 0.054, 420.8, 62100, 30, CURRENT_TIMESTAMP),
            ('eval-code-test-suite-03', 'HumanEval + Python Enterprise Libs', 'Qwen-2.5-Coder-32B', 'humaneval-enterprise', 'Native Suite', 'Completed', 0.889, 0.945, 0.871, 0.865, 0.912, 0.107, 198.5, 31400, 40, CURRENT_TIMESTAMP)
            "#,
            // Seed evaluation test cases
            r#"
            INSERT INTO evaluation_test_cases (id, evaluation_id, question, ground_truth, generated_answer, retrieved_context, faithfulness_score, relevance_score, groundedness_score, hallucination_detected, latency_ms, tokens_used, created_at)
            VALUES
            ('tc-01', 'eval-golden-rag-suite-01', 'What is the maximum reimbursement for remote workstation ergonomics per fiscal year?', 'Employees may claim up to $750 per fiscal year with receipts submitted within 30 days.', 'Under the remote work policy section 4.2, employees are eligible for up to $750 per fiscal year for ergonomic workstation accessories.', 'Section 4.2 Workstation Equipment: Eligible employees may claim up to $750 per fiscal year. Invoices must be submitted within 30 days of purchase.', 0.98, 0.96, 0.99, 0, 245.0, 340, CURRENT_TIMESTAMP),
            ('tc-02', 'eval-golden-rag-suite-01', 'Can temporary contractors access SOC-2 Type II customer audit logs directly?', 'No, external contractors require VP of Security written exemption and temporary jump-host bastions.', 'Contractors are strictly prohibited from direct access to SOC-2 logs unless an approved VP exception ticket is filed.', 'Security Policy Rule 18: No third-party or contractor may directly query customer data or SOC-2 evidence without an explicit VP Security exemption.', 0.95, 0.94, 0.96, 0, 310.0, 412, CURRENT_TIMESTAMP),
            ('tc-03', 'eval-golden-rag-suite-01', 'What is the data retention window for unindexed vector embeddings in the staging cluster?', 'Raw chunk embeddings are retained for 90 days before cold storage archiving.', 'Vector embeddings are deleted immediately after 14 days without archiving.', 'Data Governance Chapter 9: Embedding vectors in staging environments are retained for 90 days before automated lifecycle migration to cold S3 tier.', 0.42, 0.65, 0.38, 1, 290.0, 380, CURRENT_TIMESTAMP)
            "#,
            // Seed request traces
            r#"
            INSERT INTO request_traces (id, trace_id, span_name, model_name, provider, status_code, latency_ms, prompt_tokens, completion_tokens, total_tokens, estimated_cost, redacted_prompt, redacted_completion, metadata, error_message, created_at)
            VALUES
            ('tr-8821a', 'tr-20261027-001', 'rag_orchestrator::execute', 'Llama-3.1-70B-Instruct', 'vllm_local', 200, 268.4, 842, 185, 1027, 0.00059, 'User query: Summarize security policy for [REDACTED_SECRET] rotation interval.', 'According to Policy 9.1, service credentials and API keys must be rotated every 90 days.', '{"env": "production", "user_tier": "enterprise", "cache_hit": false}', NULL, CURRENT_TIMESTAMP),
            ('tr-8822b', 'tr-20261027-002', 'agent_chat::stream', 'Llama-3.1-70B-Instruct', 'vllm_local', 200, 312.1, 1250, 320, 1570, 0.00091, 'Synthesize incident response timeline for ticket #49122 on database failover.', 'Primary database failover initiated at 04:12 UTC. Secondary promoted in 42 seconds.', '{"env": "production", "user_tier": "internal_sre"}', NULL, CURRENT_TIMESTAMP),
            ('tr-8823c', 'tr-20261027-003', 'code_assistant::complete', 'Qwen-2.5-Coder-32B', 'vllm_local', 200, 145.8, 620, 140, 760, 0.00052, 'Generate rust axum handler with timeout middleware and sqlx query.', 'pub async fn handler(State(pool): State<Pool>) -> Result<Json<Data>, AppError> { ... }', '{"env": "development", "ide": "vscode"}', NULL, CURRENT_TIMESTAMP),
            ('tr-8824d', 'tr-20261027-004', 'doc_analyzer::extract', 'Mistral-Large-2', 'tgi_cluster', 504, 3000.0, 3400, 0, 3400, 0.00680, 'Extract tables from 80-page financial disclosure document.', NULL, '{"env": "staging"}', 'Gateway Timeout: vLLM inference node exceeded 3000ms threshold', CURRENT_TIMESTAMP)
            "#,
            // Seed deployments
            r#"
            INSERT INTO deployments (id, name, model_name, version, environment, cluster, replicas_desired, replicas_ready, status, image_tag, cpu_usage, memory_usage, updated_at)
            VALUES
            ('dep-llama-prod', 'llama-3.1-70b-vllm-serving', 'Llama-3.1-70B-Instruct', 'v2.1.0', 'Production', 'k8s-prod-us-east-1', 4, 4, 'Healthy', 'vllm/vllm-openai:v0.6.2-cuda12.4', '62%', '48.2 GB / 64 GB', CURRENT_TIMESTAMP),
            ('dep-mistral-stage', 'mistral-large-candidate-serving', 'Mistral-Large-2', 'v1.4.0', 'Staging', 'k8s-staging-us-east-1', 2, 2, 'Healthy', 'ghcr.io/huggingface/text-generation-inference:2.4.0', '35%', '38.4 GB / 48 GB', CURRENT_TIMESTAMP),
            ('dep-qwen-dev', 'qwen-coder-dev-instance', 'Qwen-2.5-Coder-32B', 'v0.9.3', 'Development', 'k8s-dev-cluster', 1, 1, 'Healthy', 'vllm/vllm-openai:v0.6.2', '18%', '16.1 GB / 24 GB', CURRENT_TIMESTAMP)
            "#,
            // Seed CI/CD workflows
            r#"
            INSERT INTO cicd_workflows (id, workflow_name, run_id, branch, commit_sha, commit_message, status, trigger_event, duration_seconds, url, started_at)
            VALUES
            ('run-1092', 'Model Evaluation & Benchmark Gates', 'gh-9281921', 'main', '8b4f12a', 'feat: update prompt template for RAG synthesis', 'Success', 'push', 342, 'https://github.com/enterprise/llmops-platform/actions/runs/9281921', CURRENT_TIMESTAMP),
            ('run-1091', 'vLLM Serving Container Build & Security Scan', 'gh-9281890', 'release/v2.1.0', '3c19e88', 'chore: bump base CUDA image to 12.4.1', 'Success', 'pull_request', 512, 'https://github.com/enterprise/llmops-platform/actions/runs/9281890', CURRENT_TIMESTAMP),
            ('run-1090', 'Canary Rollout to Staging Kubernetes', 'gh-9281745', 'staging', '1e4a902', 'deploy: rollout mistral-large candidate v1.4.0', 'Success', 'workflow_dispatch', 184, 'https://github.com/enterprise/llmops-platform/actions/runs/9281745', CURRENT_TIMESTAMP)
            "#,
            // Seed integrations configuration
            r#"
            INSERT INTO integrations_config (id, service_name, endpoint_url, auth_type, auth_token_masked, is_enabled, last_status, last_checked_at)
            VALUES
            ('int-mlflow', 'mlflow', 'http://localhost:5500', 'Bearer', 'mlf_***d8a', 1, 'Configured (Local / Mock Fallback)', CURRENT_TIMESTAMP),
            ('int-wandb', 'wandb', 'https://api.wandb.ai', 'API Key', 'wandb_***4e9', 1, 'Configured (API Ready)', CURRENT_TIMESTAMP),
            ('int-prometheus', 'prometheus', 'http://localhost:9190', 'None', NULL, 1, 'Configured (Scrape Ready)', CURRENT_TIMESTAMP),
            ('int-grafana', 'grafana', 'http://localhost:3310', 'Bearer', 'glsa_***88f', 1, 'Configured (Dashboards Linked)', CURRENT_TIMESTAMP),
            ('int-ragas', 'ragas', 'http://localhost:8210/api/v1/evaluations/ragas', 'Native Rust + HTTP', NULL, 1, 'Connected (Native Suite Active)', CURRENT_TIMESTAMP),
            ('int-trulens', 'trulens', 'http://localhost:8210/api/v1/evaluations/trulens', 'Native Triad + HTTP', NULL, 1, 'Connected (Native Triad Active)', CURRENT_TIMESTAMP),
            ('int-k8s', 'kubernetes', 'https://kubernetes.default.svc', 'ServiceAccount', 'eyJhb***k8s', 1, 'Connected (Local / Cluster Mode)', CURRENT_TIMESTAMP),
            ('int-gh', 'github_actions', 'https://api.github.com/repos/enterprise/llmops-platform', 'Token', 'ghp_***9a2', 1, 'Configured (API Ready)', CURRENT_TIMESTAMP)
            "#,
        ];

        for stmt in statements {
            let adapted = adapt_sql(stmt, is_pg);
            sqlx::query(&adapted).execute(pool).await?;
        }

        info!("Baseline data successfully seeded into database.");
    }

    // Seed Prompt Registry templates independently for smooth migration upgrades
    let prompt_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM prompts")
        .fetch_one(pool)
        .await?;
    if prompt_count.0 == 0 {
        let prompt_sql = adapt_sql(
            r#"
            INSERT INTO prompts (id, name, slug, version, template, variables, model_target, stage, description, tags, avg_faithfulness, avg_latency_ms, usage_count, created_at, updated_at)
            VALUES
            ('prm-rag-v1', 'Enterprise RAG Policy Synthesizer', 'rag-policy-synthesizer', 'v1.0.0',
             'You are an enterprise policy assistant.\nUse the context below to answer the question concisely.\nContext:\n{{context}}\n\nQuestion: {{question}}\nAnswer:',
             '["context", "question"]', 'Llama-3.1-70B-Instruct', 'Staging',
             'Baseline RAG system prompt before strict citation grounding rules.',
             '["rag", "policy", "baseline"]', 0.892, 295.0, 1420, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
            ('prm-rag-v2', 'Enterprise RAG Policy Synthesizer (Strict Grounding)', 'rag-policy-synthesizer', 'v2.0.0',
             'You are an enterprise compliance assistant.\nSTRICT RULE: Rely ONLY on verified facts inside <context>. If unsure, say "Insufficient evidence".\nCite section headers inline.\n<context>\n{{context}}\n</context>\n\nQuestion: {{question}}\nGrounded Answer:',
             '["context", "question"]', 'Llama-3.1-70B-Instruct', 'Production',
             'Hardened RAG prompt with XML boundaries and strict ungrounded-claim refusal.',
             '["rag", "policy", "production", "grounded"]', 0.964, 268.4, 4890, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
            ('prm-code-v1', 'Rust Axum Security Code Reviewer', 'code-reviewer-rust', 'v1.0.0',
             'Review the following Rust handler for SQL injection, unredacted secrets, and missing timeouts.\nCode:\n{{code_diff}}\nProvide structured findings with severity:',
             '["code_diff"]', 'Qwen-2.5-Coder-32B', 'Production',
             'Automated PR security review prompt for Rust microservices.',
             '["security", "rust", "cicd"]', 0.931, 184.2, 615, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
            "#,
            is_pg,
        );
        sqlx::query(&prompt_sql).execute(pool).await?;
    }

    // Seed Alerting Rules independently for smooth migration upgrades
    let alert_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM alert_rules")
        .fetch_one(pool)
        .await?;
    if alert_count.0 == 0 {
        let alert_sql = adapt_sql(
            r#"
            INSERT INTO alert_rules (id, name, metric_type, operator, threshold, severity, webhook_url, channel_type, is_enabled, last_triggered_at, created_at)
            VALUES
            ('rule-hallucination-guard', 'Hallucination Index SLA Guard', 'hallucination_score', 'gt', 0.35, 'Critical', NULL, 'slack', 1, NULL, CURRENT_TIMESTAMP),
            ('rule-latency-p95', 'Inference p95 Latency SLA', 'latency_p95_ms', 'gt', 500.0, 'High', NULL, 'webhook', 1, NULL, CURRENT_TIMESTAMP),
            ('rule-budget-threshold', 'Monthly Token Budget Cap Guard', 'budget_used_pct', 'gte', 80.0, 'Medium', NULL, 'discord', 1, NULL, CURRENT_TIMESTAMP)
            "#,
            is_pg,
        );
        sqlx::query(&alert_sql).execute(pool).await?;
    }

    Ok(())
}
