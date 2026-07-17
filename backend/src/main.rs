use axum::Router;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tracing::info;

use halo_server::adapters::api::rate_limit::RateLimiter;
use halo_server::adapters::api::routes::build_router;
use halo_server::adapters::api::state::AppState;
use halo_server::adapters::auth::jwt::JwtAuthAdapter;
use halo_server::adapters::db::conversation::PostgresConversationRepository;
use halo_server::adapters::db::create_pool_async;
use halo_server::adapters::db::group::PostgresGroupRepository;
use halo_server::adapters::db::pre_key::PostgresPreKeyRepository;
use halo_server::adapters::db::session::PostgresSessionRepository;
use halo_server::adapters::db::user::PostgresUserRepository;
use halo_server::config::Config;

#[tokio::main]
async fn main() {
    let config = Config::load().expect("Failed to load configuration");
    init_tracing(&config);

    let app = build_app(&config).await;

    let addr = config.server.socket_addr();
    info!("Starting server on {}", addr);

    let listener = TcpListener::bind(addr)
        .await
        .expect("Failed to bind address");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
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
                .with(tracing_subscriber::fmt::layer().json().with_target(true))
                .with(
                    tracing_subscriber::EnvFilter::try_from_default_env()
                        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(level)),
                )
                .init();
        }
        _ => {
            tracing_subscriber::registry()
                .with(tracing_subscriber::fmt::layer().pretty().with_target(true))
                .with(
                    tracing_subscriber::EnvFilter::try_from_default_env()
                        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(level)),
                )
                .init();
        }
    }
}

async fn build_app(config: &Config) -> Router {
    let db_pool = create_pool_async(&config.database.url, config.database.max_connections).await;

    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .expect("Failed to run database migrations");

    let user_repo = PostgresUserRepository::new(db_pool.clone());
    let session_repo = PostgresSessionRepository::new(db_pool.clone());
    let conversation_repo = PostgresConversationRepository::new(db_pool.clone());
    let group_repo = PostgresGroupRepository::new(db_pool.clone());
    let pre_key_repo = PostgresPreKeyRepository::new(db_pool.clone());

    let auth_port = JwtAuthAdapter::new(
        config.jwt.secret.clone(),
        config.jwt.access_token_expiration_secs,
        config.jwt.refresh_token_expiration_secs,
        config.jwt.issuer.clone(),
    );

    let rate_limiter = RateLimiter::new(120, 2.0);

    let app_state = AppState::new(
        db_pool,
        user_repo,
        session_repo,
        conversation_repo,
        group_repo,
        pre_key_repo,
        auth_port,
        rate_limiter,
    );

    build_router(app_state).layer(CorsLayer::permissive())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
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
