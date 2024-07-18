// cleanUp.js
// Functions to overwrite / burn sensitive data, to protected it from hackers

const fs = require("fs");
const path = require("path");
const crypto = require("crypto");

function burnFile(filePath, iterations = 3, autoDelete = true) {
	if (!fs.existsSync(filePath)) return false;
	var fileLength = fs.statSync(filePath).size;
	var i = 0;
	while (i != iterations) {
		fs.writeFileSync(filePath, crypto.randomBytes(fileLength));
		i++;
	}
	if (autoDelete) fs.unlinkSync(filePath);
	return true;
}

function deleteFilesRecursively(directory, burn = true) {
	// Read the contents of the directory
	const files = fs.readdirSync(directory);

	// Iterate over each file/folder in the directory
	files.forEach((file) => {
		const filePath = path.join(directory, file);

		// Get the stats of the file/folder
		const stats = fs.statSync(filePath);

		if (stats.isDirectory()) {
			// If it's a directory, recursively delete files in the directory
			deleteFilesRecursively(filePath, burn);
		} else {
			// If it's a file, delete it
			if (burn) {
				burnFile(filePath, 3, true);
			} else {
				fs.unlinkSync(filePath);
			}
		}
	});
	fs.rmSync(directory, { recursive: true });
	fs.mkdirSync(directory);
}

module.exports = { deleteFilesRecursively, burnFile };
