use axum::{
    extract::{Path, State},
    routing::{get, post, put},
    Json, Router,
};
use redis::AsyncCommands;
use serde_json::json;
use sqlx::Row;
use uuid::Uuid;

use crate::auth::AuthUser;
use crate::error::{AppError, AppResult};
use crate::models::{
    CheckoutRequest, Order, OrderItem, OrderItemResponse, OrderResponse, Product,
};
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/checkout", post(checkout))
        .route("/orders", get(list_orders))
        .route("/orders/{id}", get(get_order))
        .route("/orders/{id}/cancel", put(cancel_order))
        .route("/orders/{id}/status", put(update_order_status))
}

fn to_cents(amount: f64) -> i64 {
    (amount * 100.0).round() as i64
}

fn from_cents(cents: i64) -> f64 {
    cents as f64 / 100.0
}

// ── Checkout ───────────────────────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/api/checkout",
    request_body = CheckoutRequest,
    responses(
        (status = 200, description = "Pedido criado com sucesso", body = OrderResponse),
        (status = 400, description = "Carrinho vazio ou cupom inválido"),
    ),
    tag = "checkout"
)]
async fn checkout(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Json(req): Json<CheckoutRequest>,
) -> AppResult<Json<OrderResponse>> {
    // 1. Get cart items with product details
    let rows = sqlx::query(
        r#"
        SELECT ci.id, ci.product_id, p.name, p.image_url, p.price, ci.quantity,
               CAST(p.price AS DOUBLE PRECISION) * ci.quantity AS subtotal
        FROM cart_items ci
        JOIN products p ON p.id = ci.product_id
        WHERE ci.user_id = $1
        "#,
    )
    .bind(user.id)
    .fetch_all(&state.db)
    .await?;

    let cart_items: Vec<(Uuid, Uuid, String, String, sqlx::types::BigDecimal, i32, f64)> = rows
        .iter()
        .map(|row| {
            Ok((
                row.try_get::<Uuid, _>("id")?,
                row.try_get::<Uuid, _>("product_id")?,
                row.try_get::<String, _>("name")?,
                row.try_get::<String, _>("image_url")?,
                row.try_get::<sqlx::types::BigDecimal, _>("price")?,
                row.try_get::<i32, _>("quantity")?,
                row.try_get::<f64, _>("subtotal")?,
            ))
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()?;

    if cart_items.is_empty() {
        return Err(AppError::BadRequest("Carrinho vazio".into()));
    }

    let total_cents: i64 = cart_items.iter().map(|(_, _, _, _, _, _, sub)| to_cents(*sub)).sum();
    let total = from_cents(total_cents);

    // 2. Coupon validation
    let mut discount_cents: i64 = 0;
    let mut coupon_id: Option<Uuid> = None;

    if let Some(ref code) = req.coupon_code {
        let code = code.trim().to_uppercase();
        let coupon = sqlx::query_as::<_, crate::models::Coupon>(
            "SELECT * FROM coupons WHERE UPPER(code) = $1",
        )
        .bind(&code)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError::BadRequest("Cupom não encontrado".into()))?;

        // Validate: active
        if !coupon.is_active {
            return Err(AppError::BadRequest("Cupom inativo".into()));
        }

        // Validate: not expired
        if coupon.expires_at < chrono::Utc::now() {
            return Err(AppError::BadRequest("Cupom expirado".into()));
        }

        // Validate: uses remaining
        if coupon.current_uses >= coupon.max_uses {
            return Err(AppError::BadRequest("Cupom sem usos disponíveis".into()));
        }

        // Validate: min order value
        let min_cents = crate::models::to_bigdecimal_cents(&coupon.min_order_value);
        if total_cents < min_cents {
            return Err(AppError::BadRequest(format!(
                "Valor mínimo do pedido: R$ {:.2}",
                from_cents(min_cents)
            )));
        }

        // Validate: user hasn't used it
        let already_used = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM order_coupons WHERE coupon_id = $1 AND user_id = $2)",
        )
        .bind(coupon.id)
        .bind(user.id)
        .fetch_one(&state.db)
        .await?;

        if already_used {
            return Err(AppError::BadRequest("Você já usou este cupom".into()));
        }

        // Calculate discount
        let val_cents = crate::models::to_bigdecimal_cents(&coupon.discount_value);
        discount_cents = match coupon.discount_type.as_str() {
            "percentage" => {
                let pct = val_cents.min(10000); // max 100%
                total_cents * pct / 10000
            }
            "fixed" => val_cents.min(total_cents),
            _ => 0,
        };

        coupon_id = Some(coupon.id);
    }

    let final_total_cents = total_cents - discount_cents;
    let discount = from_cents(discount_cents);
    let final_total = from_cents(final_total_cents);

    // 3. Start transaction
    let mut tx = state.db.begin().await?;

    // 4. Lock products and validate stock
    let _product_ids: Vec<Uuid> = cart_items.iter().map(|(_, pid, _, _, _, _, _)| *pid).collect();
    // We lock each product individually via SELECT FOR UPDATE
    for (_ci_id, pid, _pname, _pimg, _price_bd, qty, _subtotal_f) in &cart_items {
        let product: Product = sqlx::query_as::<_, Product>(
            "SELECT * FROM products WHERE id = $1 FOR UPDATE",
        )
        .bind(pid)
        .fetch_one(&mut *tx)
        .await?;

        if product.stock < *qty {
            tx.rollback().await?;
            return Err(AppError::BadRequest(format!(
                "Estoque insuficiente para '{}'. Disponível: {}, solicitado: {}",
                product.name, product.stock, qty
            )));
        }

        // 5. Deduct stock
        sqlx::query("UPDATE products SET stock = stock - $1, updated_at = NOW() WHERE id = $2")
            .bind(qty)
            .bind(pid)
            .execute(&mut *tx)
            .await?;
    }

    // 6. Create order
    let order_id = Uuid::new_v4();
    let payment_id = format!("pay_mock_{}", Uuid::new_v4());

    sqlx::query(
        r#"
        INSERT INTO orders (id, user_id, status, total, discount, final_total, payment_id)
        VALUES ($1, $2, 'paid', $3, $4, $5, $6)
        "#,
    )
    .bind(order_id)
    .bind(user.id)
    .bind(crate::models::bigdecimal_from_f64(total))
    .bind(crate::models::bigdecimal_from_f64(discount))
    .bind(crate::models::bigdecimal_from_f64(final_total))
    .bind(&payment_id)
    .execute(&mut *tx)
    .await?;

    // 7. Create order items
    for (_ci_id, pid, pname, _pimg, _price_bd, qty, subtotal_f) in &cart_items {
        let unit_price_cents = to_cents(*subtotal_f / *qty as f64);
        let item_subtotal_cents = unit_price_cents * *qty as i64;

        sqlx::query(
            r#"
            INSERT INTO order_items (order_id, product_id, product_name, quantity, unit_price, subtotal)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(order_id)
        .bind(pid)
        .bind(pname)
        .bind(qty)
        .bind(crate::models::bigdecimal_from_f64(from_cents(unit_price_cents)))
        .bind(crate::models::bigdecimal_from_f64(from_cents(item_subtotal_cents)))
        .execute(&mut *tx)
        .await?;
    }

    // 8. If coupon, record usage
    if let Some(cid) = coupon_id {
        sqlx::query(
            r#"
            INSERT INTO order_coupons (order_id, coupon_id, user_id, discount_amount)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(order_id)
        .bind(cid)
        .bind(user.id)
        .bind(crate::models::bigdecimal_from_f64(discount))
        .execute(&mut *tx)
        .await?;

        sqlx::query("UPDATE coupons SET current_uses = current_uses + 1 WHERE id = $1")
            .bind(cid)
            .execute(&mut *tx)
            .await?;
    }

    // 9. Clear cart
    sqlx::query("DELETE FROM cart_items WHERE user_id = $1")
        .bind(user.id)
        .execute(&mut *tx)
        .await?;

    // 10. Commit
    tx.commit().await?;

    // 11. Invalidate Redis cache for affected products
    if let Some(ref conn) = state.redis {
        for (_, pid, _, _, _, _, _) in &cart_items {
            let _: Result<(), _> = conn
                .clone()
                .del(format!("product:{pid}"))
                .await;
        }
    }

    // 12. Build response
    let order_items: Vec<OrderItemResponse> = cart_items
        .iter()
        .map(|(_, pid, pname, _, _price_bd, qty, subtotal_f)| OrderItemResponse {
            product_id: *pid,
            product_name: pname.clone(),
            quantity: *qty,
            unit_price: from_cents(to_cents(*subtotal_f / *qty as f64)),
            subtotal: *subtotal_f,
        })
        .collect();

    Ok(Json(OrderResponse {
        id: order_id,
        status: "paid".into(),
        total,
        discount,
        final_total,
        items: order_items,
        created_at: chrono::Utc::now(),
    }))
}

// ── List orders ─────────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/orders",
    responses(
        (status = 200, description = "Lista de pedidos do usuário", body = Vec<OrderResponse>),
    ),
    tag = "checkout"
)]
async fn list_orders(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
) -> AppResult<Json<Vec<OrderResponse>>> {
    let orders = sqlx::query_as::<_, Order>(
        "SELECT * FROM orders WHERE user_id = $1 ORDER BY created_at DESC",
    )
    .bind(user.id)
    .fetch_all(&state.db)
    .await?;

    let mut responses = Vec::new();
    for order in orders {
        let items = sqlx::query_as::<_, OrderItem>(
            "SELECT * FROM order_items WHERE order_id = $1",
        )
        .bind(order.id)
        .fetch_all(&state.db)
        .await?;

        responses.push(OrderResponse {
            id: order.id,
            status: order.status,
            total: order.total.to_string().parse().unwrap_or(0.0),
            discount: order.discount.to_string().parse().unwrap_or(0.0),
            final_total: order.final_total.to_string().parse().unwrap_or(0.0),
            items: items
                .into_iter()
                .map(|oi| OrderItemResponse {
                    product_id: oi.product_id,
                    product_name: oi.product_name,
                    quantity: oi.quantity,
                    unit_price: oi.unit_price.to_string().parse().unwrap_or(0.0),
                    subtotal: oi.subtotal.to_string().parse().unwrap_or(0.0),
                })
                .collect(),
            created_at: order.created_at,
        });
    }

    Ok(Json(responses))
}

// ── Get single order ────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/orders/{id}",
    params(
        ("id" = Uuid, Path, description = "ID do pedido"),
    ),
    responses(
        (status = 200, description = "Detalhes do pedido", body = OrderResponse),
        (status = 404, description = "Pedido não encontrado"),
    ),
    tag = "checkout"
)]
async fn get_order(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<OrderResponse>> {
    let order = sqlx::query_as::<_, Order>(
        "SELECT * FROM orders WHERE id = $1 AND user_id = $2",
    )
    .bind(id)
    .bind(user.id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Pedido não encontrado".into()))?;

    let items = sqlx::query_as::<_, OrderItem>(
        "SELECT * FROM order_items WHERE order_id = $1",
    )
    .bind(order.id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(OrderResponse {
        id: order.id,
        status: order.status,
        total: order.total.to_string().parse().unwrap_or(0.0),
        discount: order.discount.to_string().parse().unwrap_or(0.0),
        final_total: order.final_total.to_string().parse().unwrap_or(0.0),
        items: items
            .into_iter()
            .map(|oi| OrderItemResponse {
                product_id: oi.product_id,
                product_name: oi.product_name,
                quantity: oi.quantity,
                unit_price: oi.unit_price.to_string().parse().unwrap_or(0.0),
                subtotal: oi.subtotal.to_string().parse().unwrap_or(0.0),
            })
            .collect(),
        created_at: order.created_at,
    }))
}

// ── Cancel order ────────────────────────────────────────────────────────

#[utoipa::path(
    put,
    path = "/api/orders/{id}/cancel",
    params(
        ("id" = Uuid, Path, description = "ID do pedido"),
    ),
    responses(
        (status = 200, description = "Pedido cancelado"),
        (status = 400, description = "Pedido não pode ser cancelado"),
        (status = 404, description = "Pedido não encontrado"),
    ),
    tag = "checkout"
)]
async fn cancel_order(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let order = sqlx::query_as::<_, Order>(
        "SELECT * FROM orders WHERE id = $1 AND user_id = $2 FOR UPDATE",
    )
    .bind(id)
    .bind(user.id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Pedido não encontrado".into()))?;

    if order.status != "pending" && order.status != "paid" {
        return Err(AppError::BadRequest(
            "Só é possível cancelar pedidos pendentes ou pagos".into(),
        ));
    }

    let mut tx = state.db.begin().await?;

    // Return stock
    let items = sqlx::query_as::<_, OrderItem>(
        "SELECT * FROM order_items WHERE order_id = $1",
    )
    .bind(order.id)
    .fetch_all(&mut *tx)
    .await?;

    for item in &items {
        sqlx::query("UPDATE products SET stock = stock + $1, updated_at = NOW() WHERE id = $2")
            .bind(item.quantity)
            .bind(item.product_id)
            .execute(&mut *tx)
            .await?;

        // Invalidate Redis cache for affected products
        if let Some(ref conn) = state.redis {
            let _: Result<(), _> = conn
                .clone()
                .del(format!("product:{}", item.product_id))
                .await;
        }
    }

    sqlx::query("UPDATE orders SET status = 'cancelled', updated_at = NOW() WHERE id = $1")
        .bind(order.id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(Json(json!({ "message": "Pedido cancelado com sucesso", "order_id": order.id })))
}

// ── Update order status (admin only) ────────────────────────────────────

#[utoipa::path(
    put,
    path = "/api/orders/{id}/status",
    params(
        ("id" = Uuid, Path, description = "ID do pedido"),
    ),
    request_body(content = serde_json::Value, description = "{\"status\": \"shipped\"}"),
    responses(
        (status = 200, description = "Status atualizado"),
        (status = 400, description = "Transição inválida"),
        (status = 403, description = "Apenas administradores"),
        (status = 404, description = "Pedido não encontrado"),
    ),
    tag = "checkout"
)]
async fn update_order_status(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<serde_json::Value>,
) -> AppResult<Json<serde_json::Value>> {
    if user.role != "admin" {
        return Err(AppError::Forbidden("Apenas administradores podem alterar status".into()));
    }

    let new_status = body
        .get("status")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("Campo 'status' é obrigatório".into()))?;

    let valid_transitions: [(&str, &[&str]); 5] = [
        ("pending", &["paid", "cancelled"]),
        ("paid", &["shipped", "cancelled"]),
        ("shipped", &["delivered"]),
        ("delivered", &[]),
        ("cancelled", &[]),
    ];

    let order = sqlx::query_as::<_, Order>(
        "SELECT * FROM orders WHERE id = $1 FOR UPDATE",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Pedido não encontrado".into()))?;

    let allowed = valid_transitions
        .iter()
        .find(|(s, _)| *s == order.status)
        .map(|(_, next)| next.contains(&new_status))
        .unwrap_or(false);

    if !allowed {
        return Err(AppError::BadRequest(format!(
            "Transição inválida: '{}' → '{}'",
            order.status, new_status
        )));
    }

    sqlx::query("UPDATE orders SET status = $1, updated_at = NOW() WHERE id = $2")
        .bind(new_status)
        .bind(id)
        .execute(&state.db)
        .await?;

    Ok(Json(json!({ "message": format!("Status atualizado para '{}'", new_status), "order_id": id })))
}
