use super::{admin_authorized_response, response_error, ApiError, API_BASE};
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(
    inline_js = "export function saleImageUrl(bytes, mime) { return URL.createObjectURL(new Blob([bytes], {type:mime})); } export function revokeSaleImageUrl(url) { URL.revokeObjectURL(url); }"
)]
extern "C" {
    fn saleImageUrl(bytes: &[u8], mime: &str) -> String;
    pub fn revokeSaleImageUrl(url: &str);
}

pub async fn admin_photo_url(photo: &Photo) -> Result<String, ApiError> {
    let url = format!("{API_BASE}/api/v1/sale-photos/{}/content", photo.photo_id);
    let response = admin_authorized_response(move |token| {
        let url = url.clone();
        async move {
            Request::get(&url)
                .header("Authorization", &format!("Bearer {token}"))
                .send()
                .await
                .map_err(|e| ApiError::client(e.to_string()))
        }
    })
    .await?;
    if !response.ok() {
        return Err(response_error(response).await);
    }
    let mime = response
        .headers()
        .get("Content-Type")
        .unwrap_or_else(|| "image/jpeg".into());
    let bytes = response
        .binary()
        .await
        .map_err(|e| ApiError::client(e.to_string()))?;
    Ok(saleImageUrl(&bytes, &mime))
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct Photo {
    pub photo_id: String,
    pub source_url: String,
    pub alt_text: String,
    pub sort_order: i32,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct Listing {
    pub sale_id: String,
    pub title: String,
    pub model_year: Option<i32>,
    pub manufacturer: String,
    pub model: String,
    pub rv_type: String,
    pub length_ft: Option<String>,
    pub sleeps: Option<i32>,
    pub condition: String,
    pub location: String,
    pub price: Option<String>,
    pub summary: String,
    pub description: String,
    pub status: String,
    pub sort_order: i32,
    pub updated_at: String,
    pub photos: Vec<Photo>,
}

#[derive(Clone, PartialEq, Serialize)]
pub struct Payload {
    pub title: String,
    pub model_year: Option<i32>,
    pub manufacturer: String,
    pub model: String,
    pub rv_type: String,
    pub length_ft: Option<String>,
    pub sleeps: Option<i32>,
    pub condition: String,
    pub location: String,
    pub price: Option<String>,
    pub summary: String,
    pub description: String,
    pub status: String,
    pub sort_order: i32,
    pub expected_updated_at: Option<String>,
}

impl Default for Payload {
    fn default() -> Self {
        Self {
            title: String::new(),
            model_year: None,
            manufacturer: String::new(),
            model: String::new(),
            rv_type: "travel_trailer".into(),
            length_ft: None,
            sleeps: None,
            condition: "used".into(),
            location: "Kelowna, BC".into(),
            price: None,
            summary: String::new(),
            description: String::new(),
            status: "draft".into(),
            sort_order: 0,
            expected_updated_at: None,
        }
    }
}

impl From<&Listing> for Payload {
    fn from(v: &Listing) -> Self {
        Self {
            title: v.title.clone(),
            model_year: v.model_year,
            manufacturer: v.manufacturer.clone(),
            model: v.model.clone(),
            rv_type: v.rv_type.clone(),
            length_ft: v.length_ft.clone(),
            sleeps: v.sleeps,
            condition: v.condition.clone(),
            location: v.location.clone(),
            price: v.price.clone(),
            summary: v.summary.clone(),
            description: v.description.clone(),
            status: v.status.clone(),
            sort_order: v.sort_order,
            expected_updated_at: Some(v.updated_at.clone()),
        }
    }
}

pub async fn listings(admin: bool) -> Result<Vec<Listing>, ApiError> {
    let url = format!(
        "{API_BASE}/api/v1/{}rv-sales",
        if admin { "admin/" } else { "" }
    );
    let response = if admin {
        admin_authorized_response(move |token| {
            let url = url.clone();
            async move {
                Request::get(&url)
                    .header("Authorization", &format!("Bearer {token}"))
                    .send()
                    .await
                    .map_err(|e| ApiError::client(e.to_string()))
            }
        })
        .await?
    } else {
        Request::get(&url)
            .send()
            .await
            .map_err(|e| ApiError::client(e.to_string()))?
    };
    if !response.ok() {
        return Err(response_error(response).await);
    }
    response
        .json()
        .await
        .map_err(|e| ApiError::client(e.to_string()))
}

pub async fn save(id: &str, payload: &Payload) -> Result<Listing, ApiError> {
    mutate(
        &format!("/{id}"),
        "PUT",
        Some(serde_json::to_value(payload).map_err(|e| ApiError::client(e.to_string()))?),
    )
    .await
}

pub async fn deactivate(listing: &Listing) -> Result<Listing, ApiError> {
    let mut payload = Payload::from(listing);
    payload.status = "archived".into();
    save(&listing.sale_id, &payload).await
}

pub async fn edit_photo(id: &str, photo: &Photo, make_cover: bool) -> Result<Listing, ApiError> {
    mutate(
        &format!("/{id}/photos/{}", photo.photo_id),
        "PUT",
        Some(serde_json::json!({"alt_text":photo.alt_text,"make_cover":make_cover})),
    )
    .await
}

pub async fn delete_photo(id: &str, photo: &str) -> Result<Listing, ApiError> {
    mutate(&format!("/{id}/photos/{photo}"), "DELETE", None).await
}

async fn mutate(
    path: &str,
    method: &str,
    payload: Option<serde_json::Value>,
) -> Result<Listing, ApiError> {
    let url = format!("{API_BASE}/api/v1/admin/rv-sales{path}");
    let method = method.to_string();
    let req = uuid::Uuid::new_v4().to_string();
    let response = admin_authorized_response(move |token| {
        let url = url.clone();
        let method = method.clone();
        let req = req.clone();
        let payload = payload.clone();
        async move {
            let builder = if method == "DELETE" {
                Request::delete(&url)
            } else {
                Request::put(&url)
            };
            let builder = builder
                .header("Authorization", &format!("Bearer {token}"))
                .header("x-request-id", &req);
            let request = if let Some(payload) = payload {
                builder
                    .json(&payload)
                    .map_err(|e| ApiError::client(e.to_string()))?
            } else {
                builder
                    .build()
                    .map_err(|e| ApiError::client(e.to_string()))?
            };
            request
                .send()
                .await
                .map_err(|e| ApiError::client(e.to_string()))
        }
    })
    .await?;
    if !response.ok() {
        return Err(response_error(response).await);
    }
    response
        .json()
        .await
        .map_err(|e| ApiError::client(e.to_string()))
}

pub async fn upload(id: &str, file: &web_sys::File, alt: &str) -> Result<Listing, ApiError> {
    let url = format!("{API_BASE}/api/v1/admin/rv-sales/{id}/photos");
    let req = uuid::Uuid::new_v4().to_string();
    let file = file.clone();
    let alt = alt.to_string();
    let response = admin_authorized_response(move |token| {
        let url = url.clone();
        let req = req.clone();
        let file = file.clone();
        let alt = alt.clone();
        async move {
            let form = web_sys::FormData::new()
                .map_err(|_| ApiError::client("Photo upload could not be prepared"))?;
            form.append_with_blob_and_filename("photo", &file, &file.name())
                .map_err(|_| ApiError::client("Photo could not be attached"))?;
            form.append_with_str("alt_text", &alt)
                .map_err(|_| ApiError::client("Caption could not be attached"))?;
            Request::post(&url)
                .header("Authorization", &format!("Bearer {token}"))
                .header("x-request-id", &req)
                .body(form)
                .map_err(|e| ApiError::client(e.to_string()))?
                .send()
                .await
                .map_err(|e| ApiError::client(e.to_string()))
        }
    })
    .await?;
    if !response.ok() {
        return Err(response_error(response).await);
    }
    response
        .json()
        .await
        .map_err(|e| ApiError::client(e.to_string()))
}

pub fn price_label(price: Option<&str>) -> String {
    price
        .and_then(|v| v.parse::<f64>().ok())
        .map(|v| format!("CA${v:.2}"))
        .unwrap_or_else(|| "Enquire for price".into())
}
