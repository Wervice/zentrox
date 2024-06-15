// JavaScript implementation for Mapbase database

const chpr = require("child_process");
function readDatabase(file, key) {
	return chpr
		.execSync(`./libs/mapbase/mapbase read ${file} ${key}`)
		.toString("ascii");
}

function writeDatabase(file, key, value) {
	chpr.execSync(`./libs/mapbase/mapbase write ${file} ${key} ${value}`);
}

function deleteDatabase(file, key) {
	chpr.execSync(`./libs/mapbase/mapbase delete ${file} ${key}`);
}
