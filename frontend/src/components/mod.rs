use crate::api;
use leptos::prelude::*;
use serde_json::Value;

// ── NavBar ────────────────────────────────────────────────────────────────

#[component]
pub fn NavBar() -> impl IntoView {
    let is_logged_in = move || api::get_auth_token().is_some();
    let logout = move |_| {
        api::clear_auth_token();
    };

    view! {
        <nav style="display: flex; justify-content: space-between; align-items: center; padding: 0.75rem 1.5rem; background: #2c3e50; color: #fff;">
            <div style="font-weight: bold; font-size: 1.25rem;">
                <a href="/" style="color: #fff; text-decoration: none;">
                    "LAPES"
                </a>
            </div>
            <div style="display: flex; gap: 1rem; align-items: center;">
                <a href="/products" style="color: #fff; text-decoration: none;">
                    "Produtos"
                </a>
                <a href="/cart" style="color: #fff; text-decoration: none;">
                    "Carrinho"
                </a>
                {move || {
                    if is_logged_in() {
                        view! {
                            <>
                                <a href="/admin" style="color: #fff; text-decoration: none;">
                                    "Admin"
                                </a>
                                <button
                                    style="
                                        background: transparent;
                                        color: #fff;
                                        border: 1px solid #fff;
                                        border-radius: 4px;
                                        padding: 0.25rem 0.75rem;
                                        cursor: pointer;
                                    "
                                    on:click=logout
                                >
                                    "Sair"
                                </button>
                            </>
                        }
                            .into_any()
                    } else {
                        view! {
                            <a href="/login" style="color: #fff; text-decoration: none;">
                                "Entrar"
                            </a>
                        }
                            .into_any()
                    }
                }}
            </div>
        </nav>
    }
}

// ── ProductCard ────────────────────────────────────────────────────────────

#[component]
pub fn ProductCard(product: Value) -> impl IntoView {
    let name = product["name"]
        .as_str()
        .unwrap_or("Sem nome")
        .to_string();
    let price = product["price"].as_f64().unwrap_or(0.0);
    let desc = product["description"]
        .as_str()
        .unwrap_or("")
        .to_string();
    let image = product["image_url"]
        .as_str()
        .unwrap_or("")
        .to_string();
    let id = product["id"]
        .as_str()
        .map(|s| s.to_string())
        .or_else(|| product["id"].as_i64().map(|i| i.to_string()));

    view! {
        <div style="border: 1px solid #ddd; border-radius: 8px; padding: 1rem; display: flex; flex-direction: column; gap: 0.5rem; background: #fff;">
            {if image.is_empty() {
                None
            } else {
                Some(view! {
                    <img
                        src=image
                        alt=name.clone()
                        style="width: 100%; height: 180px; object-fit: cover; border-radius: 4px;"
                    />
                })
            }}
            <h3 style="margin: 0;">{name}</h3>
            <p style="margin: 0; color: #2ecc71; font-weight: bold; font-size: 1.1rem;">
                {format!("R$ {:.2}", price)}
            </p>
            <p style="margin: 0; color: #666; font-size: 0.9rem; flex: 1;">
                {desc}
            </p>
            {id.map(|pid| {
                view! {
                    <button
                        style="
                            padding: 0.5rem;
                            background: #3498db;
                            color: #fff;
                            border: none;
                            border-radius: 4px;
                            cursor: pointer;
                            margin-top: auto;
                        "
                        on:click=move |_| {
                            leptos::task::spawn_local({
                                let id = pid.clone();
                                async move {
                                    let _ = api::api_post(
                                        "/api/cart",
                                        &serde_json::json!({ "product_id": id, "quantity": 1 }),
                                    )
                                    .await;
                                }
                            });
                        }
                    >
                        "Adicionar ao carrinho"
                    </button>
                }
            })}
        </div>
    }
}

// ── Loading Spinner ────────────────────────────────────────────────────────

#[component]
pub fn Loading() -> impl IntoView {
    view! {
        <div style="text-align: center; padding: 2rem; display: flex; justify-content: center; align-items: center; gap: 0.5rem;">
            <div
                style="
                    width: 24px;
                    height: 24px;
                    border: 3px solid #f3f3f3;
                    border-top: 3px solid #3498db;
                    border-radius: 50%;
                    animation: spin 1s linear infinite;
                "
            ></div>
            <span>"Carregando..."</span>
            <style>
                "@keyframes spin {
                    0% { transform: rotate(0deg); }
                    100% { transform: rotate(360deg); }
                }"
            </style>
        </div>
    }
}
