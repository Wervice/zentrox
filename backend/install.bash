# Use this file to install Zentrox
# Do not use setup.bash instead

echo "Installing Zentrox"
echo "This may take a while."
EXE_DIR=$HOME/.local/bin/zentrox
mkdir -p $EXE_DIR &> /dev/null
cargo build --release
cp ./target/release/zentrox ~$EXE_DIR/zentrox
cp ./ftp.py $EXE_DIR/ftp.py
cp ./setup.bash $EXE_DIR/setup.bash
cp ./robots.txt $EXE_DIR/robots.txt
cp ./manifest.json $EXE_DIR/manifest.json
cp -r static/ $EXE_DIR/static
rm -rf ./*

echo "✅ Finished Zentrox installation"
