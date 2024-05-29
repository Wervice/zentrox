// Cryptography scripts for Zentrox
// Basically just a wrapper for OpenSSl

const chpr = require("child_process");

function decryptAES(in_file, password) {
	const child = chpr.execSync(
		`openssl aes-256-cbc -d -a -pbkdf2 -in ${in_file} -pass pass:${password}`,
	);
	return child.toString("ascii").replaceAll("\n", "");
}
