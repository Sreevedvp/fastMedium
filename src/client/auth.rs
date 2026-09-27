use super::{get, request};
use crate::{
    api_types::{LoginRequest, RegisterRequest},
    models::user::User,
};
use reqwest::Method;
pub async fn get_current_user() -> Result<Option<User>, String> {
    get("/api/auth/me").await
}
pub async fn login(email: String, password: String) -> Result<User, String> {
    request(
        Method::POST,
        "/api/auth/login",
        Some(LoginRequest { email, password }),
    )
    .await
}
pub async fn register(
    username: String,
    email: String,
    password: String,
    display_name: String,
) -> Result<User, String> {
    request(
        Method::POST,
        "/api/auth/register",
        Some(RegisterRequest {
            username,
            email,
            password,
            display_name,
        }),
    )
    .await
}
pub async fn logout() -> Result<(), String> {
    request::<(), ()>(Method::POST, "/api/auth/logout", None).await
}
