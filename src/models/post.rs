use serde::{Deserialize, Serialize};

/// Post status enum shared between client and server.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PostStatus {
    Draft,
    Published,
}

impl PostStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            PostStatus::Draft => "draft",
            PostStatus::Published => "published",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "published" => PostStatus::Published,
            _ => PostStatus::Draft,
        }
    }
}

/// Full post data returned to the client.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Post {
    pub version: i64,
    pub id: String,
    pub author_id: String,
    pub author_username: String,
    pub author_display_name: String,
    pub title: String,
    pub slug: String,
    pub excerpt: String,
    pub content: String,
    pub content_html: String,
    pub status: PostStatus,
    pub created_at: String,
    pub updated_at: String,
    pub published_at: Option<String>,
    pub reading_time_minutes: u32,
}

/// Lightweight post info for feed listings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PostSummary {
    pub id: String,
    pub title: String,
    pub slug: String,
    pub excerpt: String,
    pub author_username: String,
    pub author_display_name: String,
    pub published_at: Option<String>,
    pub reading_time_minutes: u32,
}
