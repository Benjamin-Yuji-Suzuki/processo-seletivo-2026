use crate::api;
use leptos::prelude::*;

#[component]
pub fn HomePage() -> impl IntoView {
    let products = LocalResource::new(|| async move {
        api::api_get("/api/products?page=1&limit=8").await
    });

    view! {
        <div>
            <section class="hero">
                <h1>Bem-vindo à LAPES</h1>
                <p>Descubra produtos selecionados com os melhores preços para você. Confira nossas novidades e ofertas especiais.</p>
                <div class="hero-actions">
                    <a href="/produtos" class="btn btn-primary btn-lg">
                        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                            <circle cx="11" cy="11" r="8"/><path d="m21 21-4.35-4.35"/>
                        </svg>
                        Ver Produtos
                    </a>
                    <a href="/carrinho" class="btn btn-ghost btn-lg" style="color: #fff; border-color: rgba(255,255,255,0.3);">
                        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                            <circle cx="8" cy="21" r="1"/><circle cx="21" cy="21" r="1"/>
                            <path d="M3 3h2l.4 2M7 13h10l4-8H5.4"/>
                        </svg>
                        Meu Carrinho
                    </a>
                </div>
            </section>

            <div class="container">
                <div class="page-header">
                    <h2>Produtos em Destaque</h2>
                    <a href="/produtos" class="btn btn-ghost">
                        Ver todos
                        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                            <path d="m9 18 6-6-6-6"/>
                        </svg>
                    </a>
                </div>

                <Transition
                    fallback=move || view! { <crate::components::ProductSkeletonGrid/> }
                >
                    {move || {
                        products
                            .get()
                            .map(|result| match &*result {
                                Err(e) => {
                                    view! {
                                        <div class="alert alert-error">
                                            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="15" y1="9" x2="9" y2="15"/><line x1="9" y1="9" x2="15" y2="15"/></svg>
                                            <span>Erro ao carregar produtos: {e.clone()}</span>
                                        </div>
                                    }
                                        .into_any()
                                }
                                Ok(data) => {
                                    let items = data["products"]
                                        .as_array()
                                        .or_else(|| data.as_array())
                                        .cloned()
                                        .unwrap_or_default();
                                    if items.is_empty() {
                                        view! {
                                            <div class="empty-state">
                                                <div class="empty-state-icon">
                                                    <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                                                        <path d="M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4"/>
                                                    </svg>
                                                </div>
                                                <h3>Nenhum produto encontrado</h3>
                                                <p>No momento não há produtos disponíveis.</p>
                                            </div>
                                        }
                                            .into_any()
                                    } else {
                                        view! {
                                            <div class="product-grid">
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
        </div>
    }
}
