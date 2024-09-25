rm -rf ./dist/ &> /dev/null
mkdir -p dist
cp -r ../frontend/out dist/static
cp ./ftp.py dist/ftp.py
cp ./robots.txt dist/robots.txt
cargo build --release
cp target/release/zentrox dist/zentrox
cp ./install.bash dist/install.bash
