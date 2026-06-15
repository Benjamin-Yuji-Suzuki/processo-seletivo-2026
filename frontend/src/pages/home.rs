use crate::api;
use leptos::prelude::*;

#[component]
pub fn HomePage() -> impl IntoView {
    let products = LocalResource::new(|| async move {
        api::api_get("/api/products?page=1&limit=12").await
    });

    view! {
        <div style="max-width: 1200px; margin: 0 auto;">
            <h1>"Bem-vindo à LAPES"</h1>
            <p>"Confira nossos produtos em destaque."</p>

            <Transition
                fallback=move || view! { <crate::components::Loading/> }
            >
                {move || {
                    products
                        .get()
                        .map(|result| match &*result {
                            Err(e) => {
                                view! { <p style="color: red;">"Erro ao carregar produtos: " {e.clone()}</p> }
                                    .into_any()
                            }
                            Ok(data) => {
                                let items = data["data"]
                                    .as_array()
                                    .or_else(|| data.as_array())
                                    .cloned()
                                    .unwrap_or_default();
                                if items.is_empty() {
                                    view! { <p>"Nenhum produto encontrado."</p> }.into_any()
                                } else {
                                    view! {
                                        <div
                                            style="
                                                display: grid;
                                                grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
                                                gap: 1rem;
                                                margin-top: 1rem;
                                            "
                                        >
                                            {items
                                                .into_iter()
                                                .map(|product| {
                                                    view! {
                                                        <crate::components::ProductCard
                                                            product=product.clone()
                                                        />
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
                }}
            </Transition>
        </div>
    }
}
