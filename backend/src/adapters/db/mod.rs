pub mod user;
pub mod session;
pub mod conversation;
pub mod group;

use sqlx::PgPool;

pub fn create_pool(database_url: &str, _max_connections: u32) -> PgPool {
    PgPool::connect_lazy(database_url)
        .expect("Failed to create database pool")
}

pub async fn create_pool_async(database_url: &str, max_connections: u32) -> PgPool {
    sqlx::postgres::PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(database_url)
        .await
        .expect("Failed to connect to database")
}
