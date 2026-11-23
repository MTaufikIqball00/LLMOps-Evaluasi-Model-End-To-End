pub mod alerts;

use regex::Regex;
use std::sync::OnceLock;

static SECRET_REGEX: OnceLock<Vec<Regex>> = OnceLock::new();

fn get_secret_regexes() -> &'static Vec<Regex> {
    SECRET_REGEX.get_or_init(|| {
        vec![
            // OpenAI (including sk-proj-*, sk-svcacct-*) and Anthropic (sk-ant-*) hyphenated keys
            Regex::new(r"(?i)(sk-[a-zA-Z0-9_-]{20,})").unwrap(),
            // GitHub classic & fine-grained PATs
            Regex::new(r"(?i)(gh[pousr]_[a-zA-Z0-9]{20,}|github_pat_[a-zA-Z0-9_]{20,})").unwrap(),
            // Slack bot/user/app tokens
            Regex::new(r"(?i)(xox[baprs]-[a-zA-Z0-9-]{15,})").unwrap(),
            // Hugging Face tokens
            Regex::new(r"(?i)(hf_[a-zA-Z0-9]{20,})").unwrap(),
            // AWS Access Key IDs
            Regex::new(r"(AKIA[0-9A-Z]{16})").unwrap(),
            // Bearer headers
            Regex::new(r"(?i)(bearer\s+[a-zA-Z0-9_\-\.]{15,})").unwrap(),
            // Inline password assignments
            Regex::new(r#"(?i)(password\s*[:=]\s*[^\s,"']+)"#).unwrap(),
            // Inline api_key / api-key assignments
            Regex::new(r#"(?i)(api[_-]?key\s*[:=]\s*[^\s,"']+)"#).unwrap(),
        ]
    })
}

pub fn redact_secrets(input: &str) -> String {
    let mut sanitized = input.to_string();
    for re in get_secret_regexes() {
        sanitized = re.replace_all(&sanitized, "[REDACTED_SECRET]").to_string();
    }
    sanitized
}

#[derive(Debug, Clone, Copy, Default)]
pub struct LatencyPercentiles {
    pub p50: f64,
    pub p95: f64,
    pub p99: f64,
    pub avg: f64,
}

fn percentile_sorted(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    if sorted.len() == 1 {
        return (sorted[0] * 10.0).round() / 10.0;
    }
    let idx = p.clamp(0.0, 1.0) * (sorted.len() - 1) as f64;
    let lo = idx.floor() as usize;
    let hi = idx.ceil() as usize;
    let frac = idx - lo as f64;
    let val = sorted[lo] + (sorted[hi] - sorted[lo]) * frac;
    (val * 10.0).round() / 10.0
}

pub fn compute_latency_percentiles(latencies: Vec<f64>) -> LatencyPercentiles {
    let mut valid: Vec<f64> = latencies
        .into_iter()
        .filter(|v| v.is_finite() && *v >= 0.0)
        .collect();
    if valid.is_empty() {
        return LatencyPercentiles::default();
    }
    valid.sort_by(|a, b| a.total_cmp(b));
    let sum: f64 = valid.iter().sum();
    let avg = ((sum / valid.len() as f64) * 10.0).round() / 10.0;

    LatencyPercentiles {
        p50: percentile_sorted(&valid, 0.50),
        p95: percentile_sorted(&valid, 0.95),
        p99: percentile_sorted(&valid, 0.99),
        avg,
    }
}

pub fn format_prometheus_metrics(
    total_requests: i64,
    p50_latency_ms: f64,
    p95_latency_ms: f64,
    p99_latency_ms: f64,
    total_tokens: i64,
    avg_faithfulness: f64,
    active_deployments: i64,
) -> String {
    format!(
        "# HELP llmops_http_requests_total Total number of LLM requests processed\n\
         # TYPE llmops_http_requests_total counter\n\
         llmops_http_requests_total {}\n\n\
         # HELP llmops_llm_latency_p50_milliseconds Median latency in milliseconds\n\
         # TYPE llmops_llm_latency_p50_milliseconds gauge\n\
         llmops_llm_latency_p50_milliseconds {:.2}\n\n\
         # HELP llmops_llm_latency_p95_milliseconds 95th percentile latency in milliseconds\n\
         # TYPE llmops_llm_latency_p95_milliseconds gauge\n\
         llmops_llm_latency_p95_milliseconds {:.2}\n\n\
         # HELP llmops_llm_latency_p99_milliseconds 99th percentile latency in milliseconds\n\
         # TYPE llmops_llm_latency_p99_milliseconds gauge\n\
         llmops_llm_latency_p99_milliseconds {:.2}\n\n\
         # HELP llmops_llm_tokens_total Cumulative input and output tokens consumed\n\
         # TYPE llmops_llm_tokens_total counter\n\
         llmops_llm_tokens_total {}\n\n\
         # HELP llmops_evaluation_faithfulness_avg Average RAG/LLM evaluation faithfulness score [0..1]\n\
         # TYPE llmops_evaluation_faithfulness_avg gauge\n\
         llmops_evaluation_faithfulness_avg {:.3}\n\n\
         # HELP llmops_deployments_active Total active Kubernetes serving pods and deployments\n\
         # TYPE llmops_deployments_active gauge\n\
         llmops_deployments_active {}\n",
        total_requests,
        p50_latency_ms,
        p95_latency_ms,
        p99_latency_ms,
        total_tokens,
        avg_faithfulness,
        active_deployments
    )
}
