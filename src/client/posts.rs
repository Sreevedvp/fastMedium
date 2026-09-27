use super::{get, request, segment};
use crate::{
    api_types::{PostRequest, UpdatePostRequest},
    models::post::{Post, PostSummary},
};
use reqwest::Method;
pub async fn get_published_posts(
    page: u32,
    per_page: u32,
    query: String,
) -> Result<Vec<PostSummary>, String> {
    get(&format!(
        "/api/posts?page={page}&per_page={per_page}&q={}",
        segment(&query)
    ))
    .await
}
pub async fn get_post_by_slug(slug: String) -> Result<Option<Post>, String> {
    get(&format!("/api/posts/by-slug/{}", segment(&slug))).await
}
pub async fn get_user_posts() -> Result<Vec<PostSummary>, String> {
    get("/api/posts/mine").await
}
pub async fn get_post_for_edit(id: String) -> Result<Post, String> {
    get(&format!("/api/posts/{}", segment(&id))).await
}
pub async fn create_post(title: String, excerpt: String, content: String) -> Result<Post, String> {
    request(
        Method::POST,
        "/api/posts",
        Some(PostRequest {
            title,
            excerpt,
            content,
        }),
    )
    .await
}
pub async fn update_post(
    id: String,
    title: String,
    excerpt: String,
    content: String,
    version: i64,
) -> Result<(), String> {
    request(
        Method::PUT,
        &format!("/api/posts/{}", segment(&id)),
        Some(UpdatePostRequest {
            title,
            excerpt,
            content,
            version: Some(version),
        }),
    )
    .await
}
pub async fn delete_post(id: String) -> Result<(), String> {
    request::<(), ()>(
        Method::DELETE,
        &format!("/api/posts/{}", segment(&id)),
        None,
    )
    .await
}
pub async fn publish_post(id: String) -> Result<(), String> {
    request::<(), ()>(
        Method::POST,
        &format!("/api/posts/{}/publish", segment(&id)),
        None,
    )
    .await
}
pub async fn unpublish_post(id: String) -> Result<(), String> {
    request::<(), ()>(
        Method::POST,
        &format!("/api/posts/{}/unpublish", segment(&id)),
        None,
    )
    .await
}
