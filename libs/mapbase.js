// JavaScript implementation for Mapbase database

const chpr = require("child_process");
const path = require("path");
const os = require("os");

function sanitizeString(input) {
	var string = String(input);
	string = string.replace(/[^a-zA-Z0-9áéíóúñü\_\-\ \.,\"\'\`\;\,]/gm, "");
	return string.trim();
}

const zentroxInstallationPath = path.join(os.homedir(), "zentrox_data/"); // e.g. /home/test/zentrox_data or /root/zentrox_data | Contains config, user files...

function readDatabase(
	file = path.join(zentroxInstallationPath, "config.db"),
	key,
) {
	var key = sanitizeString(key);
	const readOut = chpr
		.execSync(`./libs/mapbase/mapbase read ${file} ${key}`)
		.toString("ascii");
	return readOut;
}

function writeDatabase(
	file = path.join(zentroxInstallationPath, "config.db"),
	key,
	value,
) {
	var key = sanitizeString(key);
	var value = sanitizeString(value);
	chpr.execSync(`./libs/mapbase/mapbase write ${file} ${key} ${value}`);
}

function deleteDatabase(
	file = path.join(zentroxInstallationPath, "config.db"),
	key,
) {
	var key = sanitizeString(key);
	chpr.execSync(`./libs/mapbase/mapbase delete ${file} ${key}`);
}

module.exports = { readDatabase, writeDatabase, deleteDatabase };
