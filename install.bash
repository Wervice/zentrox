#!/bin/bash

# by Wervice (Constantin Volke)
# wervice@proton.me 
# github.com/wervice/zentrox | Please 🌟 on GitHub

npm_failed() {
	echo -ne "❌ NPM failed while trying to install various NPM packages.\nDo you want to re-start the installation and ignore warnings? [y/N] "
	read
	if [[ $REPLY == "y" ]]; then
		npm -q install express body-parser cookie-parser express-session node-os-utils ejs compression
	else
		echo "Program stopped"
		exit -1
	fi
}

python_failed() {
	echo -ne "❌ The installer for Python modules failed. Do you want to ignore & restart pip3, use an alternative to pip3 or stop the program? [ignore/packageManager/N] "
	read
	if [[ $REPLY == "ignore" ]]; then
		sudo pip3 -q install pyftpdlib PyOpenSSL --break-system-packages &> /dev/null
	elif [[ $REPLY == "packageManager" ]]; then
		echo "❓ Please enter the name of your package manager [apt/dnf/pacman/zypper]"
		read PYTHON_PACKAGE_MANAGER
		if [[ $PYTHON_PACKAGE_MANAGER == "apt" ]]; then
			apt install python3-pyftpdlib python3-openssl -y &> /dev/null
		elif [[ $PYTHON_PACKAGE_MANAGER == "dnf" ]]; then
			dnf install python3-pyftpdlib python3-openssl -y &> /dev/null
		elif [[ $PYTHON_PACKAGE_MANAGER == "pacman" ]]; then
			pacman -Sy python3-pyftpdlib python3-openssl &> /dev/null
		elif [[ $PYTHON_PACKAGE_MANAGER == "zypper" ]]; then
			zypper -n install python3-pyftpdlib python3-openssl &> /dev/null
		else
			echo "❌ This package manager is not know to the installer.\nIt will attempt to run the command $PYTHON_PACKAGE_MANAGER install python3-pyopenssl python3-pyftpdlib -y"
			$PYTHON_PACKAGE_MANAGER install python3-pyopenssl python3-pyftpdlib -y
		fi
	else
		echo "Program stopped"
		exit -1
	fi
}

ufw_fail() {
	echo -ne "❌ The UFW is used to manage the firewall on your system.\nDo you want to install it now? It is required for the Firewall section under security.\nYou can also skip it for now, and install it later if you need it. [install/skip] "
	read
	if [[ $REPLY == "install" ]]; then
		echo "❓ Please enter the name of your package manager [apt/dnf/pacman/zypper]"
		read UFW_PACKAGE_MANAGER
		if [[ $UFW_PACKAGE_MANAGER == "apt" ]]; then
			apt install ufw -y &> /dev/null
		elif [[ $UFW_PACKAGE_MANAGER == "dnf" ]]; then
			dnf install ufw -y &> /dev/null
		elif [[ $UFW_PACKAGE_MANAGER == "pacman" ]]; then
			pacman -Sy ufw &> /dev/null
		elif [[ $UFW_PACKAGE_MANAGER == "zypper" ]]; then
			zypper -n install ufw &> /dev/null
		else
			echo "This package manager is not supported"
		fi
		echo "✅ Installed the UFW"
	fi
}

elevate() {
	echo "⚠️  Elevated permissions required"
	sudo echo "✅ Elevated"
}

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

echo -e "Zentrox and related applications come with absolutely no guarantee\nEvery further step is done on your own risk\nBy installing Zentrox you agree to this and the Apache 2.0 license included with Zentrox"

ACTUAL_USERNAME=""

echo -n "🤵 Please enter your linux username ($(whoami)) "
read ACTUAL_USERNAME

if [[ $ACTUAL_USERNAME == "" ]]; then
	ACTUAL_USERNAME=$(whoami)
fi

echo -n "🤵 Please enter your zentrox admin username (max. 512 characters) (e.g. johndoe) "
read ADMIN_USERNAME

if (( ${ADMIN_USERNAME} > 512 )); then
	echo "You will not be able to login with this username"
fi

echo -n "🔑 Please enter your zentrox admin password "
read -s ADMIN_PASSWORD

if (( ${ADMIN_PASSWORD} > 1024 )); then
	echo "You will not be able to login with this password"
fi

echo ""
echo -n "🔑 Enable 2FA [Y/n] "
read ENABLE_2FA

echo -n "🥏 Please enter a name for your zentrox server (e.g. glorious_server) "
read ZENTROX_SERVER_NAME

if [[ $ACTUAL_USERNAME != "root" ]]; then
	USERNAME_PATH="/home/$ACTUAL_USERNAME"
else
	USERNAME_PATH="/root"
fi

if [ -d $USERNAME_PATH ]; then
	echo "✅ Using $USERNAME_PATH/zentrox and $USERNAME_PATH/zentrox_data to install and run zentrox"
	ZENTROX_PATH="$USERNAME_PATH/zentrox"
	ZENTROX_DATA_PATH="$USERNAME_PATH/zentrox_data"
else
	echo "❌ Please enter your correct username or make sure, this username is used for your /home directory."
	exit -1
fi

mkdir $ZENTROX_DATA_PATH &> /dev/null || true

if [[ $ZENTROX_PATH == "/" || $ZENTROX_PATH == $HOME || $ZENTROX_PATH == "$HOME/" ]] ; then
	echo "⚠️ Critical problem detected: $ZENTROX_PATH equal to protected folder"
fi

echo -n "❓ Remove (rm -rf) $ZENTROX_PATH to make sure no old versions of Zentrox are left [Y/n] "

read

if [[ $REPLY == "Y" || $REPLY == "" ]]; then
	rm -rf $ZENTROX_PATH
fi

echo "🔽 Cloning Zentrox to $ZENTROX_PATH"

git clone https://github.com/Wervice/zentrox/ $ZENTROX_PATH&> /dev/null # Clones Codelink repo to current folder

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

echo "✅ Python can be used"

if ! command git -v &> /dev/null
then
	echo "❌ Git is not installed"
fi

echo "⌛ Installing dependencies"
if ! npm -q install express body-parser cookie-parser express-session node-os-utils ejs compression multiparty &> /dev/null; then
	echo "❌ NPM package installation failed";
	npm_failed;
fi

echo "✅ Installed NPM packages"
										   
if ! sudo pip3 -q install pyftpdlib PyOpenSSL &> /dev/null; then
	echo "❌ Python3 package installation failed"
	python_failed
fi

echo "✅ Installed Python packages"

echo "⌛ Compiling C programs"
if ! gcc ./libs/crypt_c.c -o ./libs/crypt_c -lcrypt; then
	echo "❌ Compiling using GCC failed"
fi

if ! openssl -v &> /dev/null; then
	echo "❌ Please install OpenSSL on your system"
fi

echo "✅ OpenSSL is installed" 

echo "🔑 Generating selfsigned keys"

echo "ℹ️ In the following, you will be asked to enter some information to generate  SSL keys and certificates."
echo "ℹ️ You can all fields except the last two empty."
echo "ℹ️ If you do not want to enter real information, you do not have to but it is recommended."

echo ""

if ! ufw -v &> /dev/null; then
	echo "❌ UFW (Uncomplicated firewall) is not installed"
	ufw_fail
fi

echo "✅ The UFW is installed"

openssl genrsa -out selfsigned.key 2048
openssl req -new -key selfsigned.key -out selfsigned.pem
openssl x509 -req -days 365 -in selfsigned.pem -signkey selfsigned.key -out selfsigned.crt

echo ""

echo "🔑 Creating PEM file from key and crt"
cat selfsigned.crt selfsigned.key > selfsigned.pem

echo "✅ Generated .key, .crt and .pem file"

echo "🤵 Creating user 'zentrox'"

elevate
sudo useradd -m -s /bin/bash -ou 0 -g 0 "zentrox" &> /dev/null
USER_PASSWORD=$(openssl rand -base64 48)
echo "ℹ️  Changed Zentrox user password to $USER_PASSWORD"
echo "zentrox:$USER_PASSWORD" | sudo chpasswd

$(echo $(echo $USER_PASSWORD | openssl aes-256-cbc -a -A -pbkdf2 -salt -pass pass:$ADMIN_PASSWORD 2> /dev/null) > "$ZENTROX_DATA_PATH/zentrox_user_password.txt") &> /dev/null

# The zentrox user password has to be stored somewhere for Zentrox to retrieve it. Instead of storing it as an ENV variable or in a plain file, it is encrypted with the admin password
# thus granting some level of protection.

# Sets up all folders and files for Zentrox
echo "📁 Creating file structure"
touch "$ZENTROX_DATA_PATH/admin.txt"
touch "$ZENTROX_DATA_PATH/setupDone.txt"
touch "$ZENTROX_DATA_PATH/users.txt"
mkdir "$ZENTROX_DATA_PATH/users" &> /dev/null
mkdir "$ZENTROX_DATA_PATH/users/$(echo $ADMIN_USERNAME | base64)" &> /dev/null
mkdir "$ZENTROX_DATA_PATH/upload_vault"
mkdir "$ZENTROX_DATA_PATH/vault_extract"
touch "$ZENTROX_DATA_PATH/zentrox.txt"
touch "$ZENTROX_DATA_PATH/vault.vlt"
openssl rand -base64 64 > "$ZENTROX_DATA_PATH/sessionSecret.txt"

touch $ZENTROX_DATA_PATH/config.db
touch $ZENTROX_DATA_PATH/locked.db

# Compile, setup and configure mapbase
cd $ZENTROX_PATH/libs/mapbase/

go build mapbase.go

cd

$ZENTROX_PATH/libs/mapbase/mapbase write $ZENTROX_DATA_PATH/config.db server_name $ZENTROX_SERVER_NAME
$ZENTROX_PATH/libs/mapbase/mapbase write $ZENTROX_DATA_PATH/config.db reg_mode linkInvite
$ZENTROX_PATH/libs/mapbase/mapbase write $ZENTROX_DATA_PATH/config.db server_name $ZENTROX_SERVER_NAME
$ZENTROX_PATH/libs/mapbase/mapbase write $ZENTROX_DATA_PATH/config.db ftp_pid 0000
$ZENTROX_PATH/libs/mapbase/mapbase write $ZENTROX_DATA_PATH/config.db ftp_running 0
$ZENTROX_PATH/libs/mapbase/mapbase write $ZENTROX_DATA_PATH/config.db ftp_username ftp_zentrox
$ZENTROX_PATH/libs/mapbase/mapbase write $ZENTROX_DATA_PATH/config.db ftp_password $(echo -n "change_me" | sha512sum | cut -d ' ' -f 1)
$ZENTROX_PATH/libs/mapbase/mapbase write $ZENTROX_DATA_PATH/config.db ftp_root /
$ZENTROX_PATH/libs/mapbase/mapbase write $ZENTROX_DATA_PATH/config.db zentrox_user_password $(echo $USER_PASSWORD | openssl aes-256-cbc -a -A -pbkdf2 -salt -pass pass:$ADMIN_PASSWORD)
$ZENTROX_PATH/libs/mapbase/mapbase write $ZENTROX_DATA_PATH/config.db vault_enabled "0"
$ZENTROX_PATH/libs/mapbase/mapbase write $ZENTROX_DATA_PATH/config.db knowsOtpSecret "0"

# Enable 2FA
if [[ $ENABLE_2FA == "Y" || $ENABLE_2FA == "" ]]; then
	$ZENTROX_PATH/libs/mapbase/mapbase write $ZENTROX_DATA_PATH/config.db useOtp 1
	echo "ℹ️ You can read the OTP secret in Zentrox' log when you first start the server.'"
	echo "ℹ️ The secret is also stored in ~/zentrox_data/config.db"
	echo "ℹ️ Please copy it and store it in a dedicated app for OTP management."
fi

# Configure admin account
echo -n "$ADMIN_USERNAME" > "$ZENTROX_DATA_PATH/admin.txt"
echo -n "true" > "$ZENTROX_DATA_PATH/setupDone.txt"

echo -n "$(echo -n $ADMIN_USERNAME | base64): $(echo -n "$ADMIN_PASSWORD" | sha512sum | cut -d ' ' -f 1): admin" > "$ZENTROX_DATA_PATH/users.txt"

echo "✅ Installation"
echo "ℹ️  You can now start Zentrox using the command  [ cd $ZENTROX_PATH; node index.js ]"
