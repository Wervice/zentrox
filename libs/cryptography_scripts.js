// Cryptography scripts for Zentrox
// Basically just a wrapper for OpenSSl

const chpr = require("child_process");
const fs = require("fs")
const path = require("path")
const os = require("os")

function decryptAES(in_string, password) {
	// Missing string handler
	const zentroxInstPath = path.join(os.homedir(), "zentrox_data/");
	let tmp_file = path.join(zentroxInstPath, btoa(Math.random()))
	fs.writeFileSync(tmp_file, in_string)	
	const child = chpr.execSync(
		`openssl aes-256-cbc -d -a -pbkdf2 -in ${tmp_file} -pass pass:${password}`,
	);
	fs.writeFileSync(tmp_file, "----------------------------------------------------------------")
	fs.unlinkSync(tmp_file)
	return child.toString("ascii").replaceAll("\n", "");
}
