# This file is intended to be started by Zentrox

update_toml() {
    local key=$1
    local value=$2
    local toml_file="$HOME/zentrox_data/zentrox_store.toml"

    # Check if the file exists
    if [[ ! -f "$toml_file" ]]; then
        echo "Error: $toml_file not found!"
        return 1
    fi

    # Check if the key exists in the file
    if grep -q "^$key[[:space:]]*=" "$toml_file"; then
        # Update the existing key with the new value
        sed -i.bak "s|^$key[[:space:]]*=.*|$key = \"$value\"|" "$toml_file"
    else
        # Add the key-value pair if the key does not exist
        echo "$key = \"$value\"" >> "$toml_file"
    fi
}

derive_key() {
	local clearv=$1
	local salt=$(openssl rand -hex 16)

	echo "$salt\$$(openssl kdf -keylen 64 -binary -kdfopt digest:SHA512 -kdfopt pass:$clearv -kdfopt salt:$salt -kdfopt iter:210000 PBKDF2 | basenc --base16 -w 0)"
}

npm_failed() {
	echo -ne "❌ NPM failed while trying to install various NPM packages.\nDo you want to re-start the installation and ignore warnings? [y/N] "
	read -r
	if [[ $REPLY == "y" ]]; then
		npm -q install express body-parser cookie-parser express-session node-os-utils ejs compression
	else
		echo "Program stopped"
		exit 1
	fi
}

python_failed() {
	echo -ne "❌ The installer for Python modules failed. Do you want to ignore this issue or install the packages using your package manager. [Ignore/Package] "
	read -r
	if [[ $REPLY == "Package" ]]; then
		echo "❓ Please enter the name of your package manager [apt/dnf/pacman]"
		read PYTHON_PACKAGE_MANAGER
		if [[ $PYTHON_PACKAGE_MANAGER == "apt" ]]; then
			apt install python3-pyftpdlib python3-openssl -y &> /dev/null
		elif [[ $PYTHON_PACKAGE_MANAGER == "dnf" ]]; then
			dnf install python3-pyftpdlib python3-openssl -y &> /dev/null
		elif [[ $PYTHON_PACKAGE_MANAGER == "pacman" ]]; then
			pacman -Sy python-pyftpdlib python-openssl &> /dev/null
		else
			echo "❌ This package manager is not know to the installer.\nIt will attempt to run the command $PYTHON_PACKAGE_MANAGER install python3-pyopenssl python3-pyftpdlib -y"
			$PYTHON_PACKAGE_MANAGER install python-pyopenssl python-pyftpdlib -y
		fi
	fi
}

ufw_fail() {
	echo -ne "❌ The UFW is used to manage the firewall on your system.\nDo you want to install it now? [install/ignore] "
	read -r
	if [[ $REPLY == "install" ]]; then
		echo "❓ Please enter the name of your package manager [apt/dnf/pacman/zypper]"
		read UFW_PACKAGE_MANAGER
		if [[ $UFW_PACKAGE_MANAGER == "apt" ]]; then
			apt install ufw -y &> /dev/null
		elif [[ $UFW_PACKAGE_MANAGER == "dnf" ]]; then
			dnf install ufw -y &> /dev/null
		elif [[ $UFW_PACKAGE_MANAGER == "pacman" ]]; then
			pacman -Sy ufw &> /dev/null
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

if (( ${#ADMIN_USERNAME} > 512 )); then
	echo "You will not be able to login with this username"
fi

echo -n "🔑 Please enter your zentrox admin password "
read -s ADMIN_PASSWORD

if (( ${#ADMIN_PASSWORD} > 1024 )); then
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

if [ -d "$USERNAME_PATH" ]; then
	echo "✅ Using $USERNAME_PATH/zentrox and $USERNAME_PATH/zentrox_data to install and run zentrox"
	ZENTROX_PATH="$USERNAME_PATH/zentrox"
	ZENTROX_DATA_PATH="$USERNAME_PATH/zentrox_data"
else
	echo "❌ Please enter your correct username or make sure, this username is used for your /home directory."
	exit 1
fi

mkdir -p "$ZENTROX_DATA_PATH" &> /dev/null || true

if ! command pip3 --version &> /dev/null
then
	echo "❌ Python Package Manager (pip3) is not installed. To fix this issue, please install python3."
	echo "❌ The command to do this may look like this:"
	echo -E "❌ sudo apt install python3"
	exit 1
fi

echo "✅ Python can be used"

if ! pip3 -q install pyftpdlib PyOpenSSL &> /dev/null; then
	echo "❌ Python3 package installation failed"
	python_failed
else
	echo "✅ Installed Python packages"
fi

if ! openssl version &> /dev/null; then
	echo "❌ Please install OpenSSL on your system"
else
	echo "✅ OpenSSL is installed" 
fi

if ! /sbin/ufw --version &> /dev/null; then
	echo "❌ UFW (Uncomplicated firewall) is not installed"
	ufw_fail
else
	echo "✅ The UFW is installed"
fi

echo "ℹ️ Adding UFW rule for port 8080 (this requires admin permissions)"
if ! sudo /usr/sbin/ufw allow from any to any port 8080 > /dev/null; then
	echo "❌ Failed to add UFW rule for Zentrox"
else
	echo "✅ Added UFW rule for Zentrox on port 8080"
fi

echo "🔑 Generating selfsigned keys"

echo "ℹ️  In the following, you will be asked to enter some information to generate  SSL keys and certificates."
echo "ℹ️  You can all fields except the last two empty."
echo "ℹ️  If you do not want to enter real information, you do not have to but it is recommended."

echo "-----------------"

openssl genrsa -out selfsigned.key 2048
openssl req -new -key selfsigned.key -out selfsigned.pem
openssl x509 -req -days 365 -in selfsigned.pem -signkey selfsigned.key -out selfsigned.crt

echo "-----------------"

echo "🔑 Creating PEM file from key and crt"
cat selfsigned.crt selfsigned.key > selfsigned.pem

echo "✅ Generated .key, .crt and .pem file"

# The zentrox user password has to be stored somewhere for Zentrox to retrieve it. Instead of storing it as an ENV variable or in a plain file, it is encrypted with the admin password
# thus granting some level of protection.

# Sets up all folders and files for Zentrox
echo "📁 Creating file structure"
touch "$ZENTROX_DATA_PATH/admin.txt" &> /dev/null
touch "$ZENTROX_DATA_PATH/setupDone.txt" &> /dev/null
mkdir -p "$ZENTROX_DATA_PATH/upload_vault" &> /dev/null
mkdir -p "$ZENTROX_DATA_PATH/vault_extract" &> /dev/null
touch "$ZENTROX_DATA_PATH/zentrox.txt" &> /dev/null
touch "$ZENTROX_DATA_PATH/vault.vlt" &> /dev/null

touch "$ZENTROX_DATA_PATH"/locked.db

touch ~/zentrox_data/zentrox_store.toml
update_toml "server_name" "$ZENTROX_SERVER_NAME"
update_toml "reg_mode" "linkInvite"
update_toml "server_name" "$ZENTROX_SERVER_NAME"
update_toml "ftp_pid" "0000"
update_toml "ftp_running" "0"
update_toml "ftp_username" "ftp_zentrox"
update_toml "ftp_password" "$(echo -n 'change_me' | sha512sum | cut -d ' ' -f 1)"
update_toml "ftp_local_root" "/"
update_toml "vault_enabled" "0"
update_toml "knows_otp_secret" "0"
update_toml "tls_cert" "selfsigned.pem"

# Enable 2FA
if [[ $ENABLE_2FA == "Y" || $ENABLE_2FA == "" ]]; then
	update_toml "use_otp" "1"
	echo "ℹ️  You can read the OTP secret in Zentrox' log when you first start the server.'"
	echo "ℹ️  The secret is also stored in ~/zentrox_data/config.toml"
	echo "ℹ️  Please copy it and store it in a dedicated app for OTP management."
else 
	update_toml "use_otp" "0"
fi

# Configure admin account
echo -n "$ADMIN_USERNAME" > "$ZENTROX_DATA_PATH/admin.txt"
echo -n "true" > "$ZENTROX_DATA_PATH/setupDone.txt"

echo -n "$(echo -n "$ADMIN_USERNAME" | base64): $(derive_key $ADMIN_PASSWORD): admin" > "$ZENTROX_DATA_PATH/users"

echo "ℹ️  You can now start Zentrox using the command  [ $ZENTROX_PATH/zentrox ]"
