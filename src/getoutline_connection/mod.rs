use anyhow::Context;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{blocking::Client as BlockingClient, header};

use serde::Deserialize;
use thiserror::Error;

pub mod auth;
pub mod documents;

/// General, inspectable error type which holds an HTTP request error
#[derive(Debug, Error)]
#[error("Failed to make request when trying to {action}")]
pub struct APIError {
    action: String,
    #[source]
    original_error: reqwest::Error,
}

impl APIError {
    /// Records an API error that occurred when we "failed trying to" complete some task
    fn failed_trying_to(action: &str, cause: reqwest::Error) -> APIError {
        APIError {
            action: action.to_string(),
            original_error: cause,
        }
    }
}

/// The current pagination status, served in response to an API request
#[derive(Deserialize)]
pub struct PaginationStatus {
    pub offset: u32,
    pub limit: u32,
}

/// The envelope containing the actual data returned from the GetOutline API
#[derive(Deserialize)]
pub struct DataEnvelope<T> {
    pub data: T,
}

/// Creates a blocking HTTP client which has the passed [auth_token] in the Authorization header
/// of every request by default
pub fn get_http_client(auth_token: &str) -> anyhow::Result<BlockingClient> {
    let mut default_headers = HeaderMap::new();
    let default_token: HeaderValue = format!("Bearer {}", auth_token)
        .parse()
        .with_context(|| format!("Could not add token {} as an HTTP header", auth_token))?;
    default_headers.insert(header::AUTHORIZATION, default_token);

    BlockingClient::builder()
        .default_headers(default_headers)
        .build()
        .with_context(|| String::from("Failed to construct HTTP client!"))
}
