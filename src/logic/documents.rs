use crate::logic::auth::{AuthReader, AuthRetrieveError};
use mockall::automock;
use thiserror::Error;

/// Options for listing a page of documents
pub struct ListOptions {
    page: u32,
    results_per_page: u32,
    own_documents_only: bool,
}

/// Entry in the document list
pub struct ListEntry {
    pub id: String,
    pub title: String,
}

/// Error produced by an implementor of [DocumentReader] when trying to fetch a list of documents
/// from the adapter
#[derive(Error, Debug)]
pub enum ReaderListError {
    #[error("Reader credentials did not work")]
    BadCredentialsError,
    #[error("Other adapter error occurred: {0}")]
    AdapterError(anyhow::Error),
}

/// Options for listing documents for a [DocumentReader]
pub struct ReaderListOptions {
    pub offset: u32,
    pub limit: u32,
    pub user: Option<String>,
}

/// Something that can read sets of GetOutline documents
#[automock]
pub trait DocumentReader {
    /// List documents in GetOutline
    fn list(&self, list_opts: &ReaderListOptions) -> Result<Vec<ListEntry>, ReaderListError>;
}

/// Errors that occur when trying to fulfill the "list" business logic
#[derive(Debug, Error)]
pub enum ListError {
    #[error("GetOutline credentials were invalid")]
    BadCredentials,
    #[error("Could not read information about the current user")]
    CouldNotGetAuth(anyhow::Error),
    #[error("Could not list available documents from GetOutline")]
    CouldNotListDocuments(anyhow::Error),
}

/// List GetOutline documents
pub fn list(
    auth_reader: &impl AuthReader,
    doc_reader: &impl DocumentReader,
    list_opts: &ListOptions,
) -> Result<(), ListError> {
    let user = if list_opts.own_documents_only {
        let auth_info = auth_reader.current().map_err(|err| match err {
            AuthRetrieveError::AdapterError(cause) => {
                ListError::CouldNotGetAuth(cause.context(
                    "Tried to read authentication information while fetching document list",
                ))
            }
        })?;

        Some(auth_info.user)
    } else {
        None
    };

    let reader_list_opts = ReaderListOptions {
        offset: list_opts.page * list_opts.results_per_page,
        limit: list_opts.results_per_page,
        user: user.map(|it| it.id),
    };

    let documents = doc_reader
        .list(&reader_list_opts)
        .map_err(|err| match err {
            ReaderListError::BadCredentialsError => ListError::BadCredentials,
            ReaderListError::AdapterError(err) => ListError::CouldNotListDocuments(
                err.context("Fetching the list of documents failed"),
            ),
        })?;

    if documents.is_empty() {
        println!("No documents found!");
        return Ok(());
    }

    println!("Retrieved documents (page {}):", list_opts.page);

    for document in documents.iter() {
        println!("\t- \"{}\"\t (ID {})", document.title, document.id);
    }

    Ok(())
}

#[cfg(test)]
mod list_tests {
    use super::*;
    use crate::logic::auth::{AuthInfo, MockAuthReader, UserInfo};
    use anyhow::anyhow;
    use speculoos::prelude::*;

    fn sample_authinfo() -> AuthInfo {
        AuthInfo {
            user: UserInfo {
                id: String::from("abc-def-ghi"),
                name: String::from("John Doe"),
            },
        }
    }

    fn sample_documents() -> Vec<ListEntry> {
        vec![
            ListEntry {
                id: String::from("abc-def-ghi"),
                title: String::from("Shopping list"),
            },
            ListEntry {
                id: String::from("ghi-jkl-mno"),
                title: String::from("Wish list"),
            },
        ]
    }

    fn default_list_opts() -> ListOptions {
        ListOptions {
            page: 0,
            results_per_page: 15,
            own_documents_only: false,
        }
    }

    #[test]
    fn list_happy_path_all_documents() {
        let auth_reader = MockAuthReader::new();
        let mut doc_reader = MockDocumentReader::new();

        doc_reader
            .expect_list()
            .returning(|_| Ok(sample_documents()));

        let result = list(&auth_reader, &doc_reader, &default_list_opts());
        assert_that(&result).is_ok();
    }

    #[test]
    fn list_happy_path_user_auth_only() {
        let mut auth_reader = MockAuthReader::new();
        let mut doc_reader = MockDocumentReader::new();

        auth_reader
            .expect_current()
            .returning(|| Ok(sample_authinfo()));
        doc_reader
            .expect_list()
            .withf(|list_opts: &ReaderListOptions| list_opts.user.is_some())
            .returning(|_| Ok(sample_documents()));

        let result = list(
            &auth_reader,
            &doc_reader,
            &ListOptions {
                own_documents_only: true,
                ..default_list_opts()
            },
        );
        assert_that(&result).is_ok();
    }

    #[test]
    fn reports_auth_error_if_it_occurs() {
        let mut auth_reader = MockAuthReader::new();
        let doc_reader = MockDocumentReader::new();

        auth_reader.expect_current().returning(|| {
            Err(AuthRetrieveError::AdapterError(anyhow!(
                "Something blew up!"
            )))
        });

        let result = list(
            &auth_reader,
            &doc_reader,
            &ListOptions {
                own_documents_only: true,
                ..default_list_opts()
            },
        );

        assert_that(&result)
            .is_err()
            .matches(|err| matches!(err, ListError::CouldNotGetAuth(_)));
    }

    #[test]
    fn reports_doc_retrieve_error_if_it_occurs() {
        let auth_reader = MockAuthReader::new();
        let mut doc_reader = MockDocumentReader::new();

        doc_reader
            .expect_list()
            .returning(|_| Err(ReaderListError::AdapterError(anyhow!("Something blew up!"))));

        let result = list(&auth_reader, &doc_reader, &default_list_opts());

        assert_that(&result)
            .is_err()
            .matches(|err| matches!(err, ListError::CouldNotListDocuments(_)));
    }
}
