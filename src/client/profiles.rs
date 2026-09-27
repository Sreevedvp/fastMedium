use super::{get, segment};
use crate::models::{post::PostSummary, user::User};
pub async fn get_profile(username: String) -> Result<Option<User>, String> {
    get(&format!("/api/profiles/{}", segment(&username))).await
}
pub async fn get_author_posts(username: String, page: u32) -> Result<Vec<PostSummary>, String> {
    get(&format!(
        "/api/profiles/{}/posts?page={page}",
        segment(&username)
    ))
    .await
}

pub async fn get_skills(username: String) -> Result<Vec<String>, String> {
    get(&format!("/api/profiles/{}/skills", segment(&username))).await
}
pub async fn set_skills(skills: Vec<String>) -> Result<Vec<String>, String> {
    super::request(
        reqwest::Method::PUT,
        "/api/me/skills",
        Some(crate::api_types::SkillsRequest { skills }),
    )
    .await
}
