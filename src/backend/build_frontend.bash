cd ../frontend/
dx bundle --release
cd ../backend/
rm -rf ./static/
mkdir static
cp -r ../frontend/target/dx/frontend/release/web/public/* ./static
