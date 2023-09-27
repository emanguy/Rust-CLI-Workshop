use std::env;
use thiserror::Error;

/// Configuration options for the CLI
pub struct Configuration {
    pub get_outline_info: GetOutlineInfo,
}

/// Config options relating to GetOutline itself
pub struct GetOutlineInfo {
    pub api_key: String,
}

/// Error returned when a required environment variable isn't present
#[derive(Error, Debug)]
#[error("Required environment variable missing: {var_name}")]
pub struct MissingEnvVarError {
    var_name: String,
}

/// Creates a [MissingEnvVarError] for the given environment variable
fn variable_was_required(var_name: &str) -> MissingEnvVarError {
    MissingEnvVarError {
        var_name: String::from(var_name),
    }
}

/// Attempt to read an environment variable, returning a domain error if it's not present
fn read_env_var(var_name: &str) -> Result<String, MissingEnvVarError> {
    env::var(var_name).map_err(|_| variable_was_required(var_name))
}

/// Read configuration from environment variables
pub fn parse_from_env() -> Result<Configuration, MissingEnvVarError> {
    Ok(Configuration {
        get_outline_info: GetOutlineInfo {
            api_key: read_env_var("GETOUTLINE_API_KEY")?,
        },
    })
}
