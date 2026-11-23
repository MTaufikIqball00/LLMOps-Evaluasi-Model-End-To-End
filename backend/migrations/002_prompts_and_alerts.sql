-- LLMOps Control Center Schema Extension: Prompt Registry & Alerting Engine
-- Compatible with SQLite & PostgreSQL

CREATE TABLE IF NOT EXISTS prompts (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    slug TEXT NOT NULL,
    version TEXT NOT NULL,
    template TEXT NOT NULL,
    variables TEXT,
    model_target TEXT NOT NULL,
    stage TEXT NOT NULL DEFAULT 'Draft',
    description TEXT,
    tags TEXT,
    avg_faithfulness REAL NOT NULL DEFAULT 0.0,
    avg_latency_ms REAL NOT NULL DEFAULT 0.0,
    usage_count INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS alert_rules (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    metric_type TEXT NOT NULL,
    operator TEXT NOT NULL DEFAULT 'gt',
    threshold REAL NOT NULL,
    severity TEXT NOT NULL DEFAULT 'High',
    webhook_url TEXT,
    channel_type TEXT NOT NULL DEFAULT 'webhook',
    is_enabled INTEGER NOT NULL DEFAULT 1,
    last_triggered_at TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS alert_events (
    id TEXT PRIMARY KEY,
    rule_id TEXT NOT NULL,
    rule_name TEXT NOT NULL,
    metric_type TEXT NOT NULL,
    observed_value REAL NOT NULL,
    threshold REAL NOT NULL,
    severity TEXT NOT NULL,
    message TEXT NOT NULL,
    delivery_status TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(rule_id) REFERENCES alert_rules(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_prompts_slug ON prompts(slug);
CREATE INDEX IF NOT EXISTS idx_prompts_stage ON prompts(stage);
CREATE INDEX IF NOT EXISTS idx_alert_events_created ON alert_events(created_at);
