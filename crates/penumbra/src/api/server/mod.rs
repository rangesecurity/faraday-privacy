pub mod handlers;
pub mod router;

use {
    anyhow::{Context, Result},
    tokio::signal,
};

pub async fn start_api(url: String, listen_url: String) -> Result<()> {
    let router = router::new(url);
    Ok(axum::serve(
        tokio::net::TcpListener::bind(listen_url)
            .await
            .with_context(|| "failed to create listener")?,
        router,
    )
    .with_graceful_shutdown(shutdown_signal())
    .await?)
}

async fn shutdown_signal() {
    // Wait for CTRL+C signal
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    // Wait for either signal
    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    log::info!("Shutdown signal received, starting graceful shutdown...");

    // Add a small delay to allow in-flight requests to complete
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    log::info!("Server shutdown complete");
}
