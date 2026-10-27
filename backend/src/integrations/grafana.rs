use crate::domain::IntegrationStatusReport;
use reqwest::Client;
use std::time::Instant;

pub async fn check_status(client: &Client, grafana_url: &str) -> IntegrationStatusReport {
    let target = format!("{}/api/health", grafana_url.trim_end_matches('/'));
    let start = Instant::now();

    match client.get(&target).send().await {
        Ok(res) if res.status().is_success() => {
            let latency = start.elapsed().as_secs_f64() * 1000.0;
            IntegrationStatusReport {
                service_name: "grafana".to_string(),
                display_name: "Grafana Dashboards".to_string(),
                category: "Observability".to_string(),
                status: "Connected (Live)".to_string(),
                endpoint_url: Some(grafana_url.to_string()),
                latency_ms: Some(latency),
                message: "Grafana API healthy. Pre-provisioned dashboards accessible.".to_string(),
                is_configured: true,
                last_checked: Some(chrono::Utc::now().to_rfc3339()),
            }
        }
        _ => IntegrationStatusReport {
            service_name: "grafana".to_string(),
            display_name: "Grafana Dashboards".to_string(),
            category: "Observability".to_string(),
            status: "Disconnected / Standalone".to_string(),
            endpoint_url: Some(grafana_url.to_string()),
            latency_ms: None,
            message: format!(
                "Grafana instance not detected on {}. Using built-in telemetry UI.",
                grafana_url
            ),
            is_configured: true,
            last_checked: Some(chrono::Utc::now().to_rfc3339()),
        },
    }
}
