<h1 align="center">Zentrox</h1>

<div align=center>
<img src="static/zentrox_lines.svg" width="100">
</div>
<br>
<div align="center">
<img src="static/readme_preview.png" alt="Preview of Zentrox" width="500">
</div>

<h3 align="center">Easy server admin and setup</h3>

![GitHub License](https://img.shields.io/github/license/Wervice/zentrox?style=for-the-badge)
![GitHub Repo stars](https://img.shields.io/github/stars/Wervice/zentrox?style=for-the-badge)
![JavaScript](https://img.shields.io/badge/JavaScript-white.svg?style=for-the-badge&logo=javascript&logoColor=black&color=gold)
![C](https://img.shields.io/badge/C%20Language-white.svg?style=for-the-badge&logo=c&logoColor=white&color=blue)
![Linux](https://img.shields.io/badge/Linux-white.svg?style=for-the-badge&logo=linux&logoColor=white&color=grey)


## Which problem does Zentrox solve?

Zentrox helps you with the setup and administration of a server.  
It gives you the right tools to do almost everything with your device.  

## 💡 Which problem does Zentrox solve?
Zentrox helps you with the setup and administration of a server.   
It gives you the right tools to do almost everything with your device.   
The installation is very simple and doesn't require great background knowledge.

## 🎯 Features

Zentrox will offer many features for different purposes:

- _File sharing protocols_
- Package manager
- _Storage_ & Files overview
- System resource measurement
- _Web shell_

###### _Italic_ means, that the feature is not fully implemented yet.

## 🛠️ Installation

> [!IMPORTANT]
> Zentrox is still in development.
> At the time, many parts of the application are not done and properly tested.
> If you want to see, what the current state is, you can test it using the commands bellow.
> Please do not ignore any disclaimers, as you may break your system.
> You can also run it in a VM.

Zentrox only supports Linux at the time.
You can use the script bellow to install Zentrox on your system.  
It will auto generate a .key and .crt file for HTTPS support.
If you already have a .key and .crt, please copy it to the folder and call it selfsigned.crt / selfsigned.key.

### Requirements

- NodeJS 18+
- git
- npm
- Linux

### Installing

To install Zentrox, please run the following commands in your terminal:

```bash
git clone https://github.com/Wervice/zentrox/ # Clones Codelink repo to current folder
cd ~/zentrox # Got to zentrox_server folder
npm install express body-parser cookie-parser express-session node-os-utils ejs # Install node_packages
openssl genrsa -out selfsigned.key 2048
openssl req -new -key selfsigned.key -out csr.pem
openssl x509 -req -days 365 -in csr.pem -signkey selfsigned.key -out selfsigned.crt
clear
node index.js # Run zentrox main JS
```
Zentrox only supports Linux at the time.

After installing Zentrox, it will store it the server code in `~/zentrox` and the config, user data... in `~/zentrox_data`
It will now be hosted on [https://localhost:3000](https://localhost:3000).

Zentrox will now be hosted on [https://localhost:3000](https://localhost:3000).

Depending on your your SSL certificates, you may be prompted with a warning about the connection being insecure.   
You can ignore this.
If you want to, you can get an official certificate for this, but your connection will still be fairly safe without one.

## 🗃️ Usage

After rebooting the server or closing Zentrox, you can restart it using:

```bash
cd ~/zentrox # Go to Zentrox code folder
node index.js # Start zentrox
```

You can now login to Zentrox using your admin credentials.
## 🗑️ Removing

You can remove Zentrox by deleting the zentrox_server folder. If you also want to erase all user & admin data, you can remove zentrox_data.

> [!IMPORTANT]
> You can not restore your data using Zentrox after removing it once.

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

# ❓ Why...

## ... JavaScript?

JavaScript is a very fast and extensible language.  
It features most of the things I was looking for and doesn't stress the hardware it runs on to much.

## ... C?

Zentrox also uses C to speed up certain tasks.  
C had the libraries and features I needed to change the system.

## ... Express?

Express is very fast and small. It can be extended using libraries.  
In addition to that, it also is very fast and lightweight on the system.

## ✏️ Contributing

You can contribute this project in many ways. I am happy about any feedback.  
If you found a bug, please open an issue and I will try to fix it.  
Also, you are very welcome to **star** this project.

## 📖 Legal

Zentrox is released under [Apache 2.0](https://github.com/Wervice/Codelink?tab=Apache-2.0-1-ov-file#readme)

Zentrox uses/requires the following resources:  
- Icons8 Icons [icons8.com](https://icons8.com)  
- Work Sans Font [OFL](https://github.com/weiweihuanghuang/Work-Sans/blob/master/OFL.txt)
- VSFTPD as an FTP server [https://security.appspot.com/vsftpd.html](https://security.appspot.com/vsftpd.html) (Has not been modified)
