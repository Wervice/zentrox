# Zentrox user documentation
## Preamble
This documentation servers as a guide to use and understand the features, interface and setup of Zentrox.
It does not function as a documentation on how the back-end or front-end are built or work. For that, please
use the developer documentation.

When talking about "(your) device" or "administered device", this documentation refers to the device you **control/administrate using** Zentrox **not** the device you use to access Zentrox.
"Your sudo password", refers to the password the user you run Zentrox as requires to authenticate with sudo.

## Setup
### Requirements
Zentrox server will only fully work on Linux distributions that meet the following requirements:
- Systemd
    - Journalctl
- Sudo
- UFW (Uncomplicated Firewall)
- One of the following package managers
    - APT
    - DNF
    - PacMan
> [!WARNING]
> The user running Zentrox has to be in the [sudoers file](https://en.wikipedia.org/wiki/Sudo#Configuration) for most Zentrox features to work properly.

This includes, but is not limited to the following distributions:
- [Debian](https://debian.org/) (and most Debian-based distributions)
- [Raspberry Pi OS](https://www.raspberrypi.com/software/)
- [Fedora](https://fedoraproject.org/server/)
- [Arch Linux](https://archlinux.org/)

If a certain requirement is not met, it is likely that certain parts of Zentrox will be broken.

|Unmet requirement|Broken feature|
|-----------------|--------------|
|Systemd          |Logging       |
|Sudo             |Privileged tasks|
|UFW              |Firewall configuration|
|Package manager  |Managing packages|

### Installation
Zentrox can be installed under any supported distribution, with the following commands:
```bash
curl -L -o zentrox.tar.gz https://github.com/wervice/zentrox/releases/latest/download/zentrox.tar.gz &&
mkdir zentrox_setup &&
tar -xf zentrox.tar.gz -C zentrox_setup &&
cd zentrox_setup &&
bash install.bash
```
The setup and compilation may take several minutes to complete.  
You are required to configure a password and a username in the terminal.  
It is also possible to enable two factor authentication using One-Time-Pad in combination with a password.  
Zentrox will now launch by default. You can start Zentrox' server by running the binary located in `~/.local/bin/zentrox`

I recommend creating a firewall ALLOW rule using ufw during the setup for port 8080, but this is optional.

Zentrox' frontend can be accessed using a browser.   
It is **not** a requirement for the device you view Zentrox with to use Linux.   
Zentrox has been tested under Firefox and Chromium.

The URL to view Zentrox is `https://[IP-OF-YOUR-DEVICE]:8080/`. Your browser may warn you about an invalid/insecure certificate.
> [!WARNING]
> If you connect to the frontend you will be shown your OTP secret.
> You can not easily get this secret another time without logging in first.

## Login
To log in to Zentrox, open `https://[IP-OF-YOUR-DEVICE]:8080/`. Now enter your password, username and if required also current OTP code.
You may be prompted to copy your current OTP code when this is your first time connecting to Zentrox. If this is the case, make sure to store your code in a safe location to prevent hackers from obtaining it.

## Overview
This page shows you general information about your device. This includes CPU & RAM statistics, networking details, uptime (the time your device has been "up") and package statistics.

## Packages
This page allows you to:
- install
- remove
- list
- update

packages.
It also shows orphaned packages. Make sure not to delete any package you may still need, even if they are flagged as orphaned.
> [!IMPORTANT]
> Installing, removing and updating require your sudo password.

## Logs
This page helps with viewing the logs provided using `journalctl`. Doing so requires your sudo password.
You can select a time window using the "Since" and "Until" handles. Make sure not to choose a very large window, as this may cause slow response time and browser side lag.

## Firewall
The firewall section can be used to enable/disable the UFW as well as create and delete firewall rules.
Doing so requires your sudo password.

## Networking
The networking page can be used to view, disable and enable network interface as well as details about those interfaces. 
Additionally, this page can also view and delete networking routes. 
> [!WARNING]
> Please make sure not to delete any routes that are relevant to your device as Zentrox does not *yet* offer a way to restore or create new routes.

## Files
The files page works like a simple file manager, offering to open ("download"), rename/move and delete files as well as upload new ones.

## Storage
The storage section helps with viewing details about connected block devices such as SSDs, HDDs or USB flash drives.
Simply click a drive to view information about it.

## Vault
Vault provides a simple interface for encrypted file storage.
First, you need to enable by clicking "Setup vault" and following the given instructions.
Then, you can unlock vault to upload files, create folders and delete, move or download said files.
The name of a file or folder can not be longer than 16 bytes (~ 16 ASCII characters).

### Cryptographic details
|detail|value|
|------|-----|
|key|32 bytes|
|nonce|12 bytes|
|key derivation algorithm|Argon2id|
|nonce generation algorithm|SHA 256|
|encryption algorithm|AES 256 GCM|

## Server
The sever section is by far the smallest section at the time.
You can use this section to upload a new TLS/SSL certificate in the `PEM` format.
The default certificate is called `selfsigned.pem` and located in `~/.local/share/zentrox/certificates`

## Media
The media section provides settings to control Zentrox media center.
Zentrox media center is a simple way to share music and video files in your local network. It is *not* intended for large-scale file sharing or streaming. 
Media center is disabled by default. Set enabled to checked to enable media center.
You can now add and remove sources that will be shown in the dashboard of media center.
A source consists of a name, folder path on the administered device and the status. The status can be on or off in order to enable or disable a media source.
Media center can now be accessed under `/media`.

## Account
You can access account settings by clicking your initials or profile picture in the top right corner of Zentrox and selecting account details.
There, you can now change your username and password as well as enable or disable 2FA using OTP.
Make sure to note down your OTP secret if you decide to (re-)enable 2FA.
You can also upload a profile picture.

## Issues, questions and more
If you run into problems while using Zentrox or have questions, you can:
- open a new issue on this GitHub
- write a mail at [wervice@proton.me](mailto:wervice@proton.me)
- ask a question on the [Zentrox subreddit](https://reddit.com/r/Zentrox)

