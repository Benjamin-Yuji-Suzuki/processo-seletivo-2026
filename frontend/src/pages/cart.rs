use crate::api;
use leptos::prelude::*;

#[component]
pub fn CartPage() -> impl IntoView {
    let cart = LocalResource::new(|| async move { api::api_get("/api/cart").await });
    let coupon = RwSignal::new(String::new());
    let checkout_msg = RwSignal::new(String::new());

    let perform_checkout = move |_| {
        let code = coupon.get();
        checkout_msg.set(String::new());
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
                    cart.refetch();
                }
                Err(e) => {
                    checkout_msg.set(format!("Erro: {}", e));
                }
            }
        });
    };

    let remove_item = move |item_id: &str| {
        let id = item_id.to_string();
        leptos::task::spawn_local(async move {
            let _ = api::api_delete(&format!("/api/cart/{}", id)).await;
            cart.refetch();
        });
    };

    let update_qty = move |item_id: &str, qty: i64| {
        let id = item_id.to_string();
        leptos::task::spawn_local(async move {
            let _ = api::api_put(
                &format!("/api/cart/{}", id),
                &serde_json::json!({ "quantity": qty }),
            )
            .await;
            cart.refetch();
        });
    };

    let token = api::get_auth_token();

    view! {
        <div style="max-width: 900px; margin: 0 auto;">
            <h1>"Carrinho"</h1>

            {if token.is_none() {
                view! {
                    <div style="text-align: center; padding: 2rem;">
                        <p>"Faça login para ver seu carrinho."</p>
                        <a href="/login" style="color: #3498db;">"Ir para Login"</a>
                    </div>
                }
                    .into_any()
            } else {
                view! {
                    <Transition fallback=move || view! { <crate::components::Loading/> }>
                        {move || {
                            cart.get().map(|result| match &*result {
                                Err(e) => {
                                    view! { <p style="color: red;">"Erro: " {e.clone()}</p> }
                                        .into_any()
                                }
                                Ok(data) => {
                                    let items = data["data"]
                                        .as_array()
                                        .or_else(|| data.as_array())
                                        .cloned()
                                        .unwrap_or_default();
                                    if items.is_empty() {
                                        view! {
                                            <div style="text-align: center; padding: 2rem;">
                                                <p>"Seu carrinho está vazio."</p>
                                            </div>
                                        }
                                            .into_any()
                                    } else {
                                        let total: f64 = items
                                            .iter()
                                            .filter_map(|item| {
                                                let qty = item["quantity"].as_i64().unwrap_or(0);
                                                let price = item["product"]["price"]
                                                    .as_f64()
                                                    .unwrap_or(0.0);
                                                Some(qty as f64 * price)
                                            })
                                            .sum();

                                        view! {
                                            <div>
                                                {items
                                                    .into_iter()
                                                    .map(|item| {
                                                        let item_id = item["id"]
                                                            .as_str()
                                                            .unwrap_or("")
                                                            .to_string();
                                                        let product_name = item["product"]
                                                            ["name"]
                                                            .as_str()
                                                            .unwrap_or("Produto")
                                                            .to_string();
                                                        let qty = item["quantity"]
                                                            .as_i64()
                                                            .unwrap_or(1);
                                                        let price = item["product"]["price"]
                                                            .as_f64()
                                                            .unwrap_or(0.0);
                                                        let subtotal = qty as f64 * price;

                                                        view! {
                                                            <div
                                                                style="
                                                                    display: flex;
                                                                    justify-content: space-between;
                                                                    align-items: center;
                                                                    border-bottom: 1px solid #eee;
                                                                    padding: 0.75rem 0;
                                                                "
                                                            >
                                                                <div style="flex: 1;">
                                                                    <strong>{product_name}</strong>
                                                                    <br/>
                                                                    <span style="color: #666;">
                                                                        {format!(
                                                                            "R$ {:.2} cada",
                                                                            price,
                                                                        )}
                                                                    </span>
                                                                </div>
                                                                <div style="display: flex; align-items: center; gap: 0.5rem;">
                                                                    <button
                                                                        style="padding: 0.25rem 0.5rem;"
                                                                        on:click={
                                                                            let id = item_id.clone();
                                                                            move |_| update_qty(&id, (qty - 1).max(1))
                                                                        }
                                                                    >
                                                                        "-"
                                                                    </button>
                                                                    <span>{qty}</span>
                                                                    <button
                                                                        style="padding: 0.25rem 0.5rem;"
                                                                        on:click={
                                                                            let id = item_id.clone();
                                                                            move |_| update_qty(&id, qty + 1)
                                                                        }
                                                                    >
                                                                        "+"
                                                                    </button>
                                                                </div>
                                                                <span style="font-weight: bold; min-width: 80px; text-align: right;">
                                                                    {format!("R$ {:.2}", subtotal)}
                                                                </span>
                                                                <button
                                                                    style="
                                                                        margin-left: 0.5rem;
                                                                        padding: 0.25rem 0.5rem;
                                                                        background: #dc3545;
                                                                        color: #fff;
                                                                        border: none;
                                                                        border-radius: 4px;
                                                                        cursor: pointer;
                                                                    "
                                                                    on:click={
                                                                        let id = item_id.clone();
                                                                        move |_| remove_item(&id)
                                                                    }
                                                                >
                                                                    "Remover"
                                                                </button>
                                                            </div>
                                                        }
                                                    })
                                                    .collect::<Vec<_>>()
                                                }

                                                <div style="margin-top: 1rem; text-align: right; font-size: 1.2rem; font-weight: bold;">
                                                    {format!("Total: R$ {:.2}", total)}
                                                </div>

                                                <div style="margin-top: 1rem; display: flex; gap: 0.5rem; align-items: center;">
                                                    <input
                                                        type="text"
                                                        placeholder="Cupom de desconto"
                                                        style="flex: 1; padding: 0.5rem;"
                                                        on:input=move |ev| {
                                                            coupon.set(event_target_value(&ev));
                                                        }
                                                        prop:value=coupon
                                                    />
                                                    <button
                                                        style="
                                                            padding: 0.5rem 1.5rem;
                                                            background: #007bff;
                                                            color: #fff;
                                                            border: none;
                                                            border-radius: 4px;
                                                            cursor: pointer;
                                                        "
                                                        on:click=perform_checkout
                                                    >
                                                        "Finalizar Pedido"
                                                    </button>
                                                </div>

                                                {if checkout_msg.get().is_empty() {
                                                    None
                                                } else {
                                                    Some(
                                                        view! {
                                                            <p style="margin-top: 0.5rem; color: #28a745;">
                                                                {move || checkout_msg.get()}
                                                            </p>
                                                        },
                                                    )
                                                }}
                                            </div>
                                        }
                                            .into_any()
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
