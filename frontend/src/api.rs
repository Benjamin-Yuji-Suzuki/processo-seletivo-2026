use gloo_net::http::Request;
use serde_json::Value;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::RequestInit;

const API_BASE: &str = "";

fn storage() -> web_sys::Storage {
    web_sys::window().unwrap().local_storage().unwrap().unwrap()
}

pub fn get_auth_token() -> Option<String> {
    storage().get_item("auth_token").ok()?
}

pub fn set_auth_token(token: &str) {
    storage().set_item("auth_token", token).ok();
}

pub fn clear_auth_token() {
    storage().remove_item("auth_token").ok();
}

/// Attach Content-Type and optional Bearer auth headers (for gloo-net GET).
fn with_auth(builder: gloo_net::http::RequestBuilder) -> gloo_net::http::RequestBuilder {
    let builder = builder.header("Content-Type", "application/json");
    if let Some(token) = get_auth_token() {
        builder.header("Authorization", &format!("Bearer {}", token))
    } else {
        builder
    }
}

/// Build auth headers as a Vec of (key, value) pairs (for web-sys fetch).
fn auth_headers() -> Vec<(&'static str, String)> {
    let mut headers = vec![("Content-Type", "application/json".to_string())];
    if let Some(token) = get_auth_token() {
        headers.push(("Authorization", format!("Bearer {}", token)));
    }
    headers
}

pub async fn api_get(path: &str) -> Result<Value, String> {
    let url = format!("{}{}", API_BASE, path);
    let resp = with_auth(Request::get(&url))
        .send()
        .await
        .map_err(|e| format!("Request error: {:?}", e))?;
    let text = resp
        .text()
        .await
        .map_err(|e| format!("Read error: {:?}", e))?;
    serde_json::from_str(&text).map_err(|e| format!("JSON parse error: {}", e))
}

pub async fn api_post(path: &str, body: &Value) -> Result<Value, String> {
    let url = format!("{}{}", API_BASE, path);
    let body_str = serde_json::to_string(body).map_err(|e| format!("Serialize error: {}", e))?;

    let headers = web_sys::Headers::new().map_err(|e| format!("Headers create: {:?}", e))?;
    for (key, val) in auth_headers() {
        headers
            .set(key, &val)
            .map_err(|e| format!("Header set: {:?}", e))?;
    }

    let opts = RequestInit::new();
    opts.set_method("POST");
    opts.set_body(&JsValue::from_str(&body_str));
    opts.set_headers(&headers);

    let window = web_sys::window().ok_or("No window")?;
    let promise = window.fetch_with_str_and_init(&url, &opts);
    let resp_val = JsFuture::from(promise)
        .await
        .map_err(|e| format!("Fetch error: {:?}", e))?;
    let resp: web_sys::Response = resp_val
        .dyn_into()
        .map_err(|_| "Response cast error".to_string())?;

    let text_promise = resp.text().map_err(|e| format!("Text error: {:?}", e))?;
    let text_val = JsFuture::from(text_promise)
        .await
        .map_err(|e| format!("Text await error: {:?}", e))?;
    let text_str = text_val
        .as_string()
        .ok_or("Response not a string".to_string())?;

    serde_json::from_str(&text_str).map_err(|e| format!("JSON parse error: {}", e))
}

pub async fn api_put(path: &str, body: &Value) -> Result<Value, String> {
    let url = format!("{}{}", API_BASE, path);
    let body_str = serde_json::to_string(body).map_err(|e| format!("Serialize error: {}", e))?;

    let headers = web_sys::Headers::new().map_err(|e| format!("Headers create: {:?}", e))?;
    for (key, val) in auth_headers() {
        headers
            .set(key, &val)
            .map_err(|e| format!("Header set: {:?}", e))?;
    }

    let opts = RequestInit::new();
    opts.set_method("PUT");
    opts.set_body(&JsValue::from_str(&body_str));
    opts.set_headers(&headers);

    let window = web_sys::window().ok_or("No window")?;
    let promise = window.fetch_with_str_and_init(&url, &opts);
    let resp_val = JsFuture::from(promise)
        .await
        .map_err(|e| format!("Fetch error: {:?}", e))?;
    let resp: web_sys::Response = resp_val
        .dyn_into()
        .map_err(|_| "Response cast error".to_string())?;

    let text_promise = resp.text().map_err(|e| format!("Text error: {:?}", e))?;
    let text_val = JsFuture::from(text_promise)
        .await
        .map_err(|e| format!("Text await error: {:?}", e))?;
    let text_str = text_val
        .as_string()
        .ok_or("Response not a string".to_string())?;

    serde_json::from_str(&text_str).map_err(|e| format!("JSON parse error: {}", e))
}

pub async fn api_delete(path: &str) -> Result<(), String> {
    let url = format!("{}{}", API_BASE, path);

    let headers = web_sys::Headers::new().map_err(|e| format!("Headers create: {:?}", e))?;
    for (key, val) in auth_headers() {
        headers
            .set(key, &val)
            .map_err(|e| format!("Header set: {:?}", e))?;
    }

    let opts = RequestInit::new();
    opts.set_method("DELETE");
    opts.set_headers(&headers);

    let window = web_sys::window().ok_or("No window")?;
    let promise = window.fetch_with_str_and_init(&url, &opts);
    let resp_val = JsFuture::from(promise)
        .await
        .map_err(|e| format!("Fetch error: {:?}", e))?;
    let _resp: web_sys::Response = resp_val
        .dyn_into()
        .map_err(|_| "Response cast error".to_string())?;

    Ok(())
}
