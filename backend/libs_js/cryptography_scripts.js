// Cryptography scripts for Zentrox
// Basically just a wrapper for OpenSSl

const chpr = require("child_process");
const fs = require("fs");
const crypto = require("crypto");
const zlog = require("./zlog");

function decryptAES(string, password) {
	// Missing string handler
	try {
		var child = chpr.execSync(
			`echo ${string} | openssl aes-256-cbc -d -a -pbkdf2 -pass pass:${password}`,
			{ stdio: "pipe" },
		);
	} catch (err) {
		zlog("Decryption Error With AES using OpenSSL command\n" + err, "error");
		return "";
	}
	return child.toString("ascii").replaceAll("\n", "");
}

function encryptAESGCM256(file, key) {
	var salt = crypto.randomBytes(32);
	var key = crypto.pbkdf2Sync(key, salt, 100000, 32, "sha256");
	// Generate a random initialization vector
	const iv = crypto.randomBytes(12);
	// Create a cipher instance
	const cipher = crypto.createCipheriv("aes-256-gcm", Buffer.from(key), iv);

	// Encrypt the file
	const input = fs.readFileSync(file);
	const encrypted = Buffer.concat([cipher.update(input), cipher.final()]);

	// Get the authentication tag
	const tag = cipher.getAuthTag();
	// Write the IV, auth tag, and encrypted data to the output file

	fs.writeFileSync(file, Buffer.concat([iv, salt, tag, encrypted]));
}

function decryptAESGCM256(file, key) {
	// Read the encrypted file
	const input = fs.readFileSync(file);

	// Extract the IV, auth tag, and encrypted data
	const file_buffer = Buffer.from(input);
	const iv = file_buffer.subarray(0, 12);
	const authTag = file_buffer.subarray(12 + 32, 12 + 16 + 32);
	const encrypted = file_buffer.subarray(12 + 16 + 32, undefined);
	const salt = file_buffer.subarray(12, 12 + 32);
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

function hash512(str) {
	// Calculate a SHA 512 hash
	var hash = crypto.createHash("sha512");
	var data = hash.update(str, "utf-8");
	return data.digest("hex");
}

module.exports = { decryptAES, encryptAESGCM256, decryptAESGCM256, hash512 };
