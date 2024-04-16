<h1 align="center">Zentrox</h1>

<div align=center>
<img src="static/zentrox_lines.svg" width="100">
</div>
<br>
<div align="center">
<img src="static/readme_preview.png" alt="Preview of Zentrox" width="250">
</div>
<br>

![GitHub License](https://img.shields.io/github/license/Wervice/Codelink?style=for-the-badge)
![GitHub Repo stars](https://img.shields.io/github/stars/Wervice/Codelink?style=for-the-badge)
![JavaScript](https://img.shields.io/badge/JavaScript-white.svg?style=for-the-badge&logo=javascript&logoColor=black&color=gold)
![C](https://img.shields.io/badge/C%20Language-white.svg?style=for-the-badge&logo=c&logoColor=white&color=blue)
![Linux](https://img.shields.io/badge/Linux-white.svg?style=for-the-badge&logo=linux&logoColor=white&color=grey)

Zentrox helps you manage and set up a NAS and collaboration applications on your server or computer.

> [!IMPORTANT]
> Zentrox is still in development. You can use the commands bellow to see what the current state is.
> Please DO NOT actually use it only test it! It is not yet ready.

## Requirements

- NodeJS >=v20
- OpenSSL
- NPM
- Git

## Installation

Zentrox only supports Linux at the time.

You can use this script to install Zentrox on your system. It will auto generate a .key and .crt file for HTTPS support.

If you already have a .key and .crt, please copy it to the folder and call it selfsigned.crt / selfsigned.key.

> [!IMPORTANT]
> Zentrox is not done yet. If you want to try the latest state of development, you can install it using the commands bellow.
> Please do NOT ignore any disclaimers, as you could potentially break your system.
> Look under [Removing](#Removing), to remove Zentrox from your system again.

```bash
git clone https://github.com/Wervice/Codelink/ # Clones Codelink repo to current folder
mv Codelink/zentrox ~/zentrox_server # Moves zentrox to ~/zentrox_server. This folder includes the zentrox code
cd ~/zentrox_server # Got to zentrox_server folder
npm install express body-parser cookie-parser express-session node-os-utils ejs # Install node_packages
openssl genrsa -out selfsigned.key 2048
openssl req -new -key selfsigned.key -out csr.pem
openssl x509 -req -days 365 -in csr.pem -signkey selfsigned.key -out selfsigned.crt
clear
node index.js # Run zentrox main JS
```

> [!NOTE]
> You can remove the Codelink folder after installing Zentrox. It doesn't contain anything important anymore

Zentrox will now be hosted on `localhost:3000`. You can continue with a GUI setup from there.

## Usage

After rebooting the server or closing Zentrox, please restart it using:

```bash
cd ~/zentrox_server # Go to Zentrox code folder
node index.js # Start zentrox
```

You can now login to Zentrox using your admin credentials.

## Features

Zentrox offers many features for different purposes:

### Administration & Management

- File sharing protocols
- Package manager
- Storage & Files overview
- System resource measurement
- Web shell

# Why...

## ... JavaScript?

JavaScript is a very fast and extensible language.  
It features most of the things I was looking for and doesn't stress the hardware it runs on to much.

## ... C?

Zentrox also uses C to speed up certain tasks.  
C had the libraries and features I needed to change the system.

## ... Express?

Express is very fast minimal. It can be extended using libraries.  
In addition to that, it also is very fast and lightweight on the system.

## Removing

You can remove Zentrox by deleting the zentrox_server folder. If you also want to erase all user & admin data, you can remove zentrox_data.

> [!IMPORTANT]
> You can not restore your data after removing it once.

### System changes

Zentrox changes a few configurations on your system, so it will work properly. These are:

#### FTP

1. Installing vsftpd and ufw
2. Enabling ufw
3. Allowing port 20 and 21 using ufw for FTP
4. Creating an FTP user called ftp_zentrox (This will change, if you change the user later using zentrox; The user has the default password `change_me` and not shell access)
   The user will also get no home folder
5. Changing /etc/vsftpd.conf (The last few lines after the last comment)
   - Use vsftpd.userlist
   - Set local root to / (you can change that later)
   - Enable userlist
   - Sub token user to $USER
6. Stopping / Starting vsftpd

## Contributing

Pull requests are welcome. For major changes, please open an issue first
to discuss what you would like to change.

## Legal

Codelink is released under [Apache 2.0](https://github.com/Wervice/Codelink?tab=Apache-2.0-1-ov-file#readme)

Codelink uses/requires the following resources:  
Icons8 Icons [icons8.com](https://icons8.com)  
VSFTPD as an FTP server [https://security.appspot.com/vsftpd.html](https://security.appspot.com/vsftpd.html) (Has not been modified)
