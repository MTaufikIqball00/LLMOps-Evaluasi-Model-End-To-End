use regex::Regex;
use std::sync::OnceLock;

static SECRET_REGEX: OnceLock<Vec<Regex>> = OnceLock::new();

fn get_secret_regexes() -> &'static Vec<Regex> {
    SECRET_REGEX.get_or_init(|| {
        vec![
            Regex::new(r"(?i)(sk-[a-zA-Z0-9]{20,})").unwrap(),
            Regex::new(r"(?i)(ghp_[a-zA-Z0-9]{20,})").unwrap(),
            Regex::new(r"(?i)(bearer\s+[a-zA-Z0-9_\-\.]{15,})").unwrap(),
            Regex::new(r"(?i)(password\s*[:=]\s*[^\s,]+)").unwrap(),
            Regex::new(r"(?i)(api[_-]?key\s*[:=]\s*[^\s,]+)").unwrap(),
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
