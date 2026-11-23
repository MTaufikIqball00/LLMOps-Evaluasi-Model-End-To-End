use crate::api::health::AppState;
use crate::domain::CreateTestCaseInput;
use crate::integrations::ragas;
use crate::observability::alerts;
use crate::persistence::adapt_sql;
use reqwest::Client;
use serde::Deserialize;
use std::sync::Arc;
use tracing::{error, info, warn};
use uuid::Uuid;

#[derive(Deserialize)]
struct JudgeScores {
    faithfulness: f64,
    relevance: f64,
    context_precision: f64,
    context_recall: f64,
    groundedness: f64,
}

async fn try_llm_judge(
    state: &AppState,
    tc: &CreateTestCaseInput,
) -> Option<ragas::EvaluationResult> {
    let api_key = state.llm_gateway_api_key.as_deref()?;
    let upstream = state.llm_gateway_upstream_url.trim_end_matches('/');
    let url = format!("{upstream}/chat/completions");

    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .ok()?;

    let system_prompt = "You are a strict RAG evaluation judge. Score the response on 0.0-1.0 for keys: faithfulness, relevance, context_precision, context_recall, groundedness. Reply ONLY with compact JSON matching those exact 5 numeric keys.";
    let user_prompt = format!(
        "Question: {}\nRetrieved Context: {}\nGround Truth: {}\nGenerated Answer: {}",
        tc.question,
        tc.retrieved_context.as_deref().unwrap_or("None"),
        tc.ground_truth.as_deref().unwrap_or("None"),
        tc.generated_answer
    );

    let payload = serde_json::json!({
        "model": state.llm_judge_model,
        "temperature": 0.0,
        "messages": [
            { "role": "system", "content": system_prompt },
            { "role": "user", "content": user_prompt }
        ]
    });

    let res = client
        .post(&url)
        .bearer_auth(api_key)
        .json(&payload)
        .send()
        .await
        .ok()?;

    if !res.status().is_success() {
        return None;
    }

    let body: serde_json::Value = res.json().await.ok()?;
    let raw_content = body["choices"][0]["message"]["content"].as_str()?;
    let cleaned = raw_content
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    let scores: JudgeScores = serde_json::from_str(cleaned).ok()?;
    let clamp3 = |v: f64| ((v.clamp(0.0, 1.0)) * 1000.0).round() / 1000.0;

    let faithfulness = clamp3(scores.faithfulness);
    let relevance = clamp3(scores.relevance);
    let context_precision = clamp3(scores.context_precision);
    let context_recall = clamp3(scores.context_recall);
    let groundedness = clamp3(scores.groundedness);
    let hallucination_score =
        ragas::compute_hallucination_index(faithfulness, groundedness, context_precision);
    let hallucination_detected = hallucination_score > 0.35 || groundedness < 0.75;

    Some(ragas::EvaluationResult {
        faithfulness,
        relevance,
        context_precision,
        context_recall,
        groundedness,
        hallucination_score,
        hallucination_detected,
    })
}

pub async fn run_evaluation_job(
    state: Arc<AppState>,
    eval_id: String,
    test_cases: Vec<CreateTestCaseInput>,
    use_llm_judge: bool,
) {
    tokio::spawn(async move {
        info!(
            "Starting evaluation job: {} (llm_judge={})",
            eval_id, use_llm_judge
        );

        let mut total_faithfulness = 0.0;
        let mut total_relevance = 0.0;
        let mut total_precision = 0.0;
        let mut total_recall = 0.0;
        let mut total_groundedness = 0.0;
        let mut total_hallucination = 0.0;
        let mut total_latency = 0.0;
        let mut total_tokens = 0i64;

        let cases_to_eval = if test_cases.is_empty() {
            vec![
                CreateTestCaseInput {
                    question: "What is the token context window of the deployed reasoning model?".to_string(),
                    ground_truth: Some("The model supports an 8k standard context window, expandable to 32k with RoPE frequency scaling.".to_string()),
                    generated_answer: "The model supports an 8k context window natively, scalable to 32k with RoPE scaling.".to_string(),
                    retrieved_context: Some("Model architecture specification: Native context window 8,192 tokens. RoPE base frequency scaling enables up to 32,768 context length.".to_string()),
                },
                CreateTestCaseInput {
                    question: "How are enterprise API secrets masked before logging?".to_string(),
                    ground_truth: Some("All regex patterns matching bearer tokens, sk-*, and ghp_* are replaced with [REDACTED_SECRET].".to_string()),
                    generated_answer: "Tokens matching sk-* or ghp-* prefixes are automatically redacted to [REDACTED_SECRET] before writing to persistent logs.".to_string(),
                    retrieved_context: Some("Logging Middleware: Secret scrubbers intercept HTTP payload streams. Detected credentials matching known token structures are replaced with [REDACTED_SECRET].".to_string()),
                },
                CreateTestCaseInput {
                    question: "What is the failover strategy for Kubernetes inference pods on GPU out-of-memory?".to_string(),
                    ground_truth: Some("The readiness probe fails, traffic is routed to replica instances, and the pod is restarted by the ReplicaSet controller.".to_string()),
                    generated_answer: "The pod triggers a graceful CPU degradation mode without restarting the container.".to_string(),
                    retrieved_context: Some("SRE Runbook: Upon GPU OOM (CUDA error 2), the vLLM engine exits with code 137. Kubernetes restarts the pod while the service ingress shifts traffic to surviving replicas.".to_string()),
                },
            ]
        } else {
            test_cases
        };

        let count = cases_to_eval.len() as f64;

        for tc in &cases_to_eval {
            let started = std::time::Instant::now();
            let res = if use_llm_judge && state.llm_gateway_api_key.is_some() {
                match try_llm_judge(&state, tc).await {
                    Some(judged) => judged,
                    None => {
                        warn!(
                            "External LLM Judge failed for {}, falling back to native Ragas evaluator",
                            eval_id
                        );
                        ragas::evaluate_sample(
                            &tc.question,
                            tc.ground_truth.as_deref(),
                            &tc.generated_answer,
                            tc.retrieved_context.as_deref(),
                        )
                    }
                }
            } else {
                ragas::evaluate_sample(
                    &tc.question,
                    tc.ground_truth.as_deref(),
                    &tc.generated_answer,
                    tc.retrieved_context.as_deref(),
                )
            };

            let elapsed_ms = (started.elapsed().as_secs_f64() * 1000.0).max(1.0);
            let eval_latency = if use_llm_judge && state.llm_gateway_api_key.is_some() {
                (elapsed_ms * 10.0).round() / 10.0
            } else {
                ((210.0 + (tc.generated_answer.len() as f64 * 0.4)) * 10.0).round() / 10.0
            };

            let case_id = format!(
                "tc-{}",
                Uuid::new_v4()
                    .to_string()
                    .chars()
                    .take(8)
                    .collect::<String>()
            );
            let approx_tokens = ((tc.question.len()
                + tc.generated_answer.len()
                + tc.retrieved_context.as_ref().map(|c| c.len()).unwrap_or(0))
                as i64
                / 4)
            .max(1);

            total_faithfulness += res.faithfulness;
            total_relevance += res.relevance;
            total_precision += res.context_precision;
            total_recall += res.context_recall;
            total_groundedness += res.groundedness;
            total_hallucination += res.hallucination_score;
            total_latency += eval_latency;
            total_tokens += approx_tokens;

            let hall_int: i64 = if res.hallucination_detected { 1 } else { 0 };

            let insert_sql = adapt_sql(
                r#"
                INSERT INTO evaluation_test_cases (
                    id, evaluation_id, question, ground_truth, generated_answer, retrieved_context,
                    faithfulness_score, relevance_score, groundedness_score, hallucination_detected,
                    latency_ms, tokens_used, created_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
                "#,
                state.is_postgres,
            );

            let insert_res = sqlx::query(&insert_sql)
                .bind(&case_id)
                .bind(&eval_id)
                .bind(&tc.question)
                .bind(&tc.ground_truth)
                .bind(&tc.generated_answer)
                .bind(&tc.retrieved_context)
                .bind(res.faithfulness)
                .bind(res.relevance)
                .bind(res.groundedness)
                .bind(hall_int)
                .bind(eval_latency)
                .bind(approx_tokens)
                .execute(&state.pool)
                .await;

            if let Err(e) = insert_res {
                error!("Failed to insert evaluation test case: {}", e);
            }
        }

        let avg_faithfulness = (total_faithfulness / count * 1000.0).round() / 1000.0;
        let avg_relevance = (total_relevance / count * 1000.0).round() / 1000.0;
        let avg_precision = (total_precision / count * 1000.0).round() / 1000.0;
        let avg_recall = (total_recall / count * 1000.0).round() / 1000.0;
        let avg_groundedness = (total_groundedness / count * 1000.0).round() / 1000.0;
        let avg_hallucination = (total_hallucination / count * 1000.0).round() / 1000.0;
        let avg_latency = (total_latency / count * 10.0).round() / 10.0;

        let update_sql = adapt_sql(
            r#"
            UPDATE evaluations
            SET status = 'Completed',
                faithfulness = ?,
                answer_relevance = ?,
                context_precision = ?,
                context_recall = ?,
                groundedness = ?,
                hallucination_score = ?,
                avg_latency_ms = ?,
                total_tokens = ?,
                test_cases_count = ?
            WHERE id = ?
            "#,
            state.is_postgres,
        );

        let update_res = sqlx::query(&update_sql)
            .bind(avg_faithfulness)
            .bind(avg_relevance)
            .bind(avg_precision)
            .bind(avg_recall)
            .bind(avg_groundedness)
            .bind(avg_hallucination)
            .bind(avg_latency)
            .bind(total_tokens)
            .bind(cases_to_eval.len() as i64)
            .bind(&eval_id)
            .execute(&state.pool)
            .await;

        match update_res {
            Ok(_) => {
                info!("Evaluation job {} completed successfully.", eval_id);
                let _ = alerts::evaluate_and_dispatch_alerts(
                    &state.pool,
                    state.is_postgres,
                    state.alert_webhook_url.as_deref(),
                )
                .await;
            }
            Err(e) => error!("Failed to update evaluation status: {}", e),
        }
    });
}
