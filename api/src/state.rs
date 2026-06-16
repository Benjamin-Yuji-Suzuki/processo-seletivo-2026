use sqlx::PgPool;

/// Shared application state injected into all Axum routes.
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub redis: Option<redis::aio::ConnectionManager>,
    pub jwt_secret: String,
}
