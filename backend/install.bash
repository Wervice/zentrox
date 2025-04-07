# Use this file to install Zentrox
# Do not use setup.bash instead

mkdir -p ~/.local/bin/zentrox
cp -r ./assets/* ~/.local/bin/zentrox
chmod +x ~/.local/bin/zentrox/zentrox
export ZENTROX_MODE=NORMAL
~/.local/bin/zentrox/zentrox
