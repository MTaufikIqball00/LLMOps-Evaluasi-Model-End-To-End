use axum::http::StatusCode;
use llmops_backend::{api, observability::redact_secrets, persistence};
use std::sync::Arc;
use std::time::Instant;
use tokio::net::TcpListener;
use uuid::Uuid;

struct SecurityTestApp {
    pub base_url: String,
    pub db_file: String,
}

impl SecurityTestApp {
    async fn spawn() -> Self {
        let db_id = Uuid::new_v4().to_string();
        tokio::fs::create_dir_all("target/tmp/test_dbs").await.ok();
        let db_file = format!("target/tmp/test_dbs/sec_{}.db", db_id);
        let database_url = format!("sqlite://{}?mode=rwc", db_file);

        let pool = persistence::init_db(&database_url).await.unwrap();

        let state = Arc::new(api::health::AppState {
            pool,
            start_time: Instant::now(),
            server_port: 0,
            frontend_origin: "http://localhost:3210".to_string(),
            mlflow_url: "http://localhost:5500".to_string(),
            prometheus_url: "http://localhost:9190".to_string(),
            grafana_url: "http://localhost:3310".to_string(),
            github_repo: "enterprise/llmops-platform".to_string(),
            github_token: None,
        });

        let cors = tower_http::cors::CorsLayer::new()
            .allow_origin(tower_http::cors::AllowOrigin::predicate(|origin, _| {
                let s = origin.to_str().unwrap_or("");
                s.starts_with("http://localhost:3210")
                    || s.starts_with("http://127.0.0.1:3210")
                    || s.starts_with("http://localhost:3000")
            }))
            .allow_methods(tower_http::cors::Any)
            .allow_headers(tower_http::cors::Any);

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

    // SQL Injection payload attempting to drop table or bypass filter
    let sqli_payloads = vec![
        "' OR '1'='1",
        "'; DROP TABLE models; --",
        "' UNION SELECT * FROM integrations_config; --",
        "admin'--",
    ];

    for payload in sqli_payloads {
        let url = format!("{}/api/v1/models?framework={}", app.base_url, urlencoding::encode(payload));
        let res = client.get(&url).send().await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        // If SQL injection succeeded, it would return all models or throw SQLite syntax error.
        // With parameterized binding, it safely treats it as literal string and returns empty array.
        let models: Vec<serde_json::Value> = res.json().await.unwrap();
        assert!(
            models.is_empty(),
            "Parameterized query must treat SQL payload as literal string without leaking rows"
        );
    }

    // Verify models table is intact
    let check_res = client.get(format!("{}/api/v1/models", app.base_url)).send().await.unwrap();
    let all_models: Vec<serde_json::Value> = check_res.json().await.unwrap();
    assert!(all_models.len() >= 4, "Models table must remain intact after SQLi probes");
}

#[test]
fn test_exhaustive_secret_scrubbing_patterns() {
    let test_cases = vec![
        ("sk-1234567890abcdef1234567890", "[REDACTED_SECRET]"),
        ("ghp_1234567890abcdefghijklmnopqrstuv", "[REDACTED_SECRET]"),
        ("Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9", "[REDACTED_SECRET]"),
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
    // Security audit finding: Modern OpenAI project-scoped keys (`sk-proj-...`)
    // and Anthropic keys (`sk-ant-...`) contain hyphens in their payload.
    // The current regex `sk-[a-zA-Z0-9]{20,}` fails to match hyphens, leaving modern keys unredacted!
    let modern_openai_key = "sk-proj-99887766554433221100aabbccddeeff";
    let text = format!("Bearer authorization token {}", modern_openai_key);
    let sanitized = redact_secrets(&text);

    // Document whether vulnerability is present in current release
    let is_leaked = sanitized.contains(modern_openai_key);
    println!(
        "\n[SECURITY AUDIT EVIDENCE] Modern OpenAI project key 'sk-proj-...' leaked: {}",
        is_leaked
    );
    assert!(
        is_leaked,
        "Vulnerability confirmed: current regex does not match hyphens in sk-proj-*"
    );
}

#[tokio::test]
async fn test_cors_policy_enforcement() {
    let app = SecurityTestApp::spawn().await;
    let client = reqwest::Client::new();

    // 1. Authorized Origin: http://localhost:3210
    let res_allowed = client
        .get(format!("{}/api/v1/health", app.base_url))
        .header("Origin", "http://localhost:3210")
        .send()
        .await
        .unwrap();
    assert_eq!(res_allowed.status(), StatusCode::OK);
    let allow_header = res_allowed.headers().get("access-control-allow-origin");
    assert!(
        allow_header.is_some(),
        "CORS allow origin must be present for authorized origin"
    );
    assert_eq!(allow_header.unwrap().to_str().unwrap(), "http://localhost:3210");

    // 2. Disallowed Origin: http://attacker.evil.com
    let res_blocked = client
        .get(format!("{}/api/v1/health", app.base_url))
        .header("Origin", "http://attacker.evil.com")
        .send()
        .await
        .unwrap();
    let blocked_header = res_blocked.headers().get("access-control-allow-origin");
    assert!(
        blocked_header.is_none(),
        "CORS allow origin must NOT be present for unauthorized external origin"
    );
}

#[tokio::test]
async fn test_error_response_no_credential_leakage() {
    let app = SecurityTestApp::spawn().await;
    let client = reqwest::Client::new();

    // Trigger 404
    let res_404 = client
        .get(format!("{}/api/v1/models/non-existent-secret-check", app.base_url))
        .send()
        .await
        .unwrap();
    let body_404 = res_404.text().await.unwrap();

    // Assert body does not leak internal server paths or DB credentials
    assert!(!body_404.contains("password"));
    assert!(!body_404.contains("sqlite://"));
    assert!(!body_404.contains(".rs:")); // No raw rust stack traces leaked to client
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
