use crate::domain::CreateTestCaseInput;
use crate::integrations::ragas;
use crate::persistence::DbPool;
use tracing::{error, info};
use uuid::Uuid;

pub async fn run_evaluation_job(
    pool: DbPool,
    eval_id: String,
    test_cases: Vec<CreateTestCaseInput>,
) {
    tokio::spawn(async move {
        info!("Starting evaluation job: {}", eval_id);

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
                    ground_truth: Some("All regex patterns matching bearer tokens, sk-*, and ghp_* are replaced with [REDACTED_API_KEY].".to_string()),
                    generated_answer: "Tokens matching sk-* or ghp-* prefixes are automatically redacted to [REDACTED_API_KEY] before writing to persistent logs.".to_string(),
                    retrieved_context: Some("Logging Middleware: Secret scrubbers intercept HTTP payload streams. Detected credentials matching known token structures are replaced with [REDACTED_API_KEY].".to_string()),
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
            let res = ragas::evaluate_sample(
                &tc.question,
                tc.ground_truth.as_deref(),
                &tc.generated_answer,
                tc.retrieved_context.as_deref(),
            );

            let case_id = format!(
                "tc-{}",
                Uuid::new_v4()
                    .to_string()
                    .chars()
                    .take(8)
                    .collect::<String>()
            );
            let simulated_latency = 210.0 + (tc.generated_answer.len() as f64 * 0.4);
            let simulated_tokens = (tc.question.len()
                + tc.generated_answer.len()
                + tc.retrieved_context.as_ref().map(|c| c.len()).unwrap_or(0))
                as i64
                / 4;

            total_faithfulness += res.faithfulness;
            total_relevance += res.relevance;
            total_precision += res.context_precision;
            total_recall += res.context_recall;
            total_groundedness += res.groundedness;
            total_hallucination += res.hallucination_score;
            total_latency += simulated_latency;
            total_tokens += simulated_tokens;

            let hall_int = if res.hallucination_detected { 1 } else { 0 };

            let insert_res = sqlx::query(
                r#"
                INSERT INTO evaluation_test_cases (
                    id, evaluation_id, question, ground_truth, generated_answer, retrieved_context,
                    faithfulness_score, relevance_score, groundedness_score, hallucination_detected,
                    latency_ms, tokens_used, created_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
                "#,
            )
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
            .bind(simulated_latency)
            .bind(simulated_tokens)
            .execute(&pool)
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

        let update_res = sqlx::query(
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
        )
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
        .execute(&pool)
        .await;

        match update_res {
            Ok(_) => info!("Evaluation job {} completed successfully.", eval_id),
            Err(e) => error!("Failed to update evaluation status: {}", e),
        }
    });
}
