/*
By: Wervice / Constantin Volke
Email: wervice@proton.me
Licence: Apache 2.0 (See GitHub repo (https://github.com/Wervice/zentrox))

This is open source software. It comes with no guarantee warranty.
I am not liable for damage caused to your hardware, software, data or anything else involving this application.
Note: Read the LICENSE file

Naming policy:
Every function, method, variable, path, key,... is named using camelCase
Every class is named using PascalCase
Every code file is named using snake_case
Package names are not important, though their const has to be named with camelCase

Permission policy:
POST /api	Admin only
GET /api	Admin only
Other paths	Anyone

Package policy:
Zentrox should use as few packages as possible, without breaking scalability or maintanability.
You can find many tools and libraries in the lib folder, which may all be used as ofter as you want.
*/

const path = require("path");
const bodyParser = require("body-parser");
const osu = require("node-os-utils"); // For CPU, RAM... metrics
const tar = require("tar"); // For tarballs in vault
const multiparty = require("multiparty");
const zlib = require("zlib");
const os = require("os");
const fs = require("fs");
const crypto = require("crypto");
const https = require("https");
const http = require("http");
const cors = require("cors");
const chpr = require("child_process");
const compression = require("compression"); // Compressing conenction
const cookieParser = require("cookie-parser");
const session = require("express-session");
const express = require("express"); // Using Express framework

const devDisAuth = false;
// const devDisAuth = true;

const MemoryStore = require("memorystore")(session);
const Worker = require("node:worker_threads").Worker; // For package cache worker

const { TarArchive, createFolderInTarSync } = require("./libs/tar.js");
const Shell = require("./libs/shell.js");

const zlog = require("./libs/zlog.js");
const {
	installPackage,
	removePackage,
	listInstalledPackages,
	listPackages,
	listAutoRemove,
	autoRemove
} = require("./libs/packages.js");
const {
	decryptAESGCM256,
	encryptAESGCM256,
	decryptAES,
} = require("./libs/cryptography_scripts.js");
const {
	generateSecret,
	otpAuth,
	firstOtp,
	firstOtpView,
	knowsOtpSecret,
} = require("./libs/otp.js");
const {
	readDatabase,
	writeDatabase,
	deleteDatabase, // Not used, but will probably be used
} = require("./libs/mapbase.js");
const { auth } = require("./libs/auth.js");

const {
	waitForVaultUnlock,
	lockFile,
	unlockFile,
} = require("./libs/lockFiles.js");
const {
	zentroxInstallationPath,
	vaultFilePath,
	configDatabasePath,
} = require("./libs/commonVariables.js");
const { deviceList, deviceInformation } = require("./libs/drives.js");
const { deleteFilesRecursively } = require("./libs/cleanUp.js");

new Worker("./libs/packageWorker.js"); // Start package cache worker (1h interval)

const key = fs.readFileSync("./selfsigned.key");
const cert = fs.readFileSync("./selfsigned.crt");
const options = {
	key: key,
	cert: cert,
};

const port = 3000;
const app = express();

var bruteForcePasswordIPs = {};

// Database to default values
writeDatabase(
	path.join(zentroxInstallationPath, "config.db"),
	"ftp_running",
	"0",
);
writeDatabase(path.join(zentroxInstallationPath, "config.db"), "ftp_pid", "1");
writeDatabase(
	path.join(zentroxInstallationPath, "config.db"),
	"ftp_may_be_killed",
	0,
);

// Generate session secret
if (!fs.existsSync(path.join(zentroxInstallationPath, "sessionSecret.txt"))) {
	if (!fs.existsSync(zentroxInstallationPath)) {
		fs.mkdirSync(zentroxInstallationPath);
		fs.writeFileSync(
			path.join(zentroxInstallationPath, "sessionSecret.txt"),
			crypto.randomBytes(64).toString("ascii"),
		);
	}
}

if (!fs.existsSync(zentroxInstallationPath)) {
	fs.mkdirSync(zentroxInstallationPath);
}

if (firstOtp()) {
	var otpSecret = generateSecret();
	zlog("Copy this to your password manager 👇");
	zlog(`OTP Secret: ${otpSecret}`, "info");
	writeDatabase(
		path.join(zentroxInstallationPath, "config.db"),
		"otpSecret",
		otpSecret,
	);
}

// Configure server
app.use(bodyParser.urlencoded({ extended: false, limit: "50mb" }));
app.use(bodyParser.json({ limit: "4gb" }));
app.use(
	cookieParser(
		fs
			.readFileSync(path.join(zentroxInstallationPath, "sessionSecret.txt"))
			.toString("utf8"),
	),
);
app.use(
	cors({
		credentials: true,
		origin: "http://localhost:3001", // DO NOT USE THIS IN RELEASE -> Use https://locahost
	}),
);

app.use(
	session({
		secret: fs
			.readFileSync(path.join(zentroxInstallationPath, "sessionSecret.txt"))
			.toString("utf8"),
		name: "currentSessionCookies",
		saveUninitialized: true,
		resave: false,
		store: new MemoryStore({
			checkPeriod: 86400000,
		}),
		cookie: {
			sameSite: true,
			secure: true,
			httpOnly: true,
		},
	}),
);

app.use(express.static("frontend/out"));
app.set("views", __dirname + "/frontend/out");
app.engine("html", require("ejs").renderFile);
app.set("view engine", "ejs");
app.use(
	compression({
		level: 9,
		memLevel: 4,
	}),
);

// General auth

function deleteUser(username) {
	// ? Delete Zentrox user
	var ostring = "";
	for (line of fs
		.readFileSync(path.join(zentroxInstallationPath, "users.txt"))
		.toString()
		.split("\n")) {
		if (line.split(": ")[0] != btoa(username)) {
			var ostring = ostring + line + "\n";
		}
	}
	var userfolder = path.join(zentroxInstallationPath, "users", btoa(username));
	if (fs.existsSync(userfolder)) {
		fs.rm(userfolder, {
			recursive: true,
		});
	}
	fs.writeFileSync(path.join(zentroxInstallationPath, "users.txt"), ostring);
}

if (devDisAuth === true) {
	zlog(
		"Developer Disable Auth is enabled! This poses a high security risk. Please report.",
		"error",
	);
}

function isAdminMw(req, res, next) {
	if (req.session.isAdmin === true || devDisAuth) {
		next();
	} else {
		var ip = req.headers["x-forwarded-for"] || req.socket.remoteAddress;
		zlog(
			`Zentrox Permission error:\t${ip} tried to access a protected resource without admin permissions.`,
			"error",
		);
		res.status(403).send("Missing permissions");
		return;
	}
}

app.get("/", async (req, res) => {
	// Login or auto redirect to dashboard
	if (!fs.existsSync(path.join(zentroxInstallationPath, "setupDone.txt"))) {
		zlog("Setup not done");
		res.render(path.join(__dirname, "templates/index.html"));
	} else {
		if (req.session.signedIn != true) {
			res.render(path.join(__dirname, "templates/welcome.html"), {
				serverName: readDatabase(
					path.join(zentroxInstallationPath, "config.db"),
					"server_name",
				),
				useOtp:
					readDatabase(
						path.join(zentroxInstallationPath, "config.db"),
						"useOtp",
					) == "1",
				firstOtp: firstOtpView(),
				otpSecret: (function () {
					if (firstOtpView()) {
						knowsOtpSecret();
						return readDatabase(
							path.join(zentroxInstallationPath, "config.db"),
							"otpSecret",
						);
					}
					return "''";
				})(),
			});
		} else {
			res.redirect("/dashboard");
		}
	}
});

app.post("/login", async (req, res) => {
	var ip = req.headers["x-forwarded-for"] || req.socket.remoteAddress;

	if (!(ip in bruteForcePasswordIPs)) {
		bruteForcePasswordIPs[ip] = [0, 0];
	}

	var nowTime = new Date().getTime();
	if (
		bruteForcePasswordIPs[ip][1] > 10 &&
		nowTime - bruteForcePasswordIPs[ip][0] < 20000
	) {
		res.send("");
		return;
	} else {
		if (nowTime - bruteForcePasswordIPs[ip][0] > 20000) {
			bruteForcePasswordIPs[ip][1] = 1;
		}
	}

	bruteForcePasswordIPs[ip] = [nowTime, bruteForcePasswordIPs[ip][1] + 1];
	var userPassword = req.body.password;
	var userUsername = req.body.username;

	if (!userPassword || !userUsername) {
		return;
	}

	if (userPassword.length > 1024) {
		return;
	}
	if (userUsername.length > 512) {
		return;
	}

	if (
		readDatabase(path.join(zentroxInstallationPath, "config.db"), "useOtp") ===
		"1"
	) {
		let otpSecret = readDatabase(
			path.join(zentroxInstallationPath, "config.db"),
			"otpSecret",
		);
		var userOtp = String(req.body.userOtp);
		if (!otpAuth(userOtp, otpSecret)) {
			res.status(403).send();
			return;
		}
	}

	var authTest = auth(userUsername, userPassword);

	if (authTest) {
		req.session.signedIn = true;
		req.session.username = req.body.username;
		delete bruteForcePasswordIPs[ip];
		if (
			req.body.username ==
			fs.readFileSync(
				path.join(zentroxInstallationPath, "admin.txt").toString("ascii"),
			)
		) {
			req.session.isAdmin = true;
			req.session.zentroxPassword = decryptAES(
				readDatabase(
					path.join(zentroxInstallationPath, "config.db"),
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

app.get("/login/otpSecret", async (req, res) => {
	if (firstOtpView() && readDatabase(configDatabasePath, "useOtp") === "1") {
		res.send({
			secret: readDatabase(
				path.join(zentroxInstallationPath, "config.db"),
				"otpSecret",
			),
		});
		knowsOtpSecret();
	} else {
		res.status(403).send({});
	}
});

app.get("/login/useOtp", async (req, res) => {
	if (readDatabase(configDatabasePath, "useOtp") === "1") {
		res.send({
			used: true,
		});
	} else {
		res.send({ used: false });
	}
});

app.get("/logout", async (req, res) => {
	//? Log user out of the Zentrox system
	req.session.signedIn = false;
	req.session.isAdmin = false;
	req.session.zentroxPassword = "";
	res.redirect("/");
});

app.get("/dashboard", async (req, res) => {
	// ? Give dashboard to the user (Admin & Front end)
	if (req.session.signedIn == true) {
		if (req.session.isAdmin === true) {
			res.render("dashboard.html");
		} else {
			res.render("dashboard_user.html");
		}
	} else {
		res.redirect("/");
	}
});

app.get("/api/cpuPercent", isAdminMw, async (req, res) => {
	osu.cpu.usage().then((info) => {
		res.send({
			status: "s",
			p: Number(info),
		});
	});
});

app.get("/api/ramPercent", isAdminMw, async (req, res) => {
	res.send({
		status: "s",
		p: Number((os.totalmem() - os.freemem()) / os.totalmem()) * 100,
	});
});

app.get("/api/diskPercent", isAdminMw, async (req, res) => {
	var stats = fs.statfsSync("/");
	var percent =
		(stats.bsize * stats.blocks - stats.bsize * stats.bfree) /
		(stats.bsize * stats.blocks);
	res.send({
		status: "s",
		p: Number(percent) * 100,
	});
});

app.get("/api/driveList", isAdminMw, async (req, res) => {
	res.send({
		status: "s",
		drives: deviceList(),
	});
});

app.get("/api/callFile/:file", isAdminMw, async (req, res) => {
	var file = req.params.file;
	res
		.set({
			"Content-Disposition": `attachment; filename=${path.basename(
				atob(file),
			)}`,
		})
		.sendFile(atob(file));
});

app.get("/api/deleteUser/:username", isAdminMw, async (req, res) => {
	var username = req.params.username;
	if (!username) {
		res.status(400).send();
		return;
	}
	deleteUser(username);
	res.send({
		status: "s",
	});
});

app.get("/api/userList", isAdminMw, async (req, res) => {
	var userTable = "<table>";
	var userList = fs
		.readFileSync(path.join(zentroxInstallationPath, "users.txt"))
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
});

app.get(
	"/api/filesRender/:path/:showHiddenFiles",
	isAdminMw,
	async (req, res) => {
		var filePath = decodeURIComponent(req.params.path);
		if (!filePath) {
			res.status(400).send();
			return;
		}
		var filesHTML = "";
		var fileN;
		try {
			for (fileN of fs.readdirSync(filePath)) {
				if (fileN[0] == ".") {
					if (
						req.params.showHiddenFiles == true ||
						req.params.showHiddenFiles == "on"
					) {
						try {
							if (fs.statSync(path.join(filePath, fileN)).isFile()) {
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
							`<button class='fileButtons' onclick="${funcToUse}('${fileN}')" oncontextmenu="contextMenuF('${fileN}')" title="${fileN}"><img src="${fileIcon}"><br>${fileN
								.replaceAll("<", "&lt;")
								.replaceAll(">", "&gt;")}</button>`;
					}
				} else {
					try {
						if (fs.statSync(path.join(filePath, fileN)).isFile()) {
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
						`<button class='fileButtons' onclick="${funcToUse}('${fileN}')" oncontextmenu="contextMenuF('${fileN}')" title="${fileN}">
								<img src="${fileIcon}"><br>${fileN
									.replaceAll("<", "&lt;")
									.replaceAll(">", "&gt;")}</button>`;
				}
			}
		} catch (e) {
			zlog(e, "error");
			res.send({
				message: "no_permissions",
			});
			return;
		}
		res.send({
			content: filesHTML,
		});
	},
);

app.get("/api/deleteFile/*", isAdminMw, async (req, res) => {
	var filePath = req.params[0];
	if (!filePath) {
		res.status(400).send();
		return;
	}
	try {
		fs.rmSync(filePath, { recursive: true, force: true });
		res.send({
			status: "s",
		});
	} catch (err) {
		console.warn("Error: " + err);
		res.status(500).send("Internal server error");
	}
});

app.get("/api/renameFile/:oldPath/:newName", isAdminMw, async (req, res) => {
	var filePath = decodeURIComponent(req.params.oldPath);
	var newName = decodeURIComponent(req.params.newName);
	if (!filePath || !newName) {
		res.status(400).send();
		return;
	}
	try {
		fs.renameSync(filePath, newName);
		res.send({
			status: "s",
		});
	} catch (err) {
		console.warn("Error: " + err);
		res.status(500).send({});
	}
});

app.get("/api/packageDatabase", isAdminMw, async (req, res) => {
	// ? Send the entire package database to the front end
	// * Early return if not admin

	zlog("Request Package Database JSON", "info");
	if (!fs.existsSync(path.join(zentroxInstallationPath, "allPackages.txt"))) {
		zlog("Database outdated");
		var packagesString = String(new Date().getTime()) + "\n";
		var allPackages = await listPackages();
		for (line of allPackages) {
			packagesString = packagesString + "\n" + line;
		}
		fs.writeFileSync(
			path.join(zentroxInstallationPath, "allPackages.txt"),
			packagesString,
		);
	}

	// * Get applications, that feature a GUI
	var guiApplications = [];
	var allInstalledPackages = listInstalledPackages(); // ? All installed packages on the system
	allPackages = fs
		.readFileSync(path.join(zentroxInstallationPath, "allPackages.txt"))
		.toString("ascii")
		.split("\n");

	allPackages.splice(0, 1);
	var applicationInUsrShare = fs.readdirSync("/usr/share/applications/");
	var desktopFileContent,
		desktopFileContentLines,
		allOtherPackages,
		guiApplications;
	applicationInUsrShare.forEach((desktopFile) => {
		var pathForFile = path.join("/usr/share/applications/", desktopFile);
		if (fs.statSync(pathForFile).isFile()) {
			desktopFileContent = fs.readFileSync(pathForFile).toString("utf-8"); // ? Read desktop file
			desktopFileContentLines = desktopFileContent.split("\n");
			allOtherPackages = [];

			desktopFileContentLines.forEach((line) => {
				const parsedLine = line.split("=");
				if (parsedLine[0] == "Name") {
					appName = line.split("=")[1];
				} else if (parsedLine[0] == "Exec") {
					appExecName = path.basename(line.split("=")[1].split(" ")[0]);
				}
			});
			guiApplications[guiApplications.length] = [appName, appExecName]; // ? The GUI application as an array
		}
	});
	var i = 0;
	var allPackages = allPackages.forEach((e) => {
		if (!allInstalledPackages.includes(e)) {
			allOtherPackages[i] = e;
			i++;
		}
	});

	// * Send results to front end

	try {
		res.send({
			content: JSON.stringify({
				apps: guiApplications, // .desktop files, but parsed
				packages: allInstalledPackages, // Everything the package manager says that it is on the system
				others: allOtherPackages, // Everything that is not in any
			}), // * Sends GUI applications and installed packages as JSON
		});
	} catch (err) {
		zlog(err, "error");
		res.status(500).send({});
	}
});

app.get("/api/packageDatabaseAutoremove", isAdminMw, async (req, res) => {
	var packagesForAutoremove = listAutoRemove();
	res.send({
		packages: packagesForAutoremove,
	});
});

app.get("/api/clearAutoRemove", isAdminMw, async (req, res) => {
	var autoRemoveReturn = await autoRemove(req.session.zentroxPassword)
	var packagesForAutoremove = listAutoRemove();
	if (!autoRemoveReturn) {
		res.status(400).send({})
		return 
	}
	res.send({
		packages: packagesForAutoremove,
	});
});

app.get("/api/removePackage/:packageName", isAdminMw, async (req, res) => {
	// ? Remove package from the system using apt, dnf, pacman

	var packageName = decodeURIComponent(req.params.packageName);
	var zentroxUserPassword = req.session.zentroxPassword
	if (!packageName) {
		res.status(400).send();
		return;
	}
	zlog("Remove package " + packageName, "info");
	if (await removePackage(packageName, zentroxUserPassword)) {
		res.send({});
		zlog("Removed package " + packageName, "info");
	} else {
		res.status(400).send({});
		zlog("Failed to remove package " + packageName, "error");
	}
});

app.get("/api/installPackage/:packageName", isAdminMw, async (req, res) => {
	//? Install a package on the system

	var packageName = decodeURIComponent(req.params.packageName);
	var zentroxUserPassword = req.session.zentroxPassword
	if (!packageName) {
		res.status(400).send();
		return;
	}
	zlog("Install package " + packageName, "info");
	if (await installPackage(packageName, zentroxUserPassword)) {
		res.send({});
		zlog("Installed package " + packageName, "info");
	} else {
		res.status(400).send({});
		zlog("Failed to install package " + packageName, "error");
	}
});

app.post("/api/updateFTPConfig", isAdminMw, async (req, res) => {
	// ? Change the FTP configuration on the system

	var enableFTP = req.body.enableFTP;
	var zentroxUserPassword = req.session.zentroxPassword;
	var enableDisable = req.body.enableDisable;
	if (
		typeof enableFTP === "undefined" ||
		typeof enableDisable === "undefined"
	) {
		res.status(400).send({});
		return;
	}
	zlog("Change FTP Settings");
	if (enableFTP == false) {
		try {
			const killShell = new Shell("zentrox", "sh", zentroxUserPassword);
			if (
				readDatabase(
					path.join(zentroxInstallationPath, "config.db"),
					"ftp_may_be_killed",
				) == "1"
			) {
				var killDelay = 0;
			} else {
				var killDelay = 3000;
			}
			try {
				var ftpServerPid = readDatabase(
					path.join(zentroxInstallationPath, "config.db"),
					"ftp_pid",
				);
				if (ftpServerPid == "") {
					res.send({})
					return;
				}
				setTimeout(async () => {
					await killShell.write(
						`sudo kill ${ftpServerPid}\n`,
					);
					killShell.kill();
				}, killDelay);
			} catch (e) {
				zlog(e, "error");
			}
		} catch (e) {
			zlog(e, "error");
		}

		writeDatabase(
			path.join(zentroxInstallationPath, "config.db"),
			"ftp_running",
			"0",
		);
	} else if (enableFTP == true) {
		if (
			readDatabase(
				path.join(zentroxInstallationPath, "config.db"),
				"ftp_running",
			) != "1"
		) {
			zlog("Starting FTP server");
			const ftpProcess = new Shell(
				"zentrox",
				"sh",
				zentroxUserPassword,
				(data) => {
					writeDatabase(
						path.join(zentroxInstallationPath, "config.db"),
						"ftp_running",
						"0",
					);
					writeDatabase(
						path.join(zentroxInstallationPath, "config.db"),
						"ftp_pid",
						"",
					);
					zlog(`FTP server exited with return of: \n${data}`);
				},
			);
			ftpProcess.write(`python3 ./libs/ftp.py ${os.userInfo().username} \n`);

			writeDatabase(
				path.join(zentroxInstallationPath, "config.db"),
				"ftp_running",
				"1",
			);
		}
	}

	// Write changes to ftp.txt
	if (typeof enableDisable === "undefined") {
		var ftpUserPassword = req.body.ftpUserPassword;
		var ftpUserUsername = req.body.ftpUserUsername;
		var ftpLocalRoot = req.body.ftpLocalRoot;
		if (!ftpUserUsername || !ftpUserPassword || !ftpLocalRoot) {
			res.status(400).send();
			return;
		}
		if (ftpUserPassword.length != 0) {
			new_ftp_password = hash512(ftpUserPassword);
			writeDatabase(
				path.join(zentroxInstallationPath, "config.db"),
				"ftp_password",
				new_ftp_password,
			);
		}
		writeDatabase(
			path.join(zentroxInstallationPath, "config.db"),
			"ftp_username",
			ftpUserUsername,
		);
		writeDatabase(
			path.join(zentroxInstallationPath, "config.db"),
			"ftp_root",
			ftpLocalRoot,
		);
		writeDatabase(
			path.join(zentroxInstallationPath, "config.db"),
			"ftp_running",
			enableFTP == true ? "1" : "0",
		);
	} else {
		zlog("Enable/Disable FTP was requested (dashboard toggle used)", "info");
	}

	res.send({});
});

app.get("/api/fetchFTPconfig", isAdminMw, async (req, res) => {
	// ? Send the current FTP information

	res.send({
		enabled:
			readDatabase(
				path.join(zentroxInstallationPath, "config.db"),
				"ftp_running",
			) == "1",
		ftpUserUsername: readDatabase(
			path.join(zentroxInstallationPath, "config.db"),
			"ftp_username",
		),
		ftpLocalRoot: readDatabase(
			path.join(zentroxInstallationPath, "config.db"),
			"ftp_root",
		),
	});
});

app.get("/api/driveInformation/:driveName", isAdminMw, async (req, res) => {
	// ? Send the current drive information to the frontent
	var driveName = decodeURIComponent(req.params.driveName);

	const dfOutput = chpr.execSync("df -P").toString("ascii");
	const dfLines = dfOutput.trim().split("\n").slice(1); // ? Split output by lines, removing header
	const dfData = dfLines.map((line) => {
		const [filesystem, size, used, available, capacity, mounted] =
			line.split(/\s+/);
		return { filesystem, size, used, available, capacity, mounted };
	});
	res.send({
		drives: deviceInformation(driveName),
		ussage: dfData,
	});
});

app.get("/api/deviceInformation", isAdminMw, async (req, res) => {
	const os_name = chpr
		.execSync("lsb_release -d", { stdio: "pipe" })
		.toString("ascii")
		.replace("Description:\t", "")
		.replace("\n", "");
	const zentrox_pid = process.pid;
	try {
		const battery_status = fs
			.readFileSync("/sys/class/power_supply/BAT0/status")
			.toString("ascii")
			.replaceAll("\n", "");
		const battery_capacity = fs
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
	const process_number = chpr
		.execSync("ps -e | wc -l", { stdio: "pipe" })
		.toString("ascii");
	const uptime = chpr
		.execSync("uptime -p")
		.toString("ascii")
		.replace("up ", "");
	const hostname = chpr
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
});

app.get("/api/powerOff", isAdminMw, async (req, res) => {
	var zentroxUserPassword = req.session.zentroxUserPassword;
	if (!zentroxUserPassword) {
		res.status(400).send();
		return;
	}
	const shutdown_handler = new Shell(
		"zentrox",
		"sh",
		zentroxUserPassword,
		() => {},
	);
	setTimeout(() => {
		shutdown_handler.write("sudo poweroff\n");
	}, 500);
	res.send({});
});

app.post("/api/vaultConfigure", isAdminMw, async (req, res) => {
	if (
		readDatabase(
			path.join(zentroxInstallationPath, "config.db"),
			"vault_enabled",
		) == "0" ||
		!fs.existsSync(path.join(zentroxInstallationPath, "vault.vlt"))
	) {
		var key = req.body.key;
		if (!key) {
			res.status(400).send();
			return;
		}
		var i = 0;
		while (i != 1000) {
			key = crypto.createHash("sha512").update(key).digest("hex");
			i++;
		}
		// ... create empty tarball
		fs.writeFileSync(path.join(zentroxInstallationPath, ".vault"), "Init");
		tar
			.c(
				{
					file: path.join(zentroxInstallationPath, "vault.tar"),
					cwd: zentroxInstallationPath,
				},
				[".vault"],
			)
			.then(() => {
				encryptAESGCM256(path.join(zentroxInstallationPath, "vault.tar"), key);
				fs.copyFileSync(
					path.join(zentroxInstallationPath, "vault.tar"),
					path.join(zentroxInstallationPath, "vault.vlt"),
				);
				fs.unlinkSync(path.join(zentroxInstallationPath, "vault.tar"));
				fs.unlinkSync(path.join(zentroxInstallationPath, ".vault"));
				writeDatabase(
					path.join(zentroxInstallationPath, "config.db"),
					"vault_enabled",
					"1",
				);
				res.send({});
			});
	} else {
		if (typeof req.body.new_key === "undefined") {
			res.send({
				code: "no_decrypt_key",
			});
		} else {
			var oldKey = req.body.oldKey;
			var newKey = req.body.newKey;
			if (!oldKey || !newKey) {
				res.status(400).send("Bad request");
				return;
			}
			var i = 0;
			var j = 0;
			while (i != 1000) {
				oldKey = crypto.createHash("sha512").update(oldKey).digest("hex");
				i++;
			}
			while (j != 1000) {
				newKey = crypto.createHash("sha512").update(newKey).digest("hex");
				j++;
			}
			try {
				waitForVaultUnlock().then(() => {
					lockFile(vaultFilePath);
					decryptAESGCM256(
						path.join(zentroxInstallationPath, "vault.vlt"),
						oldKey,
					);
					encryptAESGCM256(
						path.join(zentroxInstallationPath, "vault.vlt"),
						newKey,
					);
					unlockFile(vaultFilePath);
					res.send({
						message: "success",
					});
				});
			} catch (e) {
				zlog(e, "error");
				res.send({
					message: "auth_failed",
				});
			}
		}
	}
});

app.post("/api/vaultTree", isAdminMw, async (req, res) => {
	var key = req.body.key;
	if (!key) {
		res.status(400).send({});
		return;
	}
	var i = 0;
	if (!fs.existsSync(path.join(zentroxInstallationPath, "vault.vlt"))) {
		res.send({ message: "vault_not_configured" });
		return;
	}
	while (i != 1000) {
		key = crypto.createHash("sha512").update(key).digest("hex");
		i++;
	}
	waitForVaultUnlock().then(() => {
		lockFile(vaultFilePath);

		try {
			decryptAESGCM256(path.join(zentroxInstallationPath, "vault.vlt"), key);
		} catch (e) {
			zlog(e, "error");
			unlockFile(vaultFilePath);
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
			path.join(zentroxInstallationPath, "vault.vlt"),
		);
		encryptAESGCM256(path.join(zentroxInstallationPath, "vault.vlt"), key);
		unlockFile(vaultFilePath);
		res.send({ message: "decrypted", fs: entries });
	});
});

app.post("/api/vaultFileDownload", isAdminMw, async (req, res) => {
	var fpath = req.body.path;
	var key = req.body.key;
	if (!fpath || !key) {
		res.status(400).send();
		return;
	}
	if (fpath[0] == "/") {
		fpath = fpath.replace("/", "");
	}
	var i = 0;
	if (key.length === 0) {
		res.status(400).send("Bad request");
	}
	while (i != 1000) {
		key = crypto.createHash("sha512").update(key).digest("hex");
		i++;
	}

	waitForVaultUnlock().then(() => {
		lockFile(vaultFilePath);
		decryptAESGCM256(path.join(zentroxInstallationPath, "vault.vlt"), key);
		try {
			fs.mkdirSync(path.join(zentroxInstallationPath, "vault_extract"));
		} catch {}
		try {
			tar.x(
				{
					file: path.join(zentroxInstallationPath, "vault.vlt"),
					sync: true,
					cwd: path.join(zentroxInstallationPath, "vault_extract"),
				},
				[fpath],
			);
			fs.writeFileSync(
				path.join(zentroxInstallationPath, "vault_extract", fpath),
				zlib.unzipSync(
					fs.readFileSync(
						path.join(zentroxInstallationPath, "vault_extract", fpath),
					),
				),
			);
		} catch (e) {
			zlog(e);
		}
		encryptAESGCM256(path.join(zentroxInstallationPath, "vault.vlt"), key);
		unlockFile(vaultFilePath);
		var data = fs.readFileSync(
			path.join(zentroxInstallationPath, "vault_extract", fpath),
		);
		deleteFilesRecursively(path.join(zentroxInstallationPath, "vault_extract"));
		res.writeHead(200, {
			"Content-Type": "application/binary",
			"Content-disposition": "attachment;filename=" + path.basename(fpath),
			"Content-Length": data.length,
		});
		res.end(Buffer.from(data, "binary"));
	});
});

app.post("/api/deleteVaultFile", isAdminMw, async (req, res) => {
	var key = req.body.key;
	var deletePath = req.body.deletePath;
	if (!key || !deletePath) {
		res.status(400).send("Bad request");
		return;
	}
	var i = 0;
	while (i != 1000) {
		key = crypto.createHash("sha512").update(key).digest("hex");
		i++;
	}
	waitForVaultUnlock().then(() => {
		lockFile(vaultFilePath);
		decryptAESGCM256(path.join(zentroxInstallationPath, "vault.vlt"), key);
		try {
			const tarFile = new TarArchive(
				path.join(zentroxInstallationPath, "vault.vlt"),
			);
			tarFile.removeEntrySync(deletePath);
		} catch (err) {
			zlog(err, "error");
		}
		encryptAESGCM256(path.join(zentroxInstallationPath, "vault.vlt"), key);
		unlockFile(vaultFilePath);
		res.send({});
	});
});

app.post("/api/renameVaultFile", isAdminMw, async (req, res) => {
	var key = req.body.key;
	var oldPath = req.body.path;
	var newPath = req.body.newName;
	if (!key || !oldPath || !newPath) {
		res.status(400).send("Bad request");
		return;
	}
	var i = 0;
	while (i != 1000) {
		key = crypto.createHash("sha512").update(key).digest("hex");
		i++;
	}
	waitForVaultUnlock().then(() => {
		try {
			lockFile(vaultFilePath);
			decryptAESGCM256(path.join(zentroxInstallationPath, "vault.vlt"), key);
		} catch (err) {
			zlog(err, "error");
			res.status(500).send({
				msg: "auth_failed",
			});
			return;
		}
		try {
			const tarFile = new TarArchive(
				path.join(zentroxInstallationPath, "vault.vlt"),
			);
			tarFile.renameEntrySync(oldPath, newPath);
		} catch (err) {
			zlog(err, "error");
		}
		encryptAESGCM256(path.join(zentroxInstallationPath, "vault.vlt"), key);
		unlockFile(vaultFilePath);
		res.send({});
	});
});

app.get("/api/vaultBackup", isAdminMw, async (req, res) => {
	data = fs.readFileSync(path.join(zentroxInstallationPath, "vault.vlt"));
	res.writeHead(200, {
		"Content-Type": "application/binary",
		"Content-disposition": "attachment;filename=" + "vault.tar",
		"Content-Length": data.length,
	});
	res.end(Buffer.from(data, "binary"));
});

app.post("/api/vaultNewFolder", isAdminMw, async (req, res) => {
	var key = req.body.key;
	var folder_name = req.body.folder_name;

	if (!key || !folder_name) {
		res.status(400).send();
		return;
	}

	var i = 0;
	while (i != 1000) {
		key = crypto.createHash("sha512").update(key).digest("hex");
		i++;
	}

	waitForVaultUnlock().then(() => {
		try {
			lockFile(vaultFilePath);
			decryptAESGCM256(path.join(zentroxInstallationPath, "vault.vlt"), key);
		} catch (err) {
			res.status(500).send({ message: "auth_failed" });
		}

		try {
			createFolderInTarSync(
				path.join(zentroxInstallationPath, "vault.vlt"),
				folder_name,
			);
		} catch (err) {
			zlog(err, "error");
		} finally {
			encryptAESGCM256(path.join(zentroxInstallationPath, "vault.vlt"), key);
			unlockFile(vaultFilePath);
		}
		res.send({});
	});
});

app.get("/api/fireWallInformation", isAdminMw, async (req, res) => {
	var informationShell = new Shell(
		"zentrox",
		"sh",
		req.session.zentroxPassword,
		() => {},
		true,
		20000, // Prevent long outputs and holding the server
		false,
	);
	try {
		var ufwStatusReturnData = await informationShell.write("/usr/sbin/ufw status\n");
	} catch (err) {
		zlog(err, "error");
		res.status(500).send({
			msg: "ufw_timeout",
		});
		return;
	}
	var ufwStatus = ufwStatusReturnData.toString("ascii");
	const ufwStatusLines = ufwStatus.trim().split("\n");
	const rules = [];
	const ruleLines = ufwStatusLines.slice(4);
	var index = 1;
	ruleLines.forEach((line) => {
		const [to, action, from] = line.trim().split(/\s{2,}/);
		rules.push({ index, to, action, from });
		index++;
	});
	informationShell.kill(); // Clear this shell
	res.send({
		enabled: ufwStatus.split("\n")[0] == "Status: active",
		rules: rules,
	});
});

app.get("/api/switchUFW/:enable", isAdminMw, async (req, res) => {
	var ufwState = req.params.enable === "true";
	if (typeof ufwState === "undefined") {
		return;
	}
	const ufwShell = new Shell(
		"zentrox",
		"sh",
		req.session.zentroxPassword,
		() => {},
		true,
		1000,
		true,
	);
	try {
		if (!ufwState) {
			await ufwShell.write("/usr/sbin/ufw disable\n");
		} else {
			await ufwShell.write("/usr/sbin/ufw enable\n");
		}
	} catch (err) {
		zlog(err, "error");
		res.status(400).send({})
		return
	}
	ufwShell.kill();
	res.send({});
});

app.get("/api/deleteFireWallRule/:index", isAdminMw, async (req, res) => {
	var ruleIndex = req.params.index;

	if (typeof ruleIndex === "undefined") {
		return;
	}

	const deleteRuleShell = new Shell(
		"zentrox",
		"sh",
		req.session.zentroxPassword,
		() => {},
		false,
		2000,
		true,
	);
	try {
		await deleteRuleShell.write("/usr/sbin/ufw delete " + ruleIndex + "\n");
	} catch (error) {
		zlog("Deleting rule using UFW resulted in it throwing to Stderr. Stopped");
		res.status(500).send({
			msg: error,
		});
		return;
	}
	await deleteRuleShell.write("y\n");
	deleteRuleShell.kill();
	res.send({});
});

app.get(
	"/api/newFireWallRule/:from/:to/:action",
	isAdminMw,
	async (req, res) => {
		var ruleFrom = decodeURIComponent(req.params.from);
		var ruleTo = decodeURIComponent(req.params.to);
		var ruleAction = decodeURIComponent(req.params.action);
		zlog(ruleAction);
		if (!ruleAction) {
			return;
		}
		const newRuleShell = new Shell(
			"zentrox",
			"sh",
			req.session.zentroxPassword,
			() => {},
			true,
			20000,
			true,
		);
		if (!ruleFrom) {
			ruleFrom = "any";
		}
		if (!ruleTo) {
			ruleTo = "any";
		}
		ruleFrom = ruleFrom.replaceAll(";", "");
		ruleTo = ruleTo.replaceAll(";", "");
		const ipRegex =
			/^(25[0-5]|2[0-4][0-9]|1[0-9]{2}|[1-9]?[0-9])(\.(25[0-5]|2[0-4][0-9]|1[0-9]{2}|[1-9]?[0-9])){3}$/;
		const portRegex = /^[a-zA-Z0-9 :]*$/;
		if (!ipRegex.test(ruleFrom)) {
			res.status(500).send({
				msg: "Malformed IP",
			});
			return;
		}
		if (!portRegex.test(ruleTo)) {
			res.send({
				msg: "Malformed port",
			});
			return;
		}
		try {
			var newRuleCommandOutput = await newRuleShell.write(
				`/usr/sbin/ufw ${ruleAction} from ${ruleFrom} to any port ${ruleTo}\n`,
			);
		} catch (err) {
			newRuleShell.kill();
			res.status(500).send({
				msg: err,
			});
			return;
		}
		newRuleShell.kill();
		res.send({
			commandOutput: newRuleCommandOutput,
		});
	},
);

app.get("/api/mwOk", isAdminMw, async (req, res) => {
	res.send("If you are not logged in, you should not have gotten a response.");
});

app.post("/upload/vault", isAdminMw, async (req, res, next) => {
	var form = new multiparty.Form();
	form.parse(req, (err, fields, files) => {
		if (err) {
			zlog(err, "error");
			res.status(400).send({ message: "Failed to parse request" });
		}
		try {
			var key = fields["key"][0];
			var in_vault_path = fields["path"][0];
		} catch (err) {
			next(err);
			return;
		}
		if (!key) {
			res.status(400).send({});
			return;
		}
		var i = 0;
		while (i != 1000) {
			key = crypto.createHash("sha512").update(key).digest("hex");
			i++;
		}
		try {
			fs.copyFile(
				path.join(zentroxInstallationPath, "vault.vlt"),
				path.join(zentroxInstallationPath, "vault.vlt.bak"),
				(err) => {
					if (err) {
						zlog(err, "error");
						return;
					}
				},
			);
			decryptAESGCM256(path.join(zentroxInstallationPath, "vault.vlt"), key);
		} catch (err) {
			zlog(err, "error");
			res.send({ message: "auth_failed" });
			return;
		}
		try {
			try {
				var fpath = files["file"][0]["path"];
				var ffilename = files["file"][0]["originalFilename"];
			} catch {
				res.status(400).send();
				return;
			}
			var contents = [];
			tar.t({
				file: path.join(zentroxInstallationPath, "vault.vlt"),
				onentry: (entry) => contents.push(entry.path),
				sync: true,
			});

			while (
				contents.includes(path.join(in_vault_path, ffilename).replace("/", ""))
			) {
				var filenameSplit = ffilename.split(".");
				ffilename =
					filenameSplit[filenameSplit.length - 2] +
					"_new." +
					filenameSplit[filenameSplit.length - 1];
			}
			var new_path = path.join(
				zentroxInstallationPath,
				"vault_extract",
				in_vault_path,
				ffilename,
			);
			fs.mkdirSync(
				path.join(zentroxInstallationPath, "vault_extract", in_vault_path),
				{
					recursive: true,
				},
			);
			fs.copyFileSync(fpath, new_path);
			fs.writeFileSync(
				new_path,
				zlib.gzipSync(fs.readFileSync(new_path), {
					level: 9,
				}),
			);
			tar.update(
				{
					file: path.join(zentroxInstallationPath, "vault.vlt"),
					sync: true,
					cwd: path.join(zentroxInstallationPath, "vault_extract"),
				},
				[path.join(in_vault_path.replace("/", ""), ffilename)],
			);
		} catch (err) {
			zlog(err, "error");
			res.send({ message: err });
			encryptAESGCM256(path.join(zentroxInstallationPath, "vault.vlt"), key);
			return;
		}

		encryptAESGCM256(path.join(zentroxInstallationPath, "vault.vlt"), key);
		deleteFilesRecursively(path.join(zentroxInstallationPath, "vault_extract"));
		var j = 0;
		while (j < 4) {
			fs.writeFileSync(fpath, crypto.randomBytes(fs.statSync(fpath).size));
			fs.writeFileSync(
				path.join(zentroxInstallationPath, "vault.vlt.bak"),
				crypto.randomBytes(
					fs.statSync(path.join(zentroxInstallationPath, "vault.vlt.bak")).size,
				),
			);
			if (fs.existsSync(new_path)) {
				fs.writeFileSync(
					new_path,
					crypto.randomBytes(fs.statSync(new_path).size),
				);
			}
			j++;
		}
		if (fs.existsSync(new_path)) {
			fs.unlinkSync(new_path);
		}
		if (fs.existsSync(fpath)) {
			fs.unlinkSync(fpath);
		}
		if (fs.existsSync(path.join(zentroxInstallationPath, "vault.vlt.bak"))) {
			fs.unlinkSync(path.join(zentroxInstallationPath, "vault.vlt.bak"));
		}
		res.send({});
	});
});

app.post("/upload/fs", isAdminMw, async (req, res, next) => {
	var form = new multiparty.Form();
	form.parse(req, (err, fields, files) => {
		if (err) {
			next(err);
		}
		try {
			var systemPath = fields["filePath"][0];
			var fileName = files["file"][0]["originalFilename"];
			var fullPath = path.join(systemPath, fileName);
			var tempPath = files["file"][0]["path"];
		} catch (err) {
			next(err);
			return;
		}
		while (fs.existsSync(fullPath)) {
			var fullPathSplit = fullPath.split(".");
			fullPath =
				fullPathSplit[fullPathSplit.length - 2] +
				"_new." +
				fullPathSplit[fullPathSplit.length - 1];
		}
		fs.copyFileSync(tempPath, fullPath);
		fs.unlinkSync(tempPath);
		res.send({ message: "Uploaded" });
	});
});

process.on("beforeExit", function () {
	zlog("Process exiting...");
	fs.writeFileSync(path.join(zentroxInstallationPath, "ftp_ppid.txt"), "---");
});

server = https.createServer(options, app);

server.listen(port, () => {
	zlog(`🚀 Zentrox running on port ${port}`, "info");
});
