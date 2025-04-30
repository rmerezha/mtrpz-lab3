use anyhow::Result;
use speer_test::config::AppConfig;
use speer_test::db;
use speer_test::router::{enable_http_tracing, router};
use tokio::signal;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    let app_cfg = AppConfig::new();
    let cfg = app_cfg.config;

    tracing_subscriber::registry()
        .with(fmt::layer().with_ansi(false))
        .with(EnvFilter::new(&cfg.app_log))
        .init();

    let db_pool = db::init_db_pool(&cfg).await;
    let mut app = router(cfg.clone(), db_pool).expect("Could not create router");

    if cfg.http_tracing {
        app = enable_http_tracing(app);
    }

    let listener = tokio::net::TcpListener::bind(&cfg.server).await.unwrap();

    tracing::info!("Server start at {}", cfg.server);
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();

    tracing::info!("Shutting down...");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
