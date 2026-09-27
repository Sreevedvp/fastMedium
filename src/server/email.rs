use lettre::{
    transport::smtp::authentication::Credentials, AsyncSmtpTransport, AsyncTransport, Message,
    Tokio1Executor,
};
use sqlx::SqlitePool;
use uuid::Uuid;

/// Initialize the SMTP mailer from environment variables.
/// Returns None if SMTP is not configured (allows the app to run without email).
pub fn init_mailer() -> Option<AsyncSmtpTransport<Tokio1Executor>> {
    let host = std::env::var("SMTP_HOST").ok()?;
    let user = std::env::var("SMTP_USER").ok()?;
    let pass = std::env::var("SMTP_PASS").ok()?;

    // Don't initialize if using placeholder values
    if user.contains("your-email") || pass.contains("your-app-password") {
        println!("SMTP not configured — email notifications disabled. Update .env to enable.");
        return None;
    }

    let creds = Credentials::new(user, pass);

    let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(&host)
        .ok()?
        .credentials(creds)
        .timeout(Some(std::time::Duration::from_secs(10)))
        .build();

    println!("SMTP mailer initialized for {host}");
    Some(mailer)
}

/// Send a welcome email to a newly registered user.
pub async fn send_welcome_email(
    pool: &SqlitePool,
    user_id: &str,
    to_email: &str,
    display_name: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let from = std::env::var("SMTP_FROM").unwrap_or_else(|_| "noreply@fastmedium.com".to_string());
    let subject = "Welcome to fastMedium!".to_string();
    let body = format!(
        "Hi {display_name},\n\n\
         Welcome to fastMedium! We're excited to have you.\n\n\
         You can start writing your first story right away at /write.\n\n\
         Happy writing!\n\
         — The fastMedium Team"
    );

    send_email_and_log(pool, user_id, "welcome", &from, to_email, &subject, &body).await
}

/// Send a confirmation email when a post is published.
pub async fn send_post_published_email(
    pool: &SqlitePool,
    user_id: &str,
    to_email: &str,
    post_title: &str,
    post_slug: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let from = std::env::var("SMTP_FROM").unwrap_or_else(|_| "noreply@fastmedium.com".to_string());
    let subject = format!("Your story \"{post_title}\" is now live!");
    let body = format!(
        "Your story \"{post_title}\" has been published successfully!\n\n\
         View it at: /post/{post_slug}\n\n\
         Share it with the world!\n\
         — The fastMedium Team"
    );

    send_email_and_log(
        pool,
        user_id,
        "post_published",
        &from,
        to_email,
        &subject,
        &body,
    )
    .await
}

/// Send an email and log the notification to the database.
async fn send_email_and_log(
    pool: &SqlitePool,
    user_id: &str,
    notification_type: &str,
    from: &str,
    to: &str,
    subject: &str,
    body: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let notification_id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    // Try to send the email
    let send_result = send_raw_email(from, to, subject, body).await;

    let (status, sent_at) = match &send_result {
        Ok(()) => ("sent", Some(now.clone())),
        Err(_) => ("failed", None),
    };

    // Log to database regardless of send result
    let _ = sqlx::query(
        "INSERT INTO email_notifications (id, user_id, notification_type, subject, body, status, created_at, sent_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
    )
    .bind(&notification_id)
    .bind(user_id)
    .bind(notification_type)
    .bind(subject)
    .bind(body)
    .bind(status)
    .bind(&now)
    .bind(sent_at)
    .execute(pool)
    .await;

    send_result
}

/// Actually send an email via SMTP.
async fn send_raw_email(
    from: &str,
    to: &str,
    subject: &str,
    body: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let host = match std::env::var("SMTP_HOST") {
        Ok(h) => h,
        Err(_) => {
            return Err("SMTP is not configured".into());
        }
    };

    let user = std::env::var("SMTP_USER").unwrap_or_default();
    let pass = std::env::var("SMTP_PASS").unwrap_or_default();

    if user.contains("your-email") || pass.contains("your-app-password") {
        return Err("SMTP is not configured".into());
    }

    let email = Message::builder()
        .from(from.parse()?)
        .to(to.parse()?)
        .subject(subject)
        .body(body.to_string())?;

    let creds = Credentials::new(user, pass);
    let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(&host)?
        .credentials(creds)
        .timeout(Some(std::time::Duration::from_secs(10)))
        .build();

    mailer.send(email).await?;
    println!("Email notification sent");
    Ok(())
}
