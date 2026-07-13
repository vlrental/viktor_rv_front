use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use std::fmt;

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
#[allow(dead_code)]
pub struct RentalMedia {
    pub source_url: String,
    pub alt_text: String,
}

#[derive(Clone, Debug, Deserialize)]
#[allow(dead_code)]
pub struct RentalFeature {
    pub group_name: String,
    pub label: String,
}

#[derive(Clone, Debug, Deserialize)]
#[allow(dead_code)]
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
    #[serde(default)]
    pub delivery_address: Option<String>,
    #[serde(default)]
    pub attending_event: bool,
    #[serde(default)]
    pub towing_after_delivery: bool,
}

pub fn rv_delivery_ready(draft: &TripDraft) -> bool {
    let address_is_valid = draft
        .delivery_address
        .as_deref()
        .map(str::trim)
        .is_some_and(|address| address.len() >= 5);
    let distance_is_valid = draft
        .delivery_km
        .as_deref()
        .and_then(|distance| distance.parse::<f64>().ok())
        .is_some_and(|distance| (0.0..=150.0).contains(&distance));

    address_is_valid && distance_is_valid
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CatalogSearchDraft {
    pub location: String,
    pub radius_km: i32,
    pub starts_on: Option<String>,
    pub ends_on: Option<String>,
    pub guests: i32,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct DeliveryEstimate {
    pub resolved_address: String,
    pub one_way_km: String,
    pub round_trip_km: String,
    pub delivery_fee: String,
    pub maximum_km: String,
    pub within_range: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct AddressSuggestion {
    pub display_name: String,
    pub primary_text: String,
    pub secondary_text: String,
}

#[derive(Serialize)]
struct CreateQuotePayload<'a> {
    rental_slug: &'a str,
    starts_on: &'a str,
    ends_on: &'a str,
    guests: i32,
    addon_keys: &'a [String],
    delivery_address: Option<&'a str>,
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
    #[serde(alias = "pickup_time")]
    pub delivery_time: String,
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

#[derive(Clone, Debug, PartialEq)]
pub struct ApiError {
    pub status: u16,
    pub code: String,
    pub message: String,
}

impl ApiError {
    pub fn client(message: impl Into<String>) -> Self {
        Self { status: 0, code: "client_error".into(), message: message.into() }
    }

    pub fn is_conflict(&self) -> bool {
        self.status == 409 || self.code == "conflict"
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

#[derive(Deserialize)]
struct ErrorDetails {
    code: String,
    message: String,
}

#[derive(Deserialize)]
struct ErrorResponse {
    error: ErrorDetails,
}

fn parse_response_error(status: u16, text: String) -> ApiError {
    match serde_json::from_str::<ErrorResponse>(&text) {
        Ok(value) => ApiError {
            status,
            code: value.error.code,
            message: value.error.message,
        },
        Err(_) => ApiError {
            status,
            code: if status == 409 { "conflict" } else { "request_failed" }.into(),
            message: if text.trim().is_empty() { "Request failed".into() } else { text },
        },
    }
}

async fn response_error(response: gloo_net::http::Response) -> ApiError {
    let status = response.status();
    let text = response.text().await.unwrap_or_else(|_| "Request failed".into());
    parse_response_error(status, text)
}

fn catalog_url(search: &CatalogSearchDraft) -> String {
    let mut query = vec![format!("guests={}", search.guests.clamp(1, 10))];
    if let (Some(starts_on), Some(ends_on)) = (&search.starts_on, &search.ends_on) {
        query.push(format!("starts_on={}", urlencoding::encode(starts_on)));
        query.push(format!("ends_on={}", urlencoding::encode(ends_on)));
    }
    format!("{API_BASE}/api/v1/catalog?{}", query.join("&"))
}

pub async fn catalog(search: &CatalogSearchDraft) -> Result<Vec<Rental>, String> {
    let response = Request::get(&catalog_url(search))
        .send().await.map_err(|e| e.to_string())?;
    if !response.ok() { return Err(response_error(response).await.message); }
    response.json::<CatalogResponse>().await.map(|value| value.rentals).map_err(|e| e.to_string())
}

pub async fn rental(slug: &str) -> Result<RentalResponse, String> {
    let response = Request::get(&format!("{API_BASE}/api/v1/rentals/{}", urlencoding::encode(slug)))
        .send().await.map_err(|e| e.to_string())?;
    if !response.ok() { return Err(response_error(response).await.message); }
    response.json().await.map_err(|e| e.to_string())
}

pub async fn availability(slug: &str, starts_on: &str, ends_on: &str) -> Result<AvailabilityResponse, String> {
    let response = Request::get(&format!(
        "{API_BASE}/api/v1/rentals/{}/availability?starts_on={}&ends_on={}",
        urlencoding::encode(slug), urlencoding::encode(starts_on), urlencoding::encode(ends_on)
    )).send().await.map_err(|e| e.to_string())?;
    if !response.ok() { return Err(response_error(response).await.message); }
    response.json().await.map_err(|e| e.to_string())
}

pub async fn delivery_estimate(slug: &str, address: &str) -> Result<DeliveryEstimate, String> {
    #[derive(Serialize)]
    struct Payload<'a> { address: &'a str }
    let response = Request::post(&format!("{API_BASE}/api/v1/rentals/{}/delivery-estimate", urlencoding::encode(slug)))
        .header("Content-Type", "application/json")
        .json(&Payload { address }).map_err(|e| e.to_string())?
        .send().await.map_err(|_| "The delivery calculator is temporarily unavailable. Please try again shortly.".to_string())?;
    if !response.ok() { return Err(response_error(response).await.message); }
    response.json().await.map_err(|e| e.to_string())
}

pub async fn address_suggestions(query: &str) -> Result<Vec<AddressSuggestion>, String> {
    let response = Request::get(&format!(
        "{API_BASE}/api/v1/address-suggestions?q={}",
        urlencoding::encode(query.trim())
    ))
    .send()
    .await
    .map_err(|_| "Address search is temporarily unavailable.".to_string())?;
    if !response.ok() {
        return Err("Address search is temporarily unavailable.".to_string());
    }
    response.json().await.map_err(|_| "Could not read address suggestions.".to_string())
}

pub async fn create_quote(draft: &TripDraft) -> Result<QuoteResponse, ApiError> {
    let response = Request::post(&format!("{API_BASE}/api/v1/quotes"))
        .header("Content-Type", "application/json")
        .json(&CreateQuotePayload {
            rental_slug: &draft.rental_slug,
            starts_on: &draft.starts_on,
            ends_on: &draft.ends_on,
            guests: draft.guests,
            addon_keys: &draft.addon_keys,
            delivery_address: draft.delivery_address.as_deref(),
        }).map_err(|error| ApiError::client(error.to_string()))?
        .send().await.map_err(|error| ApiError::client(error.to_string()))?;
    if !response.ok() { return Err(response_error(response).await); }
    response.json().await.map_err(|error| ApiError::client(error.to_string()))
}

pub async fn create_booking(quote_id: &str, first_name: &str, last_name: &str, email: &str, phone: &str, notes: &str) -> Result<CreatedBooking, ApiError> {
    let token = access_token().ok_or_else(|| ApiError { status: 401, code: "unauthorized".into(), message: "Sign in before confirming your booking".into() })?;
    let response = Request::post(&format!("{API_BASE}/api/v1/bookings"))
        .header("Content-Type", "application/json")
        .header("Authorization", &format!("Bearer {token}"))
        .json(&CreateBookingPayload { quote_id, first_name, last_name, email, phone, notes: (!notes.trim().is_empty()).then_some(notes) })
        .map_err(|error| ApiError::client(error.to_string()))?.send().await.map_err(|error| ApiError::client(error.to_string()))?;
    if !response.ok() { return Err(response_error(response).await); }
    response.json().await.map_err(|error| ApiError::client(error.to_string()))
}

pub fn catalog_search_for_trip(
    saved: Option<CatalogSearchDraft>,
    draft: &TripDraft,
) -> CatalogSearchDraft {
    let mut search = saved.unwrap_or(CatalogSearchDraft {
        location: "Kelowna, BC".into(),
        radius_km: 150,
        starts_on: None,
        ends_on: None,
        guests: 2,
    });
    search.starts_on = Some(draft.starts_on.clone());
    search.ends_on = Some(draft.ends_on.clone());
    search.guests = draft.guests.clamp(1, 10);
    search
}

pub fn prepare_catalog_after_conflict(draft: &TripDraft) {
    remove_saved("vl_active_quote");
    let search = catalog_search_for_trip(load_json("vl_catalog_search"), draft);
    let _ = save_json("vl_catalog_search", &search);
}

pub async fn my_bookings() -> Result<Vec<Booking>, String> {
    let token = access_token().ok_or("Sign in to view bookings")?;
    let response = Request::get(&format!("{API_BASE}/api/v1/me/bookings"))
        .header("Authorization", &format!("Bearer {token}"))
        .send().await.map_err(|e| e.to_string())?;
    if !response.ok() { return Err(response_error(response).await.message); }
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
    if !response.ok() { return Err(response_error(response).await.message); }
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

#[cfg(test)]
mod delivery_draft_tests {
    use super::*;

    fn draft(address: Option<&str>, distance: Option<&str>) -> TripDraft {
        TripDraft {
            rental_slug: "test-rv".into(),
            starts_on: "2030-07-15".into(),
            ends_on: "2030-07-18".into(),
            guests: 2,
            addon_keys: Vec::new(),
            delivery_km: distance.map(str::to_string),
            delivery_address: address.map(str::to_string),
            attending_event: false,
            towing_after_delivery: false,
        }
    }

    #[test]
    fn old_drafts_without_a_calculated_delivery_are_rejected() {
        assert!(!rv_delivery_ready(&draft(None, None)));
        assert!(!rv_delivery_ready(&draft(Some("Kelowna, BC"), None)));
        assert!(!rv_delivery_ready(&draft(Some("   "), Some("25.0"))));
    }

    #[test]
    fn delivery_draft_must_be_within_the_policy_limit() {
        assert!(rv_delivery_ready(&draft(Some("Kelowna, BC"), Some("150.0"))));
        assert!(!rv_delivery_ready(&draft(Some("Kelowna, BC"), Some("150.1"))));
        assert!(!rv_delivery_ready(&draft(Some("Kelowna, BC"), Some("unknown"))));
    }

    #[test]
    fn availability_accepts_the_legacy_schedule_field_during_rollout() {
        let response: AvailabilityResponse = serde_json::from_str(
            r#"{
                "rental_slug":"test-rv",
                "starts_on":"2030-07-01",
                "ends_on":"2030-08-01",
                "unavailable":[],
                "pickup_time":"14:00",
                "return_time":"11:00",
                "timezone":"America/Vancouver",
                "minimum_nights":3
            }"#,
        )
        .unwrap();

        assert_eq!(response.delivery_time, "14:00");
    }

    #[test]
    fn catalog_query_contains_the_applied_dates_and_guests() {
        let search = CatalogSearchDraft {
            location: "Kelowna, BC".into(),
            radius_km: 150,
            starts_on: Some("2030-07-15".into()),
            ends_on: Some("2030-07-18".into()),
            guests: 6,
        };
        let url = catalog_url(&search);
        assert!(url.contains("guests=6"));
        assert!(url.contains("starts_on=2030-07-15"));
        assert!(url.contains("ends_on=2030-07-18"));
    }

    #[test]
    fn incomplete_dates_are_not_sent_to_catalog() {
        let search = CatalogSearchDraft {
            location: "Kelowna, BC".into(),
            radius_km: 50,
            starts_on: Some("2030-07-15".into()),
            ends_on: None,
            guests: 2,
        };
        let url = catalog_url(&search);
        assert!(!url.contains("starts_on"));
        assert!(!url.contains("ends_on"));
    }

    #[test]
    fn api_error_keeps_server_code_and_status() {
        let error = parse_response_error(
            409,
            r#"{"ok":false,"error":{"code":"conflict","message":"period unavailable"}}"#.into(),
        );

        assert_eq!(error.status, 409);
        assert_eq!(error.code, "conflict");
        assert_eq!(error.message, "period unavailable");
        assert!(error.is_conflict());
    }

    #[test]
    fn conflict_status_remains_machine_readable_for_non_json_responses() {
        let error = parse_response_error(409, "Request failed".into());

        assert_eq!(error.code, "conflict");
        assert!(error.is_conflict());
    }

    #[test]
    fn conflict_recovery_preserves_search_context_and_uses_trip_dates() {
        let saved = CatalogSearchDraft {
            location: "West Kelowna, BC".into(),
            radius_km: 75,
            starts_on: Some("2030-06-01".into()),
            ends_on: Some("2030-06-04".into()),
            guests: 2,
        };
        let trip = draft(Some("Kelowna, BC"), Some("25.0"));

        let search = catalog_search_for_trip(Some(saved), &trip);

        assert_eq!(search.location, "West Kelowna, BC");
        assert_eq!(search.radius_km, 75);
        assert_eq!(search.starts_on.as_deref(), Some("2030-07-15"));
        assert_eq!(search.ends_on.as_deref(), Some("2030-07-18"));
        assert_eq!(search.guests, 2);
    }
}
