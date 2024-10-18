<div align="center">
<img src="preview.png" alt="Preview of Zentrox" width="500">
</div>

<h3 align="center">Home server administration with batteries included</h3>

![GitHub Repo stars](https://img.shields.io/github/stars/Wervice/zentrox?style=for-the-badge)
![Rust](https://img.shields.io/badge/rust-black.svg?style=for-the-badge&logo=rust&logoColor=white&color=f74c00)

## 💡 Which problems does Zentrox solve?

Zentrox helps you with the administration of your home server and lab devices.
It provides you with important and helpful tools for managing your device.

## 🎯 Features

- System statistics
- Package managing
- Firewall overview ([UFW](https://de.wikipedia.org/wiki/Uncomplicated_Firewall) at the time)
- Encrypted file store
- File sharing (FTPS at the time)
- Block device overview

_Zentrox is still being developed and features listed above are still being developed_

## 🛠️ Installation

**Zentrox is not yet intended for active use, but for testing.**
Zentrox has the following requirements

- OpenSSL
- Uncomplicated Firewall
- Python 3.11+
- pacman/apt/dnf
- Cargo

Zentrox can be installed in one of the following two ways:

### Building Zentrox

At the time, Zentrox can only be installed by building it your self.

I am currently working on adding support for pre-built binaries or an easier way to install the project.

1. Download the latest tarball from the [Release](https://github.com/Wervice/zentrox/releases) page.
2. Unpack the tarball (`tar -xvf zentrox.tar.gz`)
3. Run the installer `bash install.bash`
4. Start Zentrox `cd ~/zentrox; ./zentrox`
5. Follow the Zentrox setup
6. Add Zentrox to your path: `export PATH=$PATH:$HOME/zentrox`

### Post installation

While installing Zentrox, `install.bash` has done the following changes to your computer:

1. Adding a UFW rule to allow Zentrox to be accessed from outside your computer. The rule allows Port 8080 for IPv4 and IPv6 traffic.
2. Creating two directories in your home directory: `~/zentrox/` and `~/zentrox_data/`.
   > [!WARNING]  
   > The ~/zentrox_data/ directory also contains vault files and other sensitive information.
   > This directory and its contents should not be modified or deleted, as doing so may break Zentrox.

#### Removing Zentrox

Zentrox can be removed by reverting the changes mentioned above and by deleting the `~/zentrox_data/` and `~/zentrox/` directory.
Doing so, will also remove your Zentrox vault files.

## ✏️ Contributing

You can contribute to this project in several different ways.  
I am glad about pull requests, issues and feedback.
You can try Zentrox at any time, but I recommend using a pre-build release as Zentrox is actively being developed on and the latest code on GitHub may not be tested yet.

## 📖 Credits & Legal

Zentrox is released under [Apache 2.0](https://github.com/Wervice/Codelink?tab=Apache-2.0-1-ov-file#readme).
The Zentrox frontend uses NextJS by Vercel, Shadcn and Lucide Icons.
The Zentrox backend and installer use OpenSSL as command line tools and/or linked libraries.
You can find more under [`legal/`](legal/) and THIRD_PARTY_LICENSES.md.
