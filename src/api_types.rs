use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}
#[derive(Serialize, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub display_name: String,
}
#[derive(Serialize, Deserialize)]
pub struct PostRequest {
    pub title: String,
    pub excerpt: String,
    pub content: String,
}
#[derive(Serialize, Deserialize)]
pub struct ApiErrorBody {
    pub error: String,
}

#[derive(Serialize, Deserialize)]
pub struct SkillsRequest {
    pub skills: Vec<String>,
}
#[derive(Serialize, Deserialize)]
pub struct PublicationRequest {
    pub name: String,
    pub description: String,
}

#[derive(Serialize, Deserialize)]
pub struct UpdatePostRequest {
    pub title: String,
    pub excerpt: String,
    pub content: String,
    pub version: Option<i64>,
}
