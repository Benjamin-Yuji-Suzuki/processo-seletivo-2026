use crate::error::AppResult; // not used directly in the struct but kept for downstream convenience
use sqlx::PgPool;

/// Shared application state injected into all Axum routes.
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub redis: redis::aio::ConnectionManager,
    pub jwt_secret: String,
}
