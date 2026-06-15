use crate::api;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use serde_json::json;

#[component]
pub fn LoginPage() -> impl IntoView {
    let is_register = RwSignal::new(false);
    let name = RwSignal::new(String::new());
    let email = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let message = RwSignal::new(String::new());
    let is_error = RwSignal::new(false);

    let submit = move |_| {
        let reg = is_register.get();
        message.set(String::new());

        leptos::task::spawn_local(async move {
            let body = if reg {
                json!({
                    "name": name.get(),
                    "email": email.get(),
                    "password": password.get(),
                })
            } else {
                json!({
                    "email": email.get(),
                    "password": password.get(),
                })
            };

            let result = if reg {
                api::api_post("/api/auth/register", &body).await
            } else {
                api::api_post("/api/auth/login", &body).await
            };

            match result {
                Ok(data) => {
                    let token = data["token"]
                        .as_str()
                        .or_else(|| data["data"]["token"].as_str())
                        .map(|s| s.to_string());
                    if let Some(t) = token {
                        api::set_auth_token(&t);
                        is_error.set(false);
                        message.set(if reg {
                            "Conta criada com sucesso!".to_string()
                        } else {
                            "Login realizado com sucesso!".to_string()
                        });
                        // Navigate to home
                        let nav = use_navigate();
                        nav("/", Default::default());
                    } else {
                        is_error.set(true);
                        message.set("Token não recebido.".to_string());
                    }
                }
                Err(e) => {
                    is_error.set(true);
                    message.set(format!("Erro: {}", e));
                }
            }
        });
    };

    let toggle_mode = move |_| {
        is_register.set(!is_register.get());
        message.set(String::new());
    };

    let logged_in = api::get_auth_token().is_some();

    view! {
        <div style="max-width: 400px; margin: 2rem auto;">
            {if logged_in {
                view! {
                    <div style="text-align: center;">
                        <h1>"Você já está logado!"</h1>
                        <p>
                            <a href="/">"Ir para o ínicio"</a>
                        </p>
                    </div>
                }
                    .into_any()
            } else {
                view! {
                    <div>
                        <h1>{move || if is_register.get() { "Criar Conta" } else { "Login" }}</h1>

                        <Show when=move || is_register.get()>
                            <div style="margin-bottom: 1rem;">
                                <label style="display: block; margin-bottom: 0.25rem;">
                                    "Nome"
                                </label>
                                <input
                                    type="text"
                                    style="width: 100%; padding: 0.5rem;"
                                    prop:value=name
                                    on:input=move |ev| name.set(event_target_value(&ev))
                                />
                            </div>
                        </Show>

                        <div style="margin-bottom: 1rem;">
                            <label style="display: block; margin-bottom: 0.25rem;">
                                "E-mail"
                            </label>
                            <input
                                type="email"
                                style="width: 100%; padding: 0.5rem;"
                                prop:value=email
                                on:input=move |ev| email.set(event_target_value(&ev))
                            />
                        </div>

                        <div style="margin-bottom: 1rem;">
                            <label style="display: block; margin-bottom: 0.25rem;">
                                "Senha"
                            </label>
                            <input
                                type="password"
                                style="width: 100%; padding: 0.5rem;"
                                prop:value=password
                                on:input=move |ev| password.set(event_target_value(&ev))
                            />
                        </div>

                        <button
                            style="
                                width: 100%; padding: 0.75rem;
                                background: #007bff; color: #fff;
                                border: none; border-radius: 4px;
                                cursor: pointer; font-size: 1rem;
                            "
                            on:click=submit
                        >
                            {move || if is_register.get() { "Cadastrar" } else { "Entrar" }}
                        </button>

                        <p style="margin-top: 1rem; text-align: center;">
                            {move || if is_register.get() {
                                "Já tem conta? "
                            } else {
                                "Não tem conta? "
                            }}
                            <a
                                href="#"
                                on:click=toggle_mode
                                style="color: #3498db; cursor: pointer;"
                            >
                                {move || if is_register.get() { "Faça login" } else { "Cadastre-se" }}
                            </a>
                        </p>

                        {if message.get().is_empty() {
                            None
                        } else {
                            Some(
                                view! {
                                    <p
                                        style=format!(
                                            "margin-top: 1rem; padding: 0.5rem; border-radius: 4px; {}",
                                            if is_error.get() {
                                                "color: #721c24; background: #f8d7da;"
                                            } else {
                                                "color: #155724; background: #d4edda;"
                                            },
                                        )
                                    >
                                        {move || message.get()}
                                    </p>
                                },
                            )
                        }}
                    </div>
                }
                    .into_any()
            }}
        </div>
    }
}
