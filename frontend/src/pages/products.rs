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
                view! { <p style="color: red;">"Erro: " {e.clone()}</p> }.into_any()
            }
            Ok(data) => {
                let items = data["data"]
                    .as_array()
                    .or_else(|| data.as_array())
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
                    view! { <p>"Nenhum produto encontrado."</p> }.into_any()
                } else {
                    view! {
                        <div
                            style="
                                display: grid;
                                grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
                                gap: 1rem;
                            "
                        >
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
                    }
                        .into_any()
                }
            }
        })
    };

    view! {
        <div style="max-width: 1200px; margin: 0 auto;">
            <h1>"Produtos"</h1>
            <div style="display: flex; gap: 0.5rem; margin-bottom: 1rem; flex-wrap: wrap;">
                <input
                    type="text"
                    placeholder="Buscar produtos..."
                    style="flex: 1; min-width: 200px; padding: 0.5rem;"
                    on:input=move |ev| {
                        search.set(event_target_value(&ev));
                    }
                    prop:value=search
                />
                <select
                    style="padding: 0.5rem;"
                    on:change=move |ev| {
                        category.set(event_target_value(&ev));
                        products.refetch();
                    }
                >
                    <option value="">"Todas as categorias"</option>
                    <option value="eletrônicos">"Eletrônicos"</option>
                    <option value="roupas">"Roupas"</option>
                    <option value="alimentos">"Alimentos"</option>
                    <option value="livros">"Livros"</option>
                    <option value="outros">"Outros"</option>
                </select>
            </div>

            <Transition fallback=move || view! { <crate::components::Loading/> }>
                {search_products}
            </Transition>

            <div style="display: flex; gap: 0.5rem; justify-content: center; margin-top: 1rem;">
                <button
                    style="padding: 0.5rem 1rem;"
                    on:click=move |_| {
                        let p = page.get();
                        if p > 1 {
                            page.set(p - 1);
                            products.refetch();
                        }
                    }
                >
                    "Anterior"
                </button>
                <span style="padding: 0.5rem;">
                    {move || format!("Página {}", page.get())}
                </span>
                <button
                    style="padding: 0.5rem 1rem;"
                    on:click=move |_| {
                        page.set(page.get() + 1);
                        products.refetch();
                    }
                >
                    "Próxima"
                </button>
            </div>
        </div>
    }
}
