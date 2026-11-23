use std::env;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub server_host: String,
    pub server_port: u16,
    pub database_url: String,
    pub frontend_origin: String,
    pub mlflow_url: String,
    pub wandb_api_key: Option<String>,
    pub prometheus_url: String,
    pub grafana_url: String,
    pub kubernetes_in_cluster: bool,
    pub github_token: Option<String>,
    pub github_repo: String,
    pub app_env: String,
    pub admin_api_key: Option<String>,
    pub request_timeout_secs: u64,
    pub rate_limit_rpm: u32,
    pub llm_gateway_upstream_url: String,
    pub llm_gateway_api_key: Option<String>,
    pub llm_judge_model: String,
    pub alert_webhook_url: Option<String>,
}

impl AppConfig {
    pub fn load() -> Self {
        dotenvy::dotenv().ok();

        let server_host = env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let server_port = env::var("SERVER_PORT")
            .ok()
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(8210);

        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "sqlite://data/llmops.db?mode=rwc".to_string());

        let frontend_origin =
            env::var("FRONTEND_ORIGIN").unwrap_or_else(|_| "http://localhost:3210".to_string());

        let mlflow_url =
            env::var("MLFLOW_TRACKING_URI").unwrap_or_else(|_| "http://localhost:5500".to_string());

        let wandb_api_key = env::var("WANDB_API_KEY").ok().filter(|s| !s.is_empty());

        let prometheus_url =
            env::var("PROMETHEUS_URL").unwrap_or_else(|_| "http://localhost:9190".to_string());

        let grafana_url =
            env::var("GRAFANA_URL").unwrap_or_else(|_| "http://localhost:3310".to_string());

        let kubernetes_in_cluster = env::var("KUBERNETES_IN_CLUSTER")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false);

        let github_token = env::var("GITHUB_TOKEN").ok().filter(|s| !s.is_empty());
        let github_repo =
            env::var("GITHUB_REPO").unwrap_or_else(|_| "enterprise/llmops-platform".to_string());

        let app_env = env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());

        let admin_api_key = env::var("ADMIN_API_KEY").ok().filter(|s| !s.is_empty());

        let request_timeout_secs = env::var("REQUEST_TIMEOUT_SECS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .filter(|&v| v > 0)
            .unwrap_or(30);

        let rate_limit_rpm = env::var("RATE_LIMIT_RPM")
            .ok()
            .and_then(|v| v.parse::<u32>().ok())
            .filter(|&v| v > 0)
            .unwrap_or(120);

        let llm_gateway_upstream_url = env::var("LLM_GATEWAY_UPSTREAM_URL")
            .ok()
            .filter(|s| !s.is_empty())
            .or_else(|| env::var("OPENAI_BASE_URL").ok().filter(|s| !s.is_empty()))
            .unwrap_or_else(|| "https://api.openai.com/v1".to_string());

        let llm_gateway_api_key = env::var("LLM_GATEWAY_API_KEY")
            .ok()
            .filter(|s| !s.is_empty())
            .or_else(|| env::var("OPENAI_API_KEY").ok().filter(|s| !s.is_empty()));

        let llm_judge_model = env::var("LLM_JUDGE_MODEL")
            .ok()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "gpt-4o-mini".to_string());

        let alert_webhook_url = env::var("ALERT_WEBHOOK_URL").ok().filter(|s| !s.is_empty());

        Self {
            server_host,
            server_port,
            database_url,
            frontend_origin,
            mlflow_url,
            wandb_api_key,
            prometheus_url,
            grafana_url,
            kubernetes_in_cluster,
            github_token,
            github_repo,
            app_env,
            admin_api_key,
            request_timeout_secs,
            rate_limit_rpm,
            llm_gateway_upstream_url,
            llm_gateway_api_key,
            llm_judge_model,
            alert_webhook_url,
        }
    }
}
