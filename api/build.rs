fn main() {
    let version = rustc_version::version().expect("Failed to determine rustc version");
    println!("cargo:rustc-env=RUSTC_VERSION={version}");
}