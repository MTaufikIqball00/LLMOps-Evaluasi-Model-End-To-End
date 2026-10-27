use crate::domain::IntegrationStatusReport;
use reqwest::Client;
use std::time::Instant;

pub async fn check_status(client: &Client, mlflow_url: &str) -> IntegrationStatusReport {
    let target = format!(
        "{}/api/2.0/mlflow/experiments/list",
        mlflow_url.trim_end_matches('/')
    );
    let start = Instant::now();

    match client.get(&target).send().await {
        Ok(res) if res.status().is_success() => {
            let latency = start.elapsed().as_secs_f64() * 1000.0;
            IntegrationStatusReport {
                service_name: "mlflow".to_string(),
                display_name: "MLflow Tracking Server".to_string(),
                category: "Experiment Tracking".to_string(),
                status: "Connected (Live)".to_string(),
                endpoint_url: Some(mlflow_url.to_string()),
                latency_ms: Some(latency),
                message: "MLflow REST API responded with HTTP 200. Tracking service active."
                    .to_string(),
                is_configured: true,
                last_checked: Some(chrono::Utc::now().to_rfc3339()),
            }
        }
        Ok(res) => {
            let latency = start.elapsed().as_secs_f64() * 1000.0;
            IntegrationStatusReport {
                service_name: "mlflow".to_string(),
                display_name: "MLflow Tracking Server".to_string(),
                category: "Experiment Tracking".to_string(),
                status: "Degraded".to_string(),
                endpoint_url: Some(mlflow_url.to_string()),
                latency_ms: Some(latency),
                message: format!(
                    "MLflow returned status code {}. Check server authentication.",
                    res.status()
                ),
                is_configured: true,
                last_checked: Some(chrono::Utc::now().to_rfc3339()),
            }
        }
        Err(err) => IntegrationStatusReport {
            service_name: "mlflow".to_string(),
            display_name: "MLflow Tracking Server".to_string(),
            category: "Experiment Tracking".to_string(),
            status: "Disconnected / Offline".to_string(),
            endpoint_url: Some(mlflow_url.to_string()),
            latency_ms: None,
            message: format!(
                "Could not reach MLflow at {}: {}. Serving local database records.",
                mlflow_url, err
            ),
            is_configured: true,
            last_checked: Some(chrono::Utc::now().to_rfc3339()),
        },
    }
}
