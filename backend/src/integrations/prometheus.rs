use crate::domain::IntegrationStatusReport;
use reqwest::Client;
use std::time::Instant;

pub async fn check_status(client: &Client, prom_url: &str) -> IntegrationStatusReport {
    let target = format!("{}/-/healthy", prom_url.trim_end_matches('/'));
    let start = Instant::now();

    match client.get(&target).send().await {
        Ok(res) if res.status().is_success() => {
            let latency = start.elapsed().as_secs_f64() * 1000.0;
            IntegrationStatusReport {
                service_name: "prometheus".to_string(),
                display_name: "Prometheus Monitoring Server".to_string(),
                category: "Observability".to_string(),
                status: "Connected (Live)".to_string(),
                endpoint_url: Some(prom_url.to_string()),
                latency_ms: Some(latency),
                message: "Prometheus instance healthy. Actively scraping /api/v1/metrics.".to_string(),
                is_configured: true,
                last_checked: Some(chrono::Utc::now().to_rfc3339()),
            }
        }
        Ok(res) => {
            let latency = start.elapsed().as_secs_f64() * 1000.0;
            IntegrationStatusReport {
                service_name: "prometheus".to_string(),
                display_name: "Prometheus Monitoring Server".to_string(),
                category: "Observability".to_string(),
                status: "Degraded".to_string(),
                endpoint_url: Some(prom_url.to_string()),
                latency_ms: Some(latency),
                message: format!("Prometheus returned status {}. Scrape targets may be failing.", res.status()),
                is_configured: true,
                last_checked: Some(chrono::Utc::now().to_rfc3339()),
            }
        }
        Err(_) => {
            IntegrationStatusReport {
                service_name: "prometheus".to_string(),
                display_name: "Prometheus Monitoring Server".to_string(),
                category: "Observability".to_string(),
                status: "Disconnected / Standalone".to_string(),
                endpoint_url: Some(prom_url.to_string()),
                latency_ms: None,
                message: format!("No external Prometheus server listening on {}. Local metrics available at /api/v1/metrics.", prom_url),
                is_configured: true,
                last_checked: Some(chrono::Utc::now().to_rfc3339()),
            }
        }
    }
}
