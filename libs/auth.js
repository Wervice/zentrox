// auth.js
// Used to authenticate a user using password and username

const fs = require("fs");
const path = require("path");
const zlog = require("./zlog");
const { hash512 } = require("./cryptography_scripts");
const { zentroxInstallationPath } = require("./commonVariables");
const crypto = require("crypto");

function auth(username, password) {
	// Check if user exists and password hash matches the database hash
	if (typeof username === "undefined" || typeof password === "undefined") {
		return false;
	}
	var users = fs
		.readFileSync(path.join(zentroxInstallationPath, "users.txt"))
		.toString()
		.split("\n");
	zlog('Auth "' + username + '"', "info");
	for (const user of users) {
		if (
			Buffer.from(user.split(": ")[0], "base64").toString("utf-8") === username
		) {
			if (
				crypto.timingSafeEqual(
					Buffer.from(hash512(password), "utf-8"),
					Buffer.from(user.split(": ")[1], "utf-8"),
				)
			) {
				return true;
			} else {
				return false;
			}
		}
	}
}

module.exports = { auth };
