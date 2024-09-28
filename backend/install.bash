# Use this file to install Zentrox
# Do not use setup.bash instead

echo "Installing Zentrox"
echo "This may take a while."
mkdir ~/zentrox &> /dev/null
cargo build --release
cp ./target/release/zentrox ~/zentrox/zentrox
cp ./ftp.py ~/zentrox/ftp.py
cp ./setup.bash ~/zentrox/setup.bash
cp ./robots.txt ~/zentrox/robots.txt
cp -r static/ ~/zentrox/static
rm -rf ./*

echo "✅ Finished Zentrox installation"
