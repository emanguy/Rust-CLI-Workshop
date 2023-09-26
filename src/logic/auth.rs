use mockall::automock;
use thiserror::Error;

/// Errors that can occur when an [AuthReader] adapter tries to fetch the current authentication information
#[derive(Debug, Error)]
pub enum AuthRetrieveError {
    #[error("Other adapter error occurred: {0}")]
    AdapterError(anyhow::Error),
}

/// Information about the user accessing GetOutline
pub struct UserInfo {
    pub id: String,
    pub name: String,
}

/// Contains authentication information about the person accessing GetOutline
pub struct AuthInfo {
    pub user: UserInfo,
}

/// Something that can read authentication information from GetOutline
#[automock]
pub trait AuthReader {
    /// Retrieve information about the currently authenticated user
    fn current(&self) -> Result<AuthInfo, AuthRetrieveError>;
}
