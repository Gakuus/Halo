pub mod auth;
pub mod groups;
pub mod users;

use axum::{
    middleware,
    routing::{delete, get, post},
    Router,
};

use crate::adapters::api::middleware::auth::auth_middleware;
use crate::adapters::api::rate_limit::rate_limit_middleware;
use crate::adapters::api::state::AppState;

pub fn build_router(app_state: AppState) -> Router {
    let protected_routes = Router::new()
        .route("/auth/logout", post(auth::logout))
        .route("/users/search", get(users::search_users))
        .route("/users/{id}", get(users::get_user))
        .route("/users/{id}/public-key", get(users::get_public_key))
        .route("/groups", post(groups::create_group))
        .route("/groups/{id}", get(groups::get_group))
        .route("/groups/{id}/members", post(groups::add_members))
        .route("/groups/{id}/members/{user_id}", delete(groups::remove_member))
        .route("/groups/{id}", delete(groups::delete_group))
        .route_layer(middleware::from_fn_with_state(
            app_state.clone(),
            auth_middleware,
        ));

    let public_routes = Router::new()
        .route("/health", get(super::health::health_check))
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login))
        .route("/auth/refresh", post(auth::refresh));

    public_routes
        .merge(protected_routes)
        .route_layer(middleware::from_fn_with_state(
            app_state.clone(),
            rate_limit_middleware,
        ))
        .with_state(app_state)
}
