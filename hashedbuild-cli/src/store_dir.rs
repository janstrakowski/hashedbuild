use std::path::PathBuf;

use thiserror::Error;
use crate::paths;

pub(crate) fn ensure() -> Result<PathBuf, EnsureError> {
    let path = paths::store()
        .map_err(EnsureError::Path)?;
    match path.try_exists() {
        Ok(true) => return Ok(path),
        Ok(false) => {},
        Err(err) => return Err(EnsureError::CheckExists(path, err)),
    }
    match inquire::Confirm::new(&format!("The store path ({}) does not exists. Would you like to create it?",
            path.to_string_lossy()))
        .with_default(true)
        .prompt() {
            Ok(true) => {},
            Ok(false) => return Err(EnsureError::NotExists(path)),
            Err(err) => return Err(EnsureError::Prompt(path, err)),
        }
    if let Err(err) = std::fs::create_dir_all(&path) {
        return Err(EnsureError::Create(path, err));
    }
    Ok(path)
}

#[derive(Debug, Error)]
pub enum EnsureError {
    #[error("{0}")]
    Path(paths::DirObtainError),
    #[error("Could not create the {0} directory: {1}.")]
    Create(PathBuf, std::io::Error),
    #[error("The store directory ({0}) does not exist.")]
    NotExists(PathBuf),
    #[error("Could not prompt the user whether to create the store directory ({0}): {1}.")]
    Prompt(PathBuf, inquire::InquireError),
    #[error("Could not check whether the store path ({0}) exists: {1}.")]
    CheckExists(PathBuf, std::io::Error)
}
