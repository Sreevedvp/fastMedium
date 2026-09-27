use super::{get, request, segment};
use crate::{
    api_types::PublicationRequest,
    models::publication::{Publication, PublicationDetail},
};
use reqwest::Method;
pub async fn list() -> Result<Vec<Publication>, String> {
    get("/api/publications").await
}
pub async fn detail(slug: String) -> Result<PublicationDetail, String> {
    get(&format!("/api/publications/by-slug/{}", segment(&slug))).await
}
pub async fn create(name: String, description: String) -> Result<Publication, String> {
    request(
        Method::POST,
        "/api/publications",
        Some(PublicationRequest { name, description }),
    )
    .await
}
pub async fn update(id: String, name: String, description: String) -> Result<(), String> {
    request(
        Method::PUT,
        &format!("/api/publications/{}", segment(&id)),
        Some(PublicationRequest { name, description }),
    )
    .await
}
pub async fn delete(id: String) -> Result<(), String> {
    request::<(), ()>(
        Method::DELETE,
        &format!("/api/publications/{}", segment(&id)),
        None,
    )
    .await
}
pub async fn attach(id: String, post: String) -> Result<(), String> {
    request::<(), ()>(
        Method::POST,
        &format!(
            "/api/publications/{}/posts/{}",
            segment(&id),
            segment(&post)
        ),
        None,
    )
    .await
}
pub async fn detach(id: String, post: String) -> Result<(), String> {
    request::<(), ()>(
        Method::DELETE,
        &format!(
            "/api/publications/{}/posts/{}",
            segment(&id),
            segment(&post)
        ),
        None,
    )
    .await
}
