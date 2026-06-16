use gloo_net::http::{Request, RequestBuilder};
use serde_json::Value;

const API_BASE: &str = "/api";

fn storage() -> web_sys::Storage {
    web_sys::window()
        .unwrap()
        .local_storage()
        .unwrap()
        .unwrap()
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

/// Attach Content-Type and optional Bearer auth headers.
fn with_auth(builder: RequestBuilder) -> RequestBuilder {
    let builder = builder.header("Content-Type", "application/json");
    if let Some(token) = get_auth_token() {
        builder.header("Authorization", &format!("Bearer {}", token))
    } else {
        builder
    }
}

pub async fn api_get(path: &str) -> Result<Value, String> {
    let url = format!("{}{}", API_BASE, path);
    let resp = with_auth(Request::get(&url))
        .send()
        .await
        .map_err(|e| format!("Request error: {:?}", e))?;
    let text = resp.text().await.map_err(|e| format!("Read error: {:?}", e))?;
    serde_json::from_str(&text).map_err(|e| format!("JSON parse error: {}", e))
}

pub async fn api_post(path: &str, body: &Value) -> Result<Value, String> {
    let url = format!("{}{}", API_BASE, path);
    let body_str =
        serde_json::to_string(body).map_err(|e| format!("Serialize error: {}", e))?;
    let req = with_auth(Request::post(&url))
        .body(body_str)
        .map_err(|e| format!("Build error: {:?}", e))?;
    let resp = req.send().await.map_err(|e| format!("Request error: {:?}", e))?;
    let text = resp.text().await.map_err(|e| format!("Read error: {:?}", e))?;
    serde_json::from_str(&text).map_err(|e| format!("JSON parse error: {}", e))
}

pub async fn api_put(path: &str, body: &Value) -> Result<Value, String> {
    let url = format!("{}{}", API_BASE, path);
    let body_str =
        serde_json::to_string(body).map_err(|e| format!("Serialize error: {}", e))?;
    let req = with_auth(Request::put(&url))
        .body(body_str)
        .map_err(|e| format!("Build error: {:?}", e))?;
    let resp = req.send().await.map_err(|e| format!("Request error: {:?}", e))?;
    let text = resp.text().await.map_err(|e| format!("Read error: {:?}", e))?;
    serde_json::from_str(&text).map_err(|e| format!("JSON parse error: {}", e))
}

pub async fn api_delete(path: &str) -> Result<(), String> {
    let url = format!("{}{}", API_BASE, path);
    let resp = with_auth(Request::delete(&url))
        .send()
        .await
        .map_err(|e| format!("Request error: {:?}", e))?;
    let _text = resp.text().await.map_err(|e| format!("Read error: {:?}", e))?;
    Ok(())
}
