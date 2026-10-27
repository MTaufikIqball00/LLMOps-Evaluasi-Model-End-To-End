-- LLMOps Control Center Database Schema
-- Compatible with SQLite & PostgreSQL

CREATE TABLE IF NOT EXISTS models (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    framework TEXT NOT NULL,
    task TEXT NOT NULL,
    stage TEXT NOT NULL,
    version TEXT NOT NULL,
    parameters_count TEXT,
    artifact_uri TEXT,
    description TEXT,
    tags TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS experiments (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    model_id TEXT,
    framework TEXT NOT NULL,
    status TEXT NOT NULL,
    parameters TEXT NOT NULL,
    metrics TEXT NOT NULL,
    tags TEXT,
    duration_seconds INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS evaluations (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    model_name TEXT NOT NULL,
    dataset_name TEXT NOT NULL,
    evaluator_framework TEXT NOT NULL,
    status TEXT NOT NULL,
    faithfulness REAL NOT NULL DEFAULT 0.0,
    answer_relevance REAL NOT NULL DEFAULT 0.0,
    context_precision REAL NOT NULL DEFAULT 0.0,
    context_recall REAL NOT NULL DEFAULT 0.0,
    groundedness REAL NOT NULL DEFAULT 0.0,
    hallucination_score REAL NOT NULL DEFAULT 0.0,
    avg_latency_ms REAL NOT NULL DEFAULT 0.0,
    total_tokens INTEGER NOT NULL DEFAULT 0,
    test_cases_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS evaluation_test_cases (
    id TEXT PRIMARY KEY,
    evaluation_id TEXT NOT NULL,
    question TEXT NOT NULL,
    ground_truth TEXT,
    generated_answer TEXT NOT NULL,
    retrieved_context TEXT,
    faithfulness_score REAL NOT NULL DEFAULT 0.0,
    relevance_score REAL NOT NULL DEFAULT 0.0,
    groundedness_score REAL NOT NULL DEFAULT 0.0,
    hallucination_detected INTEGER NOT NULL DEFAULT 0,
    latency_ms REAL NOT NULL DEFAULT 0.0,
    tokens_used INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(evaluation_id) REFERENCES evaluations(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS request_traces (
    id TEXT PRIMARY KEY,
    trace_id TEXT NOT NULL,
    span_name TEXT NOT NULL,
    model_name TEXT NOT NULL,
    provider TEXT NOT NULL,
    status_code INTEGER NOT NULL,
    latency_ms REAL NOT NULL,
    prompt_tokens INTEGER NOT NULL,
    completion_tokens INTEGER NOT NULL,
    total_tokens INTEGER NOT NULL,
    estimated_cost REAL NOT NULL,
    redacted_prompt TEXT,
    redacted_completion TEXT,
    metadata TEXT,
    error_message TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS pricing_catalog (
    id TEXT PRIMARY KEY,
    provider TEXT NOT NULL,
    model_pattern TEXT NOT NULL,
    prompt_cost_per_1k REAL NOT NULL,
    completion_cost_per_1k REAL NOT NULL,
    last_updated TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS budget_settings (
    id TEXT PRIMARY KEY,
    monthly_budget_usd REAL NOT NULL,
    alert_threshold_pct REAL NOT NULL,
    current_spend_usd REAL NOT NULL,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS deployments (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    model_name TEXT NOT NULL,
    version TEXT NOT NULL,
    environment TEXT NOT NULL,
    cluster TEXT NOT NULL,
    replicas_desired INTEGER NOT NULL,
    replicas_ready INTEGER NOT NULL,
    status TEXT NOT NULL,
    image_tag TEXT NOT NULL,
    cpu_usage TEXT,
    memory_usage TEXT,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS cicd_workflows (
    id TEXT PRIMARY KEY,
    workflow_name TEXT NOT NULL,
    run_id TEXT NOT NULL,
    branch TEXT NOT NULL,
    commit_sha TEXT NOT NULL,
    commit_message TEXT NOT NULL,
    status TEXT NOT NULL,
    trigger_event TEXT NOT NULL,
    duration_seconds INTEGER NOT NULL,
    url TEXT,
    started_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS integrations_config (
    id TEXT PRIMARY KEY,
    service_name TEXT NOT NULL UNIQUE,
    endpoint_url TEXT,
    auth_type TEXT,
    auth_token_masked TEXT,
    is_enabled INTEGER NOT NULL DEFAULT 1,
    last_status TEXT NOT NULL,
    last_checked_at TIMESTAMP
);

-- Indices for performance
CREATE INDEX IF NOT EXISTS idx_models_stage ON models(stage);
CREATE INDEX IF NOT EXISTS idx_experiments_created ON experiments(created_at);
CREATE INDEX IF NOT EXISTS idx_evaluations_created ON evaluations(created_at);
CREATE INDEX IF NOT EXISTS idx_traces_created ON request_traces(created_at);
CREATE INDEX IF NOT EXISTS idx_traces_model ON request_traces(model_name);
CREATE INDEX IF NOT EXISTS idx_deployments_env ON deployments(environment);
