use crate::api;
use leptos::prelude::*;
use serde_json::Value;

#[component]
pub fn ProductList() -> impl IntoView {
    let search = RwSignal::new(String::new());
    let category = RwSignal::new(String::new());
    let page = RwSignal::new(1u32);
    let products = LocalResource::new(move || {
        let c = category.get();
        let p = page.get();
        async move {
            let mut path = format!("/api/products?page={}&limit=12", p);
            if !c.is_empty() {
                path.push_str(&format!("&category={}", c));
            }
            api::api_get(&path).await
        }
    });

    let search_products = move || {
        products.get().map(|result| match &*result {
            Err(e) => {
                view! {
                    <div class="alert alert-error">
                        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="15" y1="9" x2="9" y2="15"/><line x1="9" y1="9" x2="15" y2="15"/></svg>
                        <span>Erro: {e.clone()}</span>
                    </div>
                }.into_any()
            }
            Ok(data) => {
                let items = data["products"]
                    .as_array()
                    .cloned()
                    .unwrap_or_default();
                let q = search.get().to_lowercase();
                let filtered: Vec<Value> = if q.is_empty() {
                    items
                } else {
                    items
                        .into_iter()
                        .filter(|p| {
                            p["name"]
                                .as_str()
                                .unwrap_or("")
                                .to_lowercase()
                                .contains(&q)
                                || p["description"]
                                    .as_str()
                                    .unwrap_or("")
                                    .to_lowercase()
                                    .contains(&q)
                        })
                        .collect()
                };

                if filtered.is_empty() {
                    view! {
                        <div class="empty-state">
                            <div class="empty-state-icon">
                                <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                                    <circle cx="11" cy="11" r="8"/><path d="m21 21-4.35-4.35"/>
                                </svg>
                            </div>
                            <h3>Nenhum produto encontrado</h3>
                            <p>Tente ajustar sua busca ou limpar os filtros.</p>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <div class="product-grid">
                            {filtered
                                .into_iter()
                                .map(|p| {
                                    view! {
                                        <crate::components::ProductCard product=p.clone()/>
                                    }
                                })
                                .collect::<Vec<_>>()
                            }
                        </div>
                    }.into_any()
                }
            }
        })
    };

    view! {
        <div class="container" style="margin-top: 1.5rem;">
            <div class="page-header">
                <h1>Produtos</h1>
                <span style="color: var(--gray-500); font-size: 0.9rem;">
                    {move || {
                        products.get().map(|result| match &*result {
                            Ok(data) => {
                                let count = data["products"]
                                    .as_array()
                                    .map(|a| a.len())
                                    .or_else(|| data.as_array().map(|a| a.len()))
                                    .unwrap_or(0);
                                format!("{} produto(s)", count)
                            }
                            _ => String::new(),
                        }).unwrap_or_default()
                    }}
                </span>
            </div>

            <div class="filter-bar">
                <div style="display: flex; gap: 8px; align-items: stretch; flex: 1;">
                    <div style="position: relative; flex: 1;">
                        <input
                            type="text"
                            placeholder="Buscar produtos..."
                            class="form-input"
                            style="padding-right: 2.5rem;"
                            on:input=move |ev| {
                                search.set(event_target_value(&ev));
                            }
                            prop:value=search
                        />
                        <button
                            style="
                                position: absolute; right: 6px; top: 50%; transform: translateY(-50%);
                                background: none; border: none; cursor: pointer;
                                color: var(--gray-500); padding: 4px; line-height: 0;
                                display: flex; align-items: center;
                            "
                            on:click=move |_| search.set(String::new())
                        >
                            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                <line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
                            </svg>
                        </button>
                    </div>
                    <select
                        class="form-select"
                        style="max-width: 200px;"
                        on:change=move |ev| {
                            category.set(event_target_value(&ev));
                            page.set(1);
                            products.refetch();
                        }
                    >
                        <option value="">Todas as categorias</option>
                        <option value="eletrônicos">Eletrônicos</option>
                        <option value="roupas">Roupas</option>
                        <option value="alimentos">Alimentos</option>
                        <option value="livros">Livros</option>
                        <option value="outros">Outros</option>
                    </select>
                </div>
            </div>

            <Transition fallback=move || view! { <crate::components::ProductSkeletonGrid/> }>
                {search_products}
            </Transition>

            <div class="pagination">
                <button
                    class="btn btn-ghost btn-sm"
                    disabled=move || page.get() <= 1
                    on:click=move |_| {
                        if page.get() > 1 {
                            page.set(page.get() - 1);
                            products.refetch();
                        }
                    }
                >
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <path d="m15 18-6-6 6-6"/>
                    </svg>
                    Anterior
                </button>
                <span class="pagination-info">
                    {move || format!("Página {}", page.get())}
                </span>
                <button
                    class="btn btn-ghost btn-sm"
                    on:click=move |_| {
                        page.set(page.get() + 1);
                        products.refetch();
                    }
                >
                    Próxima
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <path d="m9 18 6-6-6-6"/>
                    </svg>
                </button>
            </div>
        </div>
    }
}
