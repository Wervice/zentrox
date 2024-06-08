// Cryptography scripts for Zentrox
// Basically just a wrapper for OpenSSl

const chpr = require("child_process");
const fs = require("fs")

function decryptAES(string, password) {
	// Missing string handler
	const child = chpr.execSync(
		`echo ${string} | openssl aes-256-cbc -d -a -pbkdf2 -pass pass:${password}`,
	);
	return child.toString("ascii").replaceAll("\n", "");
}
