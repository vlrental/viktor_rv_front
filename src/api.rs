use dioxus::prelude::document;
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt};

pub const API_BASE: &str = match option_env!("VL_API_BASE_URL") {
    Some(value) => value,
    None => "https://api.vlrental.ca",
};

pub fn frontend_base_url() -> String {
    option_env!("VL_FRONTEND_BASE_URL")
        .filter(|value| !value.trim().is_empty())
        .map(|value| value.trim_end_matches('/').to_string())
        .or_else(|| {
            web_sys::window()
                .and_then(|window| window.location().origin().ok())
                .map(|value| value.trim_end_matches('/').to_string())
        })
        .unwrap_or_else(|| "https://vlrental.ca".to_string())
}

pub fn google_login_url() -> String {
    format!(
        "{API_BASE}/api/v1/auth/google?return_to={}",
        urlencoding::encode(&frontend_base_url())
    )
}

pub fn frontend_path(path: &str) -> String {
    format!("{}{}", frontend_base_url(), normalized_auth_path(path))
}

fn normalized_auth_path(path: &str) -> &str {
    if path.starts_with('/') && !path.starts_with("//") {
        path
    } else {
        "/account"
    }
}

pub fn remember_auth_return(path: &str) {
    if let Some(storage) = web_sys::window()
        .and_then(|window| window.session_storage().ok())
        .flatten()
    {
        let _ = storage.set_item("vl_auth_return", path);
        let _ = storage.set_item("vl_google_auth_pending", "1");
        let _ = storage.remove_item("vl_inline_auth_mode");
        let _ = storage.remove_item("vl_inline_auth_error");
    }
}

pub fn take_google_auth_pending() -> bool {
    let Some(storage) = web_sys::window()
        .and_then(|window| window.session_storage().ok())
        .flatten()
    else {
        return false;
    };
    let pending = storage
        .get_item("vl_google_auth_pending")
        .ok()
        .flatten()
        .is_some_and(|value| value == "1");
    let _ = storage.remove_item("vl_google_auth_pending");
    pending
}

pub fn take_auth_return() -> Option<String> {
    let storage = web_sys::window()?.session_storage().ok().flatten()?;
    let path = storage.get_item("vl_auth_return").ok().flatten();
    let _ = storage.remove_item("vl_auth_return");
    path
}

pub fn request_inline_auth(register: bool, message: Option<&str>) {
    if let Some(storage) = web_sys::window()
        .and_then(|window| window.session_storage().ok())
        .flatten()
    {
        let _ = storage.set_item(
            "vl_inline_auth_mode",
            if register { "register" } else { "signin" },
        );
        if let Some(message) = message {
            let _ = storage.set_item("vl_inline_auth_error", message);
        } else {
            let _ = storage.remove_item("vl_inline_auth_error");
        }
    }
}

pub fn take_inline_auth_request() -> Option<(bool, Option<String>)> {
    let storage = web_sys::window()?.session_storage().ok().flatten()?;
    let mode = storage.get_item("vl_inline_auth_mode").ok().flatten()?;
    let message = storage.get_item("vl_inline_auth_error").ok().flatten();
    let _ = storage.remove_item("vl_inline_auth_mode");
    let _ = storage.remove_item("vl_inline_auth_error");
    Some((mode == "register", message))
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

fn auth_parameters(fragment: &str, search: &str) -> Option<HashMap<String, String>> {
    [fragment, search].into_iter().find_map(|source| {
        let mut value = source.trim().trim_start_matches(['#', '?']);
        if let Some((_, query)) = value.rsplit_once('?') {
            value = query;
        }
        let parameters = value
            .split('&')
            .filter_map(|part| part.split_once('='))
            .filter_map(|(key, value)| {
                Some((
                    urlencoding::decode(key).ok()?.into_owned(),
                    urlencoding::decode(value).ok()?.into_owned(),
                ))
            })
            .collect::<HashMap<_, _>>();
        parameters
            .keys()
            .any(|key| {
                matches!(
                    key.as_str(),
                    "vl_google_auth" | "access_token" | "refresh_token"
                )
            })
            .then_some(parameters)
    })
}

pub fn finish_google_sign_in() -> Result<Option<String>, String> {
    let window = web_sys::window().ok_or("Google sign in could not access this browser window.")?;
    let fragment = window.location().hash().unwrap_or_default();
    let search = window.location().search().unwrap_or_default();
    let Some(values) = auth_parameters(&fragment, &search) else {
        return Ok(None);
    };
    let required = |key: &str| {
        values
            .get(key)
            .filter(|value| !value.trim().is_empty())
            .cloned()
            .ok_or_else(|| "Google did not return a valid sign-in session.".to_string())
    };
    let tokens = AuthTokens {
        access_token: required("access_token")?,
        refresh_token: required("refresh_token")?,
        user: AuthUser {
            user_id: required("user_id")?,
            email: required("email")?,
            role: required("role")?,
        },
    };
    save_session(&tokens)?;
    let _ = take_google_auth_pending();
    Ok(Some(
        take_auth_return()
            .filter(|path| path.starts_with('/') && !path.starts_with("//"))
            .unwrap_or_else(|| "/".to_string()),
    ))
}
#[derive(Serialize)]
struct Credentials<'a> {
    email: &'a str,
    password: &'a str,
}

#[derive(Serialize)]
struct RefreshPayload<'a> {
    refresh_token: &'a str,
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
pub fn save_auth_user(user: &AuthUser) -> Result<(), String> {
    storage()
        .ok_or("Browser storage is unavailable")?
        .set_item(
            "vl_auth_user",
            &serde_json::to_string(user).map_err(|error| error.to_string())?,
        )
        .map_err(|_| "Could not update the saved user".into())
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

fn refresh_token() -> Option<String> {
    storage()?.get_item("vl_refresh_token").ok().flatten()
}

async fn refresh_session() -> Result<AuthTokens, ApiError> {
    let refresh = refresh_token().ok_or_else(|| ApiError {
        status: 401,
        code: "unauthorized".into(),
        message: "Sign in to continue".into(),
    })?;
    let response = Request::post(&format!("{API_BASE}/api/v1/auth/refresh"))
        .header("Content-Type", "application/json")
        .json(&RefreshPayload {
            refresh_token: &refresh,
        })
        .map_err(|error| ApiError::client(error.to_string()))?
        .send()
        .await
        .map_err(|error| ApiError::client(error.to_string()))?;
    if !response.ok() {
        clear_session();
        return Err(response_error(response).await);
    }
    let tokens = response
        .json::<AuthTokens>()
        .await
        .map_err(|error| ApiError::client(error.to_string()))?;
    save_session(&tokens).map_err(ApiError::client)?;
    Ok(tokens)
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
    #[serde(default)]
    pub review_rating: Option<String>,
    #[serde(default)]
    pub review_count: i64,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct RentalReview {
    pub rental_review_id: String,
    pub rental_slug: String,
    pub rating: i32,
    pub title: String,
    pub body: String,
    pub reviewer_name: String,
    pub created_at: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct RentalReviewSummary {
    pub average_rating: Option<String>,
    pub review_count: i64,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct RentalReviewsResponse {
    pub summary: RentalReviewSummary,
    pub reviews: Vec<RentalReview>,
}

#[derive(Deserialize)]
struct RentalReviewResponse {
    review: RentalReview,
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

#[derive(Debug, Deserialize)]
struct PhotonResponse {
    features: Vec<PhotonFeature>,
}

#[derive(Debug, Deserialize)]
struct PhotonFeature {
    properties: PhotonProperties,
    geometry: PhotonGeometry,
}

#[derive(Debug, Deserialize)]
struct PhotonGeometry {
    coordinates: Vec<f64>,
}

#[derive(Debug, Deserialize)]
struct PhotonProperties {
    housenumber: Option<String>,
    street: Option<String>,
    name: Option<String>,
    city: Option<String>,
    district: Option<String>,
    state: Option<String>,
    postcode: Option<String>,
    country: Option<String>,
}

#[derive(Debug)]
struct ResolvedCanadianAddress {
    display_name: String,
    primary_text: String,
    secondary_text: String,
    longitude: f64,
    latitude: f64,
}

#[derive(Deserialize)]
struct ClientRouteResponse {
    routes: Vec<ClientRoute>,
}

#[derive(Deserialize)]
struct ClientRoute {
    distance: f64,
}

async fn search_canadian_addresses(query: &str) -> Result<Vec<ResolvedCanadianAddress>, String> {
    let trimmed = query.trim();
    let already_has_country = trimmed
        .trim_end_matches(',')
        .rsplit(',')
        .next()
        .is_some_and(|part| part.trim().eq_ignore_ascii_case("canada"));
    let locate = if already_has_country {
        trimmed.to_string()
    } else if trimmed.contains(',') {
        format!("{trimmed}, Canada")
    } else {
        format!("{trimmed}, Kelowna, BC, Canada")
    };
    let url = format!(
        "https://photon.komoot.io/api/?q={}&limit=5&lang=en&lat=50.0150675&lon=-119.3870978",
        urlencoding::encode(&locate)
    );
    let url_json = serde_json::to_string(&url)
        .map_err(|_| "Could not prepare the address search.".to_string())?;
    let script = format!(
        "const response = await fetch({url_json}); if (!response.ok) throw new Error('address search failed'); return await response.json();"
    );
    let features = document::eval(&script)
        .join::<PhotonResponse>()
        .await
        .map_err(|_| "Could not look up that Canadian address.".to_string())?
        .features;
    let mut results = Vec::new();
    for feature in features {
        let Some(longitude) = feature.geometry.coordinates.first().copied() else {
            continue;
        };
        let Some(latitude) = feature.geometry.coordinates.get(1).copied() else {
            continue;
        };
        let street = [
            feature.properties.housenumber.as_deref(),
            feature.properties.street.as_deref(),
        ]
        .into_iter()
        .flatten()
        .filter(|part| !part.trim().is_empty())
        .collect::<Vec<_>>()
        .join(" ");
        let primary_text = if street.is_empty() {
            feature
                .properties
                .name
                .clone()
                .unwrap_or_else(|| query.trim().to_string())
        } else {
            street
        };
        let city = feature
            .properties
            .city
            .or(feature.properties.district)
            .unwrap_or_else(|| "Kelowna".to_string());
        let province = feature
            .properties
            .state
            .unwrap_or_else(|| "British Columbia".to_string());
        let postal = feature.properties.postcode.unwrap_or_default();
        let country = feature
            .properties
            .country
            .unwrap_or_else(|| "Canada".to_string());
        let secondary_text = [
            city.as_str(),
            province.as_str(),
            postal.as_str(),
            country.as_str(),
        ]
        .into_iter()
        .filter(|part| !part.trim().is_empty())
        .collect::<Vec<_>>()
        .join(", ");
        let display_name = format!("{primary_text}, {secondary_text}");
        if !results
            .iter()
            .any(|item: &ResolvedCanadianAddress| item.display_name == display_name)
        {
            results.push(ResolvedCanadianAddress {
                display_name,
                primary_text,
                secondary_text,
                longitude,
                latitude,
            });
        }
    }
    Ok(results)
}

async fn resolve_canadian_address(query: &str) -> Result<ResolvedCanadianAddress, String> {
    search_canadian_addresses(query)
        .await?
        .into_iter()
        .next()
        .ok_or_else(|| "That address was not found in Canada.".to_string())
}

async fn client_delivery_estimate(address: &str) -> Result<DeliveryEstimate, String> {
    let resolved = resolve_canadian_address(address).await?;
    let route_url = format!(
        "https://router.project-osrm.org/route/v1/driving/-119.3870978,50.0150675;{},{}?overview=false",
        resolved.longitude, resolved.latitude
    );
    let route_url_json = serde_json::to_string(&route_url)
        .map_err(|_| "Could not prepare the driving route.".to_string())?;
    let route_script = format!(
        "const response = await fetch({route_url_json}); if (!response.ok) throw new Error('route failed'); return await response.json();"
    );
    let meters = document::eval(&route_script)
        .join::<ClientRouteResponse>()
        .await
        .map_err(|_| "Could not read the driving route.".to_string())?
        .routes
        .first()
        .map(|route| route.distance)
        .ok_or_else(|| "No driving route was found for that address.".to_string())?;
    let one_way_km = ((meters / 1000.0) * 10.0).round() / 10.0;
    let delivery_fee = if one_way_km <= 50.0 {
        150.0
    } else {
        150.0 + (one_way_km - 50.0) * 3.5
    };
    Ok(DeliveryEstimate {
        resolved_address: resolved.display_name,
        one_way_km: format!("{one_way_km:.1}"),
        round_trip_km: format!("{:.1}", one_way_km * 2.0),
        delivery_fee: format!("{delivery_fee:.2}"),
        maximum_km: "150.0".to_string(),
        within_range: one_way_km <= 150.0,
    })
}

fn address_for_api(address: &str) -> String {
    let address = address.trim().trim_end_matches(',').trim();
    match address.rsplit_once(',') {
        Some((without_country, country)) if country.trim().eq_ignore_ascii_case("canada") => {
            without_country
                .trim()
                .trim_end_matches(',')
                .trim()
                .to_string()
        }
        _ => address.to_string(),
    }
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
#[allow(dead_code)]
pub struct UnavailableInterval {
    pub starts_at: String,
    pub ends_at: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[allow(dead_code)]
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
    #[serde(default)]
    pub rental_slug: String,
    #[serde(default)]
    pub rental_name: String,
    pub status: String,
    pub payment_status: String,
    pub starts_at: String,
    pub ends_at: String,
    pub currency: String,
    pub total: String,
    pub amount_due_now: String,
    #[serde(default)]
    pub review_id: Option<String>,
    #[serde(default)]
    pub can_review: bool,
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

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct AdminBooking {
    pub booking_id: String,
    pub booking_number: String,
    pub rental_slug: String,
    pub rental_name: String,
    pub guests: i32,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: String,
    pub status: String,
    pub payment_status: String,
    pub starts_at: String,
    pub ends_at: String,
    pub currency: String,
    pub total: String,
    pub amount_due_now: String,
    pub created_at: String,
}

#[derive(Deserialize)]
struct AdminBookingsResponse {
    bookings: Vec<AdminBooking>,
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

#[derive(Serialize)]
struct CreateRentalReviewPayload<'a> {
    rating: i32,
    title: &'a str,
    body: &'a str,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ApiError {
    pub status: u16,
    pub code: String,
    pub message: String,
}

impl ApiError {
    pub fn client(message: impl Into<String>) -> Self {
        Self {
            status: 0,
            code: "client_error".into(),
            message: message.into(),
        }
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
            code: if status == 409 {
                "conflict"
            } else {
                "request_failed"
            }
            .into(),
            message: if text.trim().is_empty() {
                "Request failed".into()
            } else {
                text
            },
        },
    }
}

async fn response_error(response: gloo_net::http::Response) -> ApiError {
    let status = response.status();
    let text = response
        .text()
        .await
        .unwrap_or_else(|_| "Request failed".into());
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
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !response.ok() {
        return Err(response_error(response).await.message);
    }
    response
        .json::<CatalogResponse>()
        .await
        .map(|value| value.rentals)
        .map_err(|e| e.to_string())
}

pub async fn rental(slug: &str) -> Result<RentalResponse, String> {
    let response = Request::get(&format!(
        "{API_BASE}/api/v1/rentals/{}",
        urlencoding::encode(slug)
    ))
    .send()
    .await
    .map_err(|e| e.to_string())?;
    if !response.ok() {
        return Err(response_error(response).await.message);
    }
    response.json().await.map_err(|e| e.to_string())
}

pub async fn rental_reviews(slug: &str) -> Result<RentalReviewsResponse, String> {
    let response = Request::get(&format!(
        "{API_BASE}/api/v1/rentals/{}/reviews?limit=50",
        urlencoding::encode(slug)
    ))
    .send()
    .await
    .map_err(|error| error.to_string())?;
    if !response.ok() {
        return Err(response_error(response).await.message);
    }
    response.json().await.map_err(|error| error.to_string())
}

#[allow(dead_code)]
pub async fn availability(
    slug: &str,
    starts_on: &str,
    ends_on: &str,
) -> Result<AvailabilityResponse, String> {
    let response = Request::get(&format!(
        "{API_BASE}/api/v1/rentals/{}/availability?starts_on={}&ends_on={}",
        urlencoding::encode(slug),
        urlencoding::encode(starts_on),
        urlencoding::encode(ends_on)
    ))
    .send()
    .await
    .map_err(|e| e.to_string())?;
    if !response.ok() {
        return Err(response_error(response).await.message);
    }
    response.json().await.map_err(|e| e.to_string())
}

pub async fn delivery_estimate(slug: &str, address: &str) -> Result<DeliveryEstimate, String> {
    #[derive(Serialize)]
    struct Payload<'a> {
        address: &'a str,
    }
    let api_address = address_for_api(address);
    let response = Request::post(&format!(
        "{API_BASE}/api/v1/rentals/{}/delivery-estimate",
        urlencoding::encode(slug)
    ))
    .header("Content-Type", "application/json")
    .json(&Payload {
        address: &api_address,
    })
    .map_err(|e| e.to_string())?
    .send()
    .await;
    match response {
        Ok(response) if response.ok() => response.json().await.map_err(|e| e.to_string()),
        Ok(response) if response.status() != 404 => Err(response_error(response).await.message),
        _ => client_delivery_estimate(address).await,
    }
}

pub async fn address_suggestions(query: &str) -> Result<Vec<AddressSuggestion>, String> {
    let response = Request::get(&format!(
        "{API_BASE}/api/v1/address-suggestions?q={}",
        urlencoding::encode(query.trim())
    ))
    .send()
    .await;
    if let Ok(response) = response {
        if response.ok() {
            return response
                .json()
                .await
                .map_err(|_| "Could not read address suggestions.".to_string());
        }
    }
    match search_canadian_addresses(query).await {
        Ok(values) => Ok(values
            .into_iter()
            .map(|value| AddressSuggestion {
                display_name: value.display_name,
                primary_text: value.primary_text,
                secondary_text: value.secondary_text,
            })
            .collect()),
        Err(_) => Ok(Vec::new()),
    }
}

pub async fn create_quote(draft: &TripDraft) -> Result<QuoteResponse, ApiError> {
    let api_delivery_address = draft.delivery_address.as_deref().map(address_for_api);
    let response = Request::post(&format!("{API_BASE}/api/v1/quotes"))
        .header("Content-Type", "application/json")
        .json(&CreateQuotePayload {
            rental_slug: &draft.rental_slug,
            starts_on: &draft.starts_on,
            ends_on: &draft.ends_on,
            guests: draft.guests,
            addon_keys: &draft.addon_keys,
            delivery_address: api_delivery_address.as_deref(),
        })
        .map_err(|error| ApiError::client(error.to_string()))?
        .send()
        .await
        .map_err(|error| ApiError::client(error.to_string()))?;
    if !response.ok() {
        return Err(response_error(response).await);
    }
    response
        .json()
        .await
        .map_err(|error| ApiError::client(error.to_string()))
}

pub async fn create_booking(
    quote_id: &str,
    first_name: &str,
    last_name: &str,
    email: &str,
    phone: &str,
    notes: &str,
) -> Result<CreatedBooking, ApiError> {
    let mut token = access_token().ok_or_else(|| ApiError {
        status: 401,
        code: "unauthorized".into(),
        message: "Sign in before confirming your booking".into(),
    })?;
    let mut response =
        send_booking_request(&token, quote_id, first_name, last_name, email, phone, notes).await?;
    if response.status() == 401 {
        token = refresh_session().await?.access_token;
        response =
            send_booking_request(&token, quote_id, first_name, last_name, email, phone, notes)
                .await?;
    }
    if !response.ok() {
        return Err(response_error(response).await);
    }
    response
        .json()
        .await
        .map_err(|error| ApiError::client(error.to_string()))
}

async fn send_booking_request(
    token: &str,
    quote_id: &str,
    first_name: &str,
    last_name: &str,
    email: &str,
    phone: &str,
    notes: &str,
) -> Result<gloo_net::http::Response, ApiError> {
    Request::post(&format!("{API_BASE}/api/v1/bookings"))
        .header("Content-Type", "application/json")
        .header("Authorization", &format!("Bearer {token}"))
        .json(&CreateBookingPayload {
            quote_id,
            first_name,
            last_name,
            email,
            phone,
            notes: (!notes.trim().is_empty()).then_some(notes),
        })
        .map_err(|error| ApiError::client(error.to_string()))?
        .send()
        .await
        .map_err(|error| ApiError::client(error.to_string()))
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct AdminAvailabilityBlock {
    pub availability_block_id: String,
    pub rental_slug: String,
    pub rental_name: String,
    pub starts_at: String,
    pub ends_at: String,
    pub reason: String,
    pub created_at: String,
}

#[derive(Deserialize)]
struct AdminAvailabilityBlocksResponse {
    availability_blocks: Vec<AdminAvailabilityBlock>,
}

#[derive(Deserialize)]
struct AdminAvailabilityBlockResponse {
    availability_block: AdminAvailabilityBlock,
}

#[derive(Serialize)]
struct CreateAdminAvailabilityBlockPayload<'a> {
    rental_slug: &'a str,
    starts_on: &'a str,
    ends_on: &'a str,
    reason: &'a str,
}

pub async fn auth_me() -> Result<AuthUser, ApiError> {
    let mut token = access_token().ok_or_else(|| ApiError {
        status: 401,
        code: "unauthorized".into(),
        message: "Sign in to continue".into(),
    })?;
    let mut response = auth_me_request(&token).await?;
    if response.status() == 401 {
        token = refresh_session().await?.access_token;
        response = auth_me_request(&token).await?;
    }
    if !response.ok() {
        return Err(response_error(response).await);
    }
    response
        .json()
        .await
        .map_err(|error| ApiError::client(error.to_string()))
}

async fn auth_me_request(token: &str) -> Result<gloo_net::http::Response, ApiError> {
    Request::get(&format!("{API_BASE}/api/v1/auth/me"))
        .header("Authorization", &format!("Bearer {token}"))
        .send()
        .await
        .map_err(|error| ApiError::client(error.to_string()))
}

pub async fn admin_availability_blocks() -> Result<Vec<AdminAvailabilityBlock>, ApiError> {
    let token = access_token().ok_or_else(|| ApiError {
        status: 401,
        code: "unauthorized".into(),
        message: "Admin sign-in is required".into(),
    })?;
    let response = Request::get(&format!(
        "{API_BASE}/api/v1/admin/availability-blocks?limit=500"
    ))
    .header("Authorization", &format!("Bearer {token}"))
    .send()
    .await
    .map_err(|error| ApiError::client(error.to_string()))?;
    if !response.ok() {
        return Err(response_error(response).await);
    }
    response
        .json::<AdminAvailabilityBlocksResponse>()
        .await
        .map(|value| value.availability_blocks)
        .map_err(|error| ApiError::client(error.to_string()))
}

pub async fn admin_bookings() -> Result<Vec<AdminBooking>, ApiError> {
    let token = access_token().ok_or_else(|| ApiError {
        status: 401,
        code: "unauthorized".into(),
        message: "Admin sign-in is required".into(),
    })?;
    let response = Request::get(&format!("{API_BASE}/api/v1/admin/bookings?limit=500"))
        .header("Authorization", &format!("Bearer {token}"))
        .send()
        .await
        .map_err(|error| ApiError::client(error.to_string()))?;
    if !response.ok() {
        return Err(response_error(response).await);
    }
    response
        .json::<AdminBookingsResponse>()
        .await
        .map(|value| value.bookings)
        .map_err(|error| ApiError::client(error.to_string()))
}

pub async fn create_admin_availability_block(
    rental_slug: &str,
    starts_on: &str,
    ends_on: &str,
    reason: &str,
) -> Result<AdminAvailabilityBlock, ApiError> {
    let token = access_token().ok_or_else(|| ApiError {
        status: 401,
        code: "unauthorized".into(),
        message: "Admin sign-in is required".into(),
    })?;
    let response = Request::post(&format!("{API_BASE}/api/v1/admin/availability-blocks"))
        .header("Content-Type", "application/json")
        .header("Authorization", &format!("Bearer {token}"))
        .json(&CreateAdminAvailabilityBlockPayload {
            rental_slug,
            starts_on,
            ends_on,
            reason,
        })
        .map_err(|error| ApiError::client(error.to_string()))?
        .send()
        .await
        .map_err(|error| ApiError::client(error.to_string()))?;
    if !response.ok() {
        return Err(response_error(response).await);
    }
    response
        .json::<AdminAvailabilityBlockResponse>()
        .await
        .map(|value| value.availability_block)
        .map_err(|error| ApiError::client(error.to_string()))
}

pub async fn delete_admin_availability_block(block_id: &str) -> Result<(), ApiError> {
    let token = access_token().ok_or_else(|| ApiError {
        status: 401,
        code: "unauthorized".into(),
        message: "Admin sign-in is required".into(),
    })?;
    let response = Request::delete(&format!(
        "{API_BASE}/api/v1/admin/availability-blocks/{}",
        urlencoding::encode(block_id)
    ))
    .header("Authorization", &format!("Bearer {token}"))
    .send()
    .await
    .map_err(|error| ApiError::client(error.to_string()))?;
    if !response.ok() {
        return Err(response_error(response).await);
    }
    Ok(())
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
    let mut token = access_token().ok_or("Sign in to view bookings")?;
    let mut response = my_bookings_request(&token)
        .await
        .map_err(|error| error.message)?;
    if response.status() == 401 {
        token = refresh_session()
            .await
            .map_err(|error| error.message)?
            .access_token;
        response = my_bookings_request(&token)
            .await
            .map_err(|error| error.message)?;
    }
    if !response.ok() {
        return Err(response_error(response).await.message);
    }
    response
        .json::<BookingsResponse>()
        .await
        .map(|value| value.bookings)
        .map_err(|e| e.to_string())
}

async fn my_bookings_request(token: &str) -> Result<gloo_net::http::Response, ApiError> {
    Request::get(&format!("{API_BASE}/api/v1/me/bookings"))
        .header("Authorization", &format!("Bearer {token}"))
        .send()
        .await
        .map_err(|error| ApiError::client(error.to_string()))
}

pub async fn create_rental_review(
    booking_id: &str,
    rating: i32,
    title: &str,
    body: &str,
) -> Result<RentalReview, ApiError> {
    let mut token = access_token().ok_or_else(|| ApiError {
        status: 401,
        code: "unauthorized".into(),
        message: "Sign in to leave a review".into(),
    })?;
    let mut response =
        create_rental_review_request(booking_id, rating, title, body, &token).await?;
    if response.status() == 401 {
        token = refresh_session().await?.access_token;
        response = create_rental_review_request(booking_id, rating, title, body, &token).await?;
    }
    if !response.ok() {
        return Err(response_error(response).await);
    }
    response
        .json::<RentalReviewResponse>()
        .await
        .map(|value| value.review)
        .map_err(|error| ApiError::client(error.to_string()))
}

async fn create_rental_review_request(
    booking_id: &str,
    rating: i32,
    title: &str,
    body: &str,
    token: &str,
) -> Result<gloo_net::http::Response, ApiError> {
    Request::post(&format!(
        "{API_BASE}/api/v1/bookings/{}/review",
        urlencoding::encode(booking_id)
    ))
    .header("Content-Type", "application/json")
    .header("Authorization", &format!("Bearer {token}"))
    .json(&CreateRentalReviewPayload {
        rating,
        title,
        body,
    })
    .map_err(|error| ApiError::client(error.to_string()))?
    .send()
    .await
    .map_err(|error| ApiError::client(error.to_string()))
}

pub fn save_json<T: Serialize>(key: &str, value: &T) -> Result<(), String> {
    storage()
        .ok_or("Browser storage is unavailable")?
        .set_item(
            key,
            &serde_json::to_string(value).map_err(|e| e.to_string())?,
        )
        .map_err(|_| "Could not save booking progress".into())
}

pub fn load_json<T: for<'de> Deserialize<'de>>(key: &str) -> Option<T> {
    serde_json::from_str(&storage()?.get_item(key).ok()??).ok()
}

pub fn remove_saved(key: &str) {
    if let Some(storage) = storage() {
        let _ = storage.remove_item(key);
    }
}

#[derive(Serialize)]
struct ContactPayload<'a> {
    full_name: &'a str,
    email: &'a str,
    phone: &'a str,
    interest: &'a str,
    message: &'a str,
}
#[derive(Serialize)]
struct NewsletterPayload<'a> {
    email: &'a str,
}
#[derive(Serialize)]
struct SalesPayload<'a> {
    full_name: &'a str,
    email: &'a str,
    phone: &'a str,
    requested_unit: &'a str,
    message: &'a str,
}

async fn post_public<T: Serialize>(path: &str, value: &T) -> Result<(), String> {
    let response = Request::post(&format!("{API_BASE}/api/v1/{path}"))
        .header("Content-Type", "application/json")
        .json(value)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !response.ok() {
        return Err(response_error(response).await.message);
    }
    Ok(())
}

pub async fn send_contact(
    full_name: &str,
    email: &str,
    phone: &str,
    interest: &str,
    message: &str,
) -> Result<(), String> {
    post_public(
        "contact",
        &ContactPayload {
            full_name,
            email,
            phone,
            interest,
            message,
        },
    )
    .await
}

pub async fn subscribe(email: &str) -> Result<(), String> {
    post_public("newsletter", &NewsletterPayload { email }).await
}

pub async fn send_sales_inquiry(
    full_name: &str,
    email: &str,
    phone: &str,
    requested_unit: &str,
    message: &str,
) -> Result<(), String> {
    post_public(
        "sales-inquiries",
        &SalesPayload {
            full_name,
            email,
            phone,
            requested_unit,
            message,
        },
    )
    .await
}

#[cfg(test)]
mod delivery_draft_tests {
    use super::*;

    #[test]
    fn api_address_avoids_duplicate_canada_on_older_servers() {
        assert_eq!(
            address_for_api(
                "1198 Raymer Avenue, Kelowna, Pandosy, British Columbia, V1Y 5C1, Canada"
            ),
            "1198 Raymer Avenue, Kelowna, Pandosy, British Columbia, V1Y 5C1"
        );
        assert_eq!(
            address_for_api("1198 Raymer Avenue, Kelowna, BC"),
            "1198 Raymer Avenue, Kelowna, BC"
        );
    }

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
        assert!(rv_delivery_ready(&draft(
            Some("Kelowna, BC"),
            Some("150.0")
        )));
        assert!(!rv_delivery_ready(&draft(
            Some("Kelowna, BC"),
            Some("150.1")
        )));
        assert!(!rv_delivery_ready(&draft(
            Some("Kelowna, BC"),
            Some("unknown")
        )));
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

    #[test]
    fn admin_booking_response_includes_private_dashboard_fields() {
        let response: AdminBookingsResponse = serde_json::from_str(
            r#"{"bookings":[{"booking_id":"booking-1","booking_number":"VL-20260713-00000001","rental_slug":"jayco26","rental_name":"Jayco 26","guests":4,"first_name":"Test","last_name":"Guest","email":"guest@example.com","phone":"250-555-0100","status":"confirmed","payment_status":"test_paid","starts_at":"2026-07-15T21:00:00Z","ends_at":"2026-07-18T18:00:00Z","currency":"CAD","total":"1497.00","amount_due_now":"1497.00","created_at":"2026-07-13T18:00:00Z"}]}"#,
        )
        .unwrap();

        assert_eq!(response.bookings[0].guests, 4);
        assert_eq!(response.bookings[0].email, "guest@example.com");
        assert_eq!(response.bookings[0].rental_slug, "jayco26");
    }

    #[test]
    fn oauth_return_path_cannot_become_an_external_redirect() {
        assert_eq!(normalized_auth_path("/account"), "/account");
        assert_eq!(normalized_auth_path("//evil.example"), "/account");
        assert_eq!(normalized_auth_path("https://evil.example"), "/account");
    }

    #[test]
    fn google_tokens_are_read_from_path_and_hash_router_fragments() {
        let direct = auth_parameters(
            "#access_token=access&refresh_token=refresh&user_id=user&email=vl%40example.com&role=admin",
            "",
        )
        .unwrap();
        let hash_router = auth_parameters(
            "#/auth/callback?access_token=access&refresh_token=refresh&user_id=user&email=vl%40example.com&role=admin",
            "",
        )
        .unwrap();

        assert_eq!(direct["email"], "vl@example.com");
        assert_eq!(hash_router["access_token"], "access");
        assert_eq!(hash_router["role"], "admin");
    }
}
