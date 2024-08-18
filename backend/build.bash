mkdir -p dist
rm -rf dist/static &> /dev/null
cp -r ../frontend/out dist/static
cargo build
cp target/debug/backend dist/zentrox
