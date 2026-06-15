use axum::{
    extract::{Path, State},
    routing::{delete, get, post, put},
    Json, Router,
};
use serde_json::json;
use uuid::Uuid;

use crate::auth::AuthUser;
use crate::error::{AppError, AppResult};
use crate::models::{Coupon, CouponResponse, CreateCouponRequest};
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/coupons", get(list_coupons).post(create_coupon))
        .route("/coupons/validate", post(validate_coupon))
        .route("/coupons/{id}", put(update_coupon).delete(delete_coupon))
}

// ── List all coupons (admin only) ───────────────────────────────────────

async fn list_coupons(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
) -> AppResult<Json<Vec<CouponResponse>>> {
    if user.role != "admin" {
        return Err(AppError::Forbidden("Apenas administradores".into()));
    }

    let coupons = sqlx::query_as::<_, Coupon>("SELECT * FROM coupons ORDER BY created_at DESC")
        .fetch_all(&state.db)
        .await?;

    Ok(Json(coupons.into_iter().map(CouponResponse::from).collect()))
}

// ── Create coupon (admin only) ──────────────────────────────────────────

async fn create_coupon(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Json(req): Json<CreateCouponRequest>,
) -> AppResult<Json<CouponResponse>> {
    if user.role != "admin" {
        return Err(AppError::Forbidden("Apenas administradores".into()));
    }

    if req.discount_type != "percentage" && req.discount_type != "fixed" {
        return Err(AppError::Validation(
            "Tipo de desconto deve ser 'percentage' ou 'fixed'".into(),
        ));
    }

    let discount_val = crate::models::bigdecimal_from_f64(req.discount_value);
    let min_val = crate::models::bigdecimal_from_f64(req.min_order_value.unwrap_or(0.0));
    let max_u = req.max_uses.unwrap_or(1);

    let coupon = sqlx::query_as::<_, Coupon>(
        r#"
        INSERT INTO coupons (code, discount_type, discount_value, min_order_value, max_uses, expires_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING *
        "#,
    )
    .bind(&req.code.to_uppercase())
    .bind(&req.discount_type)
    .bind(&discount_val)
    .bind(&min_val)
    .bind(max_u)
    .bind(req.expires_at)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(CouponResponse::from(coupon)))
}

// ── Update coupon (admin only) ──────────────────────────────────────────

async fn update_coupon(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<serde_json::Value>,
) -> AppResult<Json<CouponResponse>> {
    if user.role != "admin" {
        return Err(AppError::Forbidden("Apenas administradores".into()));
    }

    let coupon = sqlx::query_as::<_, Coupon>(
        "SELECT * FROM coupons WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Cupom não encontrado".into()))?;

    let code = body.get("code").and_then(|v| v.as_str()).unwrap_or(&coupon.code).to_string();
    let discount_type = body
        .get("discount_type")
        .and_then(|v| v.as_str())
        .unwrap_or(&coupon.discount_type)
        .to_string();
    let discount_value = body
        .get("discount_value")
        .and_then(|v| v.as_f64())
        .map(crate::models::bigdecimal_from_f64)
        .unwrap_or(coupon.discount_value.clone());
    let min_order_value = body
        .get("min_order_value")
        .and_then(|v| v.as_f64())
        .map(crate::models::bigdecimal_from_f64)
        .unwrap_or(coupon.min_order_value.clone());
    let max_uses = body
        .get("max_uses")
        .and_then(|v| v.as_i64())
        .map(|v| v as i32)
        .unwrap_or(coupon.max_uses);
    let is_active = body
        .get("is_active")
        .and_then(|v| v.as_bool())
        .unwrap_or(coupon.is_active);
    let expires_at = body
        .get("expires_at")
        .and_then(|v| v.as_str())
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .unwrap_or(coupon.expires_at);

    let updated = sqlx::query_as::<_, Coupon>(
        r#"
        UPDATE coupons
        SET code = $1, discount_type = $2, discount_value = $3,
            min_order_value = $4, max_uses = $5, is_active = $6,
            expires_at = $7
        WHERE id = $8
        RETURNING *
        "#,
    )
    .bind(code)
    .bind(discount_type)
    .bind(&discount_value)
    .bind(&min_order_value)
    .bind(max_uses)
    .bind(is_active)
    .bind(expires_at)
    .bind(id)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(CouponResponse::from(updated)))
}

// ── Delete coupon (admin only) ──────────────────────────────────────────

async fn delete_coupon(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    if user.role != "admin" {
        return Err(AppError::Forbidden("Apenas administradores".into()));
    }

    let result = sqlx::query("DELETE FROM coupons WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Cupom não encontrado".into()));
    }

    Ok(Json(json!({ "message": "Cupom removido com sucesso" })))
}

// ── Validate coupon ─────────────────────────────────────────────────────

async fn validate_coupon(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> AppResult<Json<serde_json::Value>> {
    let code = body
        .get("code")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("Campo 'code' é obrigatório".into()))?
        .trim()
        .to_uppercase();

    let total = body
        .get("total")
        .and_then(|v| v.as_f64())
        .ok_or_else(|| AppError::BadRequest("Campo 'total' é obrigatório".into()))?;

    let coupon = sqlx::query_as::<_, Coupon>(
        "SELECT * FROM coupons WHERE UPPER(code) = $1",
    )
    .bind(&code)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::BadRequest("Cupom não encontrado".into()))?;

    // Check active
    if !coupon.is_active {
        return Ok(Json(json!({
            "valid": false, "discount": 0, "discount_type": "",
            "message": "Este cupom está inativo"
        })));
    }

    // Check expiry
    if coupon.expires_at < chrono::Utc::now() {
        return Ok(Json(json!({
            "valid": false, "discount": 0, "discount_type": "",
            "message": "Este cupom expirou"
        })));
    }

    // Check uses remaining
    if coupon.current_uses >= coupon.max_uses {
        return Ok(Json(json!({
            "valid": false, "discount": 0, "discount_type": "",
            "message": "Este cupom esgotou os usos"
        })));
    }

    // Check min order value
    let min_cents = crate::models::to_bigdecimal_cents(&coupon.min_order_value);
    let total_cents = (total * 100.0).round() as i64;
    if total_cents < min_cents {
        let min_val = min_cents as f64 / 100.0;
        return Ok(Json(json!({
            "valid": false, "discount": 0, "discount_type": "",
            "message": format!("Valor mínimo do pedido: R$ {:.2}", min_val)
        })));
    }

    // Check user hasn't used it
    let already_used = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM order_coupons WHERE coupon_id = $1 AND user_id = $2)",
    )
    .bind(coupon.id)
    .bind(user.id)
    .fetch_one(&state.db)
    .await?;

    if already_used {
        return Ok(Json(json!({
            "valid": false, "discount": 0, "discount_type": "",
            "message": "Você já utilizou este cupom"
        })));
    }

    // Calculate discount
    let val_cents = crate::models::to_bigdecimal_cents(&coupon.discount_value);
    let discount_cents = match coupon.discount_type.as_str() {
        "percentage" => {
            let pct = val_cents.min(10000);
            total_cents * pct / 10000
        }
        "fixed" => val_cents.min(total_cents),
        _ => 0,
    };

    let discount = discount_cents as f64 / 100.0;

    Ok(Json(json!({
        "valid": true,
        "discount": discount,
        "discount_type": coupon.discount_type,
        "message": "Cupom válido!"
    })))
}
