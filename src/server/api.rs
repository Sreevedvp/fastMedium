use super::{auth, error::ApiError, posts, profiles, publications};
use crate::{
    api_types::*,
    models::{
        post::{Post, PostSummary},
        user::User,
    },
};
use axum::{
    extract::{DefaultBodyLimit, Path, Query},
    http::{header, HeaderMap, HeaderValue, Method},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use tower_http::cors::CorsLayer;

pub fn router(web_port: u16) -> Result<Router, String> {
    let boundary = super::security::Boundary::new(web_port)?;
    let origins = boundary.origins.clone();
    Ok(Router::new()
        .route("/api/health", get(super::security::health))
        .route("/api/auth/register", post(register))
        .route("/api/auth/login", post(login))
        .route("/api/auth/logout", post(logout))
        .route("/api/auth/me", get(me))
        .route("/api/posts", get(feed).post(create))
        .route("/api/posts/mine", get(mine))
        .route("/api/posts/by-slug/{slug}", get(by_slug))
        .route("/api/posts/{id}", get(for_edit).put(update).delete(remove))
        .route("/api/posts/{id}/publish", post(publish))
        .route("/api/posts/{id}/unpublish", post(unpublish))
        .route("/api/profiles/{username}", get(profile))
        .route("/api/profiles/{username}/posts", get(author_posts))
        .route("/api/profiles/{username}/skills", get(profile_skills))
        .route("/api/me/skills", axum::routing::put(save_skills))
        .route(
            "/api/publications",
            get(publication_list).post(publication_create),
        )
        .route("/api/publications/by-slug/{slug}", get(publication_detail))
        .route(
            "/api/publications/{id}",
            axum::routing::put(publication_update).delete(publication_delete),
        )
        .route(
            "/api/publications/{id}/posts/{post_id}",
            post(publication_attach).delete(publication_detach),
        )
        .layer(axum::middleware::from_fn_with_state(
            boundary,
            super::security::guard,
        ))
        .layer(DefaultBodyLimit::max(1024 * 1024))
        .layer(
            CorsLayer::new()
                .allow_origin(origins)
                .allow_credentials(true)
                .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
                .allow_headers([header::CONTENT_TYPE])
                .expose_headers([
                    http::HeaderName::from_static("x-request-id"),
                    header::RETRY_AFTER,
                ]),
        ))
}

fn session_response(user: User, cookie: String) -> Result<(HeaderMap, Json<User>), ApiError> {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::SET_COOKIE,
        cookie
            .parse()
            .map_err(|_| ApiError::internal("Invalid session cookie"))?,
    );
    Ok((headers, Json(user)))
}
async fn register(Json(body): Json<RegisterRequest>) -> Result<(HeaderMap, Json<User>), ApiError> {
    let (user, cookie) =
        auth::register(body.username, body.email, body.password, body.display_name).await?;
    session_response(user, cookie)
}
async fn login(Json(body): Json<LoginRequest>) -> Result<(HeaderMap, Json<User>), ApiError> {
    let (user, cookie) = auth::login(body.email, body.password).await?;
    session_response(user, cookie)
}
async fn logout(headers: HeaderMap) -> Result<(HeaderMap, Json<()>), ApiError> {
    auth::logout(&headers).await?;
    let mut response = HeaderMap::new();
    response.insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&auth::session_cookie("", 0))
            .map_err(|_| ApiError::internal("Invalid cookie"))?,
    );
    Ok((response, Json(())))
}
async fn me(headers: HeaderMap) -> Result<Json<Option<User>>, ApiError> {
    Ok(Json(auth::get_current_user(&headers).await?))
}
#[derive(Deserialize)]
struct Pagination {
    page: Option<u32>,
    per_page: Option<u32>,
    q: Option<String>,
}
async fn feed(Query(p): Query<Pagination>) -> Result<Json<Vec<PostSummary>>, ApiError> {
    Ok(Json(
        posts::get_published_posts(
            p.page.unwrap_or(1),
            p.per_page.unwrap_or(20),
            p.q.unwrap_or_default(),
        )
        .await?,
    ))
}
async fn by_slug(Path(slug): Path<String>) -> Result<Json<Option<Post>>, ApiError> {
    Ok(Json(posts::get_post_by_slug(slug).await?))
}
async fn mine(headers: HeaderMap) -> Result<Json<Vec<PostSummary>>, ApiError> {
    Ok(Json(posts::get_user_posts(&headers).await?))
}
async fn create(headers: HeaderMap, Json(p): Json<PostRequest>) -> Result<Json<Post>, ApiError> {
    Ok(Json(
        posts::create_post(&headers, p.title, p.excerpt, p.content).await?,
    ))
}
async fn for_edit(headers: HeaderMap, Path(id): Path<String>) -> Result<Json<Post>, ApiError> {
    Ok(Json(posts::get_post_for_edit(&headers, id).await?))
}
async fn update(
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(p): Json<UpdatePostRequest>,
) -> Result<Json<()>, ApiError> {
    posts::update_post(&headers, id, p.title, p.excerpt, p.content, p.version).await?;
    Ok(Json(()))
}
async fn remove(headers: HeaderMap, Path(id): Path<String>) -> Result<Json<()>, ApiError> {
    posts::delete_post(&headers, id).await?;
    Ok(Json(()))
}
async fn publish(headers: HeaderMap, Path(id): Path<String>) -> Result<Json<()>, ApiError> {
    posts::publish_post(&headers, id).await?;
    Ok(Json(()))
}
async fn unpublish(headers: HeaderMap, Path(id): Path<String>) -> Result<Json<()>, ApiError> {
    posts::unpublish_post(&headers, id).await?;
    Ok(Json(()))
}
async fn profile(Path(username): Path<String>) -> Result<Json<Option<User>>, ApiError> {
    Ok(Json(profiles::get_profile(username).await?))
}
async fn author_posts(
    Path(username): Path<String>,
    Query(p): Query<Pagination>,
) -> Result<Json<Vec<PostSummary>>, ApiError> {
    Ok(Json(
        profiles::get_author_posts(username, p.page.unwrap_or(1)).await?,
    ))
}

async fn profile_skills(Path(username): Path<String>) -> Result<Json<Vec<String>>, ApiError> {
    Ok(Json(profiles::get_skills(username).await?))
}
async fn save_skills(
    headers: HeaderMap,
    Json(body): Json<SkillsRequest>,
) -> Result<Json<Vec<String>>, ApiError> {
    Ok(Json(profiles::set_skills(&headers, body.skills).await?))
}
async fn publication_list() -> Result<Json<Vec<crate::models::publication::Publication>>, ApiError>
{
    Ok(Json(publications::list().await?))
}
async fn publication_create(
    headers: HeaderMap,
    Json(body): Json<PublicationRequest>,
) -> Result<Json<crate::models::publication::Publication>, ApiError> {
    Ok(Json(publications::create(&headers, body).await?))
}
async fn publication_detail(
    headers: HeaderMap,
    Path(slug): Path<String>,
) -> Result<Json<crate::models::publication::PublicationDetail>, ApiError> {
    Ok(Json(publications::detail(&headers, slug).await?))
}
async fn publication_update(
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<PublicationRequest>,
) -> Result<Json<()>, ApiError> {
    publications::update(&headers, id, body).await?;
    Ok(Json(()))
}
async fn publication_delete(
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<()>, ApiError> {
    publications::delete(&headers, id).await?;
    Ok(Json(()))
}
async fn publication_attach(
    headers: HeaderMap,
    Path((id, post_id)): Path<(String, String)>,
) -> Result<Json<()>, ApiError> {
    publications::attach(&headers, id, post_id).await?;
    Ok(Json(()))
}
async fn publication_detach(
    headers: HeaderMap,
    Path((id, post_id)): Path<(String, String)>,
) -> Result<Json<()>, ApiError> {
    publications::detach(&headers, id, post_id).await?;
    Ok(Json(()))
}
