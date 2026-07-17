use std::net::SocketAddr;

use axum::Router;
use axum::routing::get;
use axum_prometheus::PrometheusMetricLayer;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use lapes_ecommerce_api::auth;
use lapes_ecommerce_api::cart;
use lapes_ecommerce_api::catalog;
use lapes_ecommerce_api::checkout;
use lapes_ecommerce_api::coupons;
use lapes_ecommerce_api::health;
use lapes_ecommerce_api::state::AppState;
use lapes_ecommerce_api::ApiDoc;

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

    // Redis connection (optional in dev)
    let redis = match redis::aio::ConnectionManager::new(
        redis::Client::open(redis_url).expect("Invalid Redis URL"),
    )
    .await
    {
        Ok(r) => {
            tracing::info!("Connected to Redis");
            Some(r)
        }
        Err(e) => {
            tracing::warn!("Redis unavailable, running without cache: {e}");
            None
        }
    };

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
    let (prometheus_layer, metric_handle) = PrometheusMetricLayer::pair();
    let metrics_handle = Arc::new(metric_handle);

    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/api/health", get(health::health_check))
        .route("/api/metrics", get({
            let mh = metrics_handle.clone();
            move || {
                let mh = mh.clone();
                async move { mh.render() }
            }
        }))
        .nest("/api/auth", auth::routes())
        .nest("/api", catalog::routes())
        .nest("/api", cart::routes())
        .nest("/api", checkout::routes())
        .nest("/api", coupons::routes())
        .layer(TraceLayer::new_for_http())
        .layer(prometheus_layer)
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8099));
    tracing::info!("Server running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
