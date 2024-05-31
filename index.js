/*
By: Wervice / Constantin Volke
Email: wervice@proton.me
Licence: Apache 2.0 (See GitHub repo (https://github.com/Wervice/Codelink))
*/

const express = require("express");
const path = require("path");
const os = require("os");
const fs = require("fs");
const compression = require("compression");
const bodyParser = require("body-parser");
const crypto = require("crypto");
const cookieParser = require("cookie-parser");
const session = require("express-session");
const https = require("https");
const osu = require("node-os-utils");
const chpr = require("child_process");
const Worker = require("node:worker_threads").Worker;

const port = 3000;
const app = express();

eval(fs.readFileSync(path.join(__dirname, "libs", "packages.js")) + "");
eval(fs.readFileSync(path.join(__dirname, "libs", "drives.js")) + "");
eval(
	fs.readFileSync(path.join(__dirname, "libs", "cryptography_scripts.js")) + "",
);

var key = fs.readFileSync(__dirname + "/selfsigned.key");
var cert = fs.readFileSync(__dirname + "/selfsigned.crt");
var options = {
	key: key,
	cert: cert,
};

const zentroxInstPath = path.join(os.homedir(), "zentrox_data/");

if (!fs.existsSync(path.join(zentroxInstPath, "sessionSecret.txt"))) {
	if (!fs.existsSync(zentroxInstPath)) {
		fs.mkdirSync(zentroxInstPath);
	}
	fs.writeFileSync(
		path.join(zentroxInstPath, "sessionSecret.txt"),
		crypto.randomBytes(64).toString("ascii"),
	);
}

if (!fs.existsSync(zentroxInstPath)) {
	fs.mkdirSync(zentroxInstPath);
}

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

new Worker("./libs/packageWorker.js");

class Shell {
	constructor(username, shell, password, exitcall) {
		// Username: Username to shell into
		// Shell: Shell that will be used
		// Password: Password to log into shell
		// Exitcall: Callback that is ran on s_process exit
		this.username = username;
		this.shell = shell;
		this.password = password;
		this.s_process = chpr.exec(`su ${username}\n`);
		console.log("Shell summoned");
		this.authed = false;
		this.s_process.stderr.on("data", (data) => {
			console.log("Stderr: " + data);
			if (data == "Password: ") {
				this.s_process.stdin.write(this.password + "\n");
				console.log("Shell In: Entered password to shell " + password);
			}
		});
		this.s_process.stdout.on("data", (data) => {
			console.log("Shell Out: " + data);
		});
		this.s_process.on("exit", (data) => {
			console.log("Process Exit");
			if (!this.authed) {
				throw new Error(`Process failed before su could be finished`);
			}
			exitcall(data);
		});

		this.authed = true;
		this.pid = this.s_process.pid;
	}
	write(command) {
		this.s_process.stdin.write(command);
		console.log("Shell In: " + command);
	}
	kill() {
		this.s_process.kill();
	}
}

function zlog(string, type) {
	// ? Custom Zentrox login to replace console.log [Supprots info and error]
	if (type == "info") {
		console.log("[ Info " + new Date().toLocaleTimeString() + "] " + string);
	} else if (type == "error") {
		console.log("[ Error " + new Date().toLocaleTimeString() + "] " + string);
	} else {
		// console.log("[ Verb " + new Date().toLocaleTimeString() + "] " + string);
	}
}

function auth(username, password) {
	// ? Check if user can be authenticated
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

try {
	let [ftp_username, ftp_root, ftp_password, ftp_state] = fs
		.readFileSync(path.join(zentroxInstPath, "ftp.txt"))
		.toString("ascii")
		.split("\n");
	fs.writeFileSync(
		path.join(zentroxInstPath, "ftp.txt"),
		`${ftp_username}\n${ftp_root}\n${ftp_password}\n0`,
	);
} catch {}

function newUser(username, password, role = "user") {
	// ? Create new Zentrox user
	zlog(`Adding new user: Name = ${username} | Role = ${role}`, "info");
	if (role == null || role == "") role = "user";
	var userEntryString = btoa(username) + ": " + hash512(password) + ": " + role;
	var alreadyExisting = false;
	for (line of fs
		.readFileSync(path.join(zentroxInstPath, "users.txt"))
		.toString()
		.split("\n")) {
		if (line.split(": ")[0] == btoa(username)) {
			var alreadyExisting = true;
		}
	}
	if (!alreadyExisting) {
		fs.appendFileSync(
			path.join(zentroxInstPath, "users.txt"),
			userEntryString + "\n",
		);
		fs.mkdirSync(path.join(zentroxInstPath, "users", btoa(username)));
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

function startsetup() {
	// ? Initliazie the Zentrox setup
	if (!fs.existsSync(zentroxInstPath)) {
		fs.mkdirSync(zentroxInstPath);
	}
	fs.mkdirSync(path.join(zentroxInstPath, "users"));
	fs.writeFileSync(path.join(zentroxInstPath, "zentrox.txt"), "");
	fs.writeFileSync(path.join(zentroxInstPath, "users.txt"), "");
}

function sudoSanitize(string) {
	return string
		.replaceAll('"', '\\"')
		.replaceAll("'", "\\'")
		.replaceAll("`", "\\`");
}

function hash512(str) {
	// ? Calculate a SHA 512
	var hash = crypto.createHash("sha512");
	var data = hash.update(str, "utf-8");
	return data.digest("hex");
}

app.get("/", (req, res) => {
	// ? Main page
	if (!fs.existsSync(path.join(zentroxInstPath, "setupDone.txt"))) {
		console.log("Setup not done");
		res.render(path.join(__dirname, "templates/index.html"));
	} else {
		if (req.session.signedIn != true) {
			res.render(path.join(__dirname, "templates/welcome.html"), {
				serverName: fs
					.readFileSync(path.join(zentroxInstPath, "custom.txt"))
					.toString()
					.split("\n")[0]
					.replaceAll("<", "&lt")
					.replaceAll(">", "&gt"),
				registrationButton: (function () {
					if (
						fs
							.readFileSync(path.join(zentroxInstPath, "regMode.txt"))
							.toString()
							.split("\n")[0] == "public"
					) {
						return "<button class=outline onclick=location.href='signup'>Sign up</button>";
					} else {
						return "";
					}
				})(),
			});
		} else {
			res.redirect("/dashboard");
		}
	}
});

app.post("/login", (req, res) => {
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
				path.join(zentroxInstPath, "zentrox_user_password.txt"),
				req.body.password,
			);
		} else {
			req.session.isAdmin = false;
		}
		res.send({
			status: "s",
		});
		if (!fs.existsSync(path.join(zentroxInstPath, "allPackages.txt"))) {
			var packagesString = String(new Date().getTime()) + "\n";
			var allPackages = listPackages();
			for (line of allPackages) {
				packagesString = packagesString + "\n" + line;
			}
			fs.writeFileSync(
				path.join(zentroxInstPath, "allPackages.txt"),
				packagesString,
			);
		}
	} else {
		res.status("403").send({});
	}
});

app.get("/signup", (req, res) => {
	// ? Signup screen
	if (
		fs.readFileSync(path.join(zentroxInstPath, "regMode.txt")).toString() ==
		"public"
	) {
		if (req.session.signedIn == true) {
			res.redirect("/dashboard");
		} else {
			res.render("signup.html", {
				serverName: fs
					.readFileSync(path.join(zentroxInstPath, "custom.txt"))
					.toString()
					.split("\n")[0],
			});
		}
	} else {
		res.status(403).send("Nice try ;)");
	}
});

app.post("/signup", (req, res) => {
	// ? Signup post request handling
	if (req.session.signedIn == true) {
		res.send({
			status: "s",
			text: "Already logged in",
		});
	} else {
		newUser(req.body.username, req.body.password);
		res.send({ status: "s" });
	}
});

app.get("/setup", (req, res) => {
	// ? Zentrox first setup
	if (!fs.existsSync(path.join(zentroxInstPath, "setupDone.txt"))) {
		res.render(path.join(__dirname, "templates/setup.html"));
	} else {
		res.redirect("/");
	}
});

app.post("/setup/registAdmin", (req, res) => {
	// ? Create admin user in setup
	if (fs.existsSync(path.join(zentroxInstPath, "admin.txt"))) {
		res.status(403).send("This action is not allowed");
	} else {
		newUser(req.body.adminUsername, req.body.adminPassword, "admin");
		fs.writeFileSync(
			path.join(zentroxInstPath, "admin.txt"),
			req.body.adminUsername,
		);
		req.session.isAdmin = true;
		res.send({
			status: "s",
		});
	}
});

app.post("/setup/regMode", (req, res) => {
	// ? Change registration mode in setup
	if (fs.existsSync(path.join(zentroxInstPath, "regMode.txt"))) {
		res.status(403).send("This action is not allowed");
	} else {
		fs.writeFileSync(
			path.join(zentroxInstPath, "regMode.txt"),
			req.body.regMode,
		);
		res.send({
			status: "s",
		});
	}
});

app.post("/setup/custom", (req, res) => {
	// ? Final installation changes
	if (fs.existsSync(path.join(zentroxInstPath, "custom.txt"))) {
		res.status(403).send("This action is not allowed");
	} else {
		fs.writeFileSync(
			path.join(zentroxInstPath, "custom.txt"),
			req.body.serverName + "\n" + req.body.cltheme,
		);

		// ? Finish setup process
		fs.writeFileSync(path.join(zentroxInstPath, "setupDone.txt"), "true");
		req.session.signedIn = true;
		req.session.isAdmin = true;

		// ? Write package list to folder for the 1. time
		var packagesString = String(new Date().getTime()) + "\n";
		var allPackages = listPackages();
		for (line of allPackages) {
			packagesString = packagesString + "\n" + line;
		}
		fs.writeFileSync(
			path.join(zentroxInstPath, "allPackages.txt"),
			packagesString,
		);

		// ? Creating system user
		// * FTP

		fs.writeFileSync(
			path.join(zentroxInstPath, "ftp.txt"),
			"ftp_zentrox\n/\n" + hash512("change_me") + "\n0",
		);
		try {
			chpr.exec(
				`echo ${sudoSanitize(req.body.sudo)}
          | sudo -S useradd ftp_zentrox`,
				{ stdio: "pipe" },
			);
		} catch {}
		res.send({
			status: "s",
		});
	}
});

app.get("/dashboard", (req, res) => {
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

app.get("/api", (req, res) => {
	// ? Handle get API
	if (req.query["r"] == "startsetup") {
		if (!fs.existsSync(path.join(zentroxInstPath, "setupDone.txt"))) {
			try {
				startsetup();
				res.send({
					status: "s",
				});
				zlog("Started setup", "verb");
			} catch (e) {
				res.send({
					status: "f",
				});
				zlog("Setup init failed\t" + e, "error");
			}
		} else {
			res.status(403).send({
				status: "f",
				text: "Can't run this command twice",
			});
		}
	} else if (req.query["r"] == "cpuPercent") {
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
		res.status("403").send({
			status: "f",
			text: "No supported command",
		});
	}
});

app.get("/logout", (req, res) => {
	//? Log user out of the Zentrox system
	req.session.signedIn = false;
	req.session.isAdmin = false;
	req.session.adminPassword = "";
	req.session.zentroxPassword = "";
	setTimeout(function () {
		res.redirect("/");
	}, 1000);
});

app.post("/api", (req, res) => {
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
		if (req.session.isAdmin == true) {
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
			for (fileN of fs.readdirSync(req.body.path)) {
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
			}
			res.send({
				content: filesHTML,
			});
		} else {
			res.status(403).send("You have no permissions to access this resource");
		}
	} else if (req.body.r == "deleteFile") {
		// ? Deletes a file from the linux file system
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

		// * Get applications, that feature a GUI
		var desktopFile = "";
		var guiApplications = [];
		var allInstalledPackages = listInstalledPackages(); // ? All installed packages on the system
		const empty_image_encoded = fs
			.readFileSync("static/empty.svg")
			.toString("base64");
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
				var appIconName = "";
				var allOtherPackages = [];

				for (line of desktopFileContentLines) {
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

				if (getIconForPackage(appIconName) != "") {
					var iconForPackage =
						"data:image/svg+xml;base64," +
						fs.readFileSync(getIconForPackage(appIconName)).toString("base64"); // ? Icon as Base64 for package
				} else {
					var iconForPackage =
						"data:image/svg+xml;base64," + empty_image_encoded; // ? "Missing icon" SVG as Base64
				}

				guiApplications[guiApplications.length] = [
					appName,
					iconForPackage,
					appExecName,
				]; // ? The GUI application as an array
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
				chpr.execSync(
					`echo ${req.body.sudo} | sudo -S kill ${fs.readFileSync(path.join(zentroxInstPath, "ftpPid.txt")).toString("ascii")}`,
				);
			} catch {}
			let [ftp_username, ftp_root, ftp_password, ftp_state] = fs
				.readFileSync(path.join(zentroxInstPath, "ftp.txt"))
				.toString("ascii")
				.split("\n");
			fs.writeFileSync(
				path.join(zentroxInstPath, "ftp.txt"),
				`${ftp_username}\n${ftp_root}\n${ftp_password}\n0`,
			);
		} else if (req.body.enableFTP == true) {
			if (
				fs
					.readFileSync(path.join(zentroxInstPath, "ftp.txt"))
					.toString("ascii")
					.split("\n")[3] != "1"
			) {
				zlog("Starting FTP server");
				let ftpProcess = new Shell(
					"zentrox",
					"sh",
					req.session.zentroxPassword,
					(data) => {
						fs.writeFileSync(
							path.join(zentroxInstPath, "ftp.txt"),
							`${ftp_username}\n${ftp_root}\n${ftp_password}\n0`,
						);
						console.log(`FTP server exited with return of: \n${data}`);
					},
				);
				setTimeout(() => {
					ftpProcess.write(`python3 ./libs/ftp.py ${os.userInfo().username} \n`);
				}, 500);

				let [ftp_username, ftp_root, ftp_password, ftp_state] = fs
					.readFileSync(path.join(zentroxInstPath, "ftp.txt"))
					.toString("ascii")
					.split("\n");
				fs.writeFileSync(
					path.join(zentroxInstPath, "ftp.txt"),
					`${ftp_username}\n${ftp_root}\n${ftp_password}\n1`,
				);
			}
		}

		
		// Write changes to ftp.txt
		if (req.body.enableDisable == undefined) {
if (req.body.ftpUserPassword.length == 0) {
			new_ftp_password = fs
				.readFileSync(path.join(zentroxInstPath, "ftp.txt"))
				.toString("ascii")
				.split("\n")[2];
		} else {
			new_ftp_password = hash512(req.body.ftpUserPassword);
		}

			fs.writeFileSync(
				path.join(zentroxInstPath, "ftp.txt"),
				req.body.ftpUserUsername +
					"\n" +
					req.body.ftpLocalRoot +
					"\n" +
					new_ftp_password +
					"\n" +
					(req.body.enableFTP == true ? "1" : "0"),
			);
		} else {
			console.log("Enable/Disable FTP");
		}

		res.send({});
	} else if (req.body.r == "fetchFTPconfig") {
		// ? Send the current FTP information
		const currentFtpUserUsername = fs
			.readFileSync(path.join(zentroxInstPath, "ftp.txt"))
			.toString("utf-8")
			.split("\n")[0];

		const localRoot = fs
			.readFileSync(path.join(zentroxInstPath, "ftp.txt"))
			.toString("utf-8")
			.split("\n")[1];

		if (!req.session.isAdmin) {
			res.status(403).send("You have no permissions to access this resource");
			return;
		}

		res.send({
			enabled:
				fs
					.readFileSync(path.join(zentroxInstPath, "ftp.txt"))
					.toString("ascii")
					.split("\n")[3] == "1",
			ftpUserUsername: currentFtpUserUsername,
			ftpLocalRoot: localRoot,
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
			.execSync("lsb_release -d", { stdio: "pipe", timeout: 500 })
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
		res.send({
			os_name: os_name,
			power_supply: battery_string,
			zentrox_pid: zentrox_pid,
			process_number: process_number,
			hostname: hostname,
			uptime: uptime,
		});
	} else if (req.body.r == "permissions") {
		res.send({ username: os.userInfo().username });
	}
});

process.on("exit", function () {
	zlog("Process exiting...");
	fs.writeFileSync(path.join(zentroxInstPath, "ftp_ppid.txt"), "---");
});

server = https.createServer(options, app);

server.listen(port, () => {
	zlog(`Zentrox running on port ${port}`, "info");
});
