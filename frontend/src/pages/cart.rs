use crate::api;
use leptos::prelude::*;

#[component]
pub fn CartPage() -> impl IntoView {
    let cart = LocalResource::new(|| async move { api::api_get("/api/cart").await });
    let coupon = RwSignal::new(String::new());
    let checkout_msg = RwSignal::new(String::new());
    let checkout_loading = RwSignal::new(false);
    let removing_id = RwSignal::new(None::<String>);

    let _perform_checkout = move |_ev: leptos::ev::MouseEvent| {
        let code = coupon.get();
        checkout_msg.set(String::new());
        checkout_loading.set(true);
        leptos::task::spawn_local(async move {
            let mut body = serde_json::json!({});
            if !code.is_empty() {
                body["coupon_code"] = serde_json::json!(code);
            }
            match api::api_post("/api/checkout", &body).await {
                Ok(resp) => {
                    let msg = resp["message"]
                        .as_str()
                        .unwrap_or("Pedido realizado com sucesso!");
                    checkout_msg.set(msg.to_string());
                    coupon.set(String::new());
                    cart.refetch();
                }
                Err(e) => {
                    checkout_msg.set(format!("Erro: {}", e));
                }
            }
            checkout_loading.set(false);
        });
    };

    let remove_item = move |item_id: &str| {
        let id = item_id.to_string();
        removing_id.set(Some(id.clone()));
        leptos::task::spawn_local(async move {
            let _ = api::api_delete(&format!("/api/cart/{:}", id)).await;
            removing_id.set(None);
            cart.refetch();
        });
    };

    let update_qty = move |item_id: &str, qty: i64| {
        let id = item_id.to_string();
        leptos::task::spawn_local(async move {
            let _ = api::api_put(
                &format!("/api/cart/{:}", id),
                &serde_json::json!({ "quantity": qty }),
            ).await;
            cart.refetch();
        });
    };

    let token = api::get_auth_token();

    view! {
        <div class="container cart-container" style="margin-top: 1.5rem;">
            <h1>Carrinho</h1>

            {if token.is_none() {
                view! {
                    <div class="empty-state" style="margin-top: 1rem;">
                        <div class="empty-state-icon">
                            <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                                <rect x="3" y="11" width="18" height="11" rx="2" ry="2"/><path d="M7 11V7a5 5 0 0110 0v4"/>
                            </svg>
                        </div>
                        <h3>Faça login para ver seu carrinho</h3>
                        <p>Você precisa estar logado para acessar o carrinho.</p>
                        <a href="/login" class="btn btn-primary">Ir para Login</a>
                    </div>
                }.into_any()
            } else {
                view! {
                    <Transition fallback=move || view! { <crate::components::Loading/> }>
                        {move || {
                            cart.get().map(|result| match &*result {
                                Err(e) => {
                                    view! {
                                        <div class="alert alert-error" style="margin-top: 1rem;">
                                            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="15" y1="9" x2="9" y2="15"/><line x1="9" y1="9" x2="15" y2="15"/></svg>
                                            <span>Erro: {e.clone()}</span>
                                        </div>
                                    }.into_any()
                                }
                                Ok(data) => {
                                    let items = data["items"]
                                        .as_array()
                                        .or_else(|| data.as_array())
                                        .cloned()
                                        .unwrap_or_default();
                                    if items.is_empty() {
                                        view! {
                                            <div class="empty-state" style="margin-top: 1rem;">
                                                <div class="empty-state-icon">
                                                    <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                                                        <circle cx="8" cy="21" r="1"/><circle cx="21" cy="21" r="1"/>
                                                        <path d="M3 3h2l.4 2M7 13h10l4-8H5.4"/>
                                                    </svg>
                                                </div>
                                                <h3>Seu carrinho está vazio</h3>
                                                <p>Adicione produtos para começar suas compras.</p>
                                                <a href="/produtos" class="btn btn-primary">Ver Produtos</a>
                                            </div>
                                        }.into_any()
                                    } else {
                                        let total: f64 = items
                                            .iter()
                                            .filter_map(|item| {
                                                let qty = item["quantity"].as_i64().unwrap_or(0);
                                                let price = item["unit_price"]
                                                    .as_f64()
                                                    .unwrap_or(0.0);
                                                Some(qty as f64 * price)
                                            })
                                            .sum();

                                        let checkout_loading2 = checkout_loading;
                                        let coupon2 = coupon;
                                        let checkout_msg2 = checkout_msg;
                                        let cart2 = cart;

                                        view! {
                                            <div style="margin-top: 1rem;">
                                                {items
                                                    .into_iter()
                                                    .map(|item| {
                                                        let item_id = item["id"]
                                                            .as_str()
                                                            .unwrap_or("")
                                                            .to_string();
                                                        let product_name = item["product_name"]
                                                            .as_str()
                                                            .unwrap_or("Produto")
                                                            .to_string();
                                                        let qty = item["quantity"]
                                                            .as_i64()
                                                            .unwrap_or(1);
                                                        let price = item["unit_price"]
                                                            .as_f64()
                                                            .unwrap_or(0.0);
                                                        let subtotal = qty as f64 * price;
                                                        let item_id_for_removing = item_id.clone();
                                                        let is_removing_fn = {
                                                            let item_id = item_id_for_removing.clone();
                                                            move || removing_id.get().as_deref() == Some(&item_id)
                                                        };
                                                        let is_removing_fn2 = {
                                                            let item_id = item_id_for_removing;
                                                            move || removing_id.get().as_deref() == Some(&item_id)
                                                        };

                                                        view! {
                                                            <div class="cart-item">
                                                                <div class="cart-item-info">
                                                                    <div class="cart-item-name">{product_name}</div>
                                                                    <div class="cart-item-price">{format!("R$ {:.2} cada", price)}</div>
                                                                </div>
                                                                <div class="cart-item-qty">
                                                                    <button
                                                                        class="btn btn-ghost btn-sm"
                                                                        disabled=move || qty <= 1
                                                                        on:click={
                                                                            let id = item_id.clone();
                                                                            move |_| update_qty(&id, (qty - 1).max(1))
                                                                        }
                                                                    >-</button>
                                                                    <span class="cart-item-qty-value">{qty}</span>
                                                                    <button
                                                                        class="btn btn-ghost btn-sm"
                                                                        on:click={
                                                                            let id = item_id.clone();
                                                                            move |_| update_qty(&id, qty + 1)
                                                                        }
                                                                    >+</button>
                                                                </div>
                                                                <span style="font-weight: 600; min-width: 80px; text-align: right;">
                                                                    {format!("R$ {:.2}", subtotal)}
                                                                </span>
                                                                <button
                                                                    class="btn btn-danger btn-sm"
                                                                    disabled=move || is_removing_fn()
                                                                    on:click={
                                                                        let id = item_id.clone();
                                                                        move |_| remove_item(&id)
                                                                    }
                                                                >
                                                                    {move || {
                                                                        if is_removing_fn2() { "Removendo..." } else { "Remover" }
                                                                    }}
                                                                </button>
                                                            </div>
                                                        }
                                                    })
                                                    .collect::<Vec<_>>()
                                                }

                                                <div class="cart-total">
                                                    {format!("Total: R$ {:.2}", total)}
                                                </div>

                                                <div class="cart-footer">
                                                    <input
                                                        type="text"
                                                        placeholder="Cupom de desconto"
                                                        class="form-input"
                                                        on:input=move |ev| {
                                                            coupon2.set(event_target_value(&ev));
                                                        }
                                                        prop:value=coupon2
                                                    />
                                                    <button
                                                        class="btn btn-success btn-lg"
                                                        disabled=move || checkout_loading2.get()
                                                        on:click=move |_| {
                                                            let code = coupon2.get();
                                                            checkout_msg2.set(String::new());
                                                            checkout_loading2.set(true);
                                                            leptos::task::spawn_local(async move {
                                                                let mut body = serde_json::json!({});
                                                                if !code.is_empty() {
                                                                    body["coupon_code"] = serde_json::json!(code);
                                                                }
                                                                match api::api_post("/api/checkout", &body).await {
                                                                    Ok(resp) => {
                                                                        let msg = resp["message"]
                                                                            .as_str()
                                                                            .unwrap_or("Pedido realizado com sucesso!");
                                                                        checkout_msg2.set(msg.to_string());
                                                                        coupon2.set(String::new());
                                                                        cart2.refetch();
                                                                    }
                                                                    Err(e) => {
                                                                        checkout_msg2.set(format!("Erro: {}", e));
                                                                    }
                                                                }
                                                                checkout_loading2.set(false);
                                                            });
                                                        }
                                                    >
                                                        {move || {
                                                            if checkout_loading2.get() {
                                                                "Finalizando..."
                                                            } else {
                                                                "Finalizar Pedido"
                                                            }
                                                        }}
                                                    </button>
                                                </div>

                                                {if checkout_msg2.get().is_empty() {
                                                    None
                                                } else {
                                                    let is_err = checkout_msg2.get().starts_with("Erro");
                                                    Some(
                                                        view! {
                                                            <div class=if is_err { "alert alert-error" } else { "alert alert-success" } style="margin-top: 1rem;">
                                                                <span>{move || checkout_msg2.get()}</span>
                                                            </div>
                                                        },
                                                    )
                                                }}
                                            </div>
                                        }.into_any()
                                    }
                                }
                            })
                        }}
                    </Transition>
                }
                    .into_any()
            }}
        </div>
    }
}
