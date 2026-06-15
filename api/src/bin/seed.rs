// Binary to seed the database with test data.
// Usage: cargo run --bin seed

use argon2::PasswordHasher;
use uuid::Uuid;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to PostgreSQL");

    // Seed admin user
    let admin_id = Uuid::new_v4();
    let admin_hash = argon2::Argon2::default()
        .hash_password(b"admin123", &argon2::password_hash::SaltString::generate(&mut argon2::password_hash::rand_core::OsRng))
        .unwrap()
        .to_string();

    let _ = sqlx::query(
        r#"
        INSERT INTO users (id, name, email, password_hash, role)
        VALUES ($1, 'Admin', 'admin@lapes.com', $2, 'admin')
        ON CONFLICT (email) DO NOTHING
        "#,
    )
    .bind(admin_id)
    .bind(&admin_hash)
    .execute(&pool)
    .await;

    println!("Admin user created (admin@lapes.com / admin123)");

    // Seed customer user
    let customer_id = Uuid::new_v4();
    let customer_hash = argon2::Argon2::default()
        .hash_password(b"customer123", &argon2::password_hash::SaltString::generate(&mut argon2::password_hash::rand_core::OsRng))
        .unwrap()
        .to_string();

    let _ = sqlx::query(
        r#"
        INSERT INTO users (id, name, email, password_hash, role)
        VALUES ($1, 'Cliente', 'cliente@lapes.com', $2, 'customer')
        ON CONFLICT (email) DO NOTHING
        "#,
    )
    .bind(customer_id)
    .bind(&customer_hash)
    .execute(&pool)
    .await;

    println!("Customer user created (cliente@lapes.com / customer123)");

    // Seed products
    let products = vec![
        ("Camiseta LAPES 2026", "Camiseta oficial do LAPES 2026", 49.90, "Vestuário", 100),
        ("Caneca Personalizada", "Caneca de cerâmica com logo LAPES", 29.90, "Acessórios", 50),
        ("Moletom Tech", "Moletom confortável estilo techwear", 129.90, "Vestuário", 30),
        ("Caderno Inteligente", "Caderno pautado capa dura", 39.90, "Papelaria", 75),
        ("Stickers Dev Rust", "Pack de 10 stickers de tecnologia", 19.90, "Acessórios", 200),
        ("Teclado Mecânico 60%", "Teclado mecânico RGB switch azul", 249.90, "Eletrônicos", 15),
        ("Mousepad XXL", "Mousepad 90x40cm com borda costurada", 79.90, "Eletrônicos", 40),
        ("Livro: Rust Avançado", "Guia completo de Rust para sistemas", 89.90, "Livros", 25),
        ("Fone Bluetooth", "Fone sem fio com cancelamento de ruído", 199.90, "Eletrônicos", 20),
        ("Capa Notebook 15\"", "Capa protetora para notebook 15 polegadas", 59.90, "Acessórios", 35),
    ];

    for (name, desc, price, category, stock) in &products {
        sqlx::query(
            "INSERT INTO products (name, description, price, category, stock) VALUES ($1, $2, $3, $4, $5) ON CONFLICT DO NOTHING",
        )
        .bind(name)
        .bind(desc)
        .bind(sqlx::types::BigDecimal::try_from(*price).unwrap())
        .bind(category)
        .bind(stock)
        .execute(&pool)
        .await
        .ok();
    }

    println!("Products seeded: {} products", products.len());

    // Seed a test coupon
    let _ = sqlx::query(
        r#"
        INSERT INTO coupons (code, discount_type, discount_value, min_order_value, max_uses, expires_at)
        VALUES ('LAPES10', 'percentage', 10.00, 50.00, 100, NOW() + INTERVAL '30 days')
        ON CONFLICT (code) DO NOTHING
        "#,
    )
    .execute(&pool)
    .await;

    let _ = sqlx::query(
        r#"
        INSERT INTO coupons (code, discount_type, discount_value, min_order_value, max_uses, expires_at)
        VALUES ('BEMVINDO', 'fixed', 15.00, 30.00, 50, NOW() + INTERVAL '30 days')
        ON CONFLICT (code) DO NOTHING
        "#,
    )
    .execute(&pool)
    .await;

    println!("Coupons created: LAPES10 (10% off), BEMVINDO (R$15 off)");
    println!("\nSeed completed!");
}
