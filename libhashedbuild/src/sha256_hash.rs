use sha2::{Digest, Sha256};

pub trait Sha256Hash {
    /// Hashes the value into the provided Sha256 state deterministically and infallibly.
    fn sha256_hash(&self, hasher: &mut Sha256);

    fn hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        self.sha256_hash(&mut hasher);
        hasher.finalize().into()
    }
}
