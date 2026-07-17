use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use serde_json::{Value, json};

use crate::state::AppState;

/// GET /api/health — verifica API, banco e cache.
#[utoipa::path(
    get,
    path = "/api/health",
    responses(
        (status = 200, description = "API saudável"),
        (status = 503, description = "API indisponível"),
    ),
)]
pub async fn health_check(State(state): State<AppState>) -> (StatusCode, Json<Value>) {
    let mut db_status = "up";
    let mut db_error: Option<String> = None;

    if let Err(e) = sqlx::query("SELECT 1").execute(&state.db).await {
        db_status = "down";
        db_error = Some(e.to_string());
    }

    let mut redis_check = json!({ "status": "disabled" });
    if let Some(ref mut conn) = state.redis.clone() {
        match redis::cmd("PING").query_async::<String>(conn).await {
            Ok(resp) => {
                redis_check = json!({ "status": "up", "response": resp });
            }
            Err(e) => {
                redis_check = json!({ "status": "down", "error": e.to_string() });
            }
        }
    }

    let healthy = db_status == "up";
    let status_code = if healthy {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    let body = json!({
        "status": if healthy { "healthy" } else { "unhealthy" },
        "version": env!("CARGO_PKG_VERSION"),
        "checks": {
            "database": {
                "status": db_status,
                "error": db_error,
            },
            "redis": redis_check,
        }
    });

    (status_code, Json(body))
}
