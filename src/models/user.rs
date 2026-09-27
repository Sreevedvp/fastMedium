use serde::{Deserialize, Serialize};

/// Public user profile — never contains password_hash.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub display_name: String,
    pub email: String,
    pub bio: String,
    pub avatar_url: Option<String>,
    pub created_at: String,
}

/// Minimal user info embedded in post listings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuthorInfo {
    pub username: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
}
