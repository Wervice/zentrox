#!/bin/bash

# by Wervice (Constantin Volke)
# wervice@proton.me 
# github.com/wervice/zentrox | Please 🌟 on GitHub

echo -e '\033[34m'
echo "███████╗███████╗███╗   ██╗████████╗██████╗  ██████╗ ██╗  ██╗"
echo "╚══███╔╝██╔════╝████╗  ██║╚══██╔══╝██╔══██╗██╔═══██╗╚██╗██╔╝"
echo "  ███╔╝ █████╗  ██╔██╗ ██║   ██║   ██████╔╝██║   ██║ ╚███╔╝ "
echo " ███╔╝  ██╔══╝  ██║╚██╗██║   ██║   ██╔══██╗██║   ██║ ██╔██╗ "
echo "███████╗███████╗██║ ╚████║   ██║   ██║  ██║╚██████╔╝██╔╝ ██╗ "
echo "╚══════╝╚══════╝╚═╝  ╚═══╝   ╚═╝   ╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝"
echo -e '\033[0m'   

echo "Zentrox Installer | by Wervice | github.com/wervice/zentrox"
echo ""
echo "This script will guide you through the process of installing and configuring zentrox."
echo ""

if [ "$EUID" -ne 0 ]; then
  echo "Please run as root"
  exit 1
fi

$ACTUAL_USERNAME=""

echo "🤵 Please enter your linux username (not root): "
read ACTUAL_USERNAME

echo "🤵 Please enter your zentrox admin username (e.g. johndoe)"
read ADMIN_USERNAME

echo "🔑 Please enter your zentrox admin password"
read -s ADMIN_PASSWORD

echo "🥏 Please enter a name for your zentrox server (e.g. glorious_server)"
read ZENTROX_SERVER_NAME

USERNAME_PATH="/home/$ACTUAL_USERNAME"

if [ -d $USERNAME_PATH ]; then
	echo "✅ Using $USERNAME_PATH/zentrox and $USERNAME_PATH/zentrox_data to install and run zentrox"
	ZENTROX_PATH="$USERNAME_PATH/zentrox"
	ZENTROX_DATA_PATH="$USERNAME_PATH/zentrox_data"
else
	echo "❌ Please enter your correct username or make sure, this username is used for your /home directory."
	exit -1
fi

mkdir $ZENTROX_DATA_PATH

if [[ $ZENTROX_PATH == "/" || $ZENTROX_PATH == $HOME || $ZENTROX_PATH == "$HOME/" ]] ; then
	echo "⚠️  Critical problem detected: $ZENTROX_PATH equal to protected folder"
fi

echo "❓ Remove (rm -rf) $ZENTROX_PATH to make sure no old versions of Zentrox are left [Y/n]"

read

if [[ $REPLY != "Y" ]]; then
	echo "Program stopped"
	exit 0
fi

rm -rf $ZENTROX_PATH

echo "🔽 Cloning Zentrox to $ZENTROX_PATH"

git clone https://github.com/Wervice/zentrox/ $ZENTROX_PATH # Clones Codelink repo to current folder

cd $ZENTROX_PATH # Got to zentrox_server folder

echo "✅ Download finished"

if ! command npm -v &> /dev/null
then
	echo "❌ Node Package Manager (npm) is not installed. To fix this issue, please install nodejs."
	echo "❌ The command to do this may look like this:"
	echo "❌ -E sudo apt install node OR sudo apt install nodejs"
	exit -1
fi

echo "✅ NPM can be used"

if ! command pip3 -v &> /dev/null
then
	echo "❌ Python Package Manager (pip3) is not installed. To fix this issue, please install python3."
	echo "❌ The command to do this may look like this:"
	echo -E "❌ sudo apt install python3"
	exit -1
fi

echo "⌛ Installing dependencies"
npm -q install express body-parser cookie-parser express-session node-os-utils ejs compression || echo "❌ Installing NPM packages failed"

echo "✅ Installed NPM packages"

sudo pip3 -H -q install pyftpdlib PyOpenSSL || echo "❌ Installing Python packages failed"

echo "✅ Installed Python packages"

echo "⌛ Compiling C programs"
gcc ./libs/crypt_c.c -o ./libs/crypt_c -lcrypt || echo "❌ Compiling using gcc failed"

echo "🔑 Generating selfsigned keys"

echo "ℹ️  In the following, you will be asked to enter some information to generate the keys."

echo ""

openssl genrsa -out selfsigned.key 2048
openssl req -new -key selfsigned.key -out selfsigned.pem
openssl x509 -req -days 365 -in selfsigned.pem -signkey selfsigned.key -out selfsigned.crt

echo ""

echo "🔑 Creating PEM file from key and crt"
cat selfsigned.crt selfsigned.key > selfsigned.pem

echo "✅ Generated .key, .crt and .pem file"

echo "🤵 Creating user 'zentrox'"

useradd -m -s /bin/bash "zentrox"
usermod -aG root "zentrox"
USER_PASSWORD=$(openssl rand -base64 48)
echo "zentrox:$USER_PASSWORD"
echo $(echo $USER_PASSWORD | openssl aes-256-cbc -a -salt -pass pass:$ADMIN_PASSWORD) > "$ZENTROX_DATA_PATH/zentrox_user_password.txt" 

sudo chown $ZENTROX_PATH/* $ACTUAL_USERNAME
sudo chown $ZENTROX_PATH/* $ACTUAL_USERNAME

echo "📁 Creating file structure"
touch "$ZENTROX_DATA_PATH/admin.txt"
touch "$ZENTROX_DATA_PATH/custom.txt"
touch "$ZENTROX_DATA_PATH/regMode.txt"
touch "$ZENTROX_DATA_PATH/setupDone.txt"
touch "$ZENTROX_DATA_PATH/users.txt"
mkdir "$ZENTROX_DATA_PATH/users"

echo "$ADMIN_USERNAME" > "$ZENTROX_DATA_PATH/admin.txt"
echo "$ZENTROX_SERVER_NAME\ndark" > "$ZENTROX_DATA_PATH/custom.txt"

echo -e "The installer has downloaded Zentrox and created a fresh set of self-signed certificates.\nThese will be used to protect your connections from hackers.\nIn addition to that several new NPM packages and Python (pip3) packages were downloaded.\n \n You can exit this change log by pressing [q] at any time. In the following you'll get a full explanaition of what the installer did.\n1. The installer clones wervice/zentrox into ~/zentrox\n2. The installer uses NPM to download these packages\n   express body-parser cookie-parser express-session node-os-utils ejs compression\n3. The installer used pip to globally install pyftpd and PyOpenSSL\n4. The installer uses OpenSSL to generate a three files: .key, .crt and .pem\n5. The installer will start Zentrox as soon as you press q\n\nIf you have any further questions, please visit the repo: https://github.com/wervice/zentrox"

echo "Starting Zentrox on port 3000"
node index.js 
