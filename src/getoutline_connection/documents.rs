use reqwest::blocking::Client as BlockingClient;
use serde::{Deserialize, Serialize};
use crate::getoutline_connection::{APIError, DataEnvelope};

/// Request for a list of documents, including pagination information
#[derive(Serialize)]
pub struct ListRequest {
    /// Pagination offset from the beginning of the results
    pub offset: u32,
    /// Number of results to return per page
    pub limit: u32,
}

impl Default for ListRequest {
    fn default() -> Self {
        ListRequest {
            offset: 0,
            limit: 15,
        }
    }
}

/// A listed document entry, used when trying to find documents that currently exist in GetOutline
#[derive(Deserialize)]
pub struct Listed {
    /// The unique ID of the document
    pub id: String,
    /// The document's title, as seen in GetOutline
    pub title: String,
}

/// Fetch a list of documents available to the current user in GetOutline
pub fn list(client: &BlockingClient, request: &ListRequest) -> Result<Vec<Listed>, APIError> {
    let http_response = client.post("https://app.getoutline.com/api/documents.list")
        .json(request)
        .send()
        .map_err(|err| APIError::failed_trying_to("list documents in GetOutline (send failure)", err))?
        .error_for_status()
        .map_err(|err| APIError::failed_trying_to("list documents in GetOutline (bad status)", err))?;

    let document_envelope: DataEnvelope<Vec<Listed>> = http_response.json().map_err(|err| APIError::failed_trying_to("read list of documents in GetOutline", err))?;

    Ok(document_envelope.data)
}
