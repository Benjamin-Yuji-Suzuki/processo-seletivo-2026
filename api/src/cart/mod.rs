use axum::{
    extract::{Path, State},
    routing::{delete, get, post, put},
    Json, Router,
};
use serde_json::{json, Value};
use sqlx::Row;
use uuid::Uuid;
use validator::Validate;

use crate::auth::AuthUser;
use crate::error::{AppError, AppResult};
use crate::models::{
    AddToCartRequest, CartItemResponse, CartResponse, UpdateCartRequest,
};
use crate::state::AppState;

/// Mount all cart routes on a `/cart`-relative router.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/cart", get(list_cart))
        .route("/cart", post(add_to_cart))
        .route("/cart/{product_id}", put(update_cart_item))
        .route("/cart/{product_id}", delete(remove_cart_item))
        .route("/cart", delete(clear_cart))
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// `GET /cart`
///
/// List every item in the authenticated user's cart, enriched with product
/// information (name, image, unit price) and a computed per-item subtotal.
/// The response includes a `total` field that sums all subtotals.
#[utoipa::path(
    get,
    path = "/api/cart",
    responses(
        (status = 200, description = "Carrinho do usuário", body = CartResponse),
    ),
    tag = "cart"
)]
async fn list_cart(
    auth: AuthUser,
    State(state): State<AppState>,
) -> AppResult<Json<CartResponse>> {
    let rows = sqlx::query(
        r#"
        SELECT
            ci.id,
            ci.product_id,
            p.name                         AS product_name,
            p.image_url                    AS product_image,
            CAST(p.price AS DOUBLE PRECISION)             AS unit_price,
            ci.quantity,
            CAST(p.price * ci.quantity AS DOUBLE PRECISION) AS subtotal
        FROM cart_items ci
        JOIN products p ON p.id = ci.product_id
        WHERE ci.user_id = $1
        ORDER BY ci.created_at ASC
        "#,
    )
    .bind(auth.0.id)
    .fetch_all(&state.db)
    .await?;

    let items: Vec<CartItemResponse> = rows
        .iter()
        .map(|row| {
            Ok(CartItemResponse {
                id: row.try_get("id")?,
                product_id: row.try_get("product_id")?,
                product_name: row.try_get("product_name")?,
                product_image: row.try_get("product_image")?,
                unit_price: row.try_get("unit_price")?,
                quantity: row.try_get("quantity")?,
                subtotal: row.try_get("subtotal")?,
            })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()?;

    let total: f64 = items.iter().map(|i| i.subtotal).sum();

    Ok(Json(CartResponse { items, total }))
}

/// `POST /cart`
///
/// Add a product to the cart (or increase its quantity if it is already
/// present — upsert semantics via `ON CONFLICT`).
///
/// Validates that:
///   - The product exists.
///   - The total requested quantity does not exceed available stock.
#[utoipa::path(
    post,
    path = "/api/cart",
    request_body = AddToCartRequest,
    responses(
        (status = 200, description = "Item adicionado/atualizado no carrinho", body = CartItemResponse),
        (status = 400, description = "Estoque insuficiente"),
        (status = 404, description = "Produto não encontrado"),
    ),
    tag = "cart"
)]
async fn add_to_cart(
    auth: AuthUser,
    State(state): State<AppState>,
    Json(payload): Json<AddToCartRequest>,
) -> AppResult<Json<CartItemResponse>> {
    payload
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    // 1. Verify product exists and snapshot current stock.
    // 1. Verify product exists and snapshot current stock.
    let available: i32 = sqlx::query_scalar("SELECT stock FROM products WHERE id = $1 AND deleted_at IS NULL")
        .bind(payload.product_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Product not found".into()))?;

    // 2. Read the quantity already in the cart (0 if not present).
    let current_qty: i32 = sqlx::query_scalar(
        "SELECT COALESCE(CAST(SUM(quantity) AS INTEGER), 0) FROM cart_items \
         WHERE user_id = $1 AND product_id = $2",
    )
    .bind(auth.0.id)
    .bind(payload.product_id)
    .fetch_one(&state.db)
    .await?;

    let new_qty = current_qty + payload.quantity;

    if new_qty > available {
        return Err(AppError::BadRequest(format!(
            "Insufficient stock. Available: {}, requested additional: {}, \
             already in cart: {}",
            available, payload.quantity, current_qty,
        )));
    }

    // 3. Upsert the cart item and return the enriched row in one round-trip.
    let row = sqlx::query(
        r#"
        WITH upsert AS (
            INSERT INTO cart_items (user_id, product_id, quantity)
            VALUES ($1, $2, $3)
            ON CONFLICT (user_id, product_id)
            DO UPDATE SET quantity = EXCLUDED.quantity, updated_at = NOW()
            RETURNING id, product_id, quantity
        )
        SELECT
            u.id,
            u.product_id,
            p.name                         AS product_name,
            p.image_url                    AS product_image,
            CAST(p.price AS DOUBLE PRECISION)             AS unit_price,
            u.quantity,
            CAST(p.price * u.quantity AS DOUBLE PRECISION) AS subtotal
        FROM upsert u
        JOIN products p ON p.id = u.product_id
        "#,
    )
    .bind(auth.0.id)
    .bind(payload.product_id)
    .bind(new_qty)
    .fetch_one(&state.db)
    .await?;

    let item = CartItemResponse {
        id: row.try_get("id")?,
        product_id: row.try_get("product_id")?,
        product_name: row.try_get("product_name")?,
        product_image: row.try_get("product_image")?,
        unit_price: row.try_get("unit_price")?,
        quantity: row.try_get("quantity")?,
        subtotal: row.try_get("subtotal")?,
    };

    Ok(Json(item))
}

/// `PUT /cart/{product_id}`
///
/// Update the quantity of an existing cart item.
///
/// - If `quantity == 0` the item is **removed** from the cart.
/// - Otherwise the product's stock is validated before applying the change.
#[utoipa::path(
    put,
    path = "/api/cart/{product_id}",
    request_body = UpdateCartRequest,
    params(
        ("product_id" = Uuid, Path, description = "ID do produto"),
    ),
    responses(
        (status = 200, description = "Item atualizado ou removido"),
        (status = 404, description = "Item não encontrado"),
    ),
    tag = "cart"
)]
async fn update_cart_item(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(product_id): Path<Uuid>,
    Json(payload): Json<UpdateCartRequest>,
) -> AppResult<Json<Value>> {
    payload
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    // --- Quantity is zero → remove the item --------------------------------
    if payload.quantity == 0 {
        let result = sqlx::query(
            "DELETE FROM cart_items WHERE user_id = $1 AND product_id = $2",
        )
        .bind(auth.0.id)
        .bind(product_id)
        .execute(&state.db)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Cart item not found".into()));
        }

        return Ok(Json(json!({"message": "Item removed from cart"})));
    }

    // --- Non-zero quantity → validate stock and update ---------------------
    let available: i32 = sqlx::query_scalar("SELECT stock FROM products WHERE id = $1 AND deleted_at IS NULL")
        .bind(product_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Product not found".into()))?;

    if payload.quantity > available {
        return Err(AppError::BadRequest(format!(
            "Insufficient stock. Available: {}, requested: {}",
            available, payload.quantity,
        )));
    }

    let row = sqlx::query(
        r#"
        WITH updated AS (
            UPDATE cart_items
            SET quantity = $3, updated_at = NOW()
            WHERE user_id = $1 AND product_id = $2
            RETURNING id, product_id, quantity
        )
        SELECT
            u.id,
            u.product_id,
            p.name                         AS product_name,
            p.image_url                    AS product_image,
            CAST(p.price AS DOUBLE PRECISION)             AS unit_price,
            u.quantity,
            CAST(p.price * u.quantity AS DOUBLE PRECISION) AS subtotal
        FROM updated u
        JOIN products p ON p.id = u.product_id
        "#,
    )
    .bind(auth.0.id)
    .bind(product_id)
    .bind(payload.quantity)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Cart item not found".into()))?;

    let item = CartItemResponse {
        id: row.try_get("id")?,
        product_id: row.try_get("product_id")?,
        product_name: row.try_get("product_name")?,
        product_image: row.try_get("product_image")?,
        unit_price: row.try_get("unit_price")?,
        quantity: row.try_get("quantity")?,
        subtotal: row.try_get("subtotal")?,
    };

    Ok(Json(json!(item)))
}

/// `DELETE /cart/{product_id}`
///
/// Remove a single product from the cart.
#[utoipa::path(
    delete,
    path = "/api/cart/{product_id}",
    params(
        ("product_id" = Uuid, Path, description = "ID do produto"),
    ),
    responses(
        (status = 200, description = "Item removido"),
        (status = 404, description = "Item não encontrado"),
    ),
    tag = "cart"
)]
async fn remove_cart_item(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(product_id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    let result = sqlx::query(
        "DELETE FROM cart_items WHERE user_id = $1 AND product_id = $2",
    )
    .bind(auth.0.id)
    .bind(product_id)
    .execute(&state.db)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Cart item not found".into()));
    }

    Ok(Json(json!({"message": "Item removed from cart"})))
}

/// `DELETE /cart`
///
/// Remove **all** items from the authenticated user's cart.
#[utoipa::path(
    delete,
    path = "/api/cart",
    responses(
        (status = 200, description = "Carrinho limpo"),
    ),
    tag = "cart"
)]
async fn clear_cart(
    auth: AuthUser,
    State(state): State<AppState>,
) -> AppResult<Json<Value>> {
    sqlx::query("DELETE FROM cart_items WHERE user_id = $1")
        .bind(auth.0.id)
        .execute(&state.db)
        .await?;

    Ok(Json(json!({"message": "Cart cleared"})))
}
