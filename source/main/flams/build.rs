fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    // Fix for ONNX on github CI, MacOS:
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        let out = std::process::Command::new("xcrun")
            .args(["clang", "--print-resource-dir"])
            .output()
            .expect("failed to run `xcrun clang --print-resource-dir`");
        let dir = String::from_utf8(out.stdout)
            .expect("invalid output from `xcrun clang --print-resource-dir`");
        println!("cargo:rustc-link-search=native={}/lib/darwin", dir.trim());
        println!("cargo:rustc-link-lib=static=clang_rt.osx");
    }
}
