pub mod auth;
pub mod cart;
pub mod catalog;
pub mod checkout;
pub mod coupons;
pub mod error;
pub mod models;
pub mod state;

use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "LAPES E-Commerce API",
        version = "0.1.0",
        description = "API do processo seletivo LAPES 2026 — E-commerce com autenticação JWT, catálogo, carrinho, checkout e cupons de desconto."
    ),
    paths(
        crate::auth::login,
        crate::auth::register,
        crate::auth::me,
        crate::catalog::list_products,
        crate::catalog::get_product,
        crate::catalog::create_product,
        crate::cart::list_cart,
        crate::cart::add_to_cart,
        crate::cart::update_cart_item,
        crate::cart::remove_cart_item,
        crate::cart::clear_cart,
        crate::checkout::checkout,
        crate::checkout::list_orders,
        crate::checkout::get_order,
        crate::checkout::cancel_order,
        crate::checkout::update_order_status,
        crate::coupons::list_coupons,
        crate::coupons::validate_coupon,
        crate::coupons::create_coupon,
    ),
    tags(
        (name = "auth", description = "Autenticação — registro, login e perfil do usuário"),
        (name = "catalog", description = "Catálogo — listar, buscar, criar, atualizar e deletar produtos"),
        (name = "cart", description = "Carrinho — adicionar, atualizar, remover e limpar itens"),
        (name = "checkout", description = "Pedidos — finalizar compra, listar e gerenciar pedidos"),
        (name = "coupons", description = "Cupons — gerenciar e validar cupons de desconto"),
    )
)]
pub struct ApiDoc;
