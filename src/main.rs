use aigc_history::{
    api::{AppState, create_router},
    config::Settings,
    db::DbClient,
    repositories::{BranchRepository, LineageRepository, ShareRepository},
    services::{BranchService, ConversationService, ForkService, ShareService},
};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[cfg(unix)]
use tokio::signal::unix::{SignalKind, signal};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "aigc_history=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let settings = Settings::from_env().map_err(|e| format!("Failed to load settings: {}", e))?;

    tracing::info!("Starting AIGC History Service");
    tracing::info!("Connecting to ScyllaDB at: {:?}", settings.scylla.nodes);

    // Initialize database client
    let db_client = DbClient::new(&settings.scylla)
        .await
        .map_err(|e| format!("Failed to connect to ScyllaDB: {}", e))?;

    tracing::info!("Successfully connected to ScyllaDB");

    // Initialize repositories
    let lineage_repo = LineageRepository::new(db_client.clone());
    let branch_repo = BranchRepository::new(db_client.clone());
    let share_repo = ShareRepository::new(db_client.clone());

    // Initialize services
    let conversation_service = Arc::new(ConversationService::new(
        lineage_repo.clone(),
        settings.app.clone(),
    ));

    let branch_service = Arc::new(BranchService::new(
        branch_repo.clone(),
        lineage_repo.clone(),
    ));

    let fork_service = Arc::new(ForkService::new(
        lineage_repo.clone(),
        branch_repo.clone(),
        settings.app.clone(),
    ));

    let share_service = Arc::new(ShareService::new(share_repo.clone()));

    // Create application state
    let app_state = AppState {
        conversation_service,
        branch_service,
        fork_service,
        share_service,
    };

    // Build router
    let app = create_router(app_state)
        .layer(CorsLayer::permissive())
        .layer(tower_http::catch_panic::CatchPanicLayer::new());

    // Start server
    let addr = format!("{}:{}", settings.server.host, settings.server.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| format!("Failed to bind to {}: {}", addr, e))?;

    tracing::info!("Server listening on {}", addr);
    tracing::info!("Health check available at: http://{}/health", addr);
    tracing::info!("API endpoints available at: http://{}/api/v1/", addr);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|e| format!("Server error: {}", e))?;

    tracing::info!("Server shutdown complete");

    Ok(())
}

async fn shutdown_signal() {
    #[cfg(unix)]
    {
        let mut terminate_signal =
            signal(SignalKind::terminate()).expect("Failed to install SIGTERM handler");

        tokio::select! {
            res = tokio::signal::ctrl_c() => {
                if let Err(err) = res {
                    tracing::error!("Failed to listen for Ctrl+C: {}", err);
                }
            },
            _ = terminate_signal.recv() => {},
        }
    }

    #[cfg(not(unix))]
    {
        if let Err(err) = tokio::signal::ctrl_c().await {
            tracing::error!("Failed to listen for Ctrl+C: {}", err);
        }
    }

    tracing::info!("Shutdown signal received, commencing graceful shutdown");
}
