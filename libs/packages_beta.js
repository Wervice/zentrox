/*
? Well, this function is still in beta, an I was to lazy to move it to another file ;-)
function getIconForPackage(packageName) {
  if (fs.existsSync(path.join(os.homedir(), "/.local/share/icons"))) {
    for (folder of fs.readdirSync(
      path.join(os.homedir(), "/.local/share/icons"),
    )) {
      if (
        fs.existsSync(
          path.join(
            os.homedir(),
            "/.local/share/icons",
            folder,
            "apps",
            "scalable",
          ),
        )
      ) {
        iconFolder = path.join(
          os.homedir(),
          "/.local/share/icons",
          folder,
          "apps",
          "scalable",
        );
      }
    }
    if (!iconFolder) {
      console.error(
        `Couldn't find an icon for this package.\nThis library looks for icon packages here: ${path.join(os.homedir(), "/.local/share/icons")}\nPlease make sure, if this package does have an icon in general.`,
      );
    }

    if (fs.existsSync(path.join(iconFolder, packageName + ".svg"))) {
      return path.join(iconFolder, packageName + ".svg");
    } else {
      console.error(
        `Couldn't find an icon for this package.\nThis library looks for icon packages here: ${path.join(os.homedir(), "/.local/share/icons")}.\nAssumed icon location: ${path.join(iconFolder, packageName + ".svg")}`,
      );
    }
  } else {
    icon = null;
    console.error(
      `Couldn't find an icon for this package.\nThis library looks for icon packages here: ${path.join(os.homedir(), "/.local/share/icons")}\nPlease make sure, if this package does have an icon in general.`,
    );
  }
} */
