pub mod github_actions;
pub mod grafana;
pub mod kubernetes;
pub mod mlflow;
pub mod prometheus;
pub mod ragas;
pub mod trulens;
pub mod wandb;

use crate::domain::IntegrationStatusReport;
use crate::persistence::DbPool;

#[allow(dead_code)]
pub struct IntegrationHub {
    pub client: reqwest::Client,
}

#[allow(dead_code)]
impl IntegrationHub {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(3))
            .build()
            .unwrap_or_default();
        Self { client }
    }

    pub async fn check_all_statuses(
        &self,
        _pool: &DbPool,
        mlflow_url: &str,
        prom_url: &str,
        grafana_url: &str,
        github_repo: &str,
        github_token: Option<&str>,
    ) -> Vec<IntegrationStatusReport> {
        vec![
            mlflow::check_status(&self.client, mlflow_url).await,
            wandb::check_status(&self.client).await,
            prometheus::check_status(&self.client, prom_url).await,
            grafana::check_status(&self.client, grafana_url).await,
            ragas::check_status().await,
            trulens::check_status().await,
            kubernetes::check_status().await,
            github_actions::check_status(&self.client, github_repo, github_token).await,
        ]
    }
}
