use crate::api::health::AppState;
use crate::api::traces::persist_trace_with_cost;
use crate::domain::{
    CreateTraceRequest, GatewayChatRequest, GatewayChatResponse, GatewayChoice, GatewayMessage,
    GatewayUsage,
};
use crate::errors::AppError;
use axum::{extract::State, Json};
use reqwest::Client;
use std::sync::Arc;
use std::time::Instant;
use uuid::Uuid;

fn estimate_tokens(text: &str) -> i64 {
    ((text.len() as f64 / 4.0).ceil() as i64).max(1)
}

pub async fn chat_completions(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<GatewayChatRequest>,
) -> Result<Json<GatewayChatResponse>, AppError> {
    if payload.messages.is_empty() {
        return Err(AppError::BadRequest(
            "Gateway request requires at least one message".to_string(),
        ));
    }

    let start = Instant::now();
    let provider = payload
        .provider
        .clone()
        .unwrap_or_else(|| "openai_compatible".to_string());
    let upstream_base = payload
        .upstream_url
        .clone()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| state.llm_gateway_upstream_url.clone());

    let joined_prompt = payload
        .messages
        .iter()
        .map(|m| format!("{}: {}", m.role, m.content))
        .collect::<Vec<_>>()
        .join("\n");

    let mut mode = "local_fallback".to_string();
    let mut status_code = 200i64;
    let mut error_message: Option<String> = None;
    let mut assistant_content = String::new();
    let mut prompt_tokens = estimate_tokens(&joined_prompt);
    let mut completion_tokens = 0i64;

    // Forward to upstream OpenAI-compatible API when API key or custom local upstream is configured
    let has_custom_upstream = payload.upstream_url.as_ref().is_some_and(|s| !s.is_empty());
    if state.llm_gateway_api_key.is_some() || has_custom_upstream {
        let target_url = format!("{}/chat/completions", upstream_base.trim_end_matches('/'));
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(state.request_timeout_secs))
            .build()
            .unwrap_or_default();

        let mut req_builder = client.post(&target_url).json(&serde_json::json!({
            "model": payload.model,
            "messages": payload.messages,
            "temperature": payload.temperature.unwrap_or(0.2),
            "max_tokens": payload.max_tokens.unwrap_or(512)
        }));

        if let Some(ref key) = state.llm_gateway_api_key {
            req_builder = req_builder.bearer_auth(key);
        }

        match req_builder.send().await {
            Ok(res) => {
                let code = res.status().as_u16() as i64;
                status_code = code;
                if res.status().is_success() {
                    if let Ok(json_body) = res.json::<serde_json::Value>().await {
                        assistant_content = json_body["choices"][0]["message"]["content"]
                            .as_str()
                            .unwrap_or("")
                            .to_string();
                        if let Some(pt) = json_body["usage"]["prompt_tokens"].as_i64() {
                            prompt_tokens = pt;
                        }
                        if let Some(ct) = json_body["usage"]["completion_tokens"].as_i64() {
                            completion_tokens = ct;
                        } else {
                            completion_tokens = estimate_tokens(&assistant_content);
                        }
                        mode = "upstream_live".to_string();
                    }
                } else {
                    let err_txt = res.text().await.unwrap_or_default();
                    error_message = Some(format!("Upstream HTTP {code}: {err_txt}"));
                    assistant_content =
                        format!("Upstream gateway error ({code}); trace logged with redaction.");
                    completion_tokens = estimate_tokens(&assistant_content);
                }
            }
            Err(e) => {
                status_code = 502;
                error_message = Some(format!("Upstream connection error: {e}"));
                assistant_content =
                    "Upstream LLM endpoint unreachable; trace logged safely.".to_string();
                completion_tokens = estimate_tokens(&assistant_content);
            }
        }
    }

    if assistant_content.is_empty() && error_message.is_none() {
        let last_user_msg = payload
            .messages
            .iter()
            .rev()
            .find(|m| m.role == "user")
            .map(|m| m.content.as_str())
            .unwrap_or("request");
        assistant_content = format!(
            "[LLMOps Gateway Local Fallback — configure LLM_GATEWAY_API_KEY for live inference] Received prompt for model '{}': {}",
            payload.model,
            last_user_msg.chars().take(180).collect::<String>()
        );
        completion_tokens = estimate_tokens(&assistant_content);
    }

    let latency_ms = ((start.elapsed().as_secs_f64() * 1000.0).max(1.0) * 10.0).round() / 10.0;

    let recorded_trace = persist_trace_with_cost(
        &state,
        CreateTraceRequest {
            span_name: "gateway::chat_completions".to_string(),
            model_name: payload.model.clone(),
            provider,
            status_code,
            latency_ms,
            prompt_tokens,
            completion_tokens,
            prompt: Some(joined_prompt),
            completion: Some(assistant_content.clone()),
            metadata: Some(serde_json::json!({
                "gateway_mode": mode,
                "upstream_url": upstream_base
            })),
            error_message,
        },
    )
    .await?;

    let completion_id = format!(
        "chatcmpl-{}",
        Uuid::new_v4()
            .to_string()
            .chars()
            .take(10)
            .collect::<String>()
    );

    Ok(Json(GatewayChatResponse {
        id: completion_id,
        object: "chat.completion".to_string(),
        created: chrono::Utc::now().timestamp(),
        model: payload.model,
        mode,
        choices: vec![GatewayChoice {
            index: 0,
            message: GatewayMessage {
                role: "assistant".to_string(),
                content: assistant_content,
            },
            finish_reason: "stop".to_string(),
        }],
        usage: GatewayUsage {
            prompt_tokens,
            completion_tokens,
            total_tokens: prompt_tokens + completion_tokens,
        },
        trace_id: recorded_trace.id,
        estimated_cost_usd: recorded_trace.estimated_cost,
        latency_ms,
    }))
}
