use dioxus::prelude::document;
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt};

pub const API_BASE: &str = match option_env!("VL_API_BASE_URL") {
    Some(value) => value,
    None => "https://api.vlrental.ca",
};

pub fn frontend_base_url() -> String {
    browser_github_pages_base()
        .or_else(|| {
            option_env!("VL_FRONTEND_BASE_URL")
                .filter(|value| !value.trim().is_empty())
                .map(|value| value.trim_end_matches('/').to_string())
        })
        .or_else(|| {
            web_sys::window()
                .and_then(|window| window.location().origin().ok())
                .map(|value| value.trim_end_matches('/').to_string())
        })
        .unwrap_or_else(|| "https://vlrental.ca".to_string())
}

fn browser_github_pages_base() -> Option<String> {
    let location = web_sys::window()?.location();
    github_pages_base(
        &location.origin().ok()?,
        &location.hostname().ok()?,
        &location.pathname().ok()?,
    )
}

fn github_pages_base(origin: &str, hostname: &str, pathname: &str) -> Option<String> {
    if !hostname.to_ascii_lowercase().ends_with(".github.io") {
        return None;
    }
    let repository = pathname
        .trim_start_matches('/')
        .split('/')
        .next()
        .filter(|value| !value.is_empty());
    Some(match repository {
        Some(repository) => format!("{}/{}", origin.trim_end_matches('/'), repository),
        None => origin.trim_end_matches('/').to_string(),
    })
}

pub fn google_login_url() -> String {
    format!(
        "{API_BASE}/api/v1/auth/google?return_to={}",
        urlencoding::encode(&frontend_base_url())
    )
}

pub fn frontend_path(path: &str) -> String {
    frontend_path_for_base(&frontend_base_url(), path)
}

fn frontend_path_for_base(base_url: &str, path: &str) -> String {
    format!(
        "{}{}",
        base_url.trim_end_matches('/'),
        normalized_frontend_path(path)
    )
}

fn normalized_frontend_path(path: &str) -> &str {
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

pub fn save_booking_auth_continuation(
    continuation: &BookingAuthContinuation,
) -> Result<(), String> {
    let storage = web_sys::window()
        .and_then(|window| window.session_storage().ok())
        .flatten()
        .ok_or("Browser session storage is unavailable")?;
    let value = serde_json::to_string(continuation).map_err(|error| error.to_string())?;
    storage
        .set_item("vl_booking_auth_continuation", &value)
        .map_err(|_| "Could not preserve the booking while signing in".to_string())
}

pub fn take_booking_auth_continuation() -> Option<BookingAuthContinuation> {
    let storage = web_sys::window()?.session_storage().ok().flatten()?;
    let value = storage
        .get_item("vl_booking_auth_continuation")
        .ok()
        .flatten();
    let _ = storage.remove_item("vl_booking_auth_continuation");
    serde_json::from_str(&value?).ok()
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
    #[serde(default)]
    pub first_name: String,
    #[serde(default)]
    pub last_name: String,
    #[serde(default)]
    pub phone: String,
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
            first_name: String::new(),
            last_name: String::new(),
            phone: String::new(),
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

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct DeliveryEstimate {
    pub resolved_address: String,
    pub one_way_km: String,
    pub round_trip_km: String,
    pub delivery_fee: String,
    pub maximum_km: String,
    pub within_range: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BookingAuthContinuation {
    pub draft: TripDraft,
    pub location: String,
    pub radius_km: i32,
    #[serde(default)]
    pub delivery_estimate: Option<DeliveryEstimate>,
    #[serde(default)]
    pub first_name: String,
    #[serde(default)]
    pub last_name: String,
    #[serde(default)]
    pub booking_email: String,
    #[serde(default)]
    pub phone: String,
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub accepted_terms: bool,
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
    #[serde(default)]
    pub access_token: String,
    #[serde(default)]
    pub notification_email_sent: bool,
    #[serde(default, alias = "checkout_client_secret")]
    pub client_secret: Option<String>,
    #[serde(default)]
    pub checkout_session_id: Option<String>,
    #[serde(default)]
    pub payment_enabled: bool,
    #[serde(default)]
    pub payment_expires_at: Option<String>,
    #[serde(default)]
    pub checkout_url: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct BookingPaymentStatus {
    #[serde(default)]
    pub booking_id: String,
    #[serde(default)]
    pub booking_number: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub payment_status: String,
    #[serde(default)]
    pub confirmed: bool,
    #[serde(default)]
    pub payment_expires_at: Option<String>,
    #[serde(default)]
    pub obligations: Vec<AdminPaymentObligation>,
}

#[derive(Deserialize)]
pub struct BookingsResponse {
    pub bookings: Vec<Booking>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct AdminBooking {
    #[serde(default)]
    pub booking_id: String,
    #[serde(default)]
    pub booking_number: String,
    #[serde(default)]
    pub rental_slug: String,
    #[serde(default)]
    pub rental_name: String,
    #[serde(default)]
    pub guests: i32,
    #[serde(default)]
    pub first_name: String,
    #[serde(default)]
    pub last_name: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub phone: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub payment_status: String,
    #[serde(default)]
    pub starts_at: String,
    #[serde(default)]
    pub ends_at: String,
    #[serde(default = "default_currency")]
    pub currency: String,
    #[serde(default)]
    pub total: String,
    #[serde(default)]
    pub amount_due_now: String,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub admin_notes: String,
    #[serde(default)]
    pub balance_due_at: Option<String>,
    #[serde(default)]
    pub payment_expires_at: Option<String>,
    #[serde(default)]
    pub payment_obligations: Vec<AdminPaymentObligation>,
    #[serde(default)]
    pub timeline: Vec<AdminTimelineEvent>,
}

fn default_currency() -> String {
    "CAD".into()
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct PaymentConfig {
    #[serde(default, alias = "payments_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub mode: String,
    #[serde(default, deserialize_with = "deserialize_nullable_string")]
    pub publishable_key: String,
    #[serde(default)]
    pub account_id: String,
}

pub const EXPECTED_STRIPE_ACCOUNT_ID: &str = "acct_1SpY7K2MR4C4rvKM";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaymentAvailability {
    Loading,
    Disabled,
    TestReady,
    Blocked,
}

pub fn payment_availability(
    config: Option<&PaymentConfig>,
    configuration_failed: bool,
) -> PaymentAvailability {
    if configuration_failed {
        return PaymentAvailability::Blocked;
    }
    let Some(config) = config else {
        return PaymentAvailability::Loading;
    };
    if !config.enabled {
        return PaymentAvailability::Disabled;
    }
    if config.mode == "test"
        && config.publishable_key.starts_with("pk_test_")
        && config.account_id == EXPECTED_STRIPE_ACCOUNT_ID
    {
        PaymentAvailability::TestReady
    } else {
        PaymentAvailability::Blocked
    }
}

fn deserialize_nullable_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(Option::<String>::deserialize(deserializer)?.unwrap_or_default())
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct AdminDashboard {
    #[serde(default)]
    pub active_bookings: i64,
    #[serde(default, alias = "pending_payment")]
    pub pending_payments: i64,
    #[serde(default, alias = "payment_failures")]
    pub payment_errors: i64,
    #[serde(default, alias = "deliveries_today")]
    pub upcoming_deliveries: i64,
    #[serde(default)]
    pub overdue_actions: i64,
    #[serde(default)]
    pub attention: Vec<AdminAttentionItem>,
    #[serde(default)]
    pub today: Vec<AdminScheduleItem>,
    #[serde(default)]
    pub confirmed: i64,
    #[serde(default)]
    pub returns_today: i64,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct AdminAttentionItem {
    #[serde(default, alias = "kind")]
    pub item_type: String,
    #[serde(default)]
    pub title: String,
    #[serde(default, alias = "message")]
    pub detail: String,
    #[serde(default)]
    pub booking_id: Option<String>,
    #[serde(default)]
    pub severity: String,
    #[serde(default)]
    pub booking_number: String,
    #[serde(default)]
    pub due_at: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct AdminScheduleItem {
    #[serde(default)]
    pub booking_id: String,
    #[serde(default)]
    pub booking_number: String,
    #[serde(default)]
    pub rental_name: String,
    #[serde(default)]
    pub customer_name: String,
    #[serde(default)]
    pub action: String,
    #[serde(default)]
    pub scheduled_at: String,
    #[serde(default)]
    pub first_name: String,
    #[serde(default)]
    pub last_name: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub starts_at: String,
    #[serde(default)]
    pub ends_at: String,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct AdminPaymentObligation {
    #[serde(default, alias = "id", alias = "obligation_id")]
    pub payment_obligation_id: String,
    #[serde(default)]
    pub payment_id: Option<String>,
    #[serde(default)]
    pub booking_id: String,
    #[serde(default)]
    pub booking_number: String,
    #[serde(default)]
    pub customer_name: String,
    #[serde(default, alias = "obligation_type")]
    pub payment_type: String,
    #[serde(default)]
    pub status: String,
    #[serde(default = "default_currency")]
    pub currency: String,
    #[serde(default)]
    pub amount: String,
    #[serde(default)]
    pub amount_paid: String,
    #[serde(default)]
    pub amount_capturable: String,
    #[serde(default)]
    pub amount_authorized: String,
    #[serde(default)]
    pub amount_captured: String,
    #[serde(default)]
    pub amount_refunded: String,
    #[serde(default)]
    pub due_at: Option<String>,
    #[serde(default)]
    pub capture_before: Option<String>,
    #[serde(default)]
    pub hosted_url: Option<String>,
    #[serde(default)]
    pub last_error: Option<String>,
    #[serde(default)]
    pub last_error_code: Option<String>,
    #[serde(default)]
    pub last_error_message: Option<String>,
    #[serde(default)]
    pub provider_reference: Option<String>,
    #[serde(default)]
    pub provider_object_type: Option<String>,
    #[serde(default)]
    pub extended_authorization_status: Option<String>,
    #[serde(default)]
    pub expires_at: Option<String>,
    #[serde(default)]
    pub attempt_count: i32,
    #[serde(default)]
    pub sequence_number: i32,
    #[serde(default)]
    pub last_provider_status: Option<String>,
    #[serde(default)]
    pub updated_at: String,
    #[serde(default)]
    pub financial_operation: bool,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct AdminFinancialOperation {
    #[serde(default)]
    pub operation_id: String,
    #[serde(default)]
    pub booking_id: String,
    #[serde(default)]
    pub obligation_id: Option<String>,
    #[serde(default)]
    pub payment_id: Option<String>,
    #[serde(default)]
    pub operation_type: String,
    #[serde(default)]
    pub sequence_number: i32,
    #[serde(default)]
    pub status: String,
    #[serde(default = "default_currency")]
    pub currency: String,
    #[serde(default)]
    pub amount: String,
    #[serde(default)]
    pub provider_reference: Option<String>,
    #[serde(default)]
    pub attempt_count: i32,
    #[serde(default)]
    pub last_provider_status: Option<String>,
    #[serde(default)]
    pub last_error_code: Option<String>,
    #[serde(default)]
    pub last_error_message: Option<String>,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub updated_at: String,
}

impl From<AdminFinancialOperation> for AdminPaymentObligation {
    fn from(operation: AdminFinancialOperation) -> Self {
        Self {
            payment_obligation_id: operation.operation_id,
            payment_id: operation.payment_id,
            booking_id: operation.booking_id,
            payment_type: operation.operation_type,
            status: operation.status,
            currency: operation.currency,
            amount: operation.amount,
            provider_reference: operation.provider_reference,
            attempt_count: operation.attempt_count,
            sequence_number: operation.sequence_number,
            last_provider_status: operation.last_provider_status,
            last_error_code: operation.last_error_code,
            last_error_message: operation.last_error_message,
            updated_at: operation.updated_at,
            financial_operation: true,
            ..Self::default()
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct AdminBookingDetail {
    pub booking: AdminBooking,
    #[serde(default)]
    pub admin_notes: String,
    #[serde(default)]
    pub payment_expires_at: Option<String>,
    #[serde(default)]
    pub balance_due_at: Option<String>,
    #[serde(default)]
    pub delivered_at: Option<String>,
    #[serde(default)]
    pub returned_at: Option<String>,
    #[serde(default)]
    pub cancelled_at: Option<String>,
    #[serde(default)]
    pub cancellation_reason: Option<String>,
    #[serde(default, alias = "payment_obligations")]
    pub obligations: Vec<AdminPaymentObligation>,
    #[serde(default)]
    pub financial_operations: Vec<AdminFinancialOperation>,
    #[serde(default)]
    pub damage_claims: Vec<AdminDamageClaim>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct AdminDamageClaim {
    #[serde(default)]
    pub damage_claim_id: String,
    #[serde(default)]
    pub payment_id: String,
    #[serde(default)]
    pub claimed_amount: String,
    #[serde(default)]
    pub reason: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub submitted_at: Option<String>,
    #[serde(default)]
    pub captured_at: Option<String>,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub updated_at: String,
    #[serde(default)]
    pub evidence: Vec<AdminDamageEvidence>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct AdminDamageEvidence {
    #[serde(default)]
    pub evidence_id: String,
    #[serde(default)]
    pub damage_claim_id: String,
    #[serde(default)]
    pub original_filename: String,
    #[serde(default)]
    pub mime_type: String,
    #[serde(default)]
    pub byte_size: i64,
    #[serde(default)]
    pub created_at: String,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct AdminDamageEvidenceAccess {
    #[serde(default)]
    pub evidence_id: String,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub expires_at: String,
    #[serde(default)]
    pub access_token: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct AdminProviderCleanup {
    #[serde(default)]
    pub object_type: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub attention_required: bool,
    #[serde(default)]
    pub message: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct AdminActionResult {
    pub booking: AdminBookingDetail,
    #[serde(default)]
    pub provider_status: Option<String>,
    #[serde(default)]
    pub provider_cleanup: Vec<AdminProviderCleanup>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct AdminTimelineEvent {
    #[serde(default)]
    pub event_type: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub detail: String,
    #[serde(default)]
    pub created_at: String,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct AdminAuditEvent {
    #[serde(default, alias = "id")]
    pub audit_event_id: String,
    #[serde(default)]
    pub action: String,
    #[serde(default)]
    pub actor_email: String,
    #[serde(default)]
    pub booking_id: Option<String>,
    #[serde(default)]
    pub booking_number: Option<String>,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub actor_user_id: Option<String>,
    #[serde(default)]
    pub entity_type: String,
    #[serde(default)]
    pub entity_id: String,
    #[serde(default)]
    pub reason: String,
    #[serde(default)]
    pub request_id: String,
}

#[derive(Deserialize)]
struct AdminBookingsResponse {
    bookings: Vec<AdminBooking>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum AdminDashboardResponse {
    Wrapped { dashboard: AdminDashboard },
    Direct(AdminDashboard),
}

#[derive(Deserialize)]
#[serde(untagged)]
enum AdminBookingDetailResponse {
    Wrapped { booking: AdminBookingDetail },
    Detail(AdminBookingDetail),
    BasicWrapped { booking: AdminBooking },
    Direct(AdminBooking),
}

#[derive(Deserialize)]
struct AdminPaymentsResponse {
    #[serde(default, alias = "payments")]
    payment_obligations: Vec<AdminPaymentObligation>,
    #[serde(default)]
    financial_operations: Vec<AdminFinancialOperation>,
}

#[derive(Deserialize)]
struct AdminAuditEventsResponse {
    #[serde(default, alias = "events")]
    audit_events: Vec<AdminAuditEvent>,
}

#[derive(Serialize)]
struct ManualBookingPayload<'a> {
    request_id: &'a str,
    rental_slug: &'a str,
    starts_on: &'a str,
    ends_on: &'a str,
    guests: i32,
    first_name: &'a str,
    last_name: &'a str,
    email: &'a str,
    phone: &'a str,
    delivery_address: &'a str,
    notes: Option<&'a str>,
    admin_notes: &'a str,
}

#[derive(Serialize)]
struct CustomerUpdatePayload<'a> {
    request_id: &'a str,
    first_name: &'a str,
    last_name: &'a str,
    email: &'a str,
    phone: &'a str,
}

#[derive(Serialize)]
struct NotesUpdatePayload<'a> {
    request_id: &'a str,
    admin_notes: &'a str,
}

#[derive(Serialize)]
struct AdminActionPayload<'a> {
    request_id: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    refund_amount: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    amount: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    evidence_ids: Option<&'a [String]>,
}

#[derive(Deserialize)]
struct PaymentConfigResponse {
    #[serde(default)]
    payment: Option<PaymentConfig>,
    #[serde(flatten)]
    config: PaymentConfig,
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

pub async fn payment_config() -> Result<PaymentConfig, ApiError> {
    let response = Request::get(&format!("{API_BASE}/api/v1/payments/config"))
        .send()
        .await
        .map_err(|error| ApiError::client(error.to_string()))?;
    if !response.ok() {
        return Err(response_error(response).await);
    }
    response
        .json::<PaymentConfigResponse>()
        .await
        .map(|value| value.payment.unwrap_or(value.config))
        .map_err(|error| ApiError::client(error.to_string()))
}

pub async fn booking_status(
    booking_id: &str,
    booking_token: &str,
) -> Result<BookingPaymentStatus, ApiError> {
    let response = Request::get(&format!(
        "{API_BASE}/api/v1/bookings/{}/payment-status",
        urlencoding::encode(booking_id)
    ))
    .header("x-booking-token", booking_token)
    .send()
    .await
    .map_err(|error| ApiError::client(error.to_string()))?;
    if !response.ok() {
        return Err(response_error(response).await);
    }
    response
        .json::<BookingPaymentStatus>()
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

fn admin_access_token() -> Result<String, ApiError> {
    access_token().ok_or_else(|| ApiError {
        status: 401,
        code: "unauthorized".into(),
        message: "Admin sign-in is required".into(),
    })
}

pub async fn admin_dashboard() -> Result<AdminDashboard, ApiError> {
    let token = admin_access_token()?;
    let response = Request::get(&format!("{API_BASE}/api/v1/admin/dashboard"))
        .header("Authorization", &format!("Bearer {token}"))
        .send()
        .await
        .map_err(|error| ApiError::client(error.to_string()))?;
    if !response.ok() {
        return Err(response_error(response).await);
    }
    response
        .json::<AdminDashboardResponse>()
        .await
        .map(|value| match value {
            AdminDashboardResponse::Wrapped { dashboard }
            | AdminDashboardResponse::Direct(dashboard) => dashboard,
        })
        .map_err(|error| ApiError::client(error.to_string()))
}

pub async fn admin_booking(booking_id: &str) -> Result<AdminBookingDetail, ApiError> {
    let token = admin_access_token()?;
    let response = Request::get(&format!(
        "{API_BASE}/api/v1/admin/bookings/{}",
        urlencoding::encode(booking_id)
    ))
    .header("Authorization", &format!("Bearer {token}"))
    .send()
    .await
    .map_err(|error| ApiError::client(error.to_string()))?;
    if !response.ok() {
        return Err(response_error(response).await);
    }
    response
        .json::<AdminBookingDetailResponse>()
        .await
        .map(|value| match value {
            AdminBookingDetailResponse::Wrapped { booking } => booking,
            AdminBookingDetailResponse::Detail(detail) => detail,
            AdminBookingDetailResponse::BasicWrapped { booking }
            | AdminBookingDetailResponse::Direct(booking) => AdminBookingDetail {
                booking,
                ..AdminBookingDetail::default()
            },
        })
        .map_err(|error| ApiError::client(error.to_string()))
}

pub async fn admin_payments() -> Result<Vec<AdminPaymentObligation>, ApiError> {
    let token = admin_access_token()?;
    let response = Request::get(&format!("{API_BASE}/api/v1/admin/payments?limit=500"))
        .header("Authorization", &format!("Bearer {token}"))
        .send()
        .await
        .map_err(|error| ApiError::client(error.to_string()))?;
    if !response.ok() {
        return Err(response_error(response).await);
    }
    response
        .json::<AdminPaymentsResponse>()
        .await
        .map(|value| {
            let mut rows = value.payment_obligations;
            rows.extend(
                value
                    .financial_operations
                    .into_iter()
                    .map(AdminPaymentObligation::from),
            );
            rows
        })
        .map_err(|error| ApiError::client(error.to_string()))
}

pub async fn admin_audit_events() -> Result<Vec<AdminAuditEvent>, ApiError> {
    let token = admin_access_token()?;
    let response = Request::get(&format!("{API_BASE}/api/v1/admin/audit-events?limit=500"))
        .header("Authorization", &format!("Bearer {token}"))
        .send()
        .await
        .map_err(|error| ApiError::client(error.to_string()))?;
    if !response.ok() {
        return Err(response_error(response).await);
    }
    response
        .json::<AdminAuditEventsResponse>()
        .await
        .map(|value| value.audit_events)
        .map_err(|error| ApiError::client(error.to_string()))
}

pub async fn admin_audit_csv() -> Result<String, ApiError> {
    let token = admin_access_token()?;
    let response = Request::get(&format!("{API_BASE}/api/v1/admin/audit-events.csv"))
        .header("Authorization", &format!("Bearer {token}"))
        .send()
        .await
        .map_err(|error| ApiError::client(error.to_string()))?;
    if !response.ok() {
        return Err(response_error(response).await);
    }
    response
        .text()
        .await
        .map_err(|error| ApiError::client(error.to_string()))
}

pub fn download_csv(filename: &str, csv: &str) -> Result<(), String> {
    use wasm_bindgen::JsCast;

    let document = web_sys::window()
        .and_then(|window| window.document())
        .ok_or("This browser cannot start the CSV download")?;
    let link = document
        .create_element("a")
        .map_err(|_| "The CSV download link could not be created")?;
    link.set_attribute(
        "href",
        &format!("data:text/csv;charset=utf-8,{}", urlencoding::encode(csv)),
    )
    .map_err(|_| "The CSV download could not be prepared")?;
    link.set_attribute("download", filename)
        .map_err(|_| "The CSV filename could not be set")?;
    link.set_attribute("hidden", "")
        .map_err(|_| "The CSV download could not be prepared")?;
    let body = document
        .body()
        .ok_or("This page cannot start the CSV download")?;
    body.append_child(&link)
        .map_err(|_| "The CSV download could not be started")?;
    link.dyn_ref::<web_sys::HtmlElement>()
        .ok_or("The CSV download link is unavailable")?
        .click();
    link.remove();
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn create_manual_admin_booking(
    rental_slug: &str,
    starts_on: &str,
    ends_on: &str,
    guests: i32,
    first_name: &str,
    last_name: &str,
    email: &str,
    phone: &str,
    delivery_address: &str,
    notes: &str,
) -> Result<CreatedBooking, ApiError> {
    let token = admin_access_token()?;
    let request_id = uuid::Uuid::new_v4().to_string();
    let response = Request::post(&format!("{API_BASE}/api/v1/admin/bookings/manual"))
        .header("Content-Type", "application/json")
        .header("Authorization", &format!("Bearer {token}"))
        .json(&ManualBookingPayload {
            request_id: &request_id,
            rental_slug,
            starts_on,
            ends_on,
            guests,
            first_name,
            last_name,
            email,
            phone,
            delivery_address,
            notes: None,
            admin_notes: notes,
        })
        .map_err(|error| ApiError::client(error.to_string()))?
        .send()
        .await
        .map_err(|error| ApiError::client(error.to_string()))?;
    if !response.ok() {
        return Err(response_error(response).await);
    }
    response
        .json::<CreatedBooking>()
        .await
        .map_err(|error| ApiError::client(error.to_string()))
}

pub async fn update_admin_booking_customer(
    booking_id: &str,
    first_name: &str,
    last_name: &str,
    email: &str,
    phone: &str,
) -> Result<AdminBookingDetail, ApiError> {
    let token = admin_access_token()?;
    let request_id = uuid::Uuid::new_v4().to_string();
    let response = Request::patch(&format!(
        "{API_BASE}/api/v1/admin/bookings/{}/customer",
        urlencoding::encode(booking_id)
    ))
    .header("Content-Type", "application/json")
    .header("Authorization", &format!("Bearer {token}"))
    .json(&CustomerUpdatePayload {
        request_id: &request_id,
        first_name,
        last_name,
        email,
        phone,
    })
    .map_err(|error| ApiError::client(error.to_string()))?
    .send()
    .await
    .map_err(|error| ApiError::client(error.to_string()))?;
    admin_booking_action_response(response).await
}

pub async fn update_admin_booking_notes(
    booking_id: &str,
    notes: &str,
) -> Result<AdminBookingDetail, ApiError> {
    let token = admin_access_token()?;
    let request_id = uuid::Uuid::new_v4().to_string();
    let response = Request::patch(&format!(
        "{API_BASE}/api/v1/admin/bookings/{}/notes",
        urlencoding::encode(booking_id)
    ))
    .header("Content-Type", "application/json")
    .header("Authorization", &format!("Bearer {token}"))
    .json(&NotesUpdatePayload {
        request_id: &request_id,
        admin_notes: notes,
    })
    .map_err(|error| ApiError::client(error.to_string()))?
    .send()
    .await
    .map_err(|error| ApiError::client(error.to_string()))?;
    admin_booking_action_response(response).await
}

async fn admin_booking_action_response(
    response: gloo_net::http::Response,
) -> Result<AdminBookingDetail, ApiError> {
    if !response.ok() {
        return Err(response_error(response).await);
    }
    response
        .json::<AdminBookingDetailResponse>()
        .await
        .map(|value| match value {
            AdminBookingDetailResponse::Wrapped { booking }
            | AdminBookingDetailResponse::Detail(booking) => booking,
            AdminBookingDetailResponse::BasicWrapped { booking }
            | AdminBookingDetailResponse::Direct(booking) => AdminBookingDetail {
                booking,
                ..AdminBookingDetail::default()
            },
        })
        .map_err(|error| ApiError::client(error.to_string()))
}

pub async fn admin_booking_action(
    booking_id: &str,
    action: &str,
    amount: Option<&str>,
    reason: Option<&str>,
    evidence_ids: Option<&[String]>,
) -> Result<AdminActionResult, ApiError> {
    let token = admin_access_token()?;
    let request_id = uuid::Uuid::new_v4().to_string();
    let path = match action {
        "delivered" => "mark-delivered",
        "returned" => "mark-returned",
        "cancel" => "cancel",
        "release" => "damage-deposit/refund",
        "capture" => "damage-deposit/settle",
        _ => return Err(ApiError::client("Unsupported admin action")),
    };
    let response = Request::post(&format!(
        "{API_BASE}/api/v1/admin/bookings/{}/{}",
        urlencoding::encode(booking_id),
        path
    ))
    .header("Content-Type", "application/json")
    .header("Authorization", &format!("Bearer {token}"))
    .json(&AdminActionPayload {
        request_id: &request_id,
        refund_amount: (action == "cancel").then_some(amount).flatten(),
        amount: (action == "capture").then_some(amount).flatten(),
        reason,
        evidence_ids,
    })
    .map_err(|error| ApiError::client(error.to_string()))?
    .send()
    .await
    .map_err(|error| ApiError::client(error.to_string()))?;
    if !response.ok() {
        return Err(response_error(response).await);
    }
    response
        .json::<AdminActionResult>()
        .await
        .map_err(|error| ApiError::client(error.to_string()))
}

pub async fn resend_admin_payment_link(obligation_id: &str) -> Result<(), ApiError> {
    let token = admin_access_token()?;
    let request_id = uuid::Uuid::new_v4().to_string();
    let response = Request::post(&format!(
        "{API_BASE}/api/v1/admin/payment-obligations/{}/resend-link",
        urlencoding::encode(obligation_id)
    ))
    .header("Content-Type", "application/json")
    .header("Authorization", &format!("Bearer {token}"))
    .json(&serde_json::json!({ "request_id": request_id }))
    .map_err(|error| ApiError::client(error.to_string()))?
    .send()
    .await
    .map_err(|error| ApiError::client(error.to_string()))?;
    if !response.ok() {
        return Err(response_error(response).await);
    }
    Ok(())
}

pub async fn refresh_admin_payment_status(
    payment_id: &str,
) -> Result<AdminPaymentObligation, ApiError> {
    let token = admin_access_token()?;
    let request_id = uuid::Uuid::new_v4().to_string();
    let response = Request::post(&format!(
        "{API_BASE}/api/v1/admin/payments/{}/refresh-status",
        urlencoding::encode(payment_id)
    ))
    .header("Content-Type", "application/json")
    .header("Authorization", &format!("Bearer {token}"))
    .json(&serde_json::json!({ "request_id": request_id }))
    .map_err(|error| ApiError::client(error.to_string()))?
    .send()
    .await
    .map_err(|error| ApiError::client(error.to_string()))?;
    if !response.ok() {
        return Err(response_error(response).await);
    }
    response
        .json::<AdminPaymentObligation>()
        .await
        .map_err(|error| ApiError::client(error.to_string()))
}

pub async fn upload_admin_damage_evidence(
    booking_id: &str,
    file: &web_sys::File,
) -> Result<String, ApiError> {
    let token = admin_access_token()?;
    let form = web_sys::FormData::new()
        .map_err(|_| ApiError::client("The evidence upload could not be prepared"))?;
    form.append_with_blob_and_filename("photo", file, &file.name())
        .map_err(|_| ApiError::client("The selected evidence photo could not be attached"))?;
    form.append_with_str("request_id", &uuid::Uuid::new_v4().to_string())
        .map_err(|_| ApiError::client("The evidence request could not be prepared"))?;
    let response = Request::post(&format!(
        "{API_BASE}/api/v1/admin/bookings/{}/damage-evidence",
        urlencoding::encode(booking_id)
    ))
    .header("Authorization", &format!("Bearer {token}"))
    .body(form)
    .map_err(|error| ApiError::client(error.to_string()))?
    .send()
    .await
    .map_err(|error| ApiError::client(error.to_string()))?;
    if !response.ok() {
        return Err(response_error(response).await);
    }
    #[derive(Deserialize)]
    struct EvidenceResponse {
        #[serde(default, alias = "id")]
        evidence_id: String,
    }
    let value = response
        .json::<EvidenceResponse>()
        .await
        .map_err(|error| ApiError::client(error.to_string()))?;
    if value.evidence_id.is_empty() {
        return Err(ApiError::client(
            "The evidence upload did not return an evidence ID",
        ));
    }
    Ok(value.evidence_id)
}

pub async fn admin_damage_evidence_access(
    evidence_id: &str,
) -> Result<AdminDamageEvidenceAccess, ApiError> {
    let token = admin_access_token()?;
    let response = Request::get(&format!(
        "{API_BASE}/api/v1/admin/damage-evidence/{}/access",
        urlencoding::encode(evidence_id)
    ))
    .header("Authorization", &format!("Bearer {token}"))
    .send()
    .await
    .map_err(|error| ApiError::client(error.to_string()))?;
    if !response.ok() {
        return Err(response_error(response).await);
    }
    let access = response
        .json::<AdminDamageEvidenceAccess>()
        .await
        .map_err(|error| ApiError::client(error.to_string()))?;
    if access.evidence_id.is_empty() || access.url.is_empty() {
        return Err(ApiError::client(
            "The private evidence access response was incomplete",
        ));
    }
    Ok(access)
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
    fn payment_config_accepts_flat_and_wrapped_test_mode_contracts() {
        let flat: PaymentConfigResponse = serde_json::from_str(
            r#"{"payments_enabled":true,"mode":"test","publishable_key":"pk_test_example","account_id":"acct_test"}"#,
        )
        .unwrap();
        let wrapped: PaymentConfigResponse = serde_json::from_str(
            r#"{"payment":{"enabled":false,"mode":"test","publishable_key":"","account_id":"acct_test"}}"#,
        )
        .unwrap();

        assert!(flat.payment.unwrap_or(flat.config).enabled);
        assert!(!wrapped.payment.unwrap_or(wrapped.config).enabled);
    }

    #[test]
    fn payment_availability_blocks_unexpected_mode_key_or_account() {
        let ready = PaymentConfig {
            enabled: true,
            mode: "test".into(),
            publishable_key: "pk_test_example".into(),
            account_id: EXPECTED_STRIPE_ACCOUNT_ID.into(),
        };
        assert_eq!(
            payment_availability(Some(&ready), false),
            PaymentAvailability::TestReady
        );

        for blocked in [
            PaymentConfig {
                mode: "live".into(),
                ..ready.clone()
            },
            PaymentConfig {
                publishable_key: "pk_live_example".into(),
                ..ready.clone()
            },
            PaymentConfig {
                account_id: "acct_unexpected".into(),
                ..ready.clone()
            },
        ] {
            assert_eq!(
                payment_availability(Some(&blocked), false),
                PaymentAvailability::Blocked
            );
        }
        assert_eq!(
            payment_availability(None, true),
            PaymentAvailability::Blocked
        );
    }

    #[test]
    fn disabled_payment_config_preserves_legacy_no_card_mode() {
        let disabled = PaymentConfig {
            enabled: false,
            mode: "test".into(),
            publishable_key: String::new(),
            account_id: EXPECTED_STRIPE_ACCOUNT_ID.into(),
        };

        assert_eq!(
            payment_availability(Some(&disabled), false),
            PaymentAvailability::Disabled
        );
        assert_eq!(
            payment_availability(None, false),
            PaymentAvailability::Loading
        );
    }

    #[test]
    fn pending_booking_accepts_embedded_checkout_fields() {
        let created: CreatedBooking = serde_json::from_str(
            r#"{"booking":{"booking_id":"booking-1","booking_number":"VL-1","status":"pending_payment","payment_status":"unpaid","starts_at":"2030-07-15T21:00:00Z","ends_at":"2030-07-18T18:00:00Z","currency":"CAD","total":"1000.00","amount_due_now":"300.00"},"access_token":"private-token","checkout_session_id":"cs_test_1","checkout_client_secret":"cs_test_secret_1","payment_expires_at":"2030-01-01T00:30:00Z"}"#,
        )
        .unwrap();

        assert_eq!(created.client_secret.as_deref(), Some("cs_test_secret_1"));
        assert_eq!(created.booking.status, "pending_payment");
    }

    #[test]
    fn manual_booking_keeps_hosted_checkout_and_notification_result() {
        let created: CreatedBooking = serde_json::from_str(
            r#"{"booking":{"booking_id":"booking-1","booking_number":"VL-1","status":"pending_payment","payment_status":"unpaid","starts_at":"2030-07-15T21:00:00Z","ends_at":"2030-07-18T18:00:00Z","currency":"CAD","total":"1000.00","amount_due_now":"300.00"},"access_token":"private-token","notification_email_sent":false,"checkout_session_id":"cs_test_1","checkout_url":"https://checkout.stripe.com/c/pay/cs_test_1","payment_expires_at":"2030-01-01T02:00:00Z"}"#,
        )
        .unwrap();

        assert!(!created.notification_email_sent);
        assert_eq!(
            created.checkout_url.as_deref(),
            Some("https://checkout.stripe.com/c/pay/cs_test_1")
        );
    }

    #[test]
    fn manual_booking_sends_empty_admin_notes_as_a_string() {
        let payload = ManualBookingPayload {
            request_id: "00000000-0000-0000-0000-000000000000",
            rental_slug: "jayco26",
            starts_on: "2030-07-15",
            ends_on: "2030-07-18",
            guests: 4,
            first_name: "Test",
            last_name: "Guest",
            email: "guest@example.com",
            phone: "2505550100",
            delivery_address: "Kelowna, BC",
            notes: None,
            admin_notes: "",
        };
        let value = serde_json::to_value(payload).unwrap();

        assert_eq!(value["admin_notes"], "");
        assert!(value["notes"].is_null());
    }

    #[test]
    fn admin_action_accepts_nested_detail_and_obligation_contract() {
        let response: AdminBookingDetailResponse = serde_json::from_str(
            r#"{"booking":{"booking":{"booking_id":"booking-1","booking_number":"VL-1","rental_slug":"jayco26","rental_name":"Jayco 26","guests":4,"first_name":"Test","last_name":"Guest","email":"guest@example.com","phone":"250-555-0100","status":"completed","payment_status":"paid","starts_at":"2030-07-15T21:00:00Z","ends_at":"2030-07-18T18:00:00Z","currency":"CAD","total":"1497.00","amount_due_now":"449.10","created_at":"2030-01-01T00:00:00Z"},"admin_notes":"Inspected","obligations":[{"obligation_id":"obligation-1","booking_id":"booking-1","obligation_type":"damage_hold","amount":"1000.00","currency":"CAD","status":"authorized","amount_authorized":"1000.00"}]},"provider_status":"requires_capture"}"#,
        )
        .unwrap();

        let AdminBookingDetailResponse::Wrapped { booking } = response else {
            panic!("expected wrapped booking detail");
        };
        assert_eq!(booking.obligations[0].payment_type, "damage_hold");
        assert_eq!(booking.admin_notes, "Inspected");
    }

    #[test]
    fn admin_action_preserves_provider_attention_from_a_success_response() {
        let response: AdminActionResult = serde_json::from_str(
            r#"{"booking":{"booking":{"booking_id":"booking-1","booking_number":"VL-1","rental_slug":"jayco26","rental_name":"Jayco 26","guests":4,"first_name":"Test","last_name":"Guest","email":"guest@example.com","phone":"250-555-0100","status":"cancelled","payment_status":"paid","starts_at":"2030-07-15T21:00:00Z","ends_at":"2030-07-18T18:00:00Z","currency":"CAD","total":"1497.00","amount_due_now":"449.10","created_at":"2030-01-01T00:00:00Z"},"obligations":[],"financial_operations":[]},"provider_status":"failed","provider_cleanup":[{"object_type":"invoice","provider_reference":"in_test","status":"failed","attention_required":true,"message":"Stripe invoice could not be voided"}]}"#,
        )
        .unwrap();

        assert_eq!(response.provider_status.as_deref(), Some("failed"));
        assert!(response.provider_cleanup[0].attention_required);
        assert_eq!(response.booking.booking.status, "cancelled");
    }

    #[test]
    fn admin_detail_accepts_private_damage_evidence_summaries_without_storage_fields() {
        let detail: AdminBookingDetail = serde_json::from_str(
            r#"{"booking":{"booking_id":"booking-1","booking_number":"VL-1","rental_slug":"jayco26","rental_name":"Jayco 26","guests":4,"first_name":"Test","last_name":"Guest","email":"guest@example.com","phone":"250-555-0100","status":"completed","payment_status":"paid","starts_at":"2030-07-15T21:00:00Z","ends_at":"2030-07-18T18:00:00Z","currency":"CAD","total":"1497.00","amount_due_now":"449.10","created_at":"2030-01-01T00:00:00Z"},"obligations":[],"financial_operations":[],"damage_claims":[{"damage_claim_id":"claim-1","payment_id":"payment-1","claimed_amount":"125.00","reason":"Awning damage","status":"submitted","submitted_at":"2030-07-19T18:00:00Z","captured_at":null,"created_at":"2030-07-19T17:00:00Z","updated_at":"2030-07-19T18:00:00Z","evidence":[{"evidence_id":"evidence-1","damage_claim_id":"claim-1","original_filename":"awning.webp","mime_type":"image/webp","byte_size":2048,"created_at":"2030-07-19T17:30:00Z"}]}]}"#,
        )
        .unwrap();

        assert_eq!(
            detail.damage_claims[0].evidence[0].evidence_id,
            "evidence-1"
        );
        assert_eq!(detail.damage_claims[0].claimed_amount, "125.00");
    }

    #[test]
    fn evidence_access_accepts_signed_and_tokenized_preview_contracts() {
        let signed: AdminDamageEvidenceAccess = serde_json::from_str(
            r#"{"evidence_id":"evidence-1","url":"https://storage.example/signed","expires_at":"2030-07-19T18:15:00Z"}"#,
        )
        .unwrap();
        let local: AdminDamageEvidenceAccess = serde_json::from_str(
            r#"{"evidence_id":"evidence-2","url":"https://api.example/api/v1/admin/damage-evidence/evidence-2/content","expires_at":"2030-07-19T18:15:00Z","access_token":"one-time-token"}"#,
        )
        .unwrap();

        assert!(signed.access_token.is_none());
        assert_eq!(local.access_token.as_deref(), Some("one-time-token"));
    }

    #[test]
    fn admin_payments_include_durable_refund_parts() {
        let response: AdminPaymentsResponse = serde_json::from_str(
            r#"{"payments":[],"financial_operations":[{"operation_id":"operation-1","booking_id":"booking-1","payment_id":"payment-1","operation_type":"refund","sequence_number":2,"status":"submitted","amount":"300.00","currency":"CAD","attempt_count":1,"last_provider_status":"pending","created_at":"2030-01-01T00:00:00Z","updated_at":"2030-01-01T00:00:01Z"}]}"#,
        )
        .unwrap();
        let operation = response.financial_operations.into_iter().next().unwrap();
        let row = AdminPaymentObligation::from(operation);

        assert!(row.financial_operation);
        assert_eq!(row.payment_type, "refund");
        assert_eq!(row.status, "submitted");
        assert_eq!(row.amount, "300.00");
        assert_eq!(row.sequence_number, 2);
        assert_eq!(row.last_provider_status.as_deref(), Some("pending"));
        assert_eq!(row.payment_id.as_deref(), Some("payment-1"));
    }

    #[test]
    fn payment_status_contract_is_webhook_backed() {
        let status: BookingPaymentStatus = serde_json::from_str(
            r#"{"booking_id":"booking-1","booking_number":"VL-1","status":"confirmed","payment_status":"paid","confirmed":true,"payment_expires_at":null,"obligations":[]}"#,
        )
        .unwrap();

        assert!(status.confirmed);
        assert_eq!(status.payment_status, "paid");
    }

    #[test]
    fn oauth_return_path_cannot_become_an_external_redirect() {
        assert_eq!(normalized_frontend_path("/account"), "/account");
        assert_eq!(normalized_frontend_path("//evil.example"), "/account");
        assert_eq!(normalized_frontend_path("https://evil.example"), "/account");
    }

    #[test]
    fn frontend_links_keep_the_github_pages_repository_prefix() {
        let base = "https://gaponovalexey.github.io/viktor_rv_front/";

        assert_eq!(
            frontend_path_for_base(base, "/#home-rentals"),
            "https://gaponovalexey.github.io/viktor_rv_front/#home-rentals"
        );
        assert_eq!(
            frontend_path_for_base(base, "/admin"),
            "https://gaponovalexey.github.io/viktor_rv_front/admin"
        );
    }

    #[test]
    fn github_pages_runtime_cannot_fall_back_to_the_production_domain() {
        assert_eq!(
            github_pages_base(
                "https://gaponovalexey.github.io",
                "gaponovalexey.github.io",
                "/viktor_rv_front/admin"
            )
            .as_deref(),
            Some("https://gaponovalexey.github.io/viktor_rv_front")
        );
        assert_eq!(
            github_pages_base("https://vlrental.ca", "vlrental.ca", "/admin"),
            None
        );
    }

    #[test]
    fn ui_source_does_not_bypass_the_router_base_with_root_absolute_links() {
        fn rust_files(path: &std::path::Path, files: &mut Vec<std::path::PathBuf>) {
            for entry in std::fs::read_dir(path).unwrap() {
                let entry = entry.unwrap();
                let path = entry.path();
                if path.is_dir() {
                    rust_files(&path, files);
                } else if path.extension().and_then(|value| value.to_str()) == Some("rs") {
                    files.push(path);
                }
            }
        }

        let source_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut files = Vec::new();
        rust_files(&source_root, &mut files);
        let forbidden = ["href:", " \"/"].concat();
        let offenders = files
            .into_iter()
            .filter_map(|path| {
                let source = std::fs::read_to_string(&path).unwrap();
                source.contains(&forbidden).then(|| {
                    path.strip_prefix(&source_root)
                        .unwrap()
                        .display()
                        .to_string()
                })
            })
            .collect::<Vec<_>>();

        assert!(
            offenders.is_empty(),
            "root-absolute UI links bypass the configured site base: {}",
            offenders.join(", ")
        );
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

    #[test]
    fn booking_auth_continuation_preserves_the_overlay_without_credentials() {
        let continuation = BookingAuthContinuation {
            draft: TripDraft {
                rental_slug: "jayco26".into(),
                starts_on: "2030-08-16".into(),
                ends_on: "2030-08-19".into(),
                guests: 4,
                addon_keys: vec!["portable_bbq".into(), "bedding".into()],
                delivery_km: Some("22.0".into()),
                delivery_address: Some("Bear Creek Provincial Park".into()),
                attending_event: false,
                towing_after_delivery: false,
            },
            location: "Kelowna, BC".into(),
            radius_km: 150,
            delivery_estimate: Some(DeliveryEstimate {
                resolved_address: "Bear Creek Provincial Park".into(),
                one_way_km: "22.0".into(),
                round_trip_km: "44.0".into(),
                delivery_fee: "150.00".into(),
                maximum_km: "150.0".into(),
                within_range: true,
            }),
            first_name: "Test".into(),
            last_name: "Guest".into(),
            booking_email: "guest@example.com".into(),
            phone: "2505550100".into(),
            notes: "Late arrival".into(),
            accepted_terms: true,
        };

        let serialized = serde_json::to_string(&continuation).unwrap();
        let restored: BookingAuthContinuation = serde_json::from_str(&serialized).unwrap();

        assert_eq!(restored, continuation);
        assert!(!serialized.contains("password"));
    }
}
