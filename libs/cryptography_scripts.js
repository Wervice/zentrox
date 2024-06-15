// Cryptography scripts for Zentrox
// Basically just a wrapper for OpenSSl

const chpr = require("child_process");
const fs = require("fs");
const crypto = require("crypto");

function decryptAES(string, password) {
	// Missing string handler
	const child = chpr.execSync(
		`echo ${string} | openssl aes-256-cbc -d -a -pbkdf2 -pass pass:${password}`,
	);
	return child.toString("ascii").replaceAll("\n", "");
}

function encryptAESGCM256(file, key) {
	var salt = crypto.randomBytes(32);
	var key = crypto.pbkdf2Sync(key, salt, 100000, 32, "sha256");
	// Generate a random initialization vector
	const iv = crypto.randomBytes(16);
	// Create a cipher instance
	const cipher = crypto.createCipheriv("aes-256-gcm", Buffer.from(key), iv);

	// Encrypt the file
	const input = fs.readFileSync(file);
	const encrypted = Buffer.concat([cipher.update(input), cipher.final()]);

	// Get the authentication tag
	const tag = cipher.getAuthTag();
	// Write the IV, auth tag, and encrypted data to the output file

	fs.writeFileSync(
		file,
		JSON.stringify({
			c: encrypted.toString("hex"),
			i: iv.toString("hex"),
			t: tag.toString("hex"),
			s: salt.toString("hex"),
		}),
	);

}

function decryptAESGCM256(file, key) {
	// Read the encrypted file
	const input = JSON.parse(fs.readFileSync(file));

	// Extract the IV, auth tag, and encrypted data
	const iv = Buffer.from(input["i"], "hex");
	const authTag = Buffer.from(input["t"], "hex");
	const encrypted = Buffer.from(input["c"], "hex");
	const salt = Buffer.from(input["s"], "hex");
	var key = crypto.pbkdf2Sync(key, salt, 100000, 32, "sha256");
	// Create a decipher instance
	const decipher = crypto.createDecipheriv("aes-256-gcm", Buffer.from(key), iv);
	decipher.setAuthTag(authTag);

	// Decrypt the file
	const decrypted = Buffer.concat([
		decipher.update(encrypted),
		decipher.final(),
	]);

	// Write the decrypted data to the output file
	fs.writeFileSync(file, decrypted);
}

