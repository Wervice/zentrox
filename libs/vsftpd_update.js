const fs = require("fs");
const process = require("process");
const path = require("path");
const os = require("os");

ftpUserUsername = process.argv[2];
ftpLocalRoot = process.argv[3];
user = process.argv[4];

const zentroxInstPath = path.join(
	path.join("/", "home", user),
	"zentrox_data/",
);

localRoot = fs
	.readFileSync(path.join(zentroxInstPath, "ftp.txt"))
	.toString("utf-8")
	.split("\n")[1];

fs.writeFileSync("/etc/vsftpd.userlist", ftpUserUsername);
fs.writeFileSync(
	"/etc/vsftpd.conf",
	fs
		.readFileSync("/etc/vsftpd.conf")
		.toString("utf8")
		.replaceAll(
			"local_root=" + localRoot + "\n",
			"local_root=" + ftpLocalRoot + "\n",
		),
);
fs.writeFileSync("/etc/vsftpd.userlist", ftpUserUsername);
