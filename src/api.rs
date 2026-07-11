use gloo_net::http::Request;
use serde::{Deserialize, Serialize};

pub const API_BASE: &str = match option_env!("VL_API_BASE_URL") {
    Some(value) => value,
    None => "https://api.vlrental.ca",
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AuthUser {
    pub user_id: String,
    pub email: String,
    pub role: String,
}
#[derive(Clone, Debug, Deserialize)]
pub struct AuthTokens {
    pub access_token: String,
    pub refresh_token: String,
    pub user: AuthUser,
}
#[derive(Serialize)]
struct Credentials<'a> {
    email: &'a str,
    password: &'a str,
}

pub async fn login(email: &str, password: &str, register: bool) -> Result<AuthTokens, String> {
    let path = if register { "register" } else { "login" };
    let response = Request::post(&format!("{API_BASE}/api/v1/auth/{path}"))
        .header("Content-Type", "application/json")
        .json(&Credentials { email, password })
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !response.ok() {
        return Err(response
            .text()
            .await
            .unwrap_or_else(|_| "Sign in failed".into()));
    }
    response.json().await.map_err(|e| e.to_string())
}

fn storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok().flatten()
}
pub fn save_session(tokens: &AuthTokens) -> Result<(), String> {
    let storage = storage().ok_or("Browser storage is unavailable")?;
    storage
        .set_item("vl_access_token", &tokens.access_token)
        .map_err(|_| "Could not save session")?;
    storage
        .set_item("vl_refresh_token", &tokens.refresh_token)
        .map_err(|_| "Could not save session")?;
    storage
        .set_item(
            "vl_auth_user",
            &serde_json::to_string(&tokens.user).map_err(|e| e.to_string())?,
        )
        .map_err(|_| "Could not save session".into())
}
pub fn current_user() -> Option<AuthUser> {
    serde_json::from_str(&storage()?.get_item("vl_auth_user").ok()??).ok()
}
pub fn clear_session() {
    if let Some(s) = storage() {
        let _ = s.remove_item("vl_access_token");
        let _ = s.remove_item("vl_refresh_token");
        let _ = s.remove_item("vl_auth_user");
    }
}
