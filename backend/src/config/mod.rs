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

        let frontend_origin = env::var("FRONTEND_ORIGIN")
            .unwrap_or_else(|_| "http://localhost:3210".to_string());

        let mlflow_url = env::var("MLFLOW_TRACKING_URI")
            .unwrap_or_else(|_| "http://localhost:5500".to_string());

        let wandb_api_key = env::var("WANDB_API_KEY").ok().filter(|s| !s.is_empty());

        let prometheus_url = env::var("PROMETHEUS_URL")
            .unwrap_or_else(|_| "http://localhost:9190".to_string());

        let grafana_url = env::var("GRAFANA_URL")
            .unwrap_or_else(|_| "http://localhost:3310".to_string());

        let kubernetes_in_cluster = env::var("KUBERNETES_IN_CLUSTER")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false);

        let github_token = env::var("GITHUB_TOKEN").ok().filter(|s| !s.is_empty());
        let github_repo = env::var("GITHUB_REPO")
            .unwrap_or_else(|_| "enterprise/llmops-platform".to_string());

        let app_env = env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());

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
        }
    }
}
