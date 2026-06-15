use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── User ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub role: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, validator::Validate)]
pub struct RegisterRequest {
    #[validate(length(min = 2, max = 255))]
    pub name: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 6, max = 128))]
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub role: String,
}

impl From<User> for UserResponse {
    fn from(u: User) -> Self {
        Self {
            id: u.id,
            name: u.name,
            email: u.email,
            role: u.role,
        }
    }
}

// ── JWT Claims ────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,       // user id
    pub email: String,
    pub role: String,
    pub exp: usize,        // expiry timestamp
    pub iat: usize,        // issued at
}

// ── Product ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Product {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub price: sqlx::types::BigDecimal,
    pub category: String,
    pub image_url: String,
    pub stock: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, validator::Validate, utoipa::ToSchema)]
pub struct CreateProductRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    pub description: String,
    #[validate(range(min = 0.0))]
    pub price: f64,
    #[validate(length(min = 1, max = 100))]
    pub category: String,
    pub image_url: String,
    #[validate(range(min = 0))]
    pub stock: i32,
}

#[derive(Debug, Deserialize, validator::Validate)]
pub struct UpdateProductRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: Option<String>,
    pub description: Option<String>,
    #[validate(range(min = 0.0))]
    pub price: Option<f64>,
    #[validate(length(min = 1, max = 100))]
    pub category: Option<String>,
    pub image_url: Option<String>,
    #[validate(range(min = 0))]
    pub stock: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct ProductQuery {
    pub search: Option<String>,
    pub category: Option<String>,
    pub min_price: Option<f64>,
    pub max_price: Option<f64>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct PaginatedProducts {
    pub products: Vec<Product>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    pub total_pages: i64,
}

// ── Cart ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CartItem {
    pub id: Uuid,
    pub user_id: Uuid,
    pub product_id: Uuid,
    pub quantity: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CartItemResponse {
    pub id: Uuid,
    pub product_id: Uuid,
    pub product_name: String,
    pub product_image: String,
    pub unit_price: f64,
    pub quantity: i32,
    pub subtotal: f64,
}

#[derive(Debug, Deserialize, validator::Validate)]
pub struct AddToCartRequest {
    pub product_id: Uuid,
    #[validate(range(min = 1, max = 99))]
    pub quantity: i32,
}

#[derive(Debug, Deserialize, validator::Validate)]
pub struct UpdateCartRequest {
    #[validate(range(min = 0, max = 99))]
    pub quantity: i32, // 0 = remove item
}

#[derive(Debug, Serialize)]
pub struct CartResponse {
    pub items: Vec<CartItemResponse>,
    pub total: f64,
}

// ── Order ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Order {
    pub id: Uuid,
    pub user_id: Uuid,
    pub status: String,
    pub total: sqlx::types::BigDecimal,
    pub discount: sqlx::types::BigDecimal,
    pub final_total: sqlx::types::BigDecimal,
    pub payment_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct OrderItem {
    pub id: Uuid,
    pub order_id: Uuid,
    pub product_id: Uuid,
    pub product_name: String,
    pub quantity: i32,
    pub unit_price: sqlx::types::BigDecimal,
    pub subtotal: sqlx::types::BigDecimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderResponse {
    pub id: Uuid,
    pub status: String,
    pub total: f64,
    pub discount: f64,
    pub final_total: f64,
    pub items: Vec<OrderItemResponse>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderItemResponse {
    pub product_id: Uuid,
    pub product_name: String,
    pub quantity: i32,
    pub unit_price: f64,
    pub subtotal: f64,
}

#[derive(Debug, Deserialize)]
pub struct CheckoutRequest {
    pub coupon_code: Option<String>,
}

// ── Coupon ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Coupon {
    pub id: Uuid,
    pub code: String,
    pub discount_type: String,
    pub discount_value: sqlx::types::BigDecimal,
    pub min_order_value: sqlx::types::BigDecimal,
    pub max_uses: i32,
    pub current_uses: i32,
    pub expires_at: DateTime<Utc>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouponResponse {
    pub id: Uuid,
    pub code: String,
    pub discount_type: String,
    pub discount_value: f64,
    pub min_order_value: f64,
    pub expires_at: DateTime<Utc>,
    pub is_active: bool,
}

impl From<Coupon> for CouponResponse {
    fn from(c: Coupon) -> Self {
        use std::str::FromStr;
        Self {
            id: c.id,
            code: c.code,
            discount_type: c.discount_type,
            discount_value: c.discount_value.to_string().parse().unwrap_or(0.0),
            min_order_value: c.min_order_value.to_string().parse().unwrap_or(0.0),
            expires_at: c.expires_at,
            is_active: c.is_active,
        }
    }
}

#[derive(Debug, Deserialize, validator::Validate)]
pub struct CreateCouponRequest {
    #[validate(length(min = 3, max = 50))]
    pub code: String,
    #[validate(length(min = 1))]
    pub discount_type: String, // "percentage" or "fixed"
    #[validate(range(min = 0.01))]
    pub discount_value: f64,
    pub min_order_value: Option<f64>,
    pub max_uses: Option<i32>,
    pub expires_at: DateTime<Utc>,
}

// ── BigDecimal helpers ──────────────────────────────────────────────────

use sqlx::types::BigDecimal;
use std::str::FromStr;

/// Convert a BigDecimal to cents (i64). Defaults to 0 on parse error.
pub fn to_bigdecimal_cents(bd: &BigDecimal) -> i64 {
    let s = bd.to_string();
    // BigDecimal "123.45" → cents "12345"
    let (int_part, frac_part) = if let Some(dot) = s.find('.') {
        let frac = &s[dot + 1..];
        let padded = format!("{}{:0<2}", &s[..dot], &frac[..frac.len().min(2)]);
        (padded, true)
    } else {
        (format!("{}00", s), false)
    };
    // Remove any non-digit except leading '-'
    let clean: String = int_part.chars().filter(|c| c.is_ascii_digit() || *c == '-').collect();
    i64::from_str(&clean).unwrap_or(0)
}

/// Create a BigDecimal from an f64, truncated to 2 decimal places (cents precision).
pub fn bigdecimal_from_f64(value: f64) -> BigDecimal {
    let cents = (value * 100.0).round() as i64;
    BigDecimal::from(cents) / BigDecimal::from(100)
}
