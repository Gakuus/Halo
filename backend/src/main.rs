use axum::Router;
use tower_http::cors::CorsLayer;
use tracing::info;

use halo_server::config::Config;
use halo_server::adapters::api::health;

#[tokio::main]
async fn main() {
    let config = Config::load().expect("Failed to load configuration");

    init_tracing(&config);

    let app = build_router(&config);

    let addr = config.server.socket_addr();
    info!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind address");

    axum::serve(listener, app)
        .await
        .expect("Server failed");
}

fn init_tracing(config: &Config) {
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    let level = &config.observability.log_level;

    match config.observability.log_format.as_str() {
        "json" => {
            tracing_subscriber::registry()
                .with(
                    tracing_subscriber::fmt::layer()
                        .json()
                        .with_target(true),
                )
                .with(
                    tracing_subscriber::EnvFilter::try_from_default_env()
                        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(level)),
                )
                .init();
        }
        _ => {
            tracing_subscriber::registry()
                .with(
                    tracing_subscriber::fmt::layer()
                        .pretty()
                        .with_target(true),
                )
                .with(
                    tracing_subscriber::EnvFilter::try_from_default_env()
                        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(level)),
                )
                .init();
        }
    }
}

fn build_router(config: &Config) -> Router {
    let _ = config;

    Router::new()
        .route("/health", axum::routing::get(health::health_check))
        .layer(CorsLayer::permissive())
}
