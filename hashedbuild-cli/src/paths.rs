use std::path::PathBuf;

use thiserror::Error;

use crate::env;
use crate::env::STORE;

fn xdg_data() -> Result<PathBuf, DirObtainError> {
    let xdg_data = if let Some(d) = dirs::data_dir() { d } else {
        return Err(DirObtainError::XdgData);
    };
    Ok(xdg_data)
}

pub fn store() -> Result<PathBuf, DirObtainError> {
    match std::env::var(env::STORE) {
        Ok(v) => return Ok(v.into()),
        Err(std::env::VarError::NotUnicode(_)) => return Err(DirObtainError::NotUnicode),
        Err(std::env::VarError::NotPresent) => {},
    }

    Ok(xdg_data()?.join("hashedbuild/store"))
}

#[derive(Debug, Error)]
pub enum DirObtainError {
    #[error("XDG_DATA_HOME directory path have been inaccesible.")]
    XdgData,
    #[error("{STORE} environment variable contains an invalid Unicode character.")]
    NotUnicode,
}
