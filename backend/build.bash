rm -rf ./dist/
mkdir -p dist
rm -rf dist/static &> /dev/null
cp -r ../frontend/out dist/static
cp ./ftp.py dist/ftp.py
cargo build --release
cp target/debug/zentrox dist/zentrox
cp ./install.bash dist/install.bash
