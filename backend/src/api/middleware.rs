use crate::api::health::AppState;
use crate::errors::AppError;
use axum::{
    body::Body,
    extract::State,
    http::{HeaderValue, Method, Request},
    middleware::Next,
    response::Response,
};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tower_http::cors::{AllowOrigin, Any, CorsLayer};

fn origin_matches_pattern(origin: &str, pattern: &str) -> bool {
    let pat = pattern.trim().trim_end_matches('/');
    if pat.is_empty() {
        return false;
    }
    if pat == "*" {
        return true;
    }
    if origin == pat {
        return true;
    }
    // Support subdomain wildcards like https://*.vercel.app
    if let Some((scheme, rest)) = pat.split_once("://*.") {
        let prefix = format!("{scheme}://");
        if let Some(after_scheme) = origin.strip_prefix(&prefix) {
            // Strip optional port, ensure no path segments inside host
            let host = after_scheme.split('/').next().unwrap_or("");
            let host_no_port = host.split(':').next().unwrap_or("");
            let expected_suffix = format!(".{rest}");
            if !after_scheme.contains('/')
                && host_no_port.ends_with(&expected_suffix)
                && host_no_port.len() > expected_suffix.len()
            {
                return true;
            }
        }
    }
    false
}

pub fn build_cors_layer(frontend_origin: &str) -> CorsLayer {
    let patterns: Vec<String> = frontend_origin
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    CorsLayer::new()
        .allow_origin(AllowOrigin::predicate(move |origin: &HeaderValue, _| {
            let s = origin.to_str().unwrap_or("");
            if s.starts_with("http://localhost:3210")
                || s.starts_with("http://127.0.0.1:3210")
                || s.starts_with("http://localhost:3000")
            {
                return true;
            }
            patterns.iter().any(|p| origin_matches_pattern(s, p))
        }))
        .allow_methods(Any)
        .allow_headers(Any)
}

pub async fn api_key_auth_middleware(
    State(state): State<Arc<AppState>>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    let is_mutating = matches!(
        *req.method(),
        Method::POST | Method::PUT | Method::PATCH | Method::DELETE
    );

    if is_mutating {
        if let Some(ref expected) = state.admin_api_key {
            let provided_x_key = req
                .headers()
                .get("x-api-key")
                .and_then(|v| v.to_str().ok())
                .map(str::trim);

            let provided_bearer = req
                .headers()
                .get("authorization")
                .and_then(|v| v.to_str().ok())
                .and_then(|h| {
                    h.strip_prefix("Bearer ")
                        .or_else(|| h.strip_prefix("bearer "))
                })
                .map(str::trim);

            let authorized = provided_x_key == Some(expected.as_str())
                || provided_bearer == Some(expected.as_str());

            if !authorized {
                return Err(AppError::Unauthorized(
                    "Missing or invalid API key in Authorization Bearer or X-API-Key header"
                        .to_string(),
                ));
            }
        }
    }

    Ok(next.run(req).await)
}

#[derive(Clone)]
pub struct RateLimiterState {
    pub limit_rpm: u32,
    windows: Arc<Mutex<HashMap<String, VecDeque<Instant>>>>,
}

impl RateLimiterState {
    pub fn new(limit_rpm: u32) -> Self {
        Self {
            limit_rpm: limit_rpm.max(1),
            windows: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

pub async fn rate_limit_middleware(
    State(limiter): State<RateLimiterState>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    let client_key = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .or_else(|| req.headers().get("x-real-ip").and_then(|v| v.to_str().ok()))
        .unwrap_or("local-client")
        .trim()
        .to_string();

    let now = Instant::now();
    let window = Duration::from_secs(60);

    {
        let mut map = limiter
            .windows
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let queue = map.entry(client_key).or_default();
        while let Some(&front) = queue.front() {
            if now.duration_since(front) > window {
                queue.pop_front();
            } else {
                break;
            }
        }
        if queue.len() >= limiter.limit_rpm as usize {
            return Err(AppError::TooManyRequests(format!(
                "Exceeded rate limit of {} requests per minute",
                limiter.limit_rpm
            )));
        }
        queue.push_back(now);
    }

    Ok(next.run(req).await)
}
