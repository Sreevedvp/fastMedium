use super::error::ApiError;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::{Duration, Utc};
use http::HeaderMap;
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;
use uuid::Uuid;
static PASSWORD_WORK: tokio::sync::Semaphore = tokio::sync::Semaphore::const_new(2);
pub fn token_digest(token: &str) -> String {
    format!("{:x}", Sha256::digest(token.as_bytes()))
}
pub fn session_cookie(token: &str, max_age: u32) -> String {
    let secure = if std::env::var("COOKIE_SECURE").as_deref() == Ok("true") {
        "; Secure"
    } else {
        ""
    };
    format!("fm_session={token}; Path=/; HttpOnly; SameSite=Strict; Max-Age={max_age}{secure}")
}

use super::db::get_db;
use crate::models::user::User;

/// Register a new user account.
pub async fn register(
    username: String,
    email: String,
    password: String,
    display_name: String,
) -> Result<(User, String), ApiError> {
    // Validate input
    let username = username.trim().to_lowercase();
    let email = email.trim().to_lowercase();
    let display_name = display_name.trim().to_string();

    if username.len() < 3 || username.len() > 30 {
        return Err(ApiError::new(
            "Username must be between 3 and 30 characters",
        ));
    }
    if !username
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return Err(ApiError::new(
            "Username can only contain letters, numbers, underscores, and hyphens",
        ));
    }
    if email.len() > 254 || email.parse::<lettre::Address>().is_err() {
        return Err(ApiError::new("Please provide a valid email address"));
    }
    if !(8..=1024).contains(&password.len()) {
        return Err(ApiError::new("Password must be 8–1024 bytes"));
    }
    if display_name.is_empty() || display_name.chars().count() > 80 {
        return Err(ApiError::new("Display name must be 1–80 characters"));
    }

    let pool = get_db().await;

    // Check if username or email already exists
    let existing: Option<(String,)> =
        sqlx::query_as("SELECT id FROM users WHERE username = ?1 OR email = ?2")
            .bind(&username)
            .bind(&email)
            .fetch_optional(pool)
            .await
            .map_err(ApiError::database)?;

    if existing.is_some() {
        return Err(ApiError {
            status: http::StatusCode::CONFLICT,
            message: "Username or email is already registered".into(),
        });
    }

    // Hash password
    let permit = PASSWORD_WORK.try_acquire().map_err(|_| ApiError::busy())?;
    let password_hash = tokio::task::spawn_blocking(move || {
        let _permit = permit;
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|_| ApiError::internal("Could not secure password"))
    })
    .await
    .map_err(|_| ApiError::internal("Password task failed"))??;

    let user_id = Uuid::new_v4().to_string();
    let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let mut tx = pool.begin().await.map_err(ApiError::database)?;
    sqlx::query(
        "INSERT INTO users (id, username, display_name, email, password_hash, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
    )
    .bind(&user_id)
    .bind(&username)
    .bind(&display_name)
    .bind(&email)
    .bind(&password_hash)
    .bind(&now)
    .execute(&mut *tx)
    .await
    .map_err(|e| if e.as_database_error().is_some_and(|e| e.is_unique_violation()) { ApiError { status: http::StatusCode::CONFLICT, message: "Username or email is already registered".into() } } else { ApiError::database(e) })?;

    // Create session
    let cookie = create_session_on(&mut tx, &user_id).await?;
    tx.commit().await.map_err(ApiError::database)?;

    // Send welcome email (fire and forget — don't fail registration if email fails)
    let email_clone = email.clone();
    let name_clone = display_name.clone();
    let uid_clone = user_id.clone();
    tokio::spawn(async move {
        let pool = get_db().await;
        let _ = super::email::send_welcome_email(pool, &uid_clone, &email_clone, &name_clone).await;
    });

    Ok((
        User {
            id: user_id,
            username,
            display_name,
            email,
            bio: String::new(),
            avatar_url: None,
            created_at: now,
        },
        cookie,
    ))
}

/// Log in with email and password.
pub async fn login(email: String, password: String) -> Result<(User, String), ApiError> {
    let email = email.trim().to_lowercase();

    if email.is_empty() || email.len() > 254 || password.is_empty() || password.len() > 1024 {
        return Err(ApiError::new("Email and password are required"));
    }

    let pool = get_db().await;

    // Fetch user with password hash
    let row: Option<(String, String, String, String, String, String, Option<String>, String)> =
        sqlx::query_as(
            "SELECT id, username, display_name, email, password_hash, bio, avatar_url, created_at FROM users WHERE email = ?1",
        )
        .bind(&email)
        .fetch_optional(pool)
        .await
        .map_err(ApiError::database)?;

    let (user_id, username, display_name, user_email, stored_hash, bio, avatar_url, created_at) =
        row.ok_or_else(|| ApiError::unauthorized("Invalid email or password"))?;

    // Verify password
    let permit = PASSWORD_WORK.try_acquire().map_err(|_| ApiError::busy())?;
    tokio::task::spawn_blocking(move || {
        let _permit = permit;
        let parsed_hash = PasswordHash::new(&stored_hash)
            .map_err(|_| ApiError::unauthorized("Invalid email or password"))?;
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .map_err(|_| ApiError::unauthorized("Invalid email or password"))
    })
    .await
    .map_err(|_| ApiError::internal("Password task failed"))??;

    // Create session
    let cookie = create_session_internal(pool, &user_id).await?;

    Ok((
        User {
            id: user_id,
            username,
            display_name,
            email: user_email,
            bio,
            avatar_url,
            created_at,
        },
        cookie,
    ))
}

/// Log out the current user.
pub async fn logout(headers: &HeaderMap) -> Result<(), ApiError> {
    if let Some(token) = extract_session_token(headers) {
        sqlx::query("DELETE FROM sessions WHERE token = ?1")
            .bind(token_digest(&token))
            .execute(get_db().await)
            .await
            .map_err(|_| ApiError::internal("Could not end the session"))?;
    }
    Ok(())
}

/// Get the currently authenticated user, if any.
pub async fn get_current_user(headers: &HeaderMap) -> Result<Option<User>, ApiError> {
    let pool = get_db().await;

    let token = extract_session_token(headers);
    let token = match token {
        Some(t) => t,
        None => return Ok(None),
    };

    // Look up session and join with user
    let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let row: Option<(
        String,
        String,
        String,
        String,
        String,
        Option<String>,
        String,
    )> = sqlx::query_as(
        "SELECT u.id, u.username, u.display_name, u.email, u.bio, u.avatar_url, u.created_at
             FROM sessions s
             JOIN users u ON s.user_id = u.id
             WHERE s.token = ?1 AND s.expires_at > ?2",
    )
    .bind(token_digest(&token))
    .bind(&now)
    .fetch_optional(pool)
    .await
    .map_err(ApiError::database)?;

    match row {
        Some((id, username, display_name, email, bio, avatar_url, created_at)) => Ok(Some(User {
            id,
            username,
            display_name,
            email,
            bio,
            avatar_url,
            created_at,
        })),
        None => Ok(None),
    }
}

/// Internal: create a session and set the cookie header.
async fn create_session_internal(pool: &SqlitePool, user_id: &str) -> Result<String, ApiError> {
    let mut tx = pool.begin().await.map_err(ApiError::database)?;
    let cookie = create_session_on(&mut tx, user_id).await?;
    tx.commit().await.map_err(ApiError::database)?;
    Ok(cookie)
}
async fn create_session_on(
    connection: &mut sqlx::SqliteConnection,
    user_id: &str,
) -> Result<String, ApiError> {
    let session_id = Uuid::new_v4().to_string();
    let token = Uuid::new_v4().to_string();
    let now = Utc::now();
    let expires = now + Duration::days(30);
    let now_str = now.format("%Y-%m-%d %H:%M:%S").to_string();
    let expires_str = expires.format("%Y-%m-%d %H:%M:%S").to_string();

    sqlx::query(
        "INSERT INTO sessions (id, user_id, token, created_at, expires_at) VALUES (?1, ?2, ?3, ?4, ?5)",
    )
    .bind(&session_id)
    .bind(user_id)
    .bind(token_digest(&token))
    .bind(&now_str)
    .bind(&expires_str)
    .execute(&mut *connection)
    .await
    .map_err(ApiError::database)?;

    sqlx::query("DELETE FROM sessions WHERE expires_at <= datetime('now') OR (user_id=? AND id NOT IN (SELECT id FROM sessions WHERE user_id=? ORDER BY rowid DESC LIMIT 10))")
        .bind(user_id).bind(user_id).execute(&mut *connection).await.map_err(ApiError::database)?;
    Ok(session_cookie(&token, 30 * 24 * 60 * 60))
}

/// Extract the session token from cookie headers.
fn extract_session_token(headers: &http::HeaderMap) -> Option<String> {
    for cookie_header in headers.get_all("cookie") {
        let Ok(cookie_header) = cookie_header.to_str() else {
            continue;
        };
        for cookie in cookie_header.split(';') {
            let cookie = cookie.trim();
            if let Some(token) = cookie.strip_prefix("fm_session=") {
                if uuid::Uuid::parse_str(token).is_ok() {
                    return Some(token.to_string());
                }
            }
        }
    }
    None
}

/// Internal helper: get current user_id from session, or error.
pub(crate) async fn require_auth(
    pool: &SqlitePool,
    headers: &HeaderMap,
) -> Result<String, ApiError> {
    let token = extract_session_token(headers)
        .ok_or_else(|| ApiError::unauthorized("You must be logged in to do this"))?;

    let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let row: Option<(String,)> =
        sqlx::query_as("SELECT user_id FROM sessions WHERE token = ?1 AND expires_at > ?2")
            .bind(token_digest(&token))
            .bind(&now)
            .fetch_optional(pool)
            .await
            .map_err(ApiError::database)?;

    row.map(|(uid,)| uid)
        .ok_or_else(|| ApiError::unauthorized("Session expired. Please log in again."))
}
