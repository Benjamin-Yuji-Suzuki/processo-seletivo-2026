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
    let loading = RwSignal::new(false);

    let submit = move |_| {
        let reg = is_register.get();
        message.set(String::new());
        loading.set(true);
        let toast = crate::toast::use_toast();

        // Client-side validation
        let email_val = email.get();
        let pass_val = password.get();
        let name_val = name.get();

        if reg && name_val.trim().is_empty() {
            is_error.set(true);
            loading.set(false);
            message.set("Nome é obrigatório.".to_string());
            toast.error("Nome é obrigatório.");
            return;
        }
        if !email_val.contains('@') || !email_val.contains('.') {
            is_error.set(true);
            loading.set(false);
            message.set("E-mail inválido.".to_string());
            toast.error("E-mail inválido.");
            return;
        }
        if pass_val.len() < 6 {
            is_error.set(true);
            loading.set(false);
            message.set("A senha deve ter pelo menos 6 caracteres.".to_string());
            toast.error("A senha deve ter pelo menos 6 caracteres.");
            return;
        }

        leptos::task::spawn_local(async move {
            let body = if reg {
                json!({
                    "name": name_val,
                    "email": email_val.clone(),
                    "password": pass_val.clone(),
                })
            } else {
                json!({
                    "email": email_val.clone(),
                    "password": pass_val.clone(),
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
                        toast.success(if reg { "Conta criada com sucesso!" } else { "Login realizado com sucesso!" });
                        is_error.set(false);
                        loading.set(false);
                        let nav = use_navigate();
                        nav("/", Default::default());
                    } else {
                        is_error.set(true);
                        loading.set(false);
                        message.set("Token não recebido.".to_string());
                        toast.error("Token não recebido. Tente novamente.");
                    }
                }
                Err(e) => {
                    is_error.set(true);
                    loading.set(false);
                    message.set(format!("Erro: {}", e));
                    toast.error(format!("Erro: {}", e));
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
        <div class="auth-container">
            {if logged_in {
                view! {
                    <div class="auth-card" style="text-align: center;">
                        <div style="font-size: 3rem; margin-bottom: 1rem;">
                            <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="var(--secondary)" stroke-width="1.5">
                                <path d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"/>
                            </svg>
                        </div>
                        <h1>Você já está logado!</h1>
                        <p style="color: var(--gray-500); margin-bottom: 1.5rem;">
                            Continue navegando pela loja.
                        </p>
                        <a href="/" class="btn btn-primary">Ir para o início</a>
                    </div>
                }.into_any()
            } else {
                view! {
                    <div class="auth-card">
                        <div class="auth-title">
                            <h1>{move || if is_register.get() { "Criar Conta" } else { "Entrar" }}</h1>
                            <p>{move || if is_register.get() { "Crie sua conta para começar a comprar" } else { "Faça login para acessar sua conta" }}</p>
                        </div>

                        {if !message.get().is_empty() {
                            Some(view! {
                                <div class=if is_error.get() { "alert alert-error" } else { "alert alert-success" }>
                                    <span>{move || message.get()}</span>
                                </div>
                            })
                        } else {
                            None
                        }}

                        <Show when=move || is_register.get()>
                            <div class="form-group">
                                <label class="form-label" for="reg-name">Nome</label>
                                <input
                                    id="reg-name"
                                    type="text"
                                    class="form-input"
                                    placeholder="Seu nome completo"
                                    prop:value=name
                                    on:input=move |ev| name.set(event_target_value(&ev))
                                />
                            </div>
                        </Show>

                        <div class="form-group">
                            <label class="form-label" for="login-email">E-mail</label>
                            <input
                                id="login-email"
                                type="email"
                                class="form-input"
                                placeholder="seu@email.com"
                                prop:value=email
                                on:input=move |ev| email.set(event_target_value(&ev))
                            />
                        </div>

                        <div class="form-group">
                            <label class="form-label" for="login-password">Senha</label>
                            <input
                                id="login-password"
                                type="password"
                                class="form-input"
                                placeholder="Sua senha"
                                prop:value=password
                                on:input=move |ev| password.set(event_target_value(&ev))
                            />
                        </div>

                        <button
                            class="btn btn-primary btn-lg"
                            style="width: 100%;"
                            disabled=move || loading.get()
                            on:click=submit
                        >
                            {move || {
                                if loading.get() {
                                    if is_register.get() { "Cadastrando..." } else { "Entrando..." }
                                } else {
                                    if is_register.get() { "Cadastrar" } else { "Entrar" }
                                }
                            }}
                        </button>

                        <div class="auth-toggle">
                            {move || if is_register.get() {
                                "Já tem conta? "
                            } else {
                                "Não tem conta? "
                            }}
                            <a on:click=toggle_mode>
                                {move || if is_register.get() { "Faça login" } else { "Cadastre-se" }}
                            </a>
                        </div>
                    </div>
                }.into_any()
            }}
        </div>
    }
}
