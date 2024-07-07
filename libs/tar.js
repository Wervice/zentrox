const tar = require("tar");
const os = require("os");
const fs = require("fs");
const crypto = require("crypto");
const path = require("path");

module.exports = class TarArchive {
	constructor(filePath) {
		this.filePath = filePath;
		this._tempDir = path.join(
			os.tmpdir(),
			"zt-" + crypto.randomBytes(6).toString("hex"),
		);

		fs.mkdirSync(this._tempDir);
	}

	_burnFile(tarFilePath) {
		var i = 0;
		var filePath = path.join(this._tempDir, tarFilePath); // True tar file path
		if (!fs.existsSync(filePath)) return false;
		while (i != 3) {
			fs.writeFileSync(
				filePath,
				crypto.randomBytes(fs.statSync(filePath).size),
			);
			i++;
		}
		fs.unlinkSync(filePath);
		return true;
	}

	_recursiveDelete(startingPath) {
		for (const entry of fs.readdirSync(startingPath, {
			recursive: true,
		})) {
			var realPath = path.join(startingPath, entry);
			if (fs.statSync(realPath).isFile()) this._burnFile(realPath);
		}
		fs.rmSync(startingPath, {
			recursive: true,
		});
	}

	removeEntrySync(filePath) {
		tar.extract({
			file: this.filePath,
			sync: true,
			cwd: this._tempDir,
		});
		if (fs.statSync(path.join(this._tempDir, filePath)).isFile()) {
			var deleteStatus = this._burnFile(filePath);
		} else {
			this._recursiveDelete(path.join(this._tempDir, filePath));
		}
		tar.create(
			{
				file: this.filePath,
				sync: true,
				cwd: this._tempDir,
			},
			fs.readdirSync(this._tempDir),
		);
		this._recursiveDelete(this._tempDir);
		return deleteStatus;
	}

	renameEntrySync(filePath, newFilePath) {
		tar.extract({
			file: this.filePath,
			sync: true,
			cwd: this._tempDir,
		});
		fs.renameSync(
			path.join(this._tempDir, filePath),
			path.join(this._tempDir, newFilePath),
			{
				recursive: true,
			},
		);
		this._burnFile(filePath);
		tar.create(
			{
				file: this.filePath,
				sync: true,
				cwd: this._tempDir,
			},
			fs.readdirSync(this._tempDir),
		);
		this._recursiveDelete(this._tempDir);
		return true;
	}
};
