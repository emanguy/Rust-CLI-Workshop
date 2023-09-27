use crate::getoutline_connection::{APIError, DataEnvelope};
use crate::logic;
use crate::logic::documents::{DocumentReader, ReaderListError, ReaderListOptions};
use reqwest::blocking::Client as BlockingClient;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

/// Request for a list of documents, including pagination information
#[derive(Serialize)]
pub struct ListRequest {
    /// Pagination offset from the beginning of the results
    pub offset: u32,
    /// Number of results to return per page
    pub limit: u32,
    /// Author of requested documents
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}

impl Default for ListRequest {
    fn default() -> Self {
        ListRequest {
            offset: 0,
            limit: 15,
            user: None,
        }
    }
}

impl From<&ReaderListOptions> for ListRequest {
    fn from(value: &ReaderListOptions) -> Self {
        ListRequest {
            offset: value.offset,
            limit: value.limit,
            user: value.user.clone(),
        }
    }
}

/// A listed document entry, used when trying to find documents that currently exist in GetOutline
#[derive(Deserialize)]
pub struct ListEntry {
    /// The unique ID of the document
    pub id: String,
    /// The document's title, as seen in GetOutline
    pub title: String,
}

impl From<ListEntry> for logic::documents::ListEntry {
    fn from(value: ListEntry) -> Self {
        logic::documents::ListEntry {
            id: value.id,
            title: value.title,
        }
    }
}

/// Fetch a list of documents available to the current user in GetOutline
pub fn list(client: &BlockingClient, request: &ListRequest) -> Result<Vec<ListEntry>, APIError> {
    let http_response = client
        .post("https://app.getoutline.com/api/documents.list")
        .json(request)
        .send()
        .map_err(|err| {
            APIError::failed_trying_to("list documents in GetOutline (send failure)", err)
        })?
        .error_for_status()
        .map_err(|err| {
            APIError::failed_trying_to("list documents in GetOutline (bad status)", err)
        })?;

    let document_envelope: DataEnvelope<Vec<ListEntry>> = http_response
        .json()
        .map_err(|err| APIError::failed_trying_to("read list of documents in GetOutline", err))?;

    Ok(document_envelope.data)
}

impl DocumentReader for BlockingClient {
    fn list(
        &self,
        list_opts: &ReaderListOptions,
    ) -> Result<Vec<logic::documents::ListEntry>, ReaderListError> {
        let request = ListRequest::from(list_opts);
        list(self, &request)
            .map(|results| {
                results
                    .into_iter()
                    .map(logic::documents::ListEntry::from)
                    .collect()
            })
            .map_err(|err| match err.original_error.status() {
                Some(StatusCode::UNAUTHORIZED) => ReaderListError::BadCredentialsError,
                _ => ReaderListError::AdapterError(anyhow::Error::new(err)),
            })
    }
}
