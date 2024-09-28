echo "Building tarball"

rm -rf ./dist/ &> /dev/null
mkdir -p dist

cp -r ../frontend/out dist/static
cp ./ftp.py dist/ftp.py
cp ./robots.txt dist/robots.txt
cp ./setup.bash dist/setup.bash # Setup zentrox
cp ./install.bash dist/install.bash # Prepare the Zentrox setup
cp -r ./src dist/src
cp -r ./notes/ dist/notes
cp -r Cargo.toml dist/Cargo.toml

cd dist
tar -czvf zentrox.tar.gz *
