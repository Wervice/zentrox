const fs = require("fs");
const path = require("path");
const chpr = require("child_process");
const os = require("os");
const zentroxInstPath = path.join(os.homedir(), "zentrox_data/");

function zlog(string, type) {
	if (type == "info") {
		console.log("[ Info " + new Date().toLocaleTimeString() + "] " + string);
	} else if (type == "error") {
		console.log("[ Error " + new Date().toLocaleTimeString() + "] " + string);
	} else {
		console.log("[ Verb " + new Date().toLocaleTimeString() + "] " + string);
	}
}

eval(fs.readFileSync(path.join(__dirname, "packages.js")) + "");

function updateCache() {
	var packagesString = String(new Date().getTime()) + "\n";
	allPackages = listPackages();
	for (line of allPackages) {
		packagesString = packagesString + "\n" + line;
	}
	fs.writeFileSync(
		path.join(zentroxInstPath, "allPackages.txt"),
		packagesString,
	);
}

setInterval(
	function () {
		updateCache();
		zlog("Writing package cache", "info");
	},
	60 * 60 * 1000,
);
