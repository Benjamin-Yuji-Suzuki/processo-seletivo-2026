use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{
    StaticSegment,
    components::{Route, Router, Routes},
};

pub mod api;
pub mod components;
pub mod pages;
pub mod toast;

#[component]
pub fn App() -> impl IntoView {
    // Set up panic hook to see errors in browser console
    std::panic::set_hook(Box::new(|info| {
        web_sys::console::error_1(&format!("PANIC: {}", info).into());
    }));

    provide_meta_context();
    toast::provide_toast();

    view! {
        <Title text="LAPES E-Commerce"/>
        <Stylesheet href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700;800&display=swap"/>
        <Router>
            <components::NavBar/>
            <main>
                <Routes fallback=|| view! {
                    <div class="container" style="text-align:center;padding:4rem 1rem;">
                        <div style="font-size:4rem;margin-bottom:1rem;opacity:0.3;">404</div>
                        <h1>Página não encontrada</h1>
                        <p style="color:var(--gray-500);margin-bottom:1.5rem;">A página que você procura não existe.</p>
                        <a href="/" class="btn btn-primary">Voltar ao início</a>
                    </div>
                }>
                    <Route path=StaticSegment("") view=pages::home::HomePage/>
                    <Route path=StaticSegment("produtos") view=pages::products::ProductList/>
                    <Route path=StaticSegment("carrinho") view=pages::cart::CartPage/>
                    <Route path=StaticSegment("login") view=pages::auth::LoginPage/>
                    <Route path=StaticSegment("admin") view=pages::admin::AdminPanel/>
                </Routes>
            </main>
            <toast::ToastContainer/>
            <footer style="text-align:center;padding:2rem 1rem;color:var(--gray-400);font-size:0.85rem;border-top:1px solid var(--gray-200);margin-top:3rem;">
                <p>&copy; 2026 LAPES E-Commerce. Todos os direitos reservados.</p>
            </footer>
        </Router>
    }
}
