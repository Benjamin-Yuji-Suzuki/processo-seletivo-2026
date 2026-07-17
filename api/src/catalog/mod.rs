use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use redis::AsyncCommands;
use uuid::Uuid;
use validator::Validate;

use crate::auth::AuthUser;
use crate::error::{AppError, AppResult};
use crate::models::{
    CreateProductRequest, PaginatedProducts, Product, ProductQuery, UpdateProductRequest,
};
use crate::state::AppState;

/// Redis cache TTL for product lookups (5 minutes).
const CACHE_TTL_SECS: u64 = 300;

/// Prefix for cached product keys.
const CACHE_KEY_PREFIX: &str = "product:";

// ── Router ─────────────────────────────────────────────────────────────

/// Build the catalog routes, all mounted under `/products`.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/products", get(list_products).post(create_product))
        .route(
            "/products/{id}",
            get(get_product).put(update_product).delete(delete_product),
        )
}

// ── Handlers ───────────────────────────────────────────────────────────

/// `GET /products` — list / search / filter / paginate products.
#[utoipa::path(
    get,
    path = "/api/products",
    params(ProductQuery),
    responses(
        (status = 200, description = "Lista paginada de produtos", body = PaginatedProducts),
    ),
    tag = "catalog"
)]
async fn list_products(
    State(state): State<AppState>,
    Query(params): Query<ProductQuery>,
) -> AppResult<Json<PaginatedProducts>> {
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * per_page;

    let search = params.search.as_deref();
    let category = params.category.as_deref();

    // ── Fetch products ──────────────────────────────────────────────
    let products: Vec<Product> = sqlx::query_as(
        r#"
        SELECT id, name, description, price, category, image_url,
               stock, created_at, updated_at, deleted_at
        FROM products
        WHERE deleted_at IS NULL
          AND ($1::text IS NULL OR name ILIKE '%' || $1 || '%')
          AND ($2::text IS NULL OR category = $2)
          AND ($3::float8 IS NULL OR price >= $3::numeric)
          AND ($4::float8 IS NULL OR price <= $4::numeric)
        ORDER BY created_at DESC
        LIMIT $5 OFFSET $6
        "#,
    )
    .bind(search)
    .bind(category)
    .bind(params.min_price)
    .bind(params.max_price)
    .bind(per_page)
    .bind(offset)
    .fetch_all(&state.db)
    .await?;

    // ── Count total (matching same filters) ─────────────────────────
    let total: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*) FROM products
        WHERE deleted_at IS NULL
          AND ($1::text IS NULL OR name ILIKE '%' || $1 || '%')
          AND ($2::text IS NULL OR category = $2)
          AND ($3::float8 IS NULL OR price >= $3::numeric)
          AND ($4::float8 IS NULL OR price <= $4::numeric)
        "#,
    )
    .bind(search)
    .bind(category)
    .bind(params.min_price)
    .bind(params.max_price)
    .fetch_one(&state.db)
    .await?;

    let total_pages = (total as f64 / per_page as f64).ceil() as i64;

    Ok(Json(PaginatedProducts {
        products,
        total,
        page,
        per_page,
        total_pages,
    }))
}

/// `GET /products/{id}` — fetch a single product by ID (Redis-cached).
#[utoipa::path(
    get,
    path = "/api/products/{id}",
    params(
        ("id" = Uuid, Path, description = "ID do produto"),
    ),
    responses(
        (status = 200, description = "Produto encontrado", body = Product),
        (status = 404, description = "Produto não encontrado"),
    ),
    tag = "catalog"
)]
async fn get_product(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Product>> {
    let cache_key = format!("{CACHE_KEY_PREFIX}{id}");

    // ── Check Redis cache ───────────────────────────────────────────
    if let Some(mut conn) = state.redis.clone() {
        let cached: Option<String> = conn.get(&cache_key).await?;
        if let Some(json_str) = cached {
            let product: Product = serde_json::from_str(&json_str)
                .map_err(|e| AppError::Internal(format!("Cache deserialize error: {e}")))?;
            return Ok(Json(product));
        }
    }

    // ── Cache miss — query database ─────────────────────────────────
    let product: Product = sqlx::query_as(
        r#"
        SELECT id, name, description, price, category, image_url,
               stock, created_at, updated_at, deleted_at
        FROM products
        WHERE id = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Product {id} not found")))?;

    // ── Store in Redis with 5-minute TTL ────────────────────────────
    if let Some(mut conn) = state.redis.clone() {
        let json_str = serde_json::to_string(&product)
            .map_err(|e| AppError::Internal(format!("Cache serialize error: {e}")))?;
        let _: () = conn.set_ex(&cache_key, &json_str, CACHE_TTL_SECS).await?;
    }

    Ok(Json(product))
}

/// `POST /products` — create a new product (admin only).
#[utoipa::path(
    post,
    path = "/api/products",
    request_body = CreateProductRequest,
    responses(
        (status = 201, description = "Produto criado", body = Product),
        (status = 403, description = "Apenas administradores"),
    ),
    tag = "catalog"
)]
async fn create_product(
    State(state): State<AppState>,
    user: AuthUser,
    Json(payload): Json<CreateProductRequest>,
) -> AppResult<(StatusCode, Json<Product>)> {
    // ── Authorisation ───────────────────────────────────────────────
    if user.0.role != "admin" {
        return Err(AppError::Forbidden(
            "Only administrators can create products".into(),
        ));
    }

    // ── Validation ──────────────────────────────────────────────────
    payload
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let id = Uuid::new_v4();
    let now = chrono::Utc::now();

    let product: Product = sqlx::query_as(
        r#"
        INSERT INTO products (id, name, description, price, category,
                              image_url, stock, created_at, updated_at)
        VALUES ($1, $2, $3, $4::numeric, $5, $6, $7, $8, $9)
        RETURNING id, name, description, price, category, image_url,
                  stock, created_at, updated_at, deleted_at
        "#,
    )
    .bind(id)
    .bind(&payload.name)
    .bind(&payload.description)
    .bind(payload.price)
    .bind(&payload.category)
    .bind(&payload.image_url)
    .bind(payload.stock)
    .bind(now)
    .bind(now)
    .fetch_one(&state.db)
    .await?;

    Ok((StatusCode::CREATED, Json(product)))
}

/// `PUT /products/{id}` — update a product (admin only).
async fn update_product(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateProductRequest>,
) -> AppResult<Json<Product>> {
    // ── Authorisation ───────────────────────────────────────────────
    if user.0.role != "admin" {
        return Err(AppError::Forbidden(
            "Only administrators can update products".into(),
        ));
    }

    // ── Validation ──────────────────────────────────────────────────
    payload
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let now = chrono::Utc::now();

    // Use COALESCE so that None::<T> binds to SQL NULL and the column
    // retains its current value — no separate read-and-merge needed.
    let product: Product = sqlx::query_as(
        r#"
        UPDATE products
        SET name        = COALESCE($1, name),
            description = COALESCE($2, description),
            price       = COALESCE($3::numeric, price),
            category    = COALESCE($4, category),
            image_url   = COALESCE($5, image_url),
            stock       = COALESCE($6, stock),
            updated_at  = $7
        WHERE id = $8 AND deleted_at IS NULL
        RETURNING id, name, description, price, category, image_url,
                  stock, created_at, updated_at, deleted_at
        "#,
    )
    .bind(payload.name.as_ref())
    .bind(payload.description.as_ref())
    .bind(payload.price)
    .bind(payload.category.as_ref())
    .bind(payload.image_url.as_ref())
    .bind(payload.stock)
    .bind(now)
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Product {id} not found")))?;

    // ── Invalidate Redis cache ───────────────────────────────────────
    let cache_key = format!("{CACHE_KEY_PREFIX}{id}");
    if let Some(mut conn) = state.redis.clone() {
        let _: () = conn.del(&cache_key).await?;
    }
    Ok(Json(product))
}

/// `DELETE /products/{id}` — soft-delete a product (admin only).
async fn delete_product(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    // ── Authorisation ───────────────────────────────────────────────
    if user.0.role != "admin" {
        return Err(AppError::Forbidden(
            "Only administrators can delete products".into(),
        ));
    }

    let result = sqlx::query("UPDATE products SET deleted_at = NOW() WHERE id = $1 AND deleted_at IS NULL")
        .bind(id)
        .execute(&state.db)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("Product {id} not found")));
    }

    // ── Invalidate Redis cache ───────────────────────────────────────
    let cache_key = format!("{CACHE_KEY_PREFIX}{id}");
    if let Some(mut conn) = state.redis.clone() {
        let _: () = conn.del(&cache_key).await?;
    }
    Ok(StatusCode::NO_CONTENT)
}
