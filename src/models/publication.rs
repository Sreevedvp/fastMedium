use super::post::PostSummary;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Publication {
    pub id: String,
    pub owner_id: String,
    pub owner_username: String,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub story_count: i64,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PublicationDetail {
    pub publication: Publication,
    pub posts: Vec<PostSummary>,
    pub can_manage: bool,
}
