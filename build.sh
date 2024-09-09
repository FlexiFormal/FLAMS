rm -rf bin
mkdir bin
cd web/graphs
CARGO_TARGET_DIR=../../target RUSTFLAGS="--cfg=web_sys_unstable_apis" trunk  build --features=client --release
#cd ../ui
#cargo build --release --no-default-features --features=client --target wasm32-unknown-unknown
#cargo leptos build --release
#cd ../main
#cargo build --release --features=async
cd ../..
LEPTOS_WASM_OPT_VERSION=version_119 LEPTOS_SASS_VERSION=1.71.0 cargo leptos build --release
cp target/release/immt bin/immt
cp -r target/web bin/web
#uglifyjs --compress --mangle --output bin/web/pkg/immt.min.js -- bin/web/pkg/immt.js
#uglifyjs --compress --mangle --output bin/web/graphs/immt-graphs.min.js -- bin/web/graphs/immt-graphs.js
#mv bin/web/pkg/immt.min.js bin/web/immt.js
#mv bin/web/graphs/immt-graphs.min.js bin/web/graphs/immt-graphs.js
# cp Cargo.toml ../app/Cargo.toml