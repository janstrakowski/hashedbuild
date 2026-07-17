use std::{cmp::Ordering, collections::BTreeMap, fs, io::Read, path::{Path, PathBuf}};

use sha2::{Digest, Sha256};

use crate::{runtime::Runtime, sha256_hash::Sha256Hash};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Value {
    Boolean(bool),
    Integer(i64),
    Float(u64),
    String(String),
    File(File),
    Map(Map),
    Function(Function),
}

#[derive(Debug, Clone)]
pub struct File {
    pub path: PathBuf,
    hash: [u8; 32],
}

// 2. Implement PartialEq and Eq based solely on the hash
impl PartialEq for File {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }
}

impl Eq for File {}

// 3. Implement Ord and PartialOrd based solely on the hash
impl Ord for File {
    fn cmp(&self, other: &Self) -> Ordering {
        self.hash.cmp(&other.hash)
    }
}

impl PartialOrd for File {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl File {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let path_buf = path.as_ref().to_path_buf();
        let mut hasher = Sha256::new();

        // Compute the I/O hash, swallowing errors to guarantee infallibility
        Self::compute_fs_hash(&path_buf, &mut hasher);

        Self {
            path: path_buf,
            hash: hasher.finalize().into(),
        }
    }

    /// Internal recursive function to compute the filesystem hash safely
    fn compute_fs_hash(path: &Path, hasher: &mut Sha256) {
        let metadata = match fs::symlink_metadata(path) {
            Ok(m) => m,
            Err(_) => {
                hasher.update(&[255]); // Fallback discriminator for errors
                return;
            }
        };

        let file_type = metadata.file_type();

        if file_type.is_file() {
            hasher.update(&[0]); // File discriminator
            hasher.update(&metadata.len().to_le_bytes());

            if let Ok(mut file) = fs::File::open(path) {
                let mut buffer = [0u8; 8192];
                while let Ok(bytes_read) = file.read(&mut buffer) {
                    if bytes_read == 0 { break; }
                    hasher.update(&buffer[..bytes_read]);
                }
            }
        } else if file_type.is_dir() {
            hasher.update(&[1]); // Directory discriminator

            if let Ok(entries) = fs::read_dir(path) {
                let mut paths: Vec<PathBuf> = entries
                    .filter_map(Result::ok)
                    .map(|e| e.path())
                    .collect();

                paths.sort(); // Deterministic sorting
                hasher.update(&paths.len().to_le_bytes());

                for p in paths {
                    Self::compute_fs_hash(&p, hasher);
                }
            } else {
                hasher.update(&0usize.to_le_bytes());
            }
        } else if file_type.is_symlink() {
            hasher.update(&[2]); // Symlink discriminator

            if let Ok(target) = fs::read_link(path) {
                let target_str = target.to_string_lossy();
                hasher.update(&target_str.len().to_le_bytes());
                hasher.update(target_str.as_bytes());
            }
        } else {
            hasher.update(&[3]); // Other discriminator
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Map {
    map: BTreeMap<Value, Value>,
}

impl Map {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn put(mut self, key: Value, val: Value) -> Self {
        self.map.insert(key, val);
        self
    }

    pub fn put_all(mut self, another: &Map) -> Self {
        self.map.extend(another.map.iter().map(|(k, v)| (k.clone(), v.clone())));
        self
    }

    pub fn get(&self, key: &Value) -> Option<&Value> {
        self.map.get(key)
    }
}

fn f2bit(f: f64) -> u64 {
    f.to_bits() as u64
}

fn bit2f(b: u64) -> f64 {
    f64::from_bits(b as u64)
}

fn ival(i: i64) -> Value {
    Value::Integer(i)
}

fn sval(s: impl Into<String>) -> Value {
    Value::String(s.into())
}

fn maybe_is(v: Value) -> Value {
    Value::Map(Map::new().put(sval("is"), v))
}

fn emptym() -> Value {
    Value::Map(Map::new())
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Function {
    LoadFile,
    WriteFile,
}

enum NumPair {
    Int(i64, i64),
    Float(u64, u64),
}

impl Function {
    pub fn call(&self, arg: &Value, runtime: &Runtime) -> Result<Value, String> {
        match self {
            Self::LoadFile => {
                let file = match arg {
                    Value::File(file) => file,
                    _ => return Err("Wrong value passed - expected File.".to_owned()),
                };
                Ok(Value::String(fs::read_to_string(&file.path)
                        .map_err(|err| format!("IO error: {err}."))?))
            }
            Self::WriteFile => {
                let string = match arg {
                    Value::String(str) => str,
                    _ => return Err("Wrong value passed - expected String.".to_owned()),
                };
                let path = runtime.store_path(arg.hash());
                fs::write(&path, string)
                    .map_err(|io_err| format!("IO error: {io_err}."))?;
                Ok(Value::File(File::new(path)))
            }
        }
    }
}

impl Sha256Hash for Value {
    fn sha256_hash(&self, hasher: &mut Sha256) {
        match self {
            Value::Boolean(b) => {
                hasher.update(&[0]);
                hasher.update(&[*b as u8]);
            }
            Value::Integer(i) => {
                hasher.update(&[1]);
                hasher.update(i.to_le_bytes());
            }
            Value::Float(f) => {
                hasher.update(&[2]);
                hasher.update(f.to_le_bytes());
            }
            Value::String(s) => {
                hasher.update(&[3]);
                hasher.update(s.as_bytes());
            }
            Value::File(f) => {
                hasher.update(&[4]);
                f.sha256_hash(hasher);
            }
            Value::Map(m) => {
                hasher.update(&[5]);
                m.sha256_hash(hasher);
            }
            Value::Function(f) => {
                hasher.update(&[6]);
                f.sha256_hash(hasher);
            }
        }
    }
}

impl Sha256Hash for File {
    fn sha256_hash(&self, hasher: &mut Sha256) {
        // Since we calculated this on ::new(), we just quickly feed it to the state
        hasher.update(&self.hash);
    }
}

impl Sha256Hash for Map {
    fn sha256_hash(&self, hasher: &mut Sha256) {
        // Because BTreeMap is ordered, iteration is naturally deterministic
        hasher.update(self.map.len().to_le_bytes());
        for (k, v) in &self.map {
            k.sha256_hash(hasher);
            v.sha256_hash(hasher);
        }
    }
}

impl Sha256Hash for Function {
    fn sha256_hash(&self, hasher: &mut Sha256) {
        match self {
            Function::LoadFile => hasher.update(&[0]),
            Function::WriteFile => hasher.update(&[1]),
        }
    }
}
