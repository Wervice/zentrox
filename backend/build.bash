echo "Building tarball"

rm -rf ./dist/ &> /dev/null
mkdir -p dist

cp ./ftp.py dist/ftp.py
cp ./robots.txt dist/robots.txt
cp ./install.bash dist/install.bash # Prepare the Zentrox setup
cp ./manifest.json dist/manifest.json
cp ./empty_cover.svg dist/empty_cover.svg

cp -r ../frontend/out dist/static
cp -r ./src dist/src
cp -r ./notes/ dist/notes
cp -r Cargo.toml dist/Cargo.toml

cd dist
tar -czvf zentrox.tar.gz *
