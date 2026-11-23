use axum::http::StatusCode;
use llmops_backend::{api, observability::redact_secrets, persistence};
use serde_json::json;
use std::sync::Arc;
use tokio::net::TcpListener;
use uuid::Uuid;

struct SecurityTestApp {
    pub base_url: String,
    pub db_file: String,
}

impl SecurityTestApp {
    async fn spawn() -> Self {
        Self::spawn_with("http://localhost:3210,https://*.vercel.app", None).await
    }

    async fn spawn_with(frontend_origin: &str, admin_api_key: Option<String>) -> Self {
        let db_id = Uuid::new_v4().to_string();
        tokio::fs::create_dir_all("target/tmp/test_dbs").await.ok();
        let db_file = format!("target/tmp/test_dbs/sec_{}.db", db_id);
        let database_url = format!("sqlite://{}?mode=rwc", db_file);

        let pool = persistence::init_db(&database_url).await.unwrap();

        let mut raw_state = api::health::AppState::for_test(pool);
        raw_state.frontend_origin = frontend_origin.to_string();
        raw_state.admin_api_key = admin_api_key;
        let state = Arc::new(raw_state);

        let cors = api::build_cors_layer(frontend_origin);
        let app = api::build_router(state).layer(cors);

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base_url = format!("http://{}", listener.local_addr().unwrap());

        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        SecurityTestApp { base_url, db_file }
    }
}

impl Drop for SecurityTestApp {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.db_file);
    }
}

#[tokio::test]
async fn test_sql_injection_defense_in_query_builders() {
    let app = SecurityTestApp::spawn().await;
    let client = reqwest::Client::new();

    let sqli_payloads = vec![
        "' OR '1'='1",
        "'; DROP TABLE models; --",
        "' UNION SELECT * FROM integrations_config; --",
        "admin'--",
    ];

    for payload in sqli_payloads {
        let url = format!(
            "{}/api/v1/models?framework={}",
            app.base_url,
            urlencoding::encode(payload)
        );
        let res = client.get(&url).send().await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let models: Vec<serde_json::Value> = res.json().await.unwrap();
        assert!(
            models.is_empty(),
            "Parameterized query must treat SQL payload as literal string without leaking rows"
        );
    }

    let check_res = client
        .get(format!("{}/api/v1/models", app.base_url))
        .send()
        .await
        .unwrap();
    let all_models: Vec<serde_json::Value> = check_res.json().await.unwrap();
    assert!(
        all_models.len() >= 4,
        "Models table must remain intact after SQLi probes"
    );
}

#[test]
fn test_exhaustive_secret_scrubbing_patterns() {
    let test_cases = vec![
        ("sk-1234567890abcdef1234567890", "[REDACTED_SECRET]"),
        ("ghp_1234567890abcdefghijklmnopqrstuv", "[REDACTED_SECRET]"),
        ("hf_1234567890abcdefghijklmnopqrstuv", "[REDACTED_SECRET]"),
        ("AKIAIOSFODNN7EXAMPLE", "[REDACTED_SECRET]"),
        (
            "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9",
            "[REDACTED_SECRET]",
        ),
        ("password = UltraSecretPassword123!", "[REDACTED_SECRET]"),
        ("password: SuperSecret987", "[REDACTED_SECRET]"),
        ("api_key: sk-custom-key-value-12345", "[REDACTED_SECRET]"),
        ("api-key=xyz9876543210123456", "[REDACTED_SECRET]"),
    ];

    for (raw, expected_placeholder) in test_cases {
        let text = format!("Header Authorization: {}, payload user query", raw);
        let sanitized = redact_secrets(&text);
        assert!(
            !sanitized.contains(raw),
            "Raw secret '{}' must NOT appear in sanitized text",
            raw
        );
        assert!(
            sanitized.contains(expected_placeholder),
            "Sanitized text must contain '{}'",
            expected_placeholder
        );
    }
}

#[test]
fn test_modern_project_key_vulnerability_probe() {
    // Verify hyphenated OpenAI (`sk-proj-...`) and Anthropic (`sk-ant-...`) keys are strictly scrubbed.
    let modern_keys = [
        "sk-proj-99887766554433221100aabbccddeeff",
        "sk-ant-api03-11223344556677889900aabbccddeeff",
    ];

    for key in modern_keys {
        let text = format!("Bearer authorization token {}", key);
        let sanitized = redact_secrets(&text);
        assert!(
            !sanitized.contains(key),
            "Modern hyphenated key '{}' must be completely scrubbed",
            key
        );
        assert!(
            sanitized.contains("[REDACTED_SECRET]"),
            "Placeholder [REDACTED_SECRET] must replace '{}'",
            key
        );
    }
}

#[tokio::test]
async fn test_trace_redacts_error_message_and_metadata() {
    let app = SecurityTestApp::spawn().await;
    let client = reqwest::Client::new();

    let leaked_key = "sk-proj-errorleak99887766554433221100";
    let leaked_gh = "ghp_metaSecretToken1234567890abcdef";

    let res = client
        .post(format!("{}/api/v1/traces", app.base_url))
        .json(&json!({
            "span_name": "scrub_all_fields_test",
            "model_name": "gpt-4o",
            "provider": "OpenAI",
            "status_code": 500,
            "latency_ms": 120.0,
            "prompt_tokens": 100,
            "completion_tokens": 0,
            "prompt": "Normal prompt",
            "completion": null,
            "metadata": { "debug_token": leaked_gh },
            "error_message": format!("Upstream rejected token {}", leaked_key)
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let trace: serde_json::Value = res.json().await.unwrap();

    let err_msg = trace["error_message"].as_str().unwrap();
    let meta = trace["metadata"].as_str().unwrap();
    assert!(!err_msg.contains(leaked_key));
    assert!(err_msg.contains("[REDACTED_SECRET]"));
    assert!(!meta.contains(leaked_gh));
    assert!(meta.contains("[REDACTED_SECRET]"));
}

#[tokio::test]
async fn test_cors_policy_enforcement() {
    let app = SecurityTestApp::spawn().await;
    let client = reqwest::Client::new();

    // 1. Authorized Local Origin
    let res_allowed = client
        .get(format!("{}/api/v1/health", app.base_url))
        .header("Origin", "http://localhost:3210")
        .send()
        .await
        .unwrap();
    assert_eq!(res_allowed.status(), StatusCode::OK);
    let allow_header = res_allowed.headers().get("access-control-allow-origin");
    assert_eq!(
        allow_header.unwrap().to_str().unwrap(),
        "http://localhost:3210"
    );

    // 2. Authorized Wildcard Vercel Origin
    let res_vercel = client
        .get(format!("{}/api/v1/health", app.base_url))
        .header("Origin", "https://llmops-preview-app.vercel.app")
        .send()
        .await
        .unwrap();
    assert_eq!(
        res_vercel
            .headers()
            .get("access-control-allow-origin")
            .unwrap()
            .to_str()
            .unwrap(),
        "https://llmops-preview-app.vercel.app"
    );

    // 3. Disallowed Origin & Subdomain Spoof
    for bad_origin in [
        "http://attacker.evil.com",
        "https://vercel.app.attacker.com",
    ] {
        let res_blocked = client
            .get(format!("{}/api/v1/health", app.base_url))
            .header("Origin", bad_origin)
            .send()
            .await
            .unwrap();
        assert!(
            res_blocked
                .headers()
                .get("access-control-allow-origin")
                .is_none(),
            "CORS allow origin must NOT match '{}'",
            bad_origin
        );
    }
}

#[tokio::test]
async fn test_admin_api_key_auth_enforcement() {
    let app = SecurityTestApp::spawn_with(
        "http://localhost:3210",
        Some("secret-admin-key-123".to_string()),
    )
    .await;
    let client = reqwest::Client::new();

    // GET endpoints remain readable
    let res_get = client
        .get(format!("{}/api/v1/models", app.base_url))
        .send()
        .await
        .unwrap();
    assert_eq!(res_get.status(), StatusCode::OK);

    // Mutating POST without key is rejected with 401
    let payload = json!({
        "name": "Unauthorized Model",
        "framework": "vLLM",
        "task": "Text",
        "version": "v1.0.0"
    });
    let res_unauth = client
        .post(format!("{}/api/v1/models", app.base_url))
        .json(&payload)
        .send()
        .await
        .unwrap();
    assert_eq!(res_unauth.status(), StatusCode::UNAUTHORIZED);

    // Mutating POST with valid X-API-Key succeeds
    let res_xkey = client
        .post(format!("{}/api/v1/models", app.base_url))
        .header("X-API-Key", "secret-admin-key-123")
        .json(&payload)
        .send()
        .await
        .unwrap();
    assert_eq!(res_xkey.status(), StatusCode::OK);

    // Mutating POST with valid Bearer token succeeds
    let res_bearer = client
        .post(format!("{}/api/v1/models", app.base_url))
        .bearer_auth("secret-admin-key-123")
        .json(&payload)
        .send()
        .await
        .unwrap();
    assert_eq!(res_bearer.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_error_response_no_credential_leakage() {
    let app = SecurityTestApp::spawn().await;
    let client = reqwest::Client::new();

    let res_404 = client
        .get(format!(
            "{}/api/v1/models/non-existent-secret-check",
            app.base_url
        ))
        .send()
        .await
        .unwrap();
    let body_404 = res_404.text().await.unwrap();

    assert!(!body_404.contains("password"));
    assert!(!body_404.contains("sqlite://"));
    assert!(!body_404.contains(".rs:"));
}

mod urlencoding {
    pub fn encode(data: &str) -> String {
        let mut encoded = String::new();
        for b in data.bytes() {
            match b {
                b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                    encoded.push(b as char);
                }
                _ => {
                    encoded.push_str(&format!("%{:02X}", b));
                }
            }
        }
        encoded
    }
}
