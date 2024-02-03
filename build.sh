cd app
rm -rf *
cd ../server
cargo leptos build --release -vv
cp target/release/immt-server ../app/immt-server
cp -r target/site ../app/web
# cp Cargo.toml ../app/Cargo.toml