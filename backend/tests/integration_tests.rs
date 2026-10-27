use llmops_backend::integrations::ragas::evaluate_sample;
use llmops_backend::observability::{format_prometheus_metrics, redact_secrets};

#[test]
fn test_ragas_evaluation_scoring() {
    let question = "What is the reimbursement limit for ergonomics accessories?";
    let context = "Section 4: Ergonomics equipment up to $750 may be reimbursed per fiscal year.";
    let ground_truth = "Up to $750 per fiscal year.";
    let faithful_answer = "Employees can claim up to $750 per fiscal year for ergonomics.";

    let res = evaluate_sample(question, Some(ground_truth), faithful_answer, Some(context));

    assert!(res.faithfulness > 0.70, "Faithful answer score must exceed 0.70");
    assert!(res.relevance > 0.60, "Relevant answer score must exceed 0.60");
    assert!(!res.hallucination_detected, "No hallucination should be flagged for faithful response");
    assert!(res.hallucination_score < 0.35, "Hallucination score must be below 0.35 threshold");
}

#[test]
fn test_secret_redaction() {
    let raw = "Client connected with sk-1234567890abcdef1234567890 and token ghp_99887766554433221100aa.";
    let sanitized = redact_secrets(raw);

    assert!(!sanitized.contains("sk-1234567890abcdef1234567890"));
    assert!(!sanitized.contains("ghp_99887766554433221100aa"));
    assert!(sanitized.contains("[REDACTED_SECRET]"));
}

#[test]
fn test_prometheus_metrics_format() {
    let metrics = format_prometheus_metrics(1250, 210.5, 380.0, 590.2, 450000, 0.942, 4);

    assert!(metrics.contains("llmops_http_requests_total 1250"));
    assert!(metrics.contains("llmops_llm_tokens_total 450000"));
    assert!(metrics.contains("llmops_evaluation_faithfulness_avg 0.942"));
    assert!(metrics.contains("llmops_deployments_active 4"));
}
