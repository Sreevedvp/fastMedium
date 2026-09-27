use super::{db, error::ApiError};
use axum::{
    extract::{ConnectInfo, MatchedPath, Request, State},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use http::{header, HeaderValue, Method, StatusCode};
use std::{
    collections::HashMap,
    net::{IpAddr, SocketAddr},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

#[derive(Clone)]
pub struct Boundary {
    pub origins: Vec<HeaderValue>,
    attempts: Arc<Mutex<HashMap<IpAddr, (Instant, u32)>>>,
    auth_limit: u32,
}
impl Boundary {
    pub fn new(web_port: u16) -> Result<Self, String> {
        let configured = std::env::var("ALLOWED_ORIGINS")
            .unwrap_or_else(|_| format!("http://127.0.0.1:{web_port},http://localhost:{web_port}"));
        let mut origins: Vec<HeaderValue> = Vec::new();
        for origin in configured.split(',').map(str::trim) {
            let uri: http::Uri = origin.parse().map_err(|_| "Invalid ALLOWED_ORIGINS")?;
            if !matches!(uri.scheme_str(), Some("http" | "https"))
                || uri.authority().is_none()
                || uri.authority().is_some_and(|a| a.as_str().contains('@'))
                || uri.path_and_query().is_some_and(|p| p.as_str() != "/")
            {
                return Err(
                    "ALLOWED_ORIGINS must contain exact http(s) origins without paths".into(),
                );
            }
            origins.push(
                origin
                    .trim_end_matches('/')
                    .parse()
                    .map_err(|_| "Invalid origin header")?,
            );
        }
        let auth_limit = std::env::var("AUTH_ATTEMPTS_PER_MINUTE")
            .unwrap_or_else(|_| "30".into())
            .parse::<u32>()
            .map_err(|_| "AUTH_ATTEMPTS_PER_MINUTE must be 1–300")?;
        if !(1..=300).contains(&auth_limit) {
            return Err("AUTH_ATTEMPTS_PER_MINUTE must be 1–300".into());
        }
        match std::env::var("COOKIE_SECURE").as_deref() {
            Ok("true" | "false") | Err(_) => {}
            _ => return Err("COOKIE_SECURE must be true or false".into()),
        }
        if origins
            .iter()
            .any(|o| o.to_str().is_ok_and(|o| o.starts_with("https://")))
            && std::env::var("COOKIE_SECURE").as_deref() != Ok("true")
        {
            return Err("HTTPS origins require COOKIE_SECURE=true".into());
        }
        Ok(Self {
            origins,
            attempts: Arc::new(Mutex::new(HashMap::new())),
            auth_limit,
        })
    }
    fn admit(&self, peer: IpAddr) -> bool {
        let Ok(mut attempts) = self.attempts.lock() else {
            return false;
        };
        let now = Instant::now();
        attempts.retain(|_, (start, _)| now.duration_since(*start) < Duration::from_secs(60));
        if attempts.len() >= 4096 && !attempts.contains_key(&peer) {
            return false;
        }
        let (_, count) = attempts.entry(peer).or_insert((now, 0));
        if *count >= self.auth_limit {
            return false;
        }
        *count += 1;
        true
    }
}

pub async fn guard(State(boundary): State<Boundary>, request: Request, next: Next) -> Response {
    let id = uuid::Uuid::new_v4().to_string();
    let start = Instant::now();
    let method = request.method().clone();
    // Log route templates only, never slugs, email addresses, search terms, cookies or bodies.
    let route = request
        .extensions()
        .get::<MatchedPath>()
        .map(|p| p.as_str().to_string())
        .unwrap_or_else(|| "unmatched".into());
    let mutation = !matches!(method, Method::GET | Method::HEAD | Method::OPTIONS);
    let foreign = request
        .headers()
        .get(header::ORIGIN)
        .is_some_and(|origin| !boundary.origins.contains(origin));
    let cross_site = request
        .headers()
        .get("sec-fetch-site")
        .is_some_and(|v| v == "cross-site");
    let auth = matches!(
        request.uri().path(),
        "/api/auth/login" | "/api/auth/register"
    );
    let peer = request
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|p| p.0.ip())
        .unwrap_or(IpAddr::V4(std::net::Ipv4Addr::LOCALHOST));
    let mut response = if mutation && (foreign || cross_site) {
        ApiError {
            status: StatusCode::FORBIDDEN,
            message: "This origin is not allowed to change data.".into(),
        }
        .into_response()
    } else if mutation && auth && !boundary.admit(peer) {
        let mut response = ApiError::busy().into_response();
        response
            .headers_mut()
            .insert(header::RETRY_AFTER, HeaderValue::from_static("60"));
        response
    } else {
        match tokio::time::timeout(Duration::from_secs(20), next.run(request)).await {
            Ok(response) => response,
            Err(_) => ApiError {
                status: StatusCode::GATEWAY_TIMEOUT,
                message: "The request timed out. Check the latest state before retrying a save."
                    .into(),
            }
            .into_response(),
        }
    };
    if response.status().is_client_error()
        && !response
            .headers()
            .get(header::CONTENT_TYPE)
            .is_some_and(|h| h.to_str().is_ok_and(|v| v.starts_with("application/json")))
    {
        let status = response.status();
        response = ApiError {
            status,
            message: match status {
                StatusCode::PAYLOAD_TOO_LARGE => "Request body is too large.",
                StatusCode::NOT_FOUND => "API route not found.",
                StatusCode::METHOD_NOT_ALLOWED => "Method not allowed.",
                _ => "Invalid request. Check the supplied fields and values.",
            }
            .into(),
        }
        .into_response();
    }
    let status = response.status().as_u16();
    response.headers_mut().insert(
        "x-request-id",
        HeaderValue::from_str(&id).expect("UUID header"),
    );
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response.headers_mut().insert(
        "x-content-type-options",
        HeaderValue::from_static("nosniff"),
    );
    response.headers_mut().insert(
        "referrer-policy",
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );
    eprintln!(
        "request_id={id} method={method} route={route} status={status} elapsed_ms={}",
        start.elapsed().as_millis()
    );
    response
}
pub async fn health() -> Result<Json<serde_json::Value>, ApiError> {
    match tokio::time::timeout(
        Duration::from_secs(2),
        sqlx::query("SELECT 1").execute(db::get_db().await),
    )
    .await
    {
        Ok(Ok(_)) => Ok(Json(serde_json::json!({"status":"ok"}))),
        _ => Err(ApiError {
            status: StatusCode::SERVICE_UNAVAILABLE,
            message: "Database is not ready.".into(),
        }),
    }
}
