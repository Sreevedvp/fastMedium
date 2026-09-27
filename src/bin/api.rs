use fast_medium::server::{api, db};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let port: u16 = std::env::var("API_PORT")
        .unwrap_or_else(|_| "8082".into())
        .parse()?;
    let web_port: u16 = std::env::var("WEB_PORT")
        .unwrap_or_else(|_| "8080".into())
        .parse()?;
    // Fail during startup if the database cannot be opened, before claiming readiness.
    db::initialize().await?;
    let router = api::router(web_port)?;
    let host: std::net::IpAddr = std::env::var("API_HOST")
        .unwrap_or_else(|_| "127.0.0.1".into())
        .parse()?;
    let listener = tokio::net::TcpListener::bind((host, port)).await?;
    println!("fastMedium Tokio API: http://{host}:{port}");
    axum::serve(
        listener,
        router.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown())
    .await?;
    Ok(())
}

async fn shutdown() {
    #[cfg(unix)]
    {
        let mut terminate =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                .expect("signal handler");
        tokio::select! { _ = tokio::signal::ctrl_c() => {}, _ = terminate.recv() => {} }
    }
    #[cfg(not(unix))]
    {
        let _ = tokio::signal::ctrl_c().await;
    }
}
