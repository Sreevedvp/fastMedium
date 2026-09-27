use sha2::{Digest, Sha256};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqlitePoolOptions};
use std::{str::FromStr, time::Duration};
use tokio::sync::OnceCell;

static DB: OnceCell<SqlitePool> = OnceCell::const_new();
const MIGRATIONS: &[(i64, &str)] = &[
    (1, include_str!("../../migrations/001_initial.sql")),
    (
        2,
        include_str!("../../migrations/002_skills_publications.sql"),
    ),
    (3, include_str!("../../migrations/003_hardening.sql")),
];

/// Startup initializes once; handlers never open a new database implicitly.
pub async fn get_db() -> &'static SqlitePool {
    DB.get()
        .expect("database must be initialized before serving requests")
}
pub async fn initialize() -> Result<(), sqlx::Error> {
    DB.get_or_try_init(init_pool).await?;
    Ok(())
}
async fn init_pool() -> Result<SqlitePool, sqlx::Error> {
    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:fastmedium.db".into());
    let options = SqliteConnectOptions::from_str(&database_url)?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .busy_timeout(Duration::from_secs(5))
        .foreign_keys(true);
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(5))
        .connect_with(options)
        .await?;
    // One writer lock for the complete migration set: no partial upgrades or concurrent runners.
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;
    sqlx::query("CREATE TABLE IF NOT EXISTS fm_migrations (version INTEGER PRIMARY KEY, checksum TEXT NOT NULL, applied_at TEXT NOT NULL DEFAULT (datetime('now')))")
        .execute(&mut *tx).await?;
    let newest: Option<i64> = sqlx::query_scalar("SELECT MAX(version) FROM fm_migrations")
        .fetch_one(&mut *tx)
        .await?;
    if newest.is_some_and(|v| v > 3) {
        return Err(sqlx::Error::Protocol(
            "Database schema is newer than this application".into(),
        ));
    }
    for (version, sql) in MIGRATIONS {
        let checksum = format!("{:x}", Sha256::digest(sql.as_bytes()));
        let saved: Option<String> =
            sqlx::query_scalar("SELECT checksum FROM fm_migrations WHERE version=?")
                .bind(version)
                .fetch_optional(&mut *tx)
                .await?;
        if let Some(saved) = saved {
            if saved != checksum {
                return Err(sqlx::Error::Protocol(format!(
                    "Migration {version} checksum mismatch"
                )));
            }
            continue;
        }
        sqlx::raw_sql(sql).execute(&mut *tx).await?;
        if *version == 3 {
            // Preserve old browser cookies while removing bearer credentials from storage.
            let tokens: Vec<(String, String)> = sqlx::query_as("SELECT id, token FROM sessions")
                .fetch_all(&mut *tx)
                .await?;
            for (id, token) in tokens {
                sqlx::query("UPDATE sessions SET token=? WHERE id=?")
                    .bind(super::auth::token_digest(&token))
                    .bind(id)
                    .execute(&mut *tx)
                    .await?;
            }
        }
        sqlx::query("INSERT INTO fm_migrations(version, checksum) VALUES (?, ?)")
            .bind(version)
            .bind(checksum)
            .execute(&mut *tx)
            .await?;
    }
    sqlx::query("DELETE FROM sessions WHERE expires_at <= datetime('now')")
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(pool)
}
