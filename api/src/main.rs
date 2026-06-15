use std::net::SocketAddr;

use axum::Router;
use sqlx::postgres::PgPoolOptions;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use lapes_ecommerce_api::auth;
use lapes_ecommerce_api::cart;
use lapes_ecommerce_api::catalog;
use lapes_ecommerce_api::checkout;
use lapes_ecommerce_api::coupons;
use lapes_ecommerce_api::state::AppState;

#[tokio::main]
async fn main() {
    // Load .env file
    dotenvy::dotenv().ok();

    // Setup tracing
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .json()
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://localhost:6379".into());
    let jwt_secret = std::env::var("JWT_SECRET")
        .expect("JWT_SECRET must be set");

    // Database pool
    let db = PgPoolOptions::new()
        .max_connections(20)
        .connect(&database_url)
        .await
        .expect("Failed to connect to PostgreSQL");

    tracing::info!("Connected to PostgreSQL");

    // Redis connection
    let redis_client = redis::Client::open(redis_url)
        .expect("Invalid Redis URL");
    let redis = redis::aio::ConnectionManager::new(redis_client)
        .await
        .expect("Failed to connect to Redis");

    tracing::info!("Connected to Redis");

    // Run migrations from ../migrations relative to workspace root
    sqlx::migrate!("../migrations")
        .run(&db)
        .await
        .expect("Failed to run migrations");

    tracing::info!("Migrations applied");

    let state = AppState {
        db,
        redis,
        jwt_secret,
    };

    // Build router
    let app = Router::new()
        .nest("/api/auth", auth::routes())
        .nest("/api", catalog::routes())
        .nest("/api", cart::routes())
        .nest("/api", checkout::routes())
        .nest("/api", coupons::routes())
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("Server running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
