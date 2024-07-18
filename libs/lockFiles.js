// lockFiles.js
// Used to lock and unlock files for certain tasks like Zentrox Vault

const { writeDatabase, readDatabase, deleteDatabase } = require("./mapbase");
const { vaultFilePath, lockedFileDatabasePath } = require("./commonVariables");

function isLockedFile(filePath) {
	if (filePath.includes(...[" ", "|"])) {
		zlog("This path can not be checked", "error");
	}
	filePath = Buffer.from(filePath).toString("hex");
	return readDatabase(lockedFileDatabasePath, filePath) == "locked";
}

function lockFile(filePath) {
	if (filePath.includes(...[" ", "|"])) {
		zlog("This path can not be checked", "error");
	}
	filePath = Buffer.from(filePath).toString("hex");
	writeDatabase(lockedFileDatabasePath, filePath, "locked");
}

function unlockFile(filePath) {
	if (filePath.includes(...[" ", "|"])) {
		zlog("This path can not be checked", "error");
	}
	filePath = Buffer.from(filePath).toString("hex");
	if (filePath.includes(...[" ", "|"])) {
		zlog("This path can not be checked", "error");
	}
	deleteDatabase(lockedFileDatabasePath, filePath);
}

function waitForVaultUnlock() {
	return new Promise((resolve) => {
		const interval = setInterval(() => {
			if (!isLockedFile(vaultFilePath)) {
				clearInterval(interval);
				resolve();
			}
		}, 100); // Check every 100 milliseconds
	});
}

module.exports = { isLockedFile, lockFile, unlockFile, waitForVaultUnlock };
