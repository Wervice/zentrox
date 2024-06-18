/*
By: Wervice / Constantin Volke
Email: wervice@proton.me
Licence: Apache 2.0 (See GitHub repo (https://github.com/Wervice/zentrox))

This is open source software. It comes with no guarantee.
*/

const path = require("path");
const bodyParser = require("body-parser");
const osu = require("node-os-utils"); // For CPU, RAM... metrics
const tar = require("tar"); // For tarballs in vault
const os = require("os");
const fs = require("fs");
const crypto = require("crypto");
const https = require("https");
const chpr = require("child_process");
const compression = require("compression"); // Compressing conenction
const cookieParser = require("cookie-parser");
const session = require("express-session");
const express = require("express"); // Using Express framework
const { argv0 } = require("process");

const Worker = require("node:worker_threads").Worker; // For package cache worker

eval(fs.readFileSync("./libs/packages.js").toString("utf8"));
eval(fs.readFileSync("./libs/drives.js").toString("utf8"));
eval(fs.readFileSync("./libs/cryptography_scripts.js").toString("utf8"));
eval(fs.readFileSync("./libs/mapbase.js").toString("utf8"));

var key = fs.readFileSync("./selfsigned.key");
var cert = fs.readFileSync("./selfsigned.crt");
var options = {
	key: key,
	cert: cert,
};

const zentroxInstPath = path.join(os.homedir(), "zentrox_data/"); // e.g. /home/test/zentrox_data or /root/zentrox_data | Contains config, user files...

const port = 3000;
const app = express();

// Database to default values
writeDatabase(path.join(zentroxInstPath, "config.db"), "ftp_running", "0");
writeDatabase(path.join(zentroxInstPath, "config.db"), "ftp_pid", "1");
writeDatabase(path.join(zentroxInstPath, "config.db"), "ftp_may_be_killed", 0);

// Generate session secret
if (!fs.existsSync(path.join(zentroxInstPath, "sessionSecret.txt"))) {
	if (!fs.existsSync(zentroxInstPath)) {
		fs.mkdirSync(zentroxInstPath);
		fs.writeFileSync(
			path.join(zentroxInstPath, "sessionSecret.txt"),
			crypto.randomBytes(64).toString("ascii"),
		);
	}
}

if (!fs.existsSync(zentroxInstPath)) {
	fs.mkdirSync(zentroxInstPath);
}

// Configure server
app.use(bodyParser.urlencoded({ extended: false }));
app.use(bodyParser.json());
app.use(
	cookieParser(
		fs
			.readFileSync(path.join(zentroxInstPath, "sessionSecret.txt"))
			.toString("utf8"),
	),
);

app.use(
	session({
		secret: fs
			.readFileSync(path.join(zentroxInstPath, "sessionSecret.txt"))
			.toString("utf8"),
		name: "currentSessionCookies",
		saveUninitialized: true,
		resave: false,
		cookie: {
			sameSite: true,
			secure: true,
			httpOnly: true,
		},
	}),
);

app.use(express.static("static"));
app.set("views", __dirname + "/templates");
app.engine("html", require("ejs").renderFile);
app.set("view engine", "ejs");
app.use(
	compression({
		level: 9,
		memLevel: 4,
	}),
);

new Worker("./libs/packageWorker.js"); // Start package cache worker (1h interval)

class Shell {
	// Class that generates a shell to launch a new virtual shell with command "su ..."
	constructor(username, shell, password, exitcall) {
		// Username: Username to shell into
		// Shell: Shell that will be used
		// Password: Password to log into shell
		// Exitcall: Callback that is ran on s_process exit
		this.username = username;
		this.shell = shell;
		this.password = password;
		this.s_process = chpr.exec(`su ${username}\n`);
		this.pid = this.s_process.pid;
		this.authed = false;
		zlog("Shell summoned", "info");
		this.authed = false;
		this.s_process.stderr.on("data", (data) => {
			zlog(
				"Stderr: " + data.toString("ascii").endsWith("\n")
					? data.toString("ascii").slice(0, -1)
					: data.toString("ascii"),
				"info",
			);
			if (data == "Password: " && !this.authed) {
				this.s_process.stdin.write(this.password + "\n");
				zlog("Entered password to shell", "info");
				this.authed = true;
			}
		});
		this.s_process.stdout.on("data", (data) => {
			zlog(
				"Shell Out: " + data.toString("ascii").endsWith("\n")
					? data.toString("ascii").slice(0, -1)
					: data.toString("ascii"),
				"info",
			);
		});
		this.s_process.on("exit", (data) => {
			zlog("Shell exit (" + this.pid + ")", "error");
			exitcall(data);
		});
	}
	write(command) {
		if (this.authed) {
			this.s_process.stdin.write(command);
			zlog("Shell In: " + command.replaceAll("\n", ""), "info");
		} else {
			setTimeout(() => {
				this.write(command);
			}, 500);
		}
	}
	kill() {
		this.s_process.kill();
	}
}

function zlog(string, type) {
	if (type == "info") {
		console.log("[ Info " + new Date().toLocaleTimeString() + "] " + string);
	} else if (type == "error") {
		console.log("[ Error " + new Date().toLocaleTimeString() + "] " + string);
	} else {
		// console.log("[ Verb " + new Date().toLocaleTimeString() + "] " + string);
	}
}

function delete_files_recursively(directory) {
	// Read the contents of the directory
	const files = fs.readdirSync(directory);

	// Iterate over each file/folder in the directory
	files.forEach((file) => {
		const filePath = path.join(directory, file);

		// Get the stats of the file/folder
		const stats = fs.statSync(filePath);

		if (stats.isDirectory()) {
			// If it's a directory, recursively delete files in the directory
			delete_files_recursively(filePath);
		} else {
			// If it's a file, delete it
			var j;
			while (j < 4) {
				fs.writeFileSync(
					filePath,
					crypto.randomBytes(fs.statSync(filePath).size),
				);
				j++;
			}
			fs.unlinkSync(filePath);
		}
	});
}

function auth(username, password) {
	// Check if user exists and password hash matches the database hash
	users = fs
		.readFileSync(path.join(zentroxInstPath, "users.txt"))
		.toString()
		.split("\n");
	zlog('Auth "' + username + '"', "info");
	for (user of users) {
		if (atob(user.split(": ")[0]) == username) {
			if (hash512(password) == user.split(": ")[1]) {
				zlog(`Auth for user "${username}" success`, "info");
				return true;
			} else {
				zlog(`Auth for user "${username}" failed`, "info");
				return false;
			}
		}
	}
}

function deleteUser(username) {
	// ? Delete Zentrox user
	var ostring = "";
	for (line of fs
		.readFileSync(path.join(zentroxInstPath, "users.txt"))
		.toString()
		.split("\n")) {
		if (line.split(": ")[0] != btoa(username)) {
			var ostring = ostring + line + "\n";
		}
	}
	var userfolder = path.join(zentroxInstPath, "users", btoa(username));
	if (fs.existsSync(userfolder)) {
		chpr.exec("rm -rf " + userfolder);
	}
	fs.writeFileSync(path.join(zentroxInstPath, "users.txt"), ostring);
}

function hash512(str) {
	// Calculate a SHA 512 hash
	var hash = crypto.createHash("sha512");
	var data = hash.update(str, "utf-8");
	return data.digest("hex");
}

app.get("/", async (req, res) => {
	// Login or auto redirect to dashboard
	if (!fs.existsSync(path.join(zentroxInstPath, "setupDone.txt"))) {
		console.log("Setup not done");
		res.render(path.join(__dirname, "templates/index.html"));
	} else {
		if (req.session.signedIn != true) {
			res.render(path.join(__dirname, "templates/welcome.html"), {
				serverName: readDatabase(
					path.join(zentroxInstPath, "config.db"),
					"server_name",
				),
			});
		} else {
			res.redirect("/dashboard");
		}
	}
});

app.post("/login", async (req, res) => {
	var authTest = auth(req.body.username, req.body.password, req);
	if (authTest == true) {
		req.session.signedIn = true;
		req.session.username = req.body.username;
		if (
			req.body.username ==
			fs.readFileSync(path.join(zentroxInstPath, "admin.txt"))
		) {
			req.session.isAdmin = true;
			req.session.adminPassword = req.body.password;
			req.session.zentroxPassword = decryptAES(
				readDatabase(
					path.join(zentroxInstPath, "config.db"),
					"zentrox_user_password",
				),
				req.body.password,
			);
		} else {
			req.session.isAdmin = false;
		}
		res.send({});
	} else {
		res.status(403).send({
			message: "Wrong password or username",
		});
	}
});

app.get("/dashboard", async (req, res) => {
	// ? Give dashboard to the user (Admin & Front end)
	if (req.session.signedIn == true) {
		if (req.session.isAdmin) {
			res.render("dashboard_admin.html");
		} else {
			res.render("dashboard_user.html");
		}
	} else {
		res.redirect("/");
	}
});

app.get("/api", async (req, res) => {
	// GET API (Not restful)
	if (req.query["r"] == "cpuPercent") {
		if (req.session.isAdmin == true) {
			osu.cpu.usage().then((info) => {
				res.send({
					status: "s",
					p: Number(info),
				});
			});
		}
	} else if (req.query["r"] == "ramPercent") {
		if (req.session.isAdmin == true) {
			res.send({
				status: "s",
				p: Number((os.totalmem() - os.freemem()) / os.totalmem()) * 100,
			});
		}
	} else if (req.query["r"] == "diskPercent") {
		if (req.session.isAdmin == true) {
			var stats = fs.statfsSync("/");
			var percent =
				(stats.bsize * stats.blocks - stats.bsize * stats.bfree) /
				(stats.bsize * stats.blocks);
			res.send({
				status: "s",
				p: Number(percent) * 100,
			});
		}
	} else if (req.query["r"] == "driveList") {
		if (req.session.isAdmin == true) {
			res.send({
				status: "s",
				drives: deviceList(),
			});
		}
	} else if (req.query["r"] == "callfile") {
		if (req.session.isAdmin == true) {
			res
				.set({
					"Content-Disposition": `attachment; filename=${path.basename(
						atob(req.query["file"]),
					)}`,
				})
				.sendFile(atob(req.query["file"]));
		} else {
			res.send("This file can not be shown to you");
			console.zlog(
				`Somebody tried to access ${req.query["file"]} without the correct permissions.`,
				"error",
			);
		}
	} else {
		res.status(403).send({
			status: "f",
			text: "No supported command",
		});
	}
});

app.get("/logout", async (req, res) => {
	//? Log user out of the Zentrox system
	req.session.signedIn = false;
	req.session.isAdmin = false;
	req.session.adminPassword = "";
	req.session.zentroxPassword = "";
	setTimeout(function () {
		res.redirect("/");
	}, 1000);
});

app.post("/api", async (req, res) => {
	// ? Handle post API
	if (req.body.r == "deleteUser") {
		// ? Deletes Zentrox user from the system
		if (req.session.isAdmin) {
			deleteUser(req.body.username);
			res.send({
				status: "s",
			});
		} else {
			res.status(403).send("You have no permissions to access this resource");
		}
	} else if (req.body.r == "userList") {
		// ? Lists Zentrox users
		if (req.session.isAdmin) {
			var userTable = "<table>";
			var userList = fs
				.readFileSync(path.join(zentroxInstPath, "users.txt"))
				.toString()
				.split("\n");
			i = 0;
			while (i != userList.length) {
				if (userList[i].split(": ")[2] == "admin") {
					var userStatus = "<b>Admin</b>";
				} else {
					var userStatus = `User</td><td><button style='color:red' onclick="deleteUser('${atob(
						userList[i].split(": ")[0],
					)}')">Delete</button>`;
				}
				if (userList[i].split(": ")[0] != "") {
					userTable +=
						"<tr><td>" +
						atob(userList[i].split(": ")[0]) +
						"</td><td>" +
						userStatus +
						"</td></tr>";
				}
				i++;
			}
			var userTable = userTable + "</table>";
			res.send({
				text: userTable,
			});
		} else {
			res.status(403).send("You have no permissions to access this resource");
		}
	} else if (req.body.r == "filesRender") {
		// ? List files as HTML and sends it to front end
		if (req.session.isAdmin) {
			var filesHTML = "";
			try {for (fileN of fs.readdirSync(req.body.path)) {
				if (fileN[0] == ".") {
					if (
						req.body.showHiddenFiles == true ||
						req.body.showHiddenFiles == "on"
					) {
						try {
							if (fs.statSync(path.join(req.body.path, fileN)).isFile()) {
								var fileIcon = "file.png";
								var funcToUse = "downloadFile";
							} else {
								var fileIcon = "folder.png";
								var funcToUse = "navigateFolder";
							}
						} catch {
							var fileIcon = "adminfile.png";
							var funcToUse = "alert";
						}
						var filesHTML =
							filesHTML +
							`<button class='fileButtons' onclick="${funcToUse}('${fileN}')" oncontextmenu="contextMenuF('${fileN}')"><img src="${fileIcon}"><br>${fileN
								.replaceAll("<", "&lt;")
								.replaceAll(">", "&gt;")}</button>`;
					}
				} else {
					try {
						if (fs.statSync(path.join(req.body.path, fileN)).isFile()) {
							var fileIcon = "file.png";
							var funcToUse = "downloadFile";
						} else {
							var fileIcon = "folder.png";
							var funcToUse = "navigateFolder";
						}
					} catch {
						var fileIcon = "adminfile.png";
						var funcToUse = "alert";
					}
					var filesHTML =
						filesHTML +
						`<button class='fileButtons' onclick="${funcToUse}('${fileN}')" oncontextmenu="contextMenuF('${fileN}')"><img src="${fileIcon}"><br>${fileN
							.replaceAll("<", "&lt;")
							.replaceAll(">", "&gt;")}</button>`;
				}
			}}
			catch (e) {
				console.error(e)
				res.send({
					message: "no_permissions"
				})
				return;
			}
			res.send({
				content: filesHTML,
			});
		} else {
			res.status(403).send("You have no permissions to access this resource");
		}
	} else if (req.body.r == "deleteFile") {
		// ? Deletes a file from the linux file system
		if (!req.session.isAdmin) return;
		try {
			if (req.session.isAdmin) {
				fs.rmSync(req.body.path, { recursive: true, force: true });
				res.send({
					status: "s",
				});
			}
		} catch (err) {
			console.warn("Error: " + err);
			res.status(500).send("Internal server error");
		}
	} else if (req.body.r == "renameFile") {
		// ? Renamve a file from the linux file system
		if (!req.session.isAdmin) return;
		try {
			if (req.session.isAdmin) {
				fs.renameSync(req.body.path, req.body.newName);
			}
			res.send({
				status: "s",
			});
		} catch (err) {
			console.warn("Error: " + err);
			res.status(500).send({});
		}
	} else if (req.body.r == "packageDatabase") {
		// ? Send the entire package database to the front end
		// * Early return if not admin
		
		if (!req.session.isAdmin) {
			res.status(403).send("You have no permissions to access this resource");
			return;
		}
		zlog("Request Package Database JSON", "info");
		if (!fs.existsSync(path.join(zentroxInstPath, "allPackages.txt"))) {
			var packagesString = String(new Date().getTime()) + "\n";
			var allPackages = await listPackages();
			for (line of allPackages) {
				packagesString = packagesString + "\n" + line;
			}
			fs.writeFileSync(
				path.join(zentroxInstPath, "allPackages.txt"),
				packagesString,
			);
		}
		// * Get applications, that feature a GUI
		var desktopFile = "";
		var guiApplications = [];
		var allInstalledPackages = listInstalledPackages(); // ? All installed packages on the system
		allPackages = fs
			.readFileSync(path.join(zentroxInstPath, "allPackages.txt"))
			.toString("ascii")
			.split("\n");
		allPackages.splice(0, 1);
		for (desktopFile of fs.readdirSync("/usr/share/applications")) {
			// ? Find all GUI applications using .desktop files
			var pathForFile = path.join("/usr/share/applications/", desktopFile);
			zlog(pathForFile, "verb");
			if (fs.statSync(pathForFile).isFile()) {
				var desktopFileContent = fs.readFileSync(pathForFile).toString("utf-8"); // ? Read desktop file
				var desktopFileContentLines = desktopFileContent.split("\n");
				var allOtherPackages = [];

				for (var line of desktopFileContentLines) {
					if (line.split("=")[0] == "Name") {
						// ? Find nice name

						var appName = line.split("=")[1];
						break;
					}
				}

				for (line of desktopFileContentLines) {
					if (line.split("=")[0] == "Icon") {
						// ? Find icon name

						var appIconName = line.split("=")[1].split(" ")[0];
						break;
					}
				}

				for (line of desktopFileContentLines) {
					// ? Find exec command

					if (line.split("=")[0] == "Exec") {
						var appExecName = line.split("=")[1].split(" ")[0];
						break;
					}
				}

				line = null;

				guiApplications[guiApplications.length] = [appName, appExecName]; // ? The GUI application as an array
			}
		}

		var i = 0;
		for (e of allPackages) {
			if (!allInstalledPackages.includes(e)) {
				allOtherPackages[i] = e;
				i++;
			}
		}

		// * Send results to front end

		try {
			res.send({
				content: JSON.stringify({
					gui: guiApplications,
					any: allInstalledPackages,
					all: allOtherPackages,
				}), // * Sends GUI applications and installed packages as JSON
			});
		} catch (err) {
			zlog(err, "error");
			res.status(500).send({});
		}
	} else if (req.body.r == "removePackage") {
		// ? Remove package from the system using apt, dnf, pacman

		if (!req.session.isAdmin) {
			res.status(403).send("You have no permissions to access this resource");
			return;
		}
		if (removePackage(req.body.packageName, req.body.sudoPassword)) {
			res.send({
				status: "s",
			});
			zlog("Removed package " + req.body.packageName, "info");
		} else {
			res.status(500).send({});
			zlog("Failed to remove package " + req.body.packageName, "error");
		}
	} else if (req.body.r == "installPackage") {
		//? Install a package on the system

		zlog("Install package " + req.body.packageName, "info");
		if (!req.session.isAdmin) {
			res.status(403).send("You have no permissions to access this resource");
			return;
		}
		if (installPackage(req.body.packageName, req.body.sudoPassword)) {
			res.send({
				status: "s",
			});
			zlog("Installed package " + req.body.packageName, "info");
		} else {
			res.status(500).send({});
			zlog("Failed to install package " + req.body.packageName, "error");
		}
	} else if (req.body.r == "updateFTPconfig") {
		// ? Change the FTP configuration on the system
		if (!req.session.isAdmin) {
			res.status(403).send("You have no permissions to access this resource");
			return;
		}
		zlog("Change FTP Settings");
		if (req.body.enableFTP == false) {
			try {
				let killShell = new Shell("zentrox", "sh", req.session.zentroxPassword);
				if (
					readDatabase(
						path.join(zentroxInstPath, "config.db"),
						"ftp_may_be_killed",
					) == "1"
				) {
					var killDelay = 0;
				} else {
					var killDelay = 2000;
				}
				setTimeout(() => {
					killShell.write(
						`sudo kill ${readDatabase(path.join(zentroxInstPath, "config.db"), "ftp_pid")}\n`,
					);
				}, 500 + killDelay);
			} catch (e) {
				console.error(e);
			}

			writeDatabase(
				path.join(zentroxInstPath, "config.db"),
				"ftp_running",
				"0",
			);
		} else if (req.body.enableFTP == true) {
			if (
				readDatabase(path.join(zentroxInstPath, "config.db"), "ftp_running") !=
				"1"
			) {
				zlog("Starting FTP server");
				let ftpProcess = new Shell(
					"zentrox",
					"sh",
					req.session.zentroxPassword,
					(data) => {
						writeDatabase(
							path.join(zentroxInstPath, "config.db"),
							"ftp_running",
							"0",
						);
						writeDatabase(
							path.join(zentroxInstPath, "config.db"),
							"ftp_pid",
							"",
						);
						console.log(`FTP server exited with return of: \n${data}`);
					},
				);
				setTimeout(() => {
					ftpProcess.write(
						`python3 ./libs/ftp.py ${os.userInfo().username} \n`,
					);
				}, 500);

				writeDatabase(
					path.join(zentroxInstPath, "config.db"),
					"ftp_running",
					"1",
				);
			}
		}

		// Write changes to ftp.txt
		if (req.body.enableDisable == undefined) {
			if (req.body.ftpUserPassword.length != 0) {
				new_ftp_password = hash512(req.body.ftpUserPassword);
				writeDatabase(
					path.join(zentroxInstPath, "config.db"),
					"ftp_password",
					new_ftp_password,
				);
			}
			writeDatabase(
				path.join(zentroxInstPath, "config.db"),
				"ftp_username",
				req.body.ftpUserUsername,
			);
			writeDatabase(
				path.join(zentroxInstPath, "config.db"),
				"ftp_root",
				req.body.ftpLocalRoot,
			);
			writeDatabase(
				path.join(zentroxInstPath, "config.db"),
				"ftp_running",
				req.body.enableFTP == true ? "1" : "0",
			);
		} else {
			zlog("Enable/Disable FTP was requested (dashboard toggle used)", "info");
		}

		res.send({});
	} else if (req.body.r == "fetchFTPconfig") {
		// ? Send the current FTP information
		if (!req.session.isAdmin) {
			res.status(403).send("You have no permissions to access this resource");
			return;
		}

		res.send({
			enabled:
				readDatabase(path.join(zentroxInstPath, "config.db"), "ftp_running") ==
				"1",
			ftpUserUsername: readDatabase(
				path.join(zentroxInstPath, "config.db"),
				"ftp_username",
			),
			ftpLocalRoot: readDatabase(
				path.join(zentroxInstPath, "config.db"),
				"ftp_root",
			),
		});
	} else if (req.body.r == "driveInformation") {
		// ? Send the current drive information to the frontent
		if (!req.session.isAdmin) {
			res.status(403).send("You have no permissions to access this resource");
			return;
		}

		const dfOutput = chpr.execSync("df -P").toString("ascii");
		const dfLines = dfOutput.trim().split("\n").slice(1); // ? Split output by lines, removing header
		const dfData = dfLines.map((line) => {
			const [filesystem, size, used, available, capacity, mounted] =
				line.split(/\s+/);
			return { filesystem, size, used, available, capacity, mounted };
		});
		res.send({
			drives: deviceInformation(req.body.driveName),
			ussage: dfData,
		});
	} else if (req.body.r == "deviceInformation") {
		if (!req.session.isAdmin) return;
		let os_name = chpr
			.execSync("lsb_release -d", { stdio: "pipe" })
			.toString("ascii")
			.replace("Description:\t", "")
			.replace("\n", "");
		let zentrox_pid = process.pid;
		try {
			let battery_status = fs
				.readFileSync("/sys/class/power_supply/BAT0/status")
				.toString("ascii")
				.replaceAll("\n", "");
			let battery_capacity = fs
				.readFileSync("/sys/class/power_supply/BAT0/capacity")
				.toString("ascii");

			if (battery_status == "Discharging") {
				var battery_string = `Discharging (${battery_capacity}%)`;
			} else if (battery_status != "Full") {
				var battery_string = `Charging (${battery_capacity}%)`;
			} else {
				var battery_string = battery_status;
			}

			battery_string = battery_string.replaceAll("\n", "");
		} catch {
			battery_string = `No battery`;
		}
		let process_number = chpr
			.execSync(" ps -e | wc -l", { stdio: "pipe" })
			.toString("ascii");
		let uptime = chpr
			.execSync("uptime -p")
			.toString("ascii")
			.replace("up ", "");
		let hostname = chpr
			.execSync("hostname")
			.toString("ascii")
			.replace("\n", "");
		try {
			var system_temperature =
				Math.round(
					Number(
						fs
							.readFileSync("/sys/class/thermal/thermal_zone0/temp")
							.toString("ascii"),
					) / 1000,
				) + "°C";
		} catch {
			var system_temperature = null;
		}
		res.send({
			os_name: os_name,
			power_supply: battery_string,
			zentrox_pid: zentrox_pid,
			process_number: process_number,
			hostname: hostname,
			uptime: uptime,
			temperature: system_temperature,
		});
	} else if (req.body.r == "power_off") {
		if (!req.session.isAdmin) return;
		let shutdown_handler = new Shell(
			"zentrox",
			"sh",
			req.session.zentroxPassword,
			() => {},
		);
		setTimeout(() => {
			shutdown_handler.write("sudo poweroff\n");
		}, 500);
		res.send({});
	} else if (req.body.r == "vault_configure") {
		if (!req.session.isAdmin) return;
		if (
			readDatabase(path.join(zentroxInstPath, "config.db"), "vault_enabled") ==
			"0"
		) {
			var key = req.body.key;
			var i = 0;
			while (i != 1000) {
				key = crypto.createHash("sha512").update(key).digest("hex");
				i++;
			}
			// ... create empty tarball
			fs.writeFileSync(path.join(zentroxInstPath, ".vault"), "Init");
			tar
				.c(
					{
						gzip: true,
						file: path.join(zentroxInstPath, "vault.tar"),
						cwd: zentroxInstPath,
					},
					[".vault"],
				)
				.then(() => {
					encryptAESGCM256(path.join(zentroxInstPath, "vault.tar"), key);
					fs.copyFileSync(
						path.join(zentroxInstPath, "vault.tar"),
						path.join(zentroxInstPath, "vault.vlt"),
					);
					fs.unlinkSync(path.join(zentroxInstPath, "vault.tar"));
					fs.unlinkSync(path.join(zentroxInstPath, ".vault"));
					writeDatabase(
						path.join(zentroxInstPath, "config.db"),
						"vault_enabled",
						"1",
					);
					res.send({});
				});
		} else {
			if (typeof req.body.new_key == "undefined") {
				res.send({
					code: "no_decrypt_key",
				});
			} else {
				var old_key = req.body.old_key;
				var new_key = req.body.new_key;
				var i = 0;
				var j = 0;
				while (i != 1000) {
					old_key = crypto.createHash("sha512").update(old_key).digest("hex");
					i++;
				}
				while (j != 1000) {
					new_key = crypto.createHash("sha512").update(new_key).digest("hex");
					j++;
				}
				try {
					decryptAESGCM256(path.join(zentroxInstPath, "vault.vlt"), old_key);
					encryptAESGCM256(path.join(zentroxInstPath, "vault.vlt"), new_key);
					res.send({
						message: "success",
					});
				} catch (e) {
					console.error(e);
					res.send({
						message: "auth_failed",
					});
				}
			}
		}
	} else if (req.body.r == "vault_tree") {
		if (!req.session.isAdmin) return;
		var key = req.body.key;
		var i = 0;
		if (fs.readFileSync(path.join(zentroxInstPath, "vault.vlt")).length === 0) {
			res.send({ message: "vault_not_configured" });
			return;
		}
		while (i != 1000) {
			key = crypto.createHash("sha512").update(key).digest("hex");
			i++;
		}
		try {
			decryptAESGCM256(path.join(zentroxInstPath, "vault.vlt"), key);
		} catch (e) {
			console.error(e);
			res.send({ message: "auth_failed" });
			return;
		}

		function getEntryFilenamesSync(tarballFilename) {
			const filenames = [];
			tar.t({
				file: tarballFilename,
				onentry: (entry) => filenames.push(entry.path),
				sync: true,
			});
			return filenames;
		}

		var entries = getEntryFilenamesSync(
			path.join(zentroxInstPath, "vault.vlt"),
		);
		encryptAESGCM256(path.join(zentroxInstPath, "vault.vlt"), key);
		res.send({ message: "decrypted", fs: entries });
	} else if (req.body.r == "vault_file_download") {
		if (!req.session.isAdmin) return;
		var fpath = req.body.path;
		if (fpath[0] == "/") {
			fpath = fpath.replace("/", "");
		}
		var i = 0;
		var key = req.body.key;
		while (i != 1000) {
			key = crypto.createHash("sha512").update(key).digest("hex");
			i++;
		}
		decryptAESGCM256(path.join(zentroxInstPath, "vault.vlt"), key);
		try {
			fs.mkdirSync(path.join(zentroxInstPath, "vault_extract"));
		} catch {}
		try {
			tar.x(
				{
					file: path.join(zentroxInstPath, "vault.vlt"),
					sync: true,
					cwd: path.join(zentroxInstPath, "vault_extract"),
				},
				[fpath],
			);
		} catch (e) {
			console.log(e);
		}
		encryptAESGCM256(path.join(zentroxInstPath, "vault.vlt"), key);
		var data = fs.readFileSync(
			path.join(zentroxInstPath, "vault_extract", fpath),
		);
		delete_files_recursively(path.join(zentroxInstPath, "vault_extract"));
		res.writeHead(200, {
			"Content-Type": "application/binary",
			"Content-disposition": "attachment;filename=" + path.basename(fpath),
			"Content-Length": data.length,
		});
		res.end(Buffer.from(data, "binary"));
	}
});

process.on("beforeExit", function () {
	zlog("Process exiting...");
	fs.writeFileSync(path.join(zentroxInstPath, "ftp_ppid.txt"), "---");
});

server = https.createServer(options, app);

server.listen(port, () => {
	zlog(`Zentrox running on port ${port}`, "info");
});
