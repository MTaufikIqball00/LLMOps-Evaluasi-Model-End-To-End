use axum::http::StatusCode;
use llmops_backend::{api, observability::redact_secrets, persistence};
use std::sync::Arc;
use std::time::Instant;
use tokio::net::TcpListener;
use uuid::Uuid;

struct PerfTestApp {
    pub base_url: String,
    pub pool: persistence::DbPool,
    pub db_file: String,
}

impl PerfTestApp {
    async fn spawn() -> Self {
        let db_id = Uuid::new_v4().to_string();
        tokio::fs::create_dir_all("target/tmp/test_dbs").await.ok();
        let db_file = format!("target/tmp/test_dbs/perf_{}.db", db_id);
        let database_url = format!("sqlite://{}?mode=rwc", db_file);

        let pool = persistence::init_db(&database_url).await.unwrap();

        let state = Arc::new(api::health::AppState {
            pool: pool.clone(),
            start_time: Instant::now(),
            server_port: 0,
            frontend_origin: "http://localhost:3210".to_string(),
            mlflow_url: "http://localhost:5500".to_string(),
            prometheus_url: "http://localhost:9190".to_string(),
            grafana_url: "http://localhost:3310".to_string(),
            github_repo: "enterprise/llmops-platform".to_string(),
            github_token: None,
        });

        let app = api::build_router(state);
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base_url = format!("http://{}", listener.local_addr().unwrap());

        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        PerfTestApp {
            base_url,
            pool,
            db_file,
        }
    }
}

impl Drop for PerfTestApp {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.db_file);
    }
}

#[tokio::test]
async fn test_api_latency_baselines() {
    let app = PerfTestApp::spawn().await;
    let client = reqwest::Client::new();

    let endpoints = vec![
        "/health",
        "/api/v1/health",
        "/api/v1/overview/stats",
        "/api/v1/models",
        "/api/v1/metrics",
        "/api/v1/costs/summary",
    ];

    println!("\n=== PERFORMANCE LATENCY BASELINE MEASUREMENTS ===");

    for ep in endpoints {
        let url = format!("{}{}", app.base_url, ep);
        let start = Instant::now();
        let res = client.get(&url).send().await.unwrap();
        let duration = start.elapsed();
        let ms = duration.as_secs_f64() * 1000.0;

        assert_eq!(res.status(), StatusCode::OK);
        println!("Endpoint {:<30} Latency: {:>8.2} ms", ep, ms);

        // Sanity ceiling: in-memory/local SQLite API requests should comfortably resolve within 1500ms
        assert!(ms < 1500.0, "Endpoint {} exceeded 1500ms baseline", ep);
    }
}

#[tokio::test]
async fn test_concurrent_request_stability() {
    let app = PerfTestApp::spawn().await;
    let client = Arc::new(reqwest::Client::new());

    let concurrent_workers = 30;
    let mut handles = Vec::new();

    let start = Instant::now();

    for i in 0..concurrent_workers {
        let c = client.clone();
        let url = format!("{}/api/v1/overview/stats", app.base_url);
        handles.push(tokio::spawn(async move {
            let res = c.get(&url).send().await;
            (i, res)
        }));
    }

    let mut successful = 0;
    for h in handles {
        let (_id, res) = h.await.unwrap();
        if let Ok(r) = res {
            if r.status() == StatusCode::OK {
                successful += 1;
            }
        }
    }

    let total_elapsed = start.elapsed().as_secs_f64();
    let rps = successful as f64 / total_elapsed;

    println!(
        "Concurrent stress test: {} / {} completed successfully in {:.3}s ({:.1} req/sec)",
        successful, concurrent_workers, total_elapsed, rps
    );

    assert_eq!(
        successful, concurrent_workers,
        "All concurrent requests must succeed without connection pool starvation"
    );
}

#[test]
fn test_secret_scrubber_throughput() {
    let large_sample = "User input with sk-1234567890abcdef1234567890 and normal text. \n".repeat(500);
    let start = Instant::now();
    let sanitized = redact_secrets(&large_sample);
    let elapsed = start.elapsed();

    assert!(sanitized.contains("[REDACTED_SECRET]"));
    println!(
        "Secret scrubber processed {} bytes in {:.2} ms",
        large_sample.len(),
        elapsed.as_secs_f64() * 1000.0
    );
    assert!(
        elapsed.as_secs_f64() < 1.0,
        "Scrubber throughput must process 500 lines under 1 second"
    );
}
