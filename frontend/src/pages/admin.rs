use crate::api;
use leptos::prelude::*;
use serde_json::Value;

#[component]
pub fn AdminPanel() -> impl IntoView {
    let token = api::get_auth_token();
    let tab = RwSignal::new("products".to_string());

    view! {
        <div class="container" style="margin-top: 1.5rem;">
            <div class="page-header">
                <h1>Painel Administrativo</h1>
            </div>

            {if token.is_none() {
                view! {
                    <div class="empty-state" style="margin-top: 1rem;">
                        <div class="empty-state-icon">
                            <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                                <rect x="3" y="11" width="18" height="11" rx="2" ry="2"/><path d="M7 11V7a5 5 0 0110 0v4"/>
                            </svg>
                        </div>
                        <h3>Você precisa estar logado como administrador</h3>
                        <p>Faça login para acessar o painel administrativo.</p>
                        <a href="/login" class="btn btn-primary">Ir para Login</a>
                    </div>
                }.into_any()
            } else {
                view! {
                    <div>
                        <div class="admin-tabs">
                            <button
                                class=move || {
                                    if tab.get() == "products" { "admin-tab active" } else { "admin-tab" }
                                }
                                on:click=move |_| tab.set("products".to_string())
                            >
                                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="vertical-align: middle; margin-right: 4px;">
                                    <path d="M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4"/>
                                </svg>
                                Produtos
                            </button>
                            <button
                                class=move || {
                                    if tab.get() == "orders" { "admin-tab active" } else { "admin-tab" }
                                }
                                on:click=move |_| tab.set("orders".to_string())
                            >
                                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="vertical-align: middle; margin-right: 4px;">
                                    <path d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2"/><rect x="9" y="3" width="6" height="4" rx="1"/><path d="M9 14l2 2 4-4"/>
                                </svg>
                                Pedidos
                            </button>
                            <button
                                class=move || {
                                    if tab.get() == "coupons" { "admin-tab active" } else { "admin-tab" }
                                }
                                on:click=move |_| tab.set("coupons".to_string())
                            >
                                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="vertical-align: middle; margin-right: 4px;">
                                    <path d="M20 12V8H6a2 2 0 01-2-2c0-1.1.9-2 2-2h12v4"/><path d="M4 6v12c0 1.1.9 2 2 2h14v-4"/><path d="M18 12a2 2 0 00-2 2c0 1.1.9 2 2 2h4v-4h-4z"/>
                                </svg>
                                Cupons
                            </button>
                        </div>

                        {move || match tab.get().as_str() {
                            "products" => view! { <ProductManagement/> }.into_any(),
                            "orders" => view! { <OrderManagement/> }.into_any(),
                            "coupons" => view! { <CouponManagement/> }.into_any(),
                            _ => view! { <p>Selecione uma aba.</p> }.into_any(),
                        }}
                    </div>
                }.into_any()
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
        let toast = crate::toast::use_toast();

        let name_val = name.get();
        if name_val.trim().is_empty() {
            form_msg.set("Nome do produto é obrigatório.".to_string());
            toast.error("Nome do produto é obrigatório.");
            return;
        }

        let p: f64 = match price.get().parse() {
            Ok(v) if v > 0.0 => v,
            Ok(_) => {
                form_msg.set("Preço deve ser maior que zero.".to_string());
                toast.error("Preço deve ser maior que zero.");
                return;
            }
            Err(_) => {
                form_msg.set("Preço inválido. Use números (ex: 29.90).".to_string());
                toast.error("Preço inválido.");
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
                    let msg = if is_new { "Produto criado com sucesso!" } else { "Produto atualizado com sucesso!" };
                    form_msg.set(msg.to_string());
                    toast.success(msg);
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
                    toast.error(format!("Erro ao {} produto: {}", if is_new { "criar" } else { "atualizar" }, e));
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
        let window = web_sys::window().unwrap();
        if !window.confirm_with_message("Tem certeza que deseja excluir este produto?").unwrap_or(false) {
            return;
        }
        let toast = crate::toast::use_toast();
        leptos::task::spawn_local({
            let id = product_id.clone();
            async move {
                let result = api::api_delete(&format!("/api/products/{}", id)).await;
                match result {
                    Ok(_) => {
                        toast.success("Produto excluído com sucesso!");
                        products.refetch();
                    }
                    Err(e) => {
                        toast.error(format!("Erro ao excluir produto: {}", e));
                    }
                }
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
            <div class="form-card" style="margin-bottom: 2rem;">
                <h2>
                    {move || {
                        if editing_id.get().is_some() { "Editar Produto" } else { "Novo Produto" }
                    }}
                </h2>

                {if !form_msg.get().is_empty() {
                    let is_err = form_msg.get().starts_with("Erro");
                    Some(view! {
                        <div class=if is_err { "alert alert-error" } else { "alert alert-success" }>
                            <span>{move || form_msg.get()}</span>
                        </div>
                    })
                } else {
                    None
                }}

                <div class="admin-form-layout">
                    <input
                        type="text"
                        placeholder="Nome do produto"
                        class="form-input"
                        prop:value=name
                        on:input=move |ev| name.set(event_target_value(&ev))
                    />
                    <input
                        type="text"
                        placeholder="Preço (ex: 29.90)"
                        class="form-input"
                        prop:value=price
                        on:input=move |ev| price.set(event_target_value(&ev))
                    />
                    <input
                        type="text"
                        placeholder="Descrição"
                        class="form-input"
                        prop:value=description
                        on:input=move |ev| description.set(event_target_value(&ev))
                    />
                    <input
                        type="text"
                        placeholder="Categoria"
                        class="form-input"
                        prop:value=category
                        on:input=move |ev| category.set(event_target_value(&ev))
                    />
                    <input
                        type="text"
                        placeholder="URL da imagem"
                        class="form-input full-width"
                        prop:value=image_url
                        on:input=move |ev| image_url.set(event_target_value(&ev))
                    />
                </div>

                <div style="display: flex; gap: 0.5rem;">
                    <button class="btn btn-success" on:click=submit_product>
                        {move || {
                            if editing_id.get().is_some() { "Atualizar" } else { "Criar Produto" }
                        }}
                    </button>
                    <button class="btn btn-ghost" on:click=cancel_edit>
                        Cancelar
                    </button>
                </div>
            </div>

            <h2 style="margin-bottom: 1rem;">Produtos Existentes</h2>
            <Transition fallback=move || view! { <crate::components::TableSkeleton rows=5/> }>
                {move || {
                    products.get().map(|result| match &*result {
                        Err(e) => {
                            view! {
                                <div class="alert alert-error">
                                    <span>Erro: {e.clone()}</span>
                                </div>
                            }.into_any()
                        }
                        Ok(data) => {
                            let items = data["data"]
                                .as_array()
                                .or_else(|| data.as_array())
                                .cloned()
                                .unwrap_or_default();
                            if items.is_empty() {
                                view! {
                                    <div class="empty-state">
                                        <h3>Nenhum produto cadastrado</h3>
                                        <p>Crie seu primeiro produto usando o formulário acima.</p>
                                    </div>
                                }.into_any()
                            } else {
                                view! {
                                    <div class="table-container">
                                        <table>
                                            <thead>
                                                <tr>
                                                    <th>Nome</th>
                                                    <th>Preço</th>
                                                    <th>Categoria</th>
                                                    <th>Ações</th>
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
                                                            <tr>
                                                                <td style="font-weight: 500;">{p_name}</td>
                                                                <td>{format!("R$ {:.2}", p_price)}</td>
                                                                <td>
                                                                    <span class="badge badge-info">{p_cat}</span>
                                                                </td>
                                                                <td>
                                                                    <div style="display: flex; gap: 0.25rem;">
                                                                        <button
                                                                            class="btn btn-warning btn-sm"
                                                                            on:click=move |_| edit_product(p_clone.clone())
                                                                        >
                                                                            Editar
                                                                        </button>
                                                                        <button
                                                                            class="btn btn-danger btn-sm"
                                                                            on:click=move |_| delete_product(pid.clone())
                                                                        >
                                                                            Excluir
                                                                        </button>
                                                                    </div>
                                                                </td>
                                                            </tr>
                                                        }
                                                    })
                                                    .collect::<Vec<_>>()
                                                }
                                            </tbody>
                                        </table>
                                    </div>
                                }.into_any()
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
        let toast = crate::toast::use_toast();
        leptos::task::spawn_local(async move {
            let result = api::api_put(
                &format!("/api/admin/orders/{}", id),
                &serde_json::json!({ "status": status }),
            ).await;
            match result {
                Ok(_) => {
                    toast.success("Status do pedido atualizado!");
                    orders.refetch();
                }
                Err(e) => {
                    toast.error(format!("Erro ao atualizar status: {}", e));
                }
            }
        });
    };

    fn status_badge_class(status: &str) -> &'static str {
        match status {
            "pending" => "badge badge-warning",
            "confirmed" => "badge badge-info",
            "shipped" => "badge badge-info",
            "delivered" => "badge badge-success",
            "cancelled" => "badge badge-danger",
            _ => "badge badge-gray",
        }
    }

    fn status_label(status: &str) -> &'static str {
        match status {
            "pending" => "Pendente",
            "confirmed" => "Confirmado",
            "shipped" => "Enviado",
            "delivered" => "Entregue",
            "cancelled" => "Cancelado",
            _ => "Desconhecido",
        }
    }

    view! {
        <div>
            <h2 style="margin-bottom: 1rem;">Gerenciar Pedidos</h2>
            <Transition fallback=move || view! { <crate::components::TableSkeleton rows=5/> }>
                {move || {
                    orders.get().map(|result| match &*result {
                        Err(e) => {
                            view! {
                                <div class="alert alert-error">
                                    <span>Erro: {e.clone()}</span>
                                </div>
                            }.into_any()
                        }
                        Ok(data) => {
                            let items = data["data"]
                                .as_array()
                                .or_else(|| data.as_array())
                                .cloned()
                                .unwrap_or_default();
                            if items.is_empty() {
                                view! {
                                    <div class="empty-state">
                                        <h3>Nenhum pedido encontrado</h3>
                                        <p>Os pedidos aparecerão aqui quando os clientes realizarem compras.</p>
                                    </div>
                                }.into_any()
                            } else {
                                view! {
                                    <div class="table-container">
                                        <table>
                                            <thead>
                                                <tr>
                                                    <th>ID</th>
                                                    <th>Status</th>
                                                    <th>Total</th>
                                                    <th>Ações</th>
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
                                                        let badge_class = status_badge_class(&current_status);
                                                        let label = status_label(&current_status);
                                                        view! {
                                                            <tr>
                                                                <td>
                                                                    <span class="table-mono">{oid_short}</span>
                                                                </td>
                                                                <td>
                                                                    <span class={badge_class}>{label}</span>
                                                                </td>
                                                                <td style="font-weight: 600;">
                                                                    {format!("R$ {:.2}", order_total)}
                                                                </td>
                                                                <td>
                                                                    <select
                                                                        class="form-select"
                                                                        style="width: auto; padding: 0.25rem 0.5rem; font-size: 0.85rem;"
                                                                        on:change=move |ev| {
                                                                            let val = event_target_value(&ev);
                                                                            update_status(&oid, &val);
                                                                        }
                                                                    >
                                                                        <option value="pending" selected=current_status=="pending">Pendente</option>
                                                                        <option value="confirmed" selected=current_status=="confirmed">Confirmado</option>
                                                                        <option value="shipped" selected=current_status=="shipped">Enviado</option>
                                                                        <option value="delivered" selected=current_status=="delivered">Entregue</option>
                                                                        <option value="cancelled" selected=current_status=="cancelled">Cancelado</option>
                                                                    </select>
                                                                </td>
                                                            </tr>
                                                        }
                                                    })
                                                    .collect::<Vec<_>>()
                                                }
                                            </tbody>
                                        </table>
                                    </div>
                                }.into_any()
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
        let toast = crate::toast::use_toast();

        let code_val = code.get();
        if code_val.trim().is_empty() {
            form_msg.set("Código do cupom é obrigatório.".to_string());
            toast.error("Código do cupom é obrigatório.");
            return;
        }

        let disc: f64 = match discount.get().parse() {
            Ok(v) if v > 0.0 => v,
            Ok(_) => {
                form_msg.set("Desconto deve ser maior que zero.".to_string());
                toast.error("Desconto deve ser maior que zero.");
                return;
            }
            Err(_) => {
                form_msg.set("Desconto inválido. Use números (ex: 10).".to_string());
                toast.error("Desconto inválido.");
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
                    let msg = if is_new { "Cupom criado com sucesso!" } else { "Cupom atualizado com sucesso!" };
                    form_msg.set(msg.to_string());
                    toast.success(msg);
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
                    toast.error(format!("Erro ao {} cupom: {}", if is_new { "criar" } else { "atualizar" }, e));
                }
            }
        });
    };

    let edit_coupon = move |coupon: Value| {
        code.set(coupon["code"].as_str().unwrap_or("").to_string());
        discount.set(coupon["discount"].as_f64().unwrap_or(0.0).to_string());
        kind.set(coupon["kind"].as_str().unwrap_or("percentage").to_string());
        max_uses.set(coupon["max_uses"].as_i64().unwrap_or(0).to_string());
        expires_at.set(coupon["expires_at"].as_str().unwrap_or("").to_string());
        editing_id.set(
            coupon["id"]
                .as_str()
                .map(|s| s.to_string())
                .or_else(|| coupon["id"].as_i64().map(|i| i.to_string())),
        );
    };

    let delete_coupon = move |coupon_id: String| {
        let window = web_sys::window().unwrap();
        if !window.confirm_with_message("Tem certeza que deseja excluir este cupom?").unwrap_or(false) {
            return;
        }
        let toast = crate::toast::use_toast();
        leptos::task::spawn_local({
            let id = coupon_id.clone();
            async move {
                let result = api::api_delete(&format!("/api/coupons/{}", id)).await;
                match result {
                    Ok(_) => {
                        toast.success("Cupom excluído com sucesso!");
                        coupons.refetch();
                    }
                    Err(e) => {
                        toast.error(format!("Erro ao excluir cupom: {}", e));
                    }
                }
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
            <div class="form-card" style="margin-bottom: 2rem;">
                <h2>
                    {move || {
                        if editing_id.get().is_some() { "Editar Cupom" } else { "Novo Cupom" }
                    }}
                </h2>

                {if !form_msg.get().is_empty() {
                    let is_err = form_msg.get().starts_with("Erro");
                    Some(view! {
                        <div class=if is_err { "alert alert-error" } else { "alert alert-success" }>
                            <span>{move || form_msg.get()}</span>
                        </div>
                    })
                } else {
                    None
                }}

                <div class="admin-form-layout">
                    <input
                        type="text"
                        placeholder="Código (ex: LAPES10)"
                        class="form-input"
                        prop:value=code
                        on:input=move |ev| code.set(event_target_value(&ev))
                    />
                    <input
                        type="text"
                        placeholder="Desconto (ex: 10)"
                        class="form-input"
                        prop:value=discount
                        on:input=move |ev| discount.set(event_target_value(&ev))
                    />
                    <select
                        class="form-select"
                        prop:value=kind
                        on:change=move |ev| kind.set(event_target_value(&ev))
                    >
                        <option value="percentage">Percentual</option>
                        <option value="fixed">Valor fixo</option>
                    </select>
                    <input
                        type="text"
                        placeholder="Usos máximos"
                        class="form-input"
                        prop:value=max_uses
                        on:input=move |ev| max_uses.set(event_target_value(&ev))
                    />
                    <input
                        type="text"
                        placeholder="Expira em (YYYY-MM-DD)"
                        class="form-input full-width"
                        prop:value=expires_at
                        on:input=move |ev| expires_at.set(event_target_value(&ev))
                    />
                </div>

                <div style="display: flex; gap: 0.5rem;">
                    <button class="btn btn-success" on:click=submit_coupon>
                        {move || {
                            if editing_id.get().is_some() { "Atualizar" } else { "Criar Cupom" }
                        }}
                    </button>
                    <button class="btn btn-ghost" on:click=cancel_edit>
                        Cancelar
                    </button>
                </div>
            </div>

            <h2 style="margin-bottom: 1rem;">Cupons Existentes</h2>
            <Transition fallback=move || view! { <crate::components::TableSkeleton rows=5/> }>
                {move || {
                    coupons.get().map(|result| match &*result {
                        Err(e) => {
                            view! {
                                <div class="alert alert-error">
                                    <span>Erro: {e.clone()}</span>
                                </div>
                            }.into_any()
                        }
                        Ok(data) => {
                            let items = data["data"]
                                .as_array()
                                .or_else(|| data.as_array())
                                .cloned()
                                .unwrap_or_default();
                            if items.is_empty() {
                                view! {
                                    <div class="empty-state">
                                        <h3>Nenhum cupom cadastrado</h3>
                                        <p>Crie seu primeiro cupom usando o formulário acima.</p>
                                    </div>
                                }.into_any()
                            } else {
                                view! {
                                    <div class="table-container">
                                        <table>
                                            <thead>
                                                <tr>
                                                    <th>Código</th>
                                                    <th>Desconto</th>
                                                    <th>Tipo</th>
                                                    <th>Ações</th>
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
                                                        let c_discount = c["discount"].as_f64().unwrap_or(0.0);
                                                        let c_kind = c["kind"].as_str().unwrap_or("").to_string();
                                                        let kind_label = if c_kind == "percentage" { "%" } else { "R$" };
                                                        let c_clone = c.clone();
                                                        view! {
                                                            <tr>
                                                                <td style="font-weight: 600; font-family: var(--font-mono);">{c_code}</td>
                                                                <td>{format!("{} {}", c_discount, kind_label)}</td>
                                                                <td>
                                                                    <span class=if c_kind == "percentage" { "badge badge-info" } else { "badge badge-success" }>
                                                                        {if c_kind == "percentage" { "Percentual" } else { "Fixo" }}
                                                                    </span>
                                                                </td>
                                                                <td>
                                                                    <div style="display: flex; gap: 0.25rem;">
                                                                        <button
                                                                            class="btn btn-warning btn-sm"
                                                                            on:click=move |_| edit_coupon(c_clone.clone())
                                                                        >
                                                                            Editar
                                                                        </button>
                                                                        <button
                                                                            class="btn btn-danger btn-sm"
                                                                            on:click=move |_| delete_coupon(cid.clone())
                                                                        >
                                                                            Excluir
                                                                        </button>
                                                                    </div>
                                                                </td>
                                                            </tr>
                                                        }
                                                    })
                                                    .collect::<Vec<_>>()
                                                }
                                            </tbody>
                                        </table>
                                    </div>
                                }.into_any()
                            }
                        }
                    })
                }}
            </Transition>
        </div>
    }
}
