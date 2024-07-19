# Package Management Library

This library provides a wrapper for DNF, PacMan and APT, to help you control system packages from NodeJS.  
It is written in JavaScript and only needs the standart Node libraries.

## Installation

Copy [this file](zentrox/libs/packages.js) and require it from your code.  
If you have Node installed, it will work. The license for the code is included in the file.

## Documentation

### Installing a package

You can install a package as follows:

```
installPackage(PACKAGE_NAME, SUDO_PASSWORD)
```

The function requires the package name and the sudo password of the current user.

The function will return `true`, if it worked, and `false`, if it failed. Also, you'll see all errors of stderr in your terminal.

### Removing a package

You can remove a package, just like you would install one:

```
removePackage(PACKAGE_NAME, SUDO_PASSWORD)
```

The function requires the package name and the sudo password of the current user.

The function will return `true`, if it worked, and `false`, if it failed. Also, you'll see all errors of stderr in your terminal.

### Listing installed packages

You can list all installed packages as follows:

```
listInstalledPackages()
```

The function will return an array with a list of packages. This may be very long.

### Listing all packages

You can list all packages in the database as follows:

```
listInstalledPackages()
```

The function will return an array with a list of packages. This may be very very long.

## Erros in the terminal

You'll get the entire stderr output of the child process for the installation and removal of a package.
As long as the function returns `true`, you can ignore the errors.

## Questions / Bugs

If you encounter a bug, please contact me in one of these ways:

- E-Mail: wervice@proton.me
- Fosstodon: @wervice
- Issue on GitHub
