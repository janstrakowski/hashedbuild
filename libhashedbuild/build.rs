use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let package_name = env::var("CARGO_PKG_NAME").unwrap();
    let include_dir = PathBuf::from(&crate_dir).join("include");
    fs::create_dir_all(&include_dir).expect("Unable to create \"include\" directory");
    let output_file = include_dir.join(format!("{}.h", package_name));

    cbindgen::Builder::new()
        .with_crate(crate_dir)
        .with_language(cbindgen::Language::C)
        .generate()
        .expect("Unable to generate the C bindings")
        .write_to_file(output_file);
}
