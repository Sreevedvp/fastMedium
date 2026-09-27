use super::{auth, db::get_db, error::ApiError};
use crate::{
    api_types::PublicationRequest,
    models::{
        post::PostSummary,
        publication::{Publication, PublicationDetail},
    },
};
use http::HeaderMap;
use sqlx::Row;
use uuid::Uuid;
const SELECT: &str = "SELECT p.id, p.owner_id, u.username, p.name, p.slug, p.description, (SELECT COUNT(*) FROM publication_posts pp JOIN posts s ON s.id = pp.post_id WHERE pp.publication_id = p.id AND s.status = 'published') AS story_count FROM publications p JOIN users u ON u.id = p.owner_id";
fn from_row(row: sqlx::sqlite::SqliteRow) -> Publication {
    Publication {
        id: row.get("id"),
        owner_id: row.get("owner_id"),
        owner_username: row.get("username"),
        name: row.get("name"),
        slug: row.get("slug"),
        description: row.get("description"),
        story_count: row.get("story_count"),
    }
}
fn database_error(_: sqlx::Error) -> ApiError {
    ApiError::internal("Could not save or load publication data. Please try again.")
}
fn validate(body: &PublicationRequest) -> Result<(), ApiError> {
    if !(3..=80).contains(&body.name.trim().chars().count()) {
        return Err(ApiError::new("Publication name must be 3–80 characters."));
    }
    if body.description.chars().count() > 500 {
        return Err(ApiError::new(
            "Description must be 500 characters or fewer.",
        ));
    }
    Ok(())
}
pub async fn list() -> Result<Vec<Publication>, ApiError> {
    let rows = sqlx::query(&format!("{SELECT} ORDER BY p.created_at DESC, p.name"))
        .fetch_all(get_db().await)
        .await
        .map_err(database_error)?;
    Ok(rows.into_iter().map(from_row).collect())
}
pub async fn create(
    headers: &HeaderMap,
    body: PublicationRequest,
) -> Result<Publication, ApiError> {
    let pool = get_db().await;
    let owner = auth::require_auth(pool, headers).await?;
    validate(&body)?;
    let id = Uuid::new_v4().to_string();
    let base = slug::slugify(body.name.trim());
    let slug = format!(
        "{}-{}",
        if base.is_empty() {
            "publication"
        } else {
            &base
        },
        &id[..8]
    );
    sqlx::query("INSERT INTO publications(id, owner_id, name, slug, description) VALUES (?1, ?2, ?3, ?4, ?5)")
        .bind(&id).bind(&owner).bind(body.name.trim()).bind(&slug).bind(body.description.trim()).execute(pool).await.map_err(database_error)?;
    load(&slug).await
}
async fn load(slug: &str) -> Result<Publication, ApiError> {
    let row = sqlx::query(&format!("{SELECT} WHERE p.slug = ?1"))
        .bind(slug)
        .fetch_optional(get_db().await)
        .await
        .map_err(database_error)?;
    row.map(from_row).ok_or(ApiError {
        status: http::StatusCode::NOT_FOUND,
        message: "Publication not found.".into(),
    })
}
async fn require_owner(headers: &HeaderMap, id: &str) -> Result<String, ApiError> {
    let pool = get_db().await;
    let user = auth::require_auth(pool, headers).await?;
    let owner: Option<(String,)> =
        sqlx::query_as("SELECT owner_id FROM publications WHERE id = ?1")
            .bind(id)
            .fetch_optional(pool)
            .await
            .map_err(database_error)?;
    match owner {
        Some((owner,)) if owner == user => Ok(user),
        Some(_) => Err(ApiError {
            status: http::StatusCode::FORBIDDEN,
            message: "Only the publication owner can change this publication.".into(),
        }),
        None => Err(ApiError {
            status: http::StatusCode::NOT_FOUND,
            message: "Publication not found.".into(),
        }),
    }
}
pub async fn detail(headers: &HeaderMap, slug: String) -> Result<PublicationDetail, ApiError> {
    let publication = load(&slug).await?;
    let can_manage = auth::get_current_user(headers)
        .await?
        .is_some_and(|u| u.id == publication.owner_id);
    let rows = sqlx::query("SELECT p.id, p.title, p.slug, p.excerpt, p.content, p.published_at, u.username, u.display_name FROM publication_posts pp JOIN posts p ON p.id = pp.post_id JOIN users u ON u.id = p.author_id WHERE pp.publication_id = ?1 AND (p.status = 'published' OR ?2) ORDER BY p.created_at DESC")
        .bind(&publication.id).bind(can_manage).fetch_all(get_db().await).await.map_err(database_error)?;
    let posts = rows
        .into_iter()
        .map(|r| PostSummary {
            id: r.get("id"),
            title: r.get("title"),
            slug: r.get("slug"),
            excerpt: r.get("excerpt"),
            author_username: r.get("username"),
            author_display_name: r.get("display_name"),
            published_at: r.get("published_at"),
            reading_time_minutes: ((r
                .get::<String, _>("content")
                .split_whitespace()
                .count()
                .max(1)
                + 199)
                / 200) as u32,
        })
        .collect();
    Ok(PublicationDetail {
        publication,
        posts,
        can_manage,
    })
}
pub async fn update(
    headers: &HeaderMap,
    id: String,
    body: PublicationRequest,
) -> Result<(), ApiError> {
    require_owner(headers, &id).await?;
    validate(&body)?;
    sqlx::query("UPDATE publications SET name = ?1, description = ?2 WHERE id = ?3")
        .bind(body.name.trim())
        .bind(body.description.trim())
        .bind(id)
        .execute(get_db().await)
        .await
        .map_err(database_error)?;
    Ok(())
}
pub async fn delete(headers: &HeaderMap, id: String) -> Result<(), ApiError> {
    require_owner(headers, &id).await?;
    // Cascades only collection links. Authored stories remain in the writer's workspace.
    sqlx::query("DELETE FROM publications WHERE id = ?1")
        .bind(id)
        .execute(get_db().await)
        .await
        .map_err(database_error)?;
    Ok(())
}
pub async fn attach(headers: &HeaderMap, id: String, post_id: String) -> Result<(), ApiError> {
    let owner = require_owner(headers, &id).await?;
    let pool = get_db().await;
    let post_owner: Option<(String,)> = sqlx::query_as("SELECT author_id FROM posts WHERE id = ?1")
        .bind(&post_id)
        .fetch_optional(pool)
        .await
        .map_err(database_error)?;
    if !post_owner.is_some_and(|(author,)| author == owner) {
        return Err(ApiError {
            status: http::StatusCode::FORBIDDEN,
            message: "You can only add your own stories.".into(),
        });
    }
    sqlx::query("INSERT INTO publication_posts(post_id, publication_id) VALUES (?1, ?2) ON CONFLICT(post_id) DO UPDATE SET publication_id = excluded.publication_id")
        .bind(post_id).bind(id).execute(pool).await.map_err(database_error)?;
    Ok(())
}
pub async fn detach(headers: &HeaderMap, id: String, post_id: String) -> Result<(), ApiError> {
    require_owner(headers, &id).await?;
    sqlx::query("DELETE FROM publication_posts WHERE publication_id = ?1 AND post_id = ?2")
        .bind(id)
        .bind(post_id)
        .execute(get_db().await)
        .await
        .map_err(database_error)?;
    Ok(())
}
