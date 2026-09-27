use super::{auth, db::get_db, error::ApiError, markdown};
use crate::models::post::{Post, PostStatus, PostSummary};
use http::HeaderMap;
use sqlx::{sqlite::SqliteRow, Row};
use uuid::Uuid;
const SELECT: &str = "SELECT p.*, u.username AS author_username, u.display_name AS author_display_name FROM posts p JOIN users u ON u.id=p.author_id";

pub fn reading_time(content: &str) -> u32 {
    content.split_whitespace().count().max(1).div_ceil(200) as u32
}
pub fn pagination(page: u32, per_page: u32) -> (i64, i64) {
    let limit = per_page.clamp(1, 50) as i64;
    (limit, i64::from(page.saturating_sub(1)) * limit)
}
fn validate(title: &str, excerpt: &str, content: &str) -> Result<(), ApiError> {
    if title.trim().is_empty() || title.chars().count() > 200 {
        return Err(ApiError::new("Title must be 1–200 characters."));
    }
    if excerpt.chars().count() > 500 {
        return Err(ApiError::new("Excerpt must be 500 characters or fewer."));
    }
    if content.len() > 250_000 {
        return Err(ApiError::new("Story content must be 250 KB or less."));
    }
    Ok(())
}
fn summary(row: SqliteRow) -> PostSummary {
    PostSummary {
        id: row.get("id"),
        title: row.get("title"),
        slug: row.get("slug"),
        excerpt: row.get("excerpt"),
        author_username: row.get("author_username"),
        author_display_name: row.get("author_display_name"),
        published_at: row.get("published_at"),
        reading_time_minutes: reading_time(&row.get::<String, _>("content")),
    }
}
fn post(row: SqliteRow) -> Post {
    let content: String = row.get("content");
    Post {
        id: row.get("id"),
        author_id: row.get("author_id"),
        title: row.get("title"),
        slug: row.get("slug"),
        excerpt: row.get("excerpt"),
        content_html: markdown::render(&content),
        reading_time_minutes: reading_time(&content),
        content,
        author_username: row.get("author_username"),
        author_display_name: row.get("author_display_name"),
        status: PostStatus::from_str(&row.get::<String, _>("status")),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        published_at: row.get("published_at"),
        version: row.get("version"),
    }
}
pub async fn get_published_posts(
    page: u32,
    per_page: u32,
    query: String,
) -> Result<Vec<PostSummary>, ApiError> {
    if query.chars().count() > 200 {
        return Err(ApiError::new("Search must be 200 characters or fewer."));
    }
    let (limit, offset) = pagination(page, per_page);
    let query = query
        .trim()
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_");
    let pattern = format!("%{query}%");
    let rows = sqlx::query(&format!("{SELECT} WHERE p.status='published' AND (p.title LIKE ?1 ESCAPE '\\' OR p.excerpt LIKE ?1 ESCAPE '\\' OR p.content LIKE ?1 ESCAPE '\\' OR u.display_name LIKE ?1 ESCAPE '\\') ORDER BY p.published_at DESC, p.id DESC LIMIT ?2 OFFSET ?3"))
        .bind(pattern).bind(limit).bind(offset).fetch_all(get_db().await).await.map_err(ApiError::database)?;
    Ok(rows.into_iter().map(summary).collect())
}
pub async fn get_post_by_slug(slug: String) -> Result<Option<Post>, ApiError> {
    Ok(
        sqlx::query(&format!("{SELECT} WHERE p.slug=? AND p.status='published'"))
            .bind(slug)
            .fetch_optional(get_db().await)
            .await
            .map_err(ApiError::database)?
            .map(post),
    )
}
pub async fn get_user_posts(headers: &HeaderMap) -> Result<Vec<PostSummary>, ApiError> {
    let pool = get_db().await;
    let user = auth::require_auth(pool, headers).await?;
    Ok(sqlx::query(&format!(
        "{SELECT} WHERE p.author_id=? ORDER BY p.updated_at DESC, p.id DESC"
    ))
    .bind(user)
    .fetch_all(pool)
    .await
    .map_err(ApiError::database)?
    .into_iter()
    .map(summary)
    .collect())
}
pub async fn get_post_for_edit(headers: &HeaderMap, id: String) -> Result<Post, ApiError> {
    let pool = get_db().await;
    let user = auth::require_auth(pool, headers).await?;
    let row = sqlx::query(&format!("{SELECT} WHERE p.id=?"))
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(ApiError::database)?
        .ok_or_else(ApiError::not_found)?;
    if row.get::<String, _>("author_id") != user {
        return Err(ApiError::forbidden());
    }
    Ok(post(row))
}
pub async fn create_post(
    headers: &HeaderMap,
    title: String,
    excerpt: String,
    content: String,
) -> Result<Post, ApiError> {
    let pool = get_db().await;
    let user = auth::require_auth(pool, headers).await?;
    validate(&title, &excerpt, &content)?;
    let id = Uuid::new_v4().to_string();
    let base = slug::slugify(title.trim());
    let base = if base.is_empty() { "story" } else { &base };
    // Uniqueness is decided by the database, not a check-then-insert race.
    for attempt in 0..4 {
        let slug = if attempt == 0 {
            base.to_string()
        } else {
            format!("{base}-{}", Uuid::new_v4().simple())
        };
        let result = sqlx::query("INSERT INTO posts(id,author_id,title,slug,excerpt,content) VALUES (?,?,?,?,?,?) ON CONFLICT(slug) DO NOTHING")
            .bind(&id).bind(&user).bind(title.trim()).bind(slug).bind(excerpt.trim()).bind(&content)
            .execute(pool).await.map_err(ApiError::database)?;
        if result.rows_affected() == 1 {
            return get_post_for_edit(headers, id).await;
        }
    }
    Err(ApiError::internal(
        "Could not reserve a story URL. Please try again.",
    ))
}
pub async fn update_post(
    headers: &HeaderMap,
    id: String,
    title: String,
    excerpt: String,
    content: String,
    version: Option<i64>,
) -> Result<(), ApiError> {
    let pool = get_db().await;
    let user = auth::require_auth(pool, headers).await?;
    validate(&title, &excerpt, &content)?;
    require_owner(&user, &id).await?;
    let version = version.ok_or(ApiError {
        status: http::StatusCode::PRECONDITION_REQUIRED,
        message: "Reload the story before saving; a version is required.".into(),
    })?;
    let result = sqlx::query("UPDATE posts SET title=?,excerpt=?,content=?,updated_at=datetime('now'),version=version+1 WHERE id=? AND author_id=? AND version=?")
        .bind(title.trim()).bind(excerpt.trim()).bind(content).bind(id).bind(user).bind(version).execute(pool).await.map_err(ApiError::database)?;
    if result.rows_affected() == 0 {
        return Err(ApiError::conflict());
    }
    Ok(())
}
async fn require_owner(user: &str, id: &str) -> Result<(), ApiError> {
    let owner: Option<String> = sqlx::query_scalar("SELECT author_id FROM posts WHERE id=?")
        .bind(id)
        .fetch_optional(get_db().await)
        .await
        .map_err(ApiError::database)?;
    match owner {
        Some(owner) if owner == user => Ok(()),
        Some(_) => Err(ApiError::forbidden()),
        None => Err(ApiError::not_found()),
    }
}
pub async fn publish_post(headers: &HeaderMap, id: String) -> Result<(), ApiError> {
    let pool = get_db().await;
    let user = auth::require_auth(pool, headers).await?;
    require_owner(&user, &id).await?;
    let result = sqlx::query("UPDATE posts SET status='published',published_at=datetime('now'),updated_at=datetime('now'),version=version+1 WHERE id=? AND author_id=? AND status='draft'")
        .bind(&id).bind(&user).execute(pool).await.map_err(ApiError::database)?;
    if result.rows_affected() == 1 {
        tokio::spawn(async move {
            let row: Option<(String,String,String)> = sqlx::query_as("SELECT p.title,p.slug,u.email FROM posts p JOIN users u ON u.id=p.author_id WHERE p.id=?")
                .bind(id).fetch_optional(pool).await.ok().flatten();
            if let Some((title, slug, email)) = row {
                let _ = super::email::send_post_published_email(pool, &user, &email, &title, &slug)
                    .await;
            }
        });
    }
    Ok(())
}
pub async fn unpublish_post(headers: &HeaderMap, id: String) -> Result<(), ApiError> {
    let pool = get_db().await;
    let user = auth::require_auth(pool, headers).await?;
    require_owner(&user, &id).await?;
    sqlx::query("UPDATE posts SET status='draft',published_at=NULL,updated_at=datetime('now'),version=version+1 WHERE id=? AND author_id=? AND status='published'")
        .bind(id).bind(user).execute(pool).await.map_err(ApiError::database)?;
    Ok(())
}
pub async fn delete_post(headers: &HeaderMap, id: String) -> Result<(), ApiError> {
    let pool = get_db().await;
    let user = auth::require_auth(pool, headers).await?;
    require_owner(&user, &id).await?;
    sqlx::query("DELETE FROM posts WHERE id=? AND author_id=?")
        .bind(id)
        .bind(user)
        .execute(pool)
        .await
        .map_err(ApiError::database)?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pagination_is_bounded_and_cannot_overflow() {
        assert_eq!(pagination(0, 0), (1, 0));
        assert_eq!(pagination(2, 500), (50, 50));
        assert_eq!(pagination(u32::MAX, u32::MAX), (50, 214748364700));
    }
}
