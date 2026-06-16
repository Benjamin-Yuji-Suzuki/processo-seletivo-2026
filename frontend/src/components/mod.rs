use crate::api;
use leptos::prelude::*;
use leptos_router::hooks::use_location;
use serde_json::Value;
use wasm_bindgen::JsCast;

// ── NavBar ────────────────────────────────────────────────────────────────

#[component]
pub fn NavBar() -> impl IntoView {
    let is_logged_in = move || api::get_auth_token().is_some();
    let location = use_location();
    let logout = move |_| {
        api::clear_auth_token();
    };

    let is_active = move |path: &str| {
        let loc = location.pathname.get();
        if loc == path || loc.starts_with(path) { "active" } else { "" }
    };

    view! {
        <nav class="navbar">
            <div class="navbar-inner">
                <div class="navbar-brand">
                    <a href="/">
                        <span class="navbar-brand-logo">L</span>
                        <span>LAPES</span>
                    </a>
                </div>
                <div class="navbar-links">
                    <a href="/produtos" class=move || format!("navbar-link {}", is_active("/produtos"))>
                        "Produtos"
                    </a>
                    <a href="/carrinho" class=move || format!("navbar-link {}", is_active("/carrinho"))>
                        "Carrinho"
                    </a>
                    {move || {
                        if is_logged_in() {
                            view! {
                                <>
                                    <a href="/admin" class=move || format!("navbar-link {}", is_active("/admin"))>
                                        "Admin"
                                    </a>
                                    <button class="navbar-btn" on:click=logout>
                                        "Sair"
                                    </button>
                                </>
                            }.into_any()
                        } else {
                            view! {
                                <a href="/login" class=move || format!("navbar-link {}", is_active("/login"))>
                                    "Entrar"
                                </a>
                            }.into_any()
                        }
                    }}
                </div>
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

    let added = RwSignal::new(false);
    let qty = RwSignal::new(1u32);
    let toast = crate::toast::use_toast();

    let dec_qty = move |_| {
        let current = qty.get();
        if current > 1 {
            qty.set(current - 1);
        }
    };
    let inc_qty = move |_| {
        let current = qty.get();
        if current < 10 {
            qty.set(current + 1);
        }
    };  
    let qty_display = move || qty.get().to_string();

    let handle_add = move |pid: String| {
        added.set(true);
        let q = qty.get();
        toast.success(format!("{}x adicionado ao carrinho!", q));
        qty.set(1);
        leptos::task::spawn_local({
            let id = pid.clone();
            async move {
                let _ = api::api_post(
                    "/api/cart",
                    &serde_json::json!({ "product_id": id, "quantity": q }),
                ).await;
                // Reset after 2s
                let window = web_sys::window().unwrap();
                let closure = wasm_bindgen::closure::Closure::once(move || added.set(false));
                let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                    closure.as_ref().unchecked_ref(),
                    2000,
                );
                closure.forget();
            }
        });
    };

    view! {
        <div class="card">
            {if image.is_empty() {
                view! {
                    <div class="card-img-placeholder">
                        <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                            <path d="M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4"/>
                        </svg>
                    </div>
                }.into_any()
            } else {
                view! {
                    <img
                        src=image.clone()
                        alt=name.clone()
                        class="card-img"
                        loading="lazy"
                    />
                }.into_any()
            }}
            <div class="card-body">
                <h3 class="card-title">{name}</h3>
                <div class="card-price">{format!("R$ {:.2}", price)}</div>
                {if desc.is_empty() {
                    None
                } else {
                    Some(view! { <p class="card-desc">{desc}</p> })
                }}
                <div class="card-actions">
                    {id.map(|pid| {
                        view! {
                            <div style="display: flex; flex-direction: column; gap: 8px;">
                                <div class="qty-selector">
                                    <button class="qty-btn" on:click=dec_qty disabled=move || qty.get() <= 1>
                                        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                            <line x1="5" y1="12" x2="19" y2="12"/>
                                        </svg>
                                    </button>
                                    <span class="qty-value">{qty_display}</span>
                                    <button class="qty-btn" on:click=inc_qty disabled=move || qty.get() >= 10>
                                        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                            <line x1="12" y1="5" x2="12" y2="19"/>
                                            <line x1="5" y1="12" x2="19" y2="12"/>
                                        </svg>
                                    </button>
                                </div>
                                <button
                                    class=move || {
                                        if added.get() {
                                            "btn btn-success btn-sm"
                                        } else {
                                            "btn btn-primary btn-sm"
                                        }
                                    }
                                    disabled=move || added.get()
                                    on:click=move |_| handle_add(pid.clone())
                                >
                                    {move || {
                                        if added.get() { "\u{2713} Adicionado" } else { "Adicionar" }
                                    }}
                                </button>
                            </div>
                        }
                    })}
                </div>
            </div>
        </div>
    }
}

// ── Loading Spinner ────────────────────────────────────────────────────────

#[component]
pub fn Loading() -> impl IntoView {
    view! {
        <div class="loading-container">
            <div class="spinner"></div>
            <span class="loading-text">Carregando...</span>
        </div>
    }
}

// ── Skeleton Cards ─────────────────────────────────────────────────────────

#[component]
pub fn ProductSkeletonGrid() -> impl IntoView {
    view! {
        <div class="product-grid">
            {move || (0..8).map(|_| {
                view! {
                    <div class="skeleton-card">
                        <div class="skeleton skeleton-card-img"></div>
                        <div class="skeleton-card-body">
                            <div class="skeleton skeleton-line skeleton-line-md"></div>
                            <div class="skeleton skeleton-line skeleton-line-sm"></div>
                            <div class="skeleton skeleton-line skeleton-line-lg"></div>
                            <div class="skeleton skeleton-line-price"></div>
                            <div class="skeleton skeleton-btn"></div>
                        </div>
                    </div>
                }
            }).collect::<Vec<_>>()}
        </div>
    }
}

#[component]
pub fn TableSkeleton(rows: u32) -> impl IntoView {
    let rows = rows.max(3).min(20);
    view! {
        <div class="table-container">
            <table>
                <thead>
                    <tr>
                        <th style="width: 30%;"><div class="skeleton skeleton-line skeleton-line-sm" style="height: 10px;"></div></th>
                        <th style="width: 20%;"><div class="skeleton skeleton-line skeleton-line-sm" style="height: 10px;"></div></th>
                        <th style="width: 20%;"><div class="skeleton skeleton-line skeleton-line-sm" style="height: 10px;"></div></th>
                        <th style="width: 30%;"><div class="skeleton skeleton-line skeleton-line-sm" style="height: 10px;"></div></th>
                    </tr>
                </thead>
                <tbody>
                    {(0..rows).map(|_| {
                        view! {
                            <tr>
                                <td><div class="skeleton skeleton-line skeleton-line-md" style="height: 12px;"></div></td>
                                <td><div class="skeleton skeleton-line skeleton-line-sm" style="height: 12px;"></div></td>
                                <td><div class="skeleton skeleton-line skeleton-line-sm" style="height: 12px;"></div></td>
                                <td><div class="skeleton skeleton-line skeleton-line-md" style="height: 12px;"></div></td>
                            </tr>
                        }
                    }).collect::<Vec<_>>()}
                </tbody>
            </table>
        </div>
    }
}
