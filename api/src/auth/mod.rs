//! Authentication routes — register, login, and current-user endpoint.
//!
//! Provides an [`AuthUser`] request extractor that validates a Bearer JWT and
//! looks up the corresponding user from the database, plus handler functions
//! for the three auth endpoints.

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{
    extract::{FromRequestParts, State},
    http::{header, request::Parts, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use std::future::Future;
use uuid::Uuid;
use validator::Validate;

use crate::error::{AppError, AppResult};
use crate::models::{AuthResponse, Claims, LoginRequest, RegisterRequest, User, UserResponse};
use crate::state::AppState;

// ---------------------------------------------------------------------------
// JWT helpers
// ---------------------------------------------------------------------------

/// Number of hours before an issued JWT expires.
const JWT_EXPIRY_HOURS: i64 = 24;

/// Issue a signed JWT for the given user.
fn create_jwt(user: &User, secret: &str) -> AppResult<String> {
    let now = chrono::Utc::now();
    let claims = Claims {
        sub: user.id.to_string(),
        email: user.email.clone(),
        role: user.role.clone(),
        iat: now.timestamp() as usize,
        exp: (now + chrono::Duration::hours(JWT_EXPIRY_HOURS))
            .timestamp()
            .try_into()
            .unwrap_or(0),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(format!("JWT encoding failed: {e}")))
}

/// Validate a JWT and return its claims.
fn verify_jwt(token: &str, secret: &str) -> AppResult<Claims> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|_| AppError::Unauthorized("Invalid or expired token".into()))
}

// ---------------------------------------------------------------------------
// AuthUser extractor
// ---------------------------------------------------------------------------

/// Axum request extractor that authenticates a user via a Bearer JWT.
///
/// The JWT is read from the `Authorization` header, verified, and the
/// corresponding [`User`] row is fetched from the database.
///
/// # Errors
///
/// Returns `401 Unauthorized` when the header is missing, the token is
/// invalid / expired, or the user no longer exists.
pub struct AuthUser(pub User);

impl AuthUser {
    /// Extract a Bearer token from the request headers.
    fn extract_token(parts: &Parts) -> AppResult<String> {
        let auth_header = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| AppError::Unauthorized("Missing Authorization header".into()))?;

        auth_header
            .strip_prefix("Bearer ")
            .map(ToOwned::to_owned)
            .ok_or_else(|| AppError::Unauthorized("Invalid Authorization header format".into()))
    }
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
        async move {
            let token = Self::extract_token(parts)?;
            let claims = verify_jwt(&token, &state.jwt_secret)?;

            let user_id: Uuid = claims
                .sub
                .parse()
                .map_err(|_| AppError::Unauthorized("Invalid token payload".into()))?;

            let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
                .bind(user_id)
                .fetch_optional(&state.db)
                .await?
                .ok_or_else(|| AppError::Unauthorized("User not found".into()))?;

            Ok(AuthUser(user))
        }
    }
}

// ---------------------------------------------------------------------------
// Route handlers
// ---------------------------------------------------------------------------

/// `POST /register`
///
/// Validates the registration payload, checks for duplicate emails, hashes the
/// password with Argon2, inserts the user, and returns a signed JWT along with
/// the user's public profile.
async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> AppResult<impl IntoResponse> {
    // 1. Validate input
    payload
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    // 2. Check for existing user (case-insensitive email would be better, but
    //    this keeps the query simple for the MVP).
    let existing = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM users WHERE email = $1",
    )
    .bind(&payload.email)
    .fetch_one(&state.db)
    .await?;

    if existing > 0 {
        return Err(AppError::Conflict(
            "A user with this email already exists".into(),
        ));
    }

    // 3. Hash password with Argon2id
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(payload.password.as_bytes(), &salt)
        .map_err(|e| AppError::Internal(format!("Password hashing failed: {e}")))?
        .to_string();

    // 4. Insert user record
    let user_id = Uuid::new_v4();
    let now = chrono::Utc::now();

    sqlx::query(
        r#"INSERT INTO users (id, name, email, password_hash, role, created_at, updated_at)
           VALUES ($1, $2, $3, $4, 'customer', $5, $6)"#,
    )
    .bind(user_id)
    .bind(&payload.name)
    .bind(&payload.email)
    .bind(&password_hash)
    .bind(now)
    .bind(now)
    .execute(&state.db)
    .await?;

    // 5. Build response
    let user = User {
        id: user_id,
        name: payload.name,
        email: payload.email,
        password_hash,
        role: "customer".into(),
        created_at: now,
        updated_at: now,
    };

    let token = create_jwt(&user, &state.jwt_secret)?;

    Ok((
        StatusCode::CREATED,
        Json(AuthResponse {
            token,
            user: user.into(),
        }),
    ))
}

/// `POST /login`
///
/// Verifies the email/password combination and returns a signed JWT along with
/// the user's public profile.
async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> AppResult<impl IntoResponse> {
    // 1. Look up user by email
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
        .bind(&payload.email)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError::Unauthorized("Invalid email or password".into()))?;

    // 2. Verify password against stored hash
    let parsed_hash = PasswordHash::new(&user.password_hash)
        .map_err(|e| AppError::Internal(format!("Invalid password hash: {e}")))?;

    Argon2::default()
        .verify_password(payload.password.as_bytes(), &parsed_hash)
        .map_err(|_| AppError::Unauthorized("Invalid email or password".into()))?;

    // 3. Issue token
    let token = create_jwt(&user, &state.jwt_secret)?;

    Ok(Json(AuthResponse {
        token,
        user: user.into(),
    }))
}

/// `GET /me`
///
/// Returns the currently authenticated user's public profile. Requires a valid
/// Bearer JWT (handled by the [`AuthUser`] extractor).
async fn me(AuthUser(user): AuthUser) -> impl IntoResponse {
    Json(UserResponse::from(user))
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

/// Build the auth router.
///
/// Mount at `/auth` on the main application:
///
/// ```ignore
/// app.nest("/auth", auth::routes())
/// ```
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/me", get(me))
}

#[cfg(test)]
mod tests {
    // Integration tests live alongside the main application test suite.
    // Auth handlers depend on a live database and JWT secret, so they are
    // exercised via end-to-end tests rather than unit tests here.
}
