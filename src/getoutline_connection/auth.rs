use crate::getoutline_connection::{APIError, DataEnvelope};
use reqwest::blocking::Client as BlockingClient;
use serde::Deserialize;

/// Information about the currently authenticated user
#[derive(Deserialize)]
pub struct UserInfo {
    pub id: String,
    pub name: String,
}

/// Contains authentication information about the requester
#[derive(Deserialize)]
pub struct AuthInfo {
    pub user: UserInfo,
}

/// Retrieve information about the currently authenticated user
pub fn current(client: &BlockingClient) -> Result<AuthInfo, APIError> {
    let http_response = client
        .post("https://app.getoutline.com/api/auth.info")
        .send()
        .map_err(|err| APIError::failed_trying_to("get authentication data (send failure)", err))?
        .error_for_status()
        .map_err(|err| APIError::failed_trying_to("get authentication data (bad status)", err))?;

    let auth_info_envelope: DataEnvelope<AuthInfo> = http_response
        .json()
        .map_err(|err| APIError::failed_trying_to("parse authentication data", err))?;

    Ok(auth_info_envelope.data)
}
