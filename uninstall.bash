echo -e '\033[34m'
echo "███████╗███████╗███╗   ██╗████████╗██████╗  ██████╗ ██╗  ██╗"
echo "╚══███╔╝██╔════╝████╗  ██║╚══██╔══╝██╔══██╗██╔═══██╗╚██╗██╔╝"
echo "  ███╔╝ █████╗  ██╔██╗ ██║   ██║   ██████╔╝██║   ██║ ╚███╔╝ "
echo " ███╔╝  ██╔══╝  ██║╚██╗██║   ██║   ██╔══██╗██║   ██║ ██╔██╗ "
echo "███████╗███████╗██║ ╚████║   ██║   ██║  ██║╚██████╔╝██╔╝ ██╗ "
echo "╚══════╝╚══════╝╚═╝  ╚═══╝   ╚═╝   ╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝"
echo -e '\033[0m'   

echo "Zentrox Uninstaller | by Wervice | github.com/wervice/zentrox"
echo ""
echo "This script will guide you through the process of removing Zentrox from your system."
echo ""

echo "ℹ️ Assuming ~/zentrox and ~/zentrox_data to be the folders used by Zentrox"

echo -n "❓ Do you want to remove the Zentrox server? [y/N] "
read
if [[ $REPLY == "y" || $REPLY == "" ]]; then
	rm -rf ~/zentrox/
fi
echo -n "❓ Do you want to remove the Zentrox data folder (includes user files and configuration)? [y/N] "
read
if [[ $REPLY == "y" || $REPLY == "" ]]; then
	rm -rf ~/zentrox_data/
fi
echo -n "❓ Do you want to remove the Zentrox user (zentrox)? [y/N] "
read
if [[ $REPLY == "y" || $REPLY == "" ]]; then
	sudo deluser zentrox &> /dev/null
	sudo userdel zentrox --force &> /dev/null
fi
echo -n "❓ Do you want to remove the home folder of the Zentrox user (/home/zentrox) ? [y/N] "
read
if [[ $REPLY == "y" || $REPLY == "" ]]; then
	sudo rm -rf /home/zentrox/
fi
echo "ℹ️ Removed Zentrox"
