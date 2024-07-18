// commonVariables.js
// Contains commonly used variables and path

const path = require("path");
const os = require("os");

const zentroxInstallationPath = path.join(os.homedir(), "zentrox_data/"); // e.g. /home/test/zentrox_data or /root/zentrox_data | Contains config, user files...
const vaultFilePath = path.join(zentroxInstallationPath, "vault.vlt");
const configDatabasePath = path.join(zentroxInstallationPath, "config.db");
const lockedFileDatabasePath = path.join(zentroxInstallationPath, "locked.db");

module.exports = {
	zentroxInstallationPath,
	vaultFilePath,
	configDatabasePath,
	lockedFileDatabasePath,
};
