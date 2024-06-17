<h1 align="center">Zentrox</h1>

<div align=center>
<img src="static/zentrox_light.svg" width="100">
</div>
<br>
<div align="center">
<img src="static/readme_preview.png" alt="Preview of Zentrox" width="500">
</div>

<h3 align="center">Easy server admin and setup</h3>

![GitHub Repo stars](https://img.shields.io/github/stars/Wervice/zentrox?style=for-the-badge)
![JavaScript](https://img.shields.io/badge/JavaScript-white.svg?style=for-the-badge&logo=javascript&logoColor=black&color=gold)
[![Post on X](https://img.shields.io/badge/Post%20on%20X-white.svg?style=for-the-badge&logo=x&logoColor=white&color=black)](https://x.com/intent/post?text=https%3A%2F%2Fgithub.com%2Fwervice%2Fzentrox)

> [!IMPORTANT]
> ⚠️ Zentrox is a work in progress and not done. It may still change before the first release.

## 💡 Which problems does Zentrox solve?

Zentrox helps you with the administration of your home server.
It will provide you with the most important tools for managin your device.
The is installation very fast and doesn't require great background knowledge.

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
> Zentrox is a work in progress.
> At the time, many parts of the application are not done and properly tested.
> If you want to see, what the current state is, you can test it using the commands bellow.
> Please do not ignore any disclaimers, as you may break your system.
> You can also run it in a VM.

Zentrox only supports Linux at the time.
You can use the script bellow to install Zentrox on your system.  
It will auto generate a .key and .crt file for HTTPS support.
If you already have a .key and .crt, please copy it to the folder and call it selfsigned.crt / selfsigned.key.

### Requirements

- Node.JS (Version 22+)
- Python (Version 3.11+)
- NPM
- Pip3
- Linux (OpenSUSE is not supported at the time)
- OpenSSL

### Installing

1. Run the following command in your terminal:

`curl -fsSL https://raw.githubusercontent.com/Wervice/zentrox/main/install.bash -o zentrox_installer.bash; bash zentrox_installer.bash`

2. Follow the installer.
3. Run `cd ~/zentrox; node zentrox.js` to get started.
4. Go to `https://localhost:3000` and login with your newly configured admin username and password.

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
