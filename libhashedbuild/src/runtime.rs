use std::{env, io, path::{Path, PathBuf}};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};

pub struct Runtime {
    source_dir: PathBuf,
    store_dir: PathBuf,
}

impl Runtime {
    pub fn start<P1: AsRef<Path>, P2: AsRef<Path>>(source: P1, store: P2) -> io::Result<Self> {
        env::set_current_dir(source.as_ref())?;
        Ok(Self {
            source_dir: source.as_ref().to_path_buf(),
            store_dir: store.as_ref().to_path_buf(),
        })
    }

    pub fn store_path(&self, hash: [u8; 32]) -> PathBuf {
        let base64 = URL_SAFE_NO_PAD.encode(hash);
        self.store_dir.join(base64)
    }
}
