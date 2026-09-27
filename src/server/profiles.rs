use super::error::ApiError;

use super::db::get_db;
use crate::models::post::PostSummary;
use crate::models::user::User;

/// Get a public user profile by username.
pub async fn get_profile(username: String) -> Result<Option<User>, ApiError> {
    let pool = get_db().await;

    let row: Option<(
        String,
        String,
        String,
        String,
        String,
        Option<String>,
        String,
    )> = sqlx::query_as(
        "SELECT id, username, display_name, email, bio, avatar_url, created_at
             FROM users WHERE username = ?1",
    )
    .bind(&username)
    .fetch_optional(pool)
    .await
    .map_err(ApiError::database)?;

    match row {
        Some((id, username, display_name, _email, bio, avatar_url, created_at)) => Ok(Some(User {
            id,
            username,
            display_name,
            email: String::new(),
            bio,
            avatar_url,
            created_at,
        })),
        None => Ok(None),
    }
}

/// Get published posts by a specific author.
pub async fn get_author_posts(username: String, page: u32) -> Result<Vec<PostSummary>, ApiError> {
    let pool = get_db().await;

    let per_page: u32 = 10;
    let (_, offset) = super::posts::pagination(page, per_page);

    let rows: Vec<(String, String, String, String, String, String, String, Option<String>)> =
        sqlx::query_as(
            "SELECT p.id, p.title, p.slug, p.excerpt, p.content, u.username, u.display_name, p.published_at
             FROM posts p
             JOIN users u ON p.author_id = u.id
             WHERE u.username = ?1 AND p.status = 'published'
             ORDER BY p.published_at DESC, p.id DESC
             LIMIT ?2 OFFSET ?3",
        )
        .bind(&username)
        .bind(per_page as i64)
        .bind(offset as i64)
        .fetch_all(pool)
        .await
        .map_err(ApiError::database)?;

    let summaries = rows
        .into_iter()
        .map(
            |(
                id,
                title,
                slug,
                excerpt,
                content,
                author_username,
                author_display_name,
                published_at,
            )| {
                PostSummary {
                    id,
                    title,
                    slug,
                    excerpt,
                    author_username,
                    author_display_name,
                    published_at,
                    reading_time_minutes: super::posts::reading_time(&content),
                }
            },
        )
        .collect();

    Ok(summaries)
}

pub async fn get_skills(username: String) -> Result<Vec<String>, ApiError> {
    sqlx::query_scalar("SELECT s.skill FROM user_skills s JOIN users u ON u.id = s.user_id WHERE u.username = ?1 ORDER BY s.position")
        .bind(username).fetch_all(get_db().await).await.map_err(|_| ApiError::internal("Could not load skills."))
}

pub async fn set_skills(
    headers: &http::HeaderMap,
    skills: Vec<String>,
) -> Result<Vec<String>, ApiError> {
    let pool = get_db().await;
    let user = super::auth::require_auth(pool, headers).await?;
    if skills.len() > 12 {
        return Err(ApiError::new("Add up to 12 skills."));
    }
    let mut normalized: Vec<String> = Vec::new();
    for skill in skills {
        let skill = skill.split_whitespace().collect::<Vec<_>>().join(" ");
        if skill.is_empty() || skill.chars().count() > 40 {
            return Err(ApiError::new("Each skill must be 1–40 characters."));
        }
        if !normalized
            .iter()
            .any(|s| s.to_lowercase() == skill.to_lowercase())
        {
            normalized.push(skill);
        }
    }
    let mut tx = pool
        .begin()
        .await
        .map_err(|_| ApiError::internal("Could not save skills."))?;
    sqlx::query("DELETE FROM user_skills WHERE user_id = ?1")
        .bind(&user)
        .execute(&mut *tx)
        .await
        .map_err(|_| ApiError::internal("Could not save skills."))?;
    for (index, skill) in normalized.iter().enumerate() {
        sqlx::query("INSERT INTO user_skills(user_id, skill, position) VALUES (?1, ?2, ?3)")
            .bind(&user)
            .bind(skill)
            .bind(index as i64)
            .execute(&mut *tx)
            .await
            .map_err(|_| ApiError::internal("Could not save skills."))?;
    }
    tx.commit()
        .await
        .map_err(|_| ApiError::internal("Could not save skills."))?;
    Ok(normalized)
}
