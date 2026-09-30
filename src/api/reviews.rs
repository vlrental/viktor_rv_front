use super::{admin_authorized_response, response_error, ApiError, API_BASE};
use gloo_net::http::Request;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GoogleReview {
    pub name: String,
    pub body: String,
    pub language: String,
    pub url: String,
    pub rating: u8,
    pub published: bool,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GoogleContent {
    pub rating: String,
    pub count: i64,
    pub checked_on: String,
    pub profile_url: String,
    pub write_url: String,
    pub published: bool,
    pub reviews: Vec<GoogleReview>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GoogleDocument {
    pub content: GoogleContent,
    pub revision: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExternalReview {
    pub rental_slug: String,
    pub source: String,
    pub source_review_id: String,
    pub source_url: String,
    pub reviewer_name: String,
    pub rating: String,
    pub title: String,
    pub body: String,
    pub reviewed_at: String,
    pub reviewed_at_label: String,
    pub is_published: bool,
    pub revision: Option<String>,
}
pub async fn public_google() -> Result<Option<GoogleContent>, ApiError> {
    let response = Request::get(&format!("{API_BASE}/api/v1/google-reviews"))
        .send()
        .await
        .map_err(|e| ApiError::client(e.to_string()))?;
    if !response.ok() {
        return Err(response_error(response).await);
    }
    response
        .json()
        .await
        .map_err(|e| ApiError::client(e.to_string()))
}
pub async fn admin_get<T: DeserializeOwned>(path: &str) -> Result<T, ApiError> {
    let url = format!("{API_BASE}/api/v1/admin/{path}");
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
    response
        .json()
        .await
        .map_err(|e| ApiError::client(e.to_string()))
}
pub async fn admin_save<T: Serialize, R: DeserializeOwned>(
    path: &str,
    value: &T,
) -> Result<R, ApiError> {
    let url = format!("{API_BASE}/api/v1/admin/{path}");
    let body = serde_json::to_string(value).map_err(|e| ApiError::client(e.to_string()))?;
    let response = admin_authorized_response(move |token| {
        let url = url.clone();
        let body = body.clone();
        async move {
            Request::put(&url)
                .header("Authorization", &format!("Bearer {token}"))
                .header("Content-Type", "application/json")
                .body(body)
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
