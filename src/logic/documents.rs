use crate::logic::auth::{AuthReader, AuthRetrieveError};
use mockall::automock;
use thiserror::Error;

/// Options for listing a page of documents
pub struct ListOptions {
    pub page: u32,
    pub results_per_page: u32,
    pub own_documents_only: bool,
}

impl Default for ListOptions {
    fn default() -> Self {
        ListOptions {
            page: 0,
            results_per_page: 15,
            own_documents_only: false,
        }
    }
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
    BadCredentials,
    #[error("Other adapter error occurred: {0}")]
    AdapterError(anyhow::Error),
}

/// Error produced by an implementer of [DocumentReader] which explains why fetching a single
/// document went awry
#[derive(Error, Debug)]
pub enum DocRetrieveError {
    #[error("Reader credentials did not work")]
    BadCredentials,
    #[error("Requested document did not exist")]
    DocumentNotFound,
    #[error("Other adapter error occurred: {0}")]
    AdapterError(anyhow::Error),
}

/// Options for listing documents for a [DocumentReader]
pub struct ReaderListOptions {
    pub offset: u32,
    pub limit: u32,
    pub user: Option<String>,
}

/// Raw markdown content of a document
pub struct DocContent {
    pub id: String,
    pub title: String,
    pub text: String,
}

/// Something that can read sets of GetOutline documents
#[automock]
pub trait DocumentReader {
    /// List documents in GetOutline
    fn list(&self, list_opts: &ReaderListOptions) -> Result<Vec<ListEntry>, ReaderListError>;

    /// Get a specific document from GetOutline
    fn retrieve_one(&self, document_id: &str) -> Result<DocContent, DocRetrieveError>;
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
            ReaderListError::BadCredentials => ListError::BadCredentials,
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
        assert_that!(&result).is_ok();
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
        assert_that!(&result).is_ok();
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

        assert_that!(&result)
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

        assert_that!(&result)
            .is_err()
            .matches(|err| matches!(err, ListError::CouldNotListDocuments(_)));
    }
}

/// DocumentSaver adapter error reporting a save failure
#[derive(Error, Debug)]
pub enum SaveError {
    #[error("Something with the name \"{name}\" already exists.")]
    TargetWithSameNameExists { name: String },
    #[error("Unexpected error happened when saving document: {0}")]
    AdapterError(anyhow::Error),
}

/// Something that can save documents from GetOutline
#[automock]
pub trait DocumentSaver {
    /// Save the [content] of a document in a place referenced by [name]
    fn save_document(&self, content: &str, name: &str) -> Result<(), SaveError>;
}

/// Options for retrieving a document with [retrieve]
pub struct RetrieveOptions<'refs> {
    pub suggested_name: Option<&'refs str>,
}

#[allow(clippy::derivable_impls)]
impl<'refs> Default for RetrieveOptions<'refs> {
    fn default() -> Self {
        RetrieveOptions {
            suggested_name: None,
        }
    }
}

/// Error representing things going wrong when retrieving and saving a document
#[derive(Error, Debug)]
pub enum RetrieveError {
    #[error("There is not a document with the id {requested_id}")]
    DocumentDoesNotExist { requested_id: String },
    #[error("The GetOutline credentials provided did not work. Please provide new ones.")]
    BadAuth,
    #[error("Failed to retrieve the requested document from GetOutline: {0}")]
    DocumentRetrieveFailed(anyhow::Error),
    #[error(
        "Could not save the requested document, something with the name \"{name}\" already exists!"
    )]
    SameNameCouldNotSave { name: String },
    #[error("Failed to save the file from GetOutline: {0}")]
    DocumentSaveFailed(anyhow::Error),
}

/// Retrieve a document from GetOutline and save it. If a name is provided in the [options] without
/// the ".md" file extension, it will be appended automatically. If a name suggestion isn't provided,
/// the name of the document in GetOutline will be used.
pub fn retrieve(
    reader: &impl DocumentReader,
    saver: &impl DocumentSaver,
    document_id: &str,
    options: &RetrieveOptions,
) -> Result<(), RetrieveError> {
    let getout_document_result = reader.retrieve_one(document_id);
    let getout_document = match getout_document_result {
        Ok(document) => document,
        Err(DocRetrieveError::DocumentNotFound) => {
            return Err(RetrieveError::DocumentDoesNotExist {
                requested_id: document_id.to_string(),
            })
        }
        Err(DocRetrieveError::BadCredentials) => return Err(RetrieveError::BadAuth),
        Err(DocRetrieveError::AdapterError(cause)) => {
            return Err(RetrieveError::DocumentRetrieveFailed(cause))
        }
    };

    let mut doc_name = options
        .suggested_name
        .unwrap_or(getout_document.title.as_str())
        .to_string();
    if !doc_name.to_lowercase().ends_with(".md") {
        doc_name += ".md";
    }

    let save_result = saver.save_document(&getout_document.text, &doc_name);
    if let Err(error) = save_result {
        return match error {
            SaveError::TargetWithSameNameExists { name } => {
                Err(RetrieveError::SameNameCouldNotSave { name })
            }
            SaveError::AdapterError(cause) => Err(RetrieveError::DocumentSaveFailed(cause)),
        };
    }

    Ok(())
}

#[cfg(test)]
mod retrieve_tests {
    use super::*;
    use anyhow::anyhow;
    use mockall::predicate;
    use speculoos::assert_that;
    use speculoos::prelude::*;

    fn retrieved_document() -> DocContent {
        DocContent {
            id: "12345".to_string(),
            title: "My document".to_string(),
            text: "Hello world!".to_string(),
        }
    }

    #[test]
    fn happy_path_no_rename() {
        let mut mock_reader = MockDocumentReader::new();
        let mut mock_saver = MockDocumentSaver::new();

        mock_reader
            .expect_retrieve_one()
            .with(predicate::eq("abc123"))
            .returning(|_| Ok(retrieved_document()));
        mock_saver
            .expect_save_document()
            .with(
                predicate::eq("Hello world!"),
                predicate::eq("My document.md"),
            )
            .returning(|_, _| Ok(()));

        let retrieve_result = retrieve(
            &mock_reader,
            &mock_saver,
            "abc123",
            &RetrieveOptions::default(),
        );

        assert_that!(retrieve_result).is_ok();
    }

    #[test]
    fn happy_path_with_rename_no_file_ext() {
        let mut mock_reader = MockDocumentReader::new();
        let mut mock_saver = MockDocumentSaver::new();
        let retrieve_opts = RetrieveOptions {
            suggested_name: Some("New Name"),
        };

        mock_reader
            .expect_retrieve_one()
            .with(predicate::eq("abc123"))
            .returning(|_| Ok(retrieved_document()));
        mock_saver
            .expect_save_document()
            .with(predicate::eq("Hello world!"), predicate::eq("New Name.md"))
            .returning(|_, _| Ok(()));

        let retrieve_result = retrieve(&mock_reader, &mock_saver, "abc123", &retrieve_opts);

        assert_that!(retrieve_result).is_ok();
    }

    #[test]
    fn happy_path_with_file_ext() {
        let mut mock_reader = MockDocumentReader::new();
        let mut mock_saver = MockDocumentSaver::new();
        let retrieve_opts = RetrieveOptions {
            suggested_name: Some("New Name.Md"),
        };

        mock_reader
            .expect_retrieve_one()
            .with(predicate::eq("abc123"))
            .returning(|_| Ok(retrieved_document()));
        mock_saver
            .expect_save_document()
            .with(predicate::eq("Hello world!"), predicate::eq("New Name.Md"))
            .returning(|_, _| Ok(()));

        let retrieve_result = retrieve(&mock_reader, &mock_saver, "abc123", &retrieve_opts);

        assert_that!(retrieve_result).is_ok();
    }

    #[test]
    fn fails_on_bad_auth() {
        let mut mock_reader = MockDocumentReader::new();
        let mock_saver = MockDocumentSaver::new();

        mock_reader
            .expect_retrieve_one()
            .with(predicate::eq("abc123"))
            .returning(|_| Err(DocRetrieveError::BadCredentials));

        let retrieve_result = retrieve(
            &mock_reader,
            &mock_saver,
            "abc123",
            &RetrieveOptions::default(),
        );

        assert_that!(retrieve_result)
            .is_err()
            .matches(|error| matches!(error, RetrieveError::BadAuth));
    }

    #[test]
    fn fails_when_document_doesnt_exist() {
        let mut mock_reader = MockDocumentReader::new();
        let mock_saver = MockDocumentSaver::new();

        mock_reader
            .expect_retrieve_one()
            .with(predicate::eq("abc123"))
            .returning(|_| Err(DocRetrieveError::DocumentNotFound));

        let retrieve_result = retrieve(
            &mock_reader,
            &mock_saver,
            "abc123",
            &RetrieveOptions::default(),
        );

        assert_that!(retrieve_result)
            .is_err()
            .matches(|error| matches!(error, RetrieveError::DocumentDoesNotExist { requested_id } if requested_id == "abc123"));
    }

    #[test]
    fn fails_when_document_retrieve_fails() {
        let mut mock_reader = MockDocumentReader::new();
        let mock_saver = MockDocumentSaver::new();

        mock_reader
            .expect_retrieve_one()
            .with(predicate::eq("abc123"))
            .returning(|_| Err(DocRetrieveError::AdapterError(anyhow!("Whoopsie!"))));

        let retrieve_result = retrieve(
            &mock_reader,
            &mock_saver,
            "abc123",
            &RetrieveOptions::default(),
        );

        assert_that!(retrieve_result)
            .is_err()
            .matches(|error| matches!(error, RetrieveError::DocumentRetrieveFailed(_)));
    }

    #[test]
    fn fails_when_save_fails() {
        let mut mock_reader = MockDocumentReader::new();
        let mut mock_saver = MockDocumentSaver::new();

        mock_reader
            .expect_retrieve_one()
            .with(predicate::eq("abc123"))
            .returning(|_| Ok(retrieved_document()));
        mock_saver
            .expect_save_document()
            .with(
                predicate::eq("Hello world!"),
                predicate::eq("My document.md"),
            )
            .returning(|_, _| Err(SaveError::AdapterError(anyhow!("Whoopsie!"))));

        let retrieve_result = retrieve(
            &mock_reader,
            &mock_saver,
            "abc123",
            &RetrieveOptions::default(),
        );

        assert_that!(retrieve_result)
            .is_err()
            .matches(|error| matches!(error, RetrieveError::DocumentSaveFailed(_)));
    }

    #[test]
    fn fails_on_duplicate_save_name() {
        let mut mock_reader = MockDocumentReader::new();
        let mut mock_saver = MockDocumentSaver::new();

        mock_reader
            .expect_retrieve_one()
            .with(predicate::eq("abc123"))
            .returning(|_| Ok(retrieved_document()));
        mock_saver
            .expect_save_document()
            .with(
                predicate::eq("Hello world!"),
                predicate::eq("My document.md"),
            )
            .returning(|_, _| {
                Err(SaveError::TargetWithSameNameExists {
                    name: "My ducument.md".to_string(),
                })
            });

        let retrieve_result = retrieve(
            &mock_reader,
            &mock_saver,
            "abc123",
            &RetrieveOptions::default(),
        );

        assert_that!(retrieve_result).is_err().matches(|error| matches!(error, RetrieveError::SameNameCouldNotSave { name } if name == "My document.md"));
    }
}
