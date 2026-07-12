use gloo_net::http::Request;
use serde::{Deserialize, Serialize};

pub const API_BASE: &str = match option_env!("VL_API_BASE_URL") {
    Some(value) => value,
    None => "https://api.vlrental.ca",
};

pub fn google_login_url() -> String {
    let frontend_origin = web_sys::window()
        .and_then(|window| window.location().origin().ok())
        .unwrap_or_else(|| "https://vlrental.ca".to_string());
    format!(
        "{API_BASE}/api/v1/auth/google?return_to={}",
        urlencoding::encode(&frontend_origin)
    )
}

pub fn remember_auth_return(path: &str) {
    if let Some(storage) = web_sys::window()
        .and_then(|window| window.session_storage().ok())
        .flatten()
    {
        let _ = storage.set_item("vl_auth_return", path);
    }
}

pub fn take_auth_return() -> Option<String> {
    let storage = web_sys::window()?.session_storage().ok().flatten()?;
    let path = storage.get_item("vl_auth_return").ok().flatten();
    let _ = storage.remove_item("vl_auth_return");
    path
}

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

pub fn access_token() -> Option<String> {
    storage()?.get_item("vl_access_token").ok().flatten()
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Rental {
    pub slug: String,
    pub name: String,
    pub category: String,
    pub summary: String,
    pub description: String,
    pub capacity: i32,
    pub price_unit: String,
    pub base_rate: String,
    pub currency: String,
    pub min_units: i32,
    pub refundable_deposit: String,
    pub hero_image_url: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CatalogResponse {
    pub rentals: Vec<Rental>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct RentalMedia {
    pub source_url: String,
    pub alt_text: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct RentalFeature {
    pub group_name: String,
    pub label: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct RentalAddon {
    pub addon_key: String,
    pub label: String,
    pub price: String,
    pub charge_type: String,
    pub is_recommended: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct RentalResponse {
    pub rental: Rental,
    pub media: Vec<RentalMedia>,
    pub features: Vec<RentalFeature>,
    pub addons: Vec<RentalAddon>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TripDraft {
    pub rental_slug: String,
    pub starts_on: String,
    pub ends_on: String,
    pub guests: i32,
    pub addon_keys: Vec<String>,
    pub delivery_km: Option<String>,
}

#[derive(Serialize)]
struct CreateQuotePayload<'a> {
    rental_slug: &'a str,
    starts_on: &'a str,
    ends_on: &'a str,
    guests: i32,
    addon_keys: &'a [String],
    delivery_km: Option<&'a str>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct UnavailableInterval {
    pub starts_at: String,
    pub ends_at: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct AvailabilityResponse {
    pub rental_slug: String,
    pub starts_on: String,
    pub ends_on: String,
    pub unavailable: Vec<UnavailableInterval>,
    pub pickup_time: String,
    pub return_time: String,
    pub timezone: String,
    pub minimum_nights: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Quote {
    pub quote_id: String,
    pub rental_slug: String,
    pub starts_at: String,
    pub ends_at: String,
    pub guests: i32,
    pub units: i32,
    pub currency: String,
    pub subtotal: String,
    pub tax_total: String,
    pub refundable_deposit: String,
    pub total: String,
    pub expires_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QuoteItem {
    pub item_type: String,
    pub item_key: String,
    pub label: String,
    pub quantity: String,
    pub unit_price: String,
    pub amount: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QuoteResponse {
    pub quote: Quote,
    pub items: Vec<QuoteItem>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Booking {
    pub booking_id: String,
    pub booking_number: String,
    pub status: String,
    pub payment_status: String,
    pub starts_at: String,
    pub ends_at: String,
    pub currency: String,
    pub total: String,
    pub amount_due_now: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CreatedBooking {
    pub booking: Booking,
    pub access_token: String,
    #[serde(default)]
    pub notification_email_sent: bool,
}

#[derive(Deserialize)]
pub struct BookingsResponse {
    pub bookings: Vec<Booking>,
}

#[derive(Serialize)]
struct CreateBookingPayload<'a> {
    quote_id: &'a str,
    first_name: &'a str,
    last_name: &'a str,
    email: &'a str,
    phone: &'a str,
    notes: Option<&'a str>,
}

async fn response_error(response: gloo_net::http::Response) -> String {
    #[derive(Deserialize)]
    struct ErrorDetails { message: String }
    #[derive(Deserialize)]
    struct ErrorResponse { error: ErrorDetails }

    let text = response.text().await.unwrap_or_else(|_| "Request failed".into());
    serde_json::from_str::<ErrorResponse>(&text)
        .map(|value| value.error.message)
        .unwrap_or(text)
}

pub async fn catalog() -> Result<Vec<Rental>, String> {
    let response = Request::get(&format!("{API_BASE}/api/v1/catalog"))
        .send().await.map_err(|e| e.to_string())?;
    if !response.ok() { return Err(response_error(response).await); }
    response.json::<CatalogResponse>().await.map(|value| value.rentals).map_err(|e| e.to_string())
}

pub async fn rental(slug: &str) -> Result<RentalResponse, String> {
    let response = Request::get(&format!("{API_BASE}/api/v1/rentals/{}", urlencoding::encode(slug)))
        .send().await.map_err(|e| e.to_string())?;
    if !response.ok() { return Err(response_error(response).await); }
    response.json().await.map_err(|e| e.to_string())
}

pub async fn availability(slug: &str, starts_on: &str, ends_on: &str) -> Result<AvailabilityResponse, String> {
    let response = Request::get(&format!(
        "{API_BASE}/api/v1/rentals/{}/availability?starts_on={}&ends_on={}",
        urlencoding::encode(slug), urlencoding::encode(starts_on), urlencoding::encode(ends_on)
    )).send().await.map_err(|e| e.to_string())?;
    if !response.ok() { return Err(response_error(response).await); }
    response.json().await.map_err(|e| e.to_string())
}

pub async fn create_quote(draft: &TripDraft) -> Result<QuoteResponse, String> {
    let response = Request::post(&format!("{API_BASE}/api/v1/quotes"))
        .header("Content-Type", "application/json")
        .json(&CreateQuotePayload {
            rental_slug: &draft.rental_slug,
            starts_on: &draft.starts_on,
            ends_on: &draft.ends_on,
            guests: draft.guests,
            addon_keys: &draft.addon_keys,
            delivery_km: draft.delivery_km.as_deref(),
        }).map_err(|e| e.to_string())?
        .send().await.map_err(|e| e.to_string())?;
    if !response.ok() { return Err(response_error(response).await); }
    response.json().await.map_err(|e| e.to_string())
}

pub async fn create_booking(quote_id: &str, first_name: &str, last_name: &str, email: &str, phone: &str, notes: &str) -> Result<CreatedBooking, String> {
    let token = access_token().ok_or("Sign in before confirming your booking")?;
    let response = Request::post(&format!("{API_BASE}/api/v1/bookings"))
        .header("Content-Type", "application/json")
        .header("Authorization", &format!("Bearer {token}"))
        .json(&CreateBookingPayload { quote_id, first_name, last_name, email, phone, notes: (!notes.trim().is_empty()).then_some(notes) })
        .map_err(|e| e.to_string())?.send().await.map_err(|e| e.to_string())?;
    if !response.ok() { return Err(response_error(response).await); }
    response.json().await.map_err(|e| e.to_string())
}

pub async fn my_bookings() -> Result<Vec<Booking>, String> {
    let token = access_token().ok_or("Sign in to view bookings")?;
    let response = Request::get(&format!("{API_BASE}/api/v1/me/bookings"))
        .header("Authorization", &format!("Bearer {token}"))
        .send().await.map_err(|e| e.to_string())?;
    if !response.ok() { return Err(response_error(response).await); }
    response.json::<BookingsResponse>().await.map(|value| value.bookings).map_err(|e| e.to_string())
}

pub fn save_json<T: Serialize>(key: &str, value: &T) -> Result<(), String> {
    storage().ok_or("Browser storage is unavailable")?.set_item(key, &serde_json::to_string(value).map_err(|e| e.to_string())?).map_err(|_| "Could not save booking progress".into())
}

pub fn load_json<T: for<'de> Deserialize<'de>>(key: &str) -> Option<T> {
    serde_json::from_str(&storage()?.get_item(key).ok()??).ok()
}

pub fn remove_saved(key: &str) {
    if let Some(storage) = storage() { let _ = storage.remove_item(key); }
}

#[derive(Serialize)]
struct ContactPayload<'a> { full_name: &'a str, email: &'a str, phone: &'a str, interest: &'a str, message: &'a str }
#[derive(Serialize)]
struct NewsletterPayload<'a> { email: &'a str }
#[derive(Serialize)]
struct SalesPayload<'a> { full_name: &'a str, email: &'a str, phone: &'a str, requested_unit: &'a str, message: &'a str }

async fn post_public<T: Serialize>(path: &str, value: &T) -> Result<(), String> {
    let response = Request::post(&format!("{API_BASE}/api/v1/{path}"))
        .header("Content-Type", "application/json").json(value).map_err(|e| e.to_string())?
        .send().await.map_err(|e| e.to_string())?;
    if !response.ok() { return Err(response_error(response).await); }
    Ok(())
}

pub async fn send_contact(full_name: &str, email: &str, phone: &str, interest: &str, message: &str) -> Result<(), String> {
    post_public("contact", &ContactPayload { full_name, email, phone, interest, message }).await
}

pub async fn subscribe(email: &str) -> Result<(), String> {
    post_public("newsletter", &NewsletterPayload { email }).await
}

pub async fn send_sales_inquiry(full_name: &str, email: &str, phone: &str, requested_unit: &str, message: &str) -> Result<(), String> {
    post_public("sales-inquiries", &SalesPayload { full_name, email, phone, requested_unit, message }).await
}
