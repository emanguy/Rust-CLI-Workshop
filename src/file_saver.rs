use crate::logic::documents::{DocumentSaver, SaveError};
use std::fs::{File};
use std::io::{BufWriter, Write};
use std::path::PathBuf;

/// Document saver which saves documents to the user's hard drive
pub struct FileDocumentSaver;

impl DocumentSaver for FileDocumentSaver {
    fn save_document(&self, content: &str, name: &str) -> Result<(), SaveError> {
        let target_path = PathBuf::from(".").join(name);
        if target_path.exists() {
            return Err(SaveError::TargetWithSameNameExists { name: name.to_string() });
        }

        let file = File::create(&target_path)
            .map_err(|err| {
                SaveError::AdapterError(
                    anyhow::Error::new(err).context("failed opening file to save document"),
                )
            })?;
        let mut writer = BufWriter::new(file);
        writer.write_all(content.as_bytes()).map_err(|err| {
            SaveError::AdapterError(anyhow::Error::new(err).context("failed writing bytes to file"))
        })?;
        writer.flush().map_err(|err| {
            SaveError::AdapterError(anyhow::Error::new(err).context("failed completion of write"))
        })?;

        Ok(())
    }
}
