pub mod auth;
pub mod posts;
pub mod profiles;
use crate::api_types::ApiErrorBody;
use serde::{de::DeserializeOwned, Serialize};

fn base_url() -> String {
    if let Some(url) = option_env!("FASTMEDIUM_API_URL") {
        return url.trim_end_matches('/').to_string();
    }
    #[cfg(target_arch = "wasm32")]
    let host = web_sys::window()
        .and_then(|w| w.location().hostname().ok())
        .unwrap_or_else(|| "127.0.0.1".into());
    #[cfg(not(target_arch = "wasm32"))]
    let host = "127.0.0.1";
    let port = option_env!("FASTMEDIUM_API_PORT").unwrap_or("8082");
    format!("http://{host}:{port}")
}

pub async fn request<T: DeserializeOwned, B: Serialize>(
    method: reqwest::Method,
    path: &str,
    body: Option<B>,
) -> Result<T, String> {
    let request = reqwest::Client::new()
        .request(method, format!("{}{path}", base_url()))
        .timeout(std::time::Duration::from_secs(25));
    #[cfg(target_arch = "wasm32")]
    let request = request.fetch_credentials_include();
    let request = if let Some(body) = body {
        request.json(&body)
    } else {
        request
    };
    let response = request.send().await.map_err(|_| {
        "Cannot reach the server. Your text is still here. Check your connection and the latest saved state before retrying.".to_string()
    })?;
    if !response.status().is_success() {
        let status = response.status();
        return Err(response
            .json::<ApiErrorBody>()
            .await
            .map(|e| e.error)
            .unwrap_or_else(|_| format!("Request failed ({status})")));
    }
    response
        .json()
        .await
        .map_err(|_| "The API returned an unexpected response.".to_string())
}

pub async fn get<T: DeserializeOwned>(path: &str) -> Result<T, String> {
    request::<T, ()>(reqwest::Method::GET, path, None).await
}
// URL-encode a dynamic path segment without adding another dependency.
pub fn segment(value: &str) -> String {
    value
        .bytes()
        .map(|b| {
            if b.is_ascii_alphanumeric() || b"-._~".contains(&b) {
                (b as char).to_string()
            } else {
                format!("%{b:02X}")
            }
        })
        .collect()
}

pub mod publications;
