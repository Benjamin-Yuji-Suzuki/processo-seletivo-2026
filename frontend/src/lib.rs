use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};

pub mod api;
pub mod components;
pub mod pages;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Title text="LAPES E-Commerce"/>
        <Stylesheet href="https://cdn.jsdelivr.net/npm/@exampledev/new.css@1/new.min.css"/>
        <Router>
            <components::NavBar/>
            <main style="padding: 1rem;">
                <Routes fallback=|| view! { <p>"Página não encontrada"</p> }>
                    <Route path=StaticSegment("") view=pages::home::HomePage/>
                    <Route path=StaticSegment("produtos") view=pages::products::ProductList/>
                    <Route path=StaticSegment("carrinho") view=pages::cart::CartPage/>
                    <Route path=StaticSegment("login") view=pages::auth::LoginPage/>
                    <Route path=StaticSegment("admin") view=pages::admin::AdminPanel/>
                </Routes>
            </main>
        </Router>
    }
}
