use crate::api;
use leptos::prelude::*;
use serde_json::Value;

#[component]
pub fn AdminPanel() -> impl IntoView {
    let token = api::get_auth_token();
    let tab = RwSignal::new("products".to_string());

    view! {
        <div style="max-width: 1200px; margin: 0 auto;">
            <h1>"Painel Administrativo"</h1>

            {if token.is_none() {
                view! {
                    <div style="text-align: center; padding: 2rem;">
                        <p>"Você precisa estar logado como administrador."</p>
                        <a href="/login" style="color: #3498db;">"Ir para Login"</a>
                    </div>
                }
                    .into_any()
            } else {
                view! {
                    <div>
                        <div style="display: flex; gap: 0.5rem; margin-bottom: 1rem;">
                            <button
                                style=move || {
                                    format!(
                                        "padding: 0.5rem 1rem; border: 1px solid #ccc; border-radius: 4px; cursor: pointer; {}",
                                        if tab.get() == "products" {
                                            "background: #007bff; color: #fff;"
                                        } else {
                                            "background: #fff; color: #333;"
                                        },
                                    )
                                }
                                on:click=move |_| tab.set("products".to_string())
                            >
                                "Produtos"
                            </button>
                            <button
                                style=move || {
                                    format!(
                                        "padding: 0.5rem 1rem; border: 1px solid #ccc; border-radius: 4px; cursor: pointer; {}",
                                        if tab.get() == "orders" {
                                            "background: #007bff; color: #fff;"
                                        } else {
                                            "background: #fff; color: #333;"
                                        },
                                    )
                                }
                                on:click=move |_| tab.set("orders".to_string())
                            >
                                "Pedidos"
                            </button>
                            <button
                                style=move || {
                                    format!(
                                        "padding: 0.5rem 1rem; border: 1px solid #ccc; border-radius: 4px; cursor: pointer; {}",
                                        if tab.get() == "coupons" {
                                            "background: #007bff; color: #fff;"
                                        } else {
                                            "background: #fff; color: #333;"
                                        },
                                    )
                                }
                                on:click=move |_| tab.set("coupons".to_string())
                            >
                                "Cupons"
                            </button>
                        </div>

                        {move || match tab.get().as_str() {
                            "products" => view! { <ProductManagement/> }.into_any(),
                            "orders" => view! { <OrderManagement/> }.into_any(),
                            "coupons" => view! { <CouponManagement/> }.into_any(),
                            _ => view! { <p>"Selecione uma aba."</p> }.into_any(),
                        }}
                    </div>
                }
                    .into_any()
            }}
        </div>
    }
}

// ── Product Management ─────────────────────────────────────────────────────

#[component]
fn ProductManagement() -> impl IntoView {
    let products = LocalResource::new(|| async move {
        api::api_get("/api/products?page=1&limit=100").await
    });

    let name = RwSignal::new(String::new());
    let description = RwSignal::new(String::new());
    let price = RwSignal::new(String::new());
    let category = RwSignal::new(String::new());
    let image_url = RwSignal::new(String::new());
    let editing_id = RwSignal::new(None::<String>);
    let form_msg = RwSignal::new(String::new());

    let submit_product = move |_| {
        let id = editing_id.get();
        form_msg.set(String::new());

        let p: f64 = match price.get().parse() {
            Ok(v) => v,
            Err(_) => {
                form_msg.set("Preço inválido.".to_string());
                return;
            }
        };

        let body = serde_json::json!({
            "name": name.get(),
            "description": description.get(),
            "price": p,
            "category": category.get(),
            "image_url": image_url.get(),
        });

        let is_new = id.is_none();
        let path = match &id {
            Some(id) => format!("/api/products/{}", id),
            None => "/api/products".to_string(),
        };

        leptos::task::spawn_local(async move {
            let result = if is_new {
                api::api_post(&path, &body).await
            } else {
                api::api_put(&path, &body).await
            };
            match result {
                Ok(_) => {
                    form_msg.set(if is_new {
                        "Produto criado!".to_string()
                    } else {
                        "Produto atualizado!".to_string()
                    });
                    name.set(String::new());
                    description.set(String::new());
                    price.set(String::new());
                    category.set(String::new());
                    image_url.set(String::new());
                    editing_id.set(None);
                    products.refetch();
                }
                Err(e) => {
                    form_msg.set(format!("Erro: {}", e));
                }
            }
        });
    };

    let edit_product = move |product: Value| {
        name.set(product["name"].as_str().unwrap_or("").to_string());
        description.set(product["description"].as_str().unwrap_or("").to_string());
        price.set(product["price"].as_f64().unwrap_or(0.0).to_string());
        category.set(product["category"].as_str().unwrap_or("").to_string());
        image_url.set(product["image_url"].as_str().unwrap_or("").to_string());
        editing_id.set(
            product["id"]
                .as_str()
                .map(|s| s.to_string())
                .or_else(|| product["id"].as_i64().map(|i| i.to_string())),
        );
    };

    let delete_product = move |product_id: String| {
        leptos::task::spawn_local({
            let id = product_id.clone();
            async move {
                let _ = api::api_delete(&format!("/api/products/{}", id)).await;
                products.refetch();
            }
        });
    };

    let cancel_edit = move |_| {
        name.set(String::new());
        description.set(String::new());
        price.set(String::new());
        category.set(String::new());
        image_url.set(String::new());
        editing_id.set(None);
        form_msg.set(String::new());
    };

    view! {
        <div>
            <h2>
                {move || {
                    if editing_id.get().is_some() {
                        "Editar Produto"
                    } else {
                        "Novo Produto"
                    }
                }}
            </h2>

            {if !form_msg.get().is_empty() {
                Some(
                    view! {
                        <p
                            style=format!(
                                "padding: 0.5rem; border-radius: 4px; {}",
                                if form_msg.get().starts_with("Erro") {
                                    "color: #721c24; background: #f8d7da;"
                                } else {
                                    "color: #155724; background: #d4edda;"
                                },
                            )
                        >
                            {move || form_msg.get()}
                        </p>
                    },
                )
            } else {
                None
            }}

            <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 0.5rem; margin-bottom: 1rem;">
                <input
                    type="text"
                    placeholder="Nome"
                    style="padding: 0.5rem;"
                    prop:value=name
                    on:input=move |ev| name.set(event_target_value(&ev))
                />
                <input
                    type="text"
                    placeholder="Preço (ex: 29.90)"
                    style="padding: 0.5rem;"
                    prop:value=price
                    on:input=move |ev| price.set(event_target_value(&ev))
                />
                <input
                    type="text"
                    placeholder="Descrição"
                    style="padding: 0.5rem;"
                    prop:value=description
                    on:input=move |ev| description.set(event_target_value(&ev))
                />
                <input
                    type="text"
                    placeholder="Categoria"
                    style="padding: 0.5rem;"
                    prop:value=category
                    on:input=move |ev| category.set(event_target_value(&ev))
                />
                <input
                    type="text"
                    placeholder="URL da imagem"
                    style="padding: 0.5rem; grid-column: span 2;"
                    prop:value=image_url
                    on:input=move |ev| image_url.set(event_target_value(&ev))
                />
            </div>

            <div style="display: flex; gap: 0.5rem;">
                <button
                    style="
                        padding: 0.5rem 1.5rem;
                        background: #28a745; color: #fff;
                        border: none; border-radius: 4px; cursor: pointer;
                    "
                    on:click=submit_product
                >
                    {move || {
                        if editing_id.get().is_some() {
                            "Atualizar"
                        } else {
                            "Criar Produto"
                        }
                    }}
                </button>
                <button
                    style="
                        padding: 0.5rem 1.5rem;
                        background: #6c757d; color: #fff;
                        border: none; border-radius: 4px; cursor: pointer;
                    "
                    on:click=cancel_edit
                >
                    "Cancelar"
                </button>
            </div>

            <h3 style="margin-top: 2rem;">"Produtos Existentes"</h3>
            <Transition fallback=move || view! { <crate::components::Loading/> }>
                {move || {
                    products.get().map(|result| match &*result {
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
                                view! { <p>"Nenhum produto cadastrado."</p> }.into_any()
                            } else {
                                view! {
                                    <table style="width: 100%; border-collapse: collapse;">
                                        <thead>
                                            <tr style="background: #f5f5f5;">
                                                <th style="padding: 0.5rem; text-align: left;">"Nome"</th>
                                                <th style="padding: 0.5rem; text-align: left;">"Preço"</th>
                                                <th style="padding: 0.5rem; text-align: left;">"Categoria"</th>
                                                <th style="padding: 0.5rem; text-align: left;">"Ações"</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {items
                                                .into_iter()
                                                .map(|p| {
                                                    let pid = p["id"]
                                                        .as_str()
                                                        .unwrap_or("")
                                                        .to_string();
                                                    let p_name = p["name"]
                                                        .as_str()
                                                        .unwrap_or("")
                                                        .to_string();
                                                    let p_price = p["price"].as_f64().unwrap_or(0.0);
                                                    let p_cat = p["category"]
                                                        .as_str()
                                                        .unwrap_or("")
                                                        .to_string();
                                                    let p_clone = p.clone();
                                                    view! {
                                                        <tr style="border-bottom: 1px solid #eee;">
                                                            <td style="padding: 0.5rem;">
                                                                {p_name}
                                                            </td>
                                                            <td style="padding: 0.5rem;">
                                                                {format!("R$ {:.2}", p_price)}
                                                            </td>
                                                            <td style="padding: 0.5rem;">
                                                                {p_cat}
                                                            </td>
                                                            <td style="padding: 0.5rem;">
                                                                <button
                                                                    style="
                                                                        padding: 0.25rem 0.5rem;
                                                                        background: #ffc107;
                                                                        border: none;
                                                                        border-radius: 4px;
                                                                        cursor: pointer;
                                                                        margin-right: 0.25rem;
                                                                    "
                                                                    on:click=move |_| edit_product(p_clone.clone())
                                                                >
                                                                    "Editar"
                                                                </button>
                                                                <button
                                                                    style="
                                                                        padding: 0.25rem 0.5rem;
                                                                        background: #dc3545;
                                                                        color: #fff;
                                                                        border: none;
                                                                        border-radius: 4px;
                                                                        cursor: pointer;
                                                                    "
                                                                    on:click=move |_| {
                                                                        delete_product(pid.clone())
                                                                    }
                                                                >
                                                                    "Excluir"
                                                                </button>
                                                            </td>
                                                        </tr>
                                                    }
                                                })
                                                .collect::<Vec<_>>()
                                            }
                                        </tbody>
                                    </table>
                                }
                                    .into_any()
                            }
                        }
                    })
                }}
            </Transition>
        </div>
    }
}

// ── Order Management ───────────────────────────────────────────────────────

#[component]
fn OrderManagement() -> impl IntoView {
    let orders = LocalResource::new(|| async move {
        api::api_get("/api/admin/orders").await
    });

    let update_status = move |order_id: &str, new_status: &str| {
        let id = order_id.to_string();
        let status = new_status.to_string();
        leptos::task::spawn_local(async move {
            let _ = api::api_put(
                &format!("/api/admin/orders/{}", id),
                &serde_json::json!({ "status": status }),
            )
            .await;
            orders.refetch();
        });
    };

    view! {
        <div>
            <h2>"Gerenciar Pedidos"</h2>
            <Transition fallback=move || view! { <crate::components::Loading/> }>
                {move || {
                    orders.get().map(|result| match &*result {
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
                                view! { <p>"Nenhum pedido encontrado."</p> }.into_any()
                            } else {
                                view! {
                                    <table style="width: 100%; border-collapse: collapse;">
                                        <thead>
                                            <tr style="background: #f5f5f5;">
                                                <th style="padding: 0.5rem; text-align: left;">"ID"</th>
                                                <th style="padding: 0.5rem; text-align: left;">"Status"</th>
                                                <th style="padding: 0.5rem; text-align: left;">"Total"</th>
                                                <th style="padding: 0.5rem; text-align: left;">"Ações"</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {items
                                                .into_iter()
                                                .map(|order| {
                                                    let oid = order["id"]
                                                        .as_str()
                                                        .unwrap_or("")
                                                        .to_string();
                                                    let oid_short = oid[..oid.len().min(8)].to_string();
                                                    let current_status = order["status"]
                                                        .as_str()
                                                        .unwrap_or("unknown")
                                                        .to_string();
                                                    let order_total = order["total"]
                                                        .as_f64()
                                                        .unwrap_or(0.0);
                                                    view! {
                                                        <tr style="border-bottom: 1px solid #eee;">
                                                            <td style="padding: 0.5rem; font-family: monospace; font-size: 0.85rem;">
                                                                {oid_short}
                                                            </td>
                                                            <td style="padding: 0.5rem;">
                                                                {current_status.clone()}
                                                            </td>
                                                            <td style="padding: 0.5rem;">
                                                                {format!("R$ {:.2}", order_total)}
                                                            </td>
                                                            <td style="padding: 0.5rem;">
                                                                <select
                                                                    style="padding: 0.25rem;"
                                                                    on:change=move |ev| {
                                                                        let val = event_target_value(&ev);
                                                                        update_status(&oid, &val);
                                                                    }
                                                                >
                                                                    <option
                                                                        value="pending"
                                                                        selected=current_status == "pending"
                                                                    >
                                                                        "Pendente"
                                                                    </option>
                                                                    <option
                                                                        value="confirmed"
                                                                        selected=current_status == "confirmed"
                                                                    >
                                                                        "Confirmado"
                                                                    </option>
                                                                    <option
                                                                        value="shipped"
                                                                        selected=current_status == "shipped"
                                                                    >
                                                                        "Enviado"
                                                                    </option>
                                                                    <option
                                                                        value="delivered"
                                                                        selected=current_status == "delivered"
                                                                    >
                                                                        "Entregue"
                                                                    </option>
                                                                    <option
                                                                        value="cancelled"
                                                                        selected=current_status == "cancelled"
                                                                    >
                                                                        "Cancelado"
                                                                    </option>
                                                                </select>
                                                            </td>
                                                        </tr>
                                                    }
                                                })
                                                .collect::<Vec<_>>()
                                            }
                                        </tbody>
                                    </table>
                                }
                                    .into_any()
                            }
                        }
                    })
                }}
            </Transition>
        </div>
    }
}

// ── Coupon Management ──────────────────────────────────────────────────────

#[component]
fn CouponManagement() -> impl IntoView {
    let coupons = LocalResource::new(|| async move {
        api::api_get("/api/coupons").await
    });

    let code = RwSignal::new(String::new());
    let discount = RwSignal::new(String::new());
    let kind = RwSignal::new("percentage".to_string());
    let max_uses = RwSignal::new(String::new());
    let expires_at = RwSignal::new(String::new());
    let editing_id = RwSignal::new(None::<String>);
    let form_msg = RwSignal::new(String::new());

    let submit_coupon = move |_| {
        let id = editing_id.get();
        form_msg.set(String::new());

        let disc: f64 = match discount.get().parse() {
            Ok(v) => v,
            Err(_) => {
                form_msg.set("Desconto inválido.".to_string());
                return;
            }
        };

        let body = serde_json::json!({
            "code": code.get(),
            "discount": disc,
            "kind": kind.get(),
            "max_uses": max_uses.get().parse::<i64>().unwrap_or(0),
            "expires_at": expires_at.get(),
        });

        let is_new = id.is_none();
        let path = match &id {
            Some(id) => format!("/api/coupons/{}", id),
            None => "/api/coupons".to_string(),
        };

        leptos::task::spawn_local(async move {
            let result = if is_new {
                api::api_post(&path, &body).await
            } else {
                api::api_put(&path, &body).await
            };
            match result {
                Ok(_) => {
                    form_msg.set(if is_new {
                        "Cupom criado!".to_string()
                    } else {
                        "Cupom atualizado!".to_string()
                    });
                    code.set(String::new());
                    discount.set(String::new());
                    kind.set("percentage".to_string());
                    max_uses.set(String::new());
                    expires_at.set(String::new());
                    editing_id.set(None);
                    coupons.refetch();
                }
                Err(e) => {
                    form_msg.set(format!("Erro: {}", e));
                }
            }
        });
    };

    let edit_coupon = move |coupon: Value| {
        code.set(coupon["code"].as_str().unwrap_or("").to_string());
        discount.set(coupon["discount"].as_f64().unwrap_or(0.0).to_string());
        kind.set(
            coupon["kind"]
                .as_str()
                .unwrap_or("percentage")
                .to_string(),
        );
        max_uses.set(
            coupon["max_uses"]
                .as_i64()
                .unwrap_or(0)
                .to_string(),
        );
        expires_at.set(coupon["expires_at"].as_str().unwrap_or("").to_string());
        editing_id.set(
            coupon["id"]
                .as_str()
                .map(|s| s.to_string())
                .or_else(|| coupon["id"].as_i64().map(|i| i.to_string())),
        );
    };

    let delete_coupon = move |coupon_id: String| {
        leptos::task::spawn_local({
            let id = coupon_id.clone();
            async move {
                let _ = api::api_delete(&format!("/api/coupons/{}", id)).await;
                coupons.refetch();
            }
        });
    };

    let cancel_edit = move |_| {
        code.set(String::new());
        discount.set(String::new());
        kind.set("percentage".to_string());
        max_uses.set(String::new());
        expires_at.set(String::new());
        editing_id.set(None);
        form_msg.set(String::new());
    };

    view! {
        <div>
            <h2>
                {move || {
                    if editing_id.get().is_some() {
                        "Editar Cupom"
                    } else {
                        "Novo Cupom"
                    }
                }}
            </h2>

            {if !form_msg.get().is_empty() {
                Some(
                    view! {
                        <p
                            style=format!(
                                "padding: 0.5rem; border-radius: 4px; {}",
                                if form_msg.get().starts_with("Erro") {
                                    "color: #721c24; background: #f8d7da;"
                                } else {
                                    "color: #155724; background: #d4edda;"
                                },
                            )
                        >
                            {move || form_msg.get()}
                        </p>
                    },
                )
            } else {
                None
            }}

            <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 0.5rem; margin-bottom: 1rem;">
                <input
                    type="text"
                    placeholder="Código (ex: LAPES10)"
                    style="padding: 0.5rem;"
                    prop:value=code
                    on:input=move |ev| code.set(event_target_value(&ev))
                />
                <input
                    type="text"
                    placeholder="Desconto (ex: 10)"
                    style="padding: 0.5rem;"
                    prop:value=discount
                    on:input=move |ev| discount.set(event_target_value(&ev))
                />
                <select
                    style="padding: 0.5rem;"
                    prop:value=kind
                    on:change=move |ev| kind.set(event_target_value(&ev))
                >
                    <option value="percentage">"Percentual"</option>
                    <option value="fixed">"Valor fixo"</option>
                </select>
                <input
                    type="text"
                    placeholder="Usos máximos"
                    style="padding: 0.5rem;"
                    prop:value=max_uses
                    on:input=move |ev| max_uses.set(event_target_value(&ev))
                />
                <input
                    type="text"
                    placeholder="Expira em (YYYY-MM-DD)"
                    style="padding: 0.5rem; grid-column: span 2;"
                    prop:value=expires_at
                    on:input=move |ev| expires_at.set(event_target_value(&ev))
                />
            </div>

            <div style="display: flex; gap: 0.5rem;">
                <button
                    style="
                        padding: 0.5rem 1.5rem;
                        background: #28a745; color: #fff;
                        border: none; border-radius: 4px; cursor: pointer;
                    "
                    on:click=submit_coupon
                >
                    {move || {
                        if editing_id.get().is_some() {
                            "Atualizar"
                        } else {
                            "Criar Cupom"
                        }
                    }}
                </button>
                <button
                    style="
                        padding: 0.5rem 1.5rem;
                        background: #6c757d; color: #fff;
                        border: none; border-radius: 4px; cursor: pointer;
                    "
                    on:click=cancel_edit
                >
                    "Cancelar"
                </button>
            </div>

            <h3 style="margin-top: 2rem;">"Cupons Existentes"</h3>
            <Transition fallback=move || view! { <crate::components::Loading/> }>
                {move || {
                    coupons.get().map(|result| match &*result {
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
                                view! { <p>"Nenhum cupom cadastrado."</p> }.into_any()
                            } else {
                                view! {
                                    <table style="width: 100%; border-collapse: collapse;">
                                        <thead>
                                            <tr style="background: #f5f5f5;">
                                                <th style="padding: 0.5rem; text-align: left;">"Código"</th>
                                                <th style="padding: 0.5rem; text-align: left;">"Desconto"</th>
                                                <th style="padding: 0.5rem; text-align: left;">"Tipo"</th>
                                                <th style="padding: 0.5rem; text-align: left;">"Ações"</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {items
                                                .into_iter()
                                                .map(|c| {
                                                    let cid = c["id"]
                                                        .as_str()
                                                        .unwrap_or("")
                                                        .to_string();
                                                    let c_code = c["code"]
                                                        .as_str()
                                                        .unwrap_or("")
                                                        .to_string();
                                                    let c_discount = c["discount"]
                                                        .as_f64()
                                                        .unwrap_or(0.0);
                                                    let c_kind = c["kind"]
                                                        .as_str()
                                                        .unwrap_or("")
                                                        .to_string();
                                                    let c_clone = c.clone();
                                                    view! {
                                                        <tr style="border-bottom: 1px solid #eee;">
                                                            <td style="padding: 0.5rem;">
                                                                {c_code}
                                                            </td>
                                                            <td style="padding: 0.5rem;">
                                                                {c_discount}
                                                            </td>
                                                            <td style="padding: 0.5rem;">
                                                                {c_kind}
                                                            </td>
                                                            <td style="padding: 0.5rem;">
                                                                <button
                                                                    style="
                                                                        padding: 0.25rem 0.5rem;
                                                                        background: #ffc107;
                                                                        border: none;
                                                                        border-radius: 4px;
                                                                        cursor: pointer;
                                                                        margin-right: 0.25rem;
                                                                    "
                                                                    on:click=move |_| edit_coupon(c_clone.clone())
                                                                >
                                                                    "Editar"
                                                                </button>
                                                                <button
                                                                    style="
                                                                        padding: 0.25rem 0.5rem;
                                                                        background: #dc3545;
                                                                        color: #fff;
                                                                        border: none;
                                                                        border-radius: 4px;
                                                                        cursor: pointer;
                                                                    "
                                                                    on:click=move |_| {
                                                                        delete_coupon(cid.clone())
                                                                    }
                                                                >
                                                                    "Excluir"
                                                                </button>
                                                            </td>
                                                        </tr>
                                                    }
                                                })
                                                .collect::<Vec<_>>()
                                            }
                                        </tbody>
                                    </table>
                                }
                                    .into_any()
                            }
                        }
                    })
                }}
            </Transition>
        </div>
    }
}
