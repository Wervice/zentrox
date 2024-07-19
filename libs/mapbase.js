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
	const readOut = chpr.spawnSync(`./libs/mapbase/mapbase`, ["read", file, key]);
	const stdOut = readOut.stdout.toString("utf8");
	return stdOut;
}

function writeDatabase(
	file = path.join(zentroxInstallationPath, "config.db"),
	key,
	value,
) {
	var key = sanitizeString(key);
	var value = sanitizeString(value);
	const writeOut = chpr.spawnSync(`./libs/mapbase/mapbase`, [
		"write",
		file,
		key,
		value,
	]);
	const stdErr = writeOut.stderr.toString("utf8");

	if (stdErr.length > 0) {
		console.error("Failed to write to Mapbase");
		return false;
	}
	return true;
}

function deleteDatabase(
	file = path.join(zentroxInstallationPath, "config.db"),
	key,
) {
	var key = sanitizeString(key);
	const deleteOut = chpr.spawnSync(`./libs/mapbase/mapbase`, [
		"delete",
		file,
		key,
	]);
	if (deleteOut.stderr.toString("utf8").length > 0) return false;
	return true;
}

module.exports = { readDatabase, writeDatabase, deleteDatabase };
