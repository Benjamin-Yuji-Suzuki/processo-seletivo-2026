use leptos::prelude::*;
use wasm_bindgen::JsCast;

/// Types of toast messages
#[derive(Clone, Debug)]
pub enum ToastType {
    Success,
    Error,
    Info,
}

impl ToastType {
    fn css_class(&self) -> &'static str {
        match self {
            ToastType::Success => "toast toast-success",
            ToastType::Error => "toast toast-error",
            ToastType::Info => "toast toast-info",
        }
    }

    fn icon(&self) -> &'static str {
        match self {
            ToastType::Success => "\u{2713}",
            ToastType::Error => "\u{2717}",
            ToastType::Info => "\u{2139}",
        }
    }
}

#[derive(Clone, Debug)]
struct Toast {
    id: u64,
    message: String,
    toast_type: ToastType,
}

static NEXT_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

/// Context that provides toast functionality
#[derive(Clone)]
pub struct ToastContext {
    toasts: RwSignal<Vec<Toast>>,
}

impl ToastContext {
    pub fn new() -> Self {
        Self {
            toasts: RwSignal::new(Vec::new()),
        }
    }

    pub fn show(&self, message: impl Into<String>, toast_type: ToastType) {
        let id = NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let toast = Toast {
            id,
            message: message.into(),
            toast_type,
        };
        self.toasts.update(|t| t.push(toast));

        let toasts = self.toasts;
        leptos::task::spawn_local(async move {
            let window = web_sys::window().unwrap();
            let closure = wasm_bindgen::closure::Closure::once(move || {
                toasts.update(|t| t.retain(|toast| toast.id != id));
            });
            let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                closure.as_ref().unchecked_ref(),
                3500,
            );
            closure.forget();
        });
    }

    pub fn success(&self, message: impl Into<String>) {
        self.show(message, ToastType::Success);
    }

    pub fn error(&self, message: impl Into<String>) {
        self.show(message, ToastType::Error);
    }

    pub fn info(&self, message: impl Into<String>) {
        self.show(message, ToastType::Info);
    }

    pub fn remove(&self, id: u64) {
        self.toasts.update(|t| t.retain(|toast| toast.id != id));
    }
}

/// Provide toast context at the app level
pub fn provide_toast() {
    let context = ToastContext::new();
    provide_context(context);
}

/// Get the toast context in any component
pub fn use_toast() -> ToastContext {
    expect_context::<ToastContext>()
}

/// The visual container that renders active toasts
#[component]
pub fn ToastContainer() -> impl IntoView {
    let toast_ctx = use_toast();

    view! {
        <div class="toast-container">
            {move || {
                toast_ctx
                    .toasts
                    .get()
                    .into_iter()
                    .map(|toast| {
                        let id = toast.id;
                        let toast_ctx_clone = toast_ctx.clone();
                        view! {
                            <div class={toast.toast_type.css_class()}>
                                <span style="font-weight: 700; flex-shrink: 0;">{toast.toast_type.icon()}</span>
                                <span style="flex: 1;">{toast.message.clone()}</span>
                                <button
                                    style="
                                        background: none; border: none; color: inherit;
                                        cursor: pointer; font-size: 1.1rem; padding: 0 0 0 0.5rem;
                                        opacity: 0.7; flex-shrink: 0;
                                    "
                                    on:click=move |_| toast_ctx_clone.remove(id)
                                >
                                    "x"
                                </button>
                            </div>
                        }
                    })
                    .collect::<Vec<_>>()
            }}
        </div>
    }
}
