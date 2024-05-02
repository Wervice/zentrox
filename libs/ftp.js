const ftp = require("ftp-srv");
const fs = require("fs");
const crypto = require("crypto");
const path = require("path");
const os = require("os");
const tls = require('tls');

const zentroxInstPath = path.join(os.homedir(), "zentrox_data/");
const ftp_txt = fs
  .readFileSync(path.join(zentroxInstPath, "ftp.txt"))
  .toString("ascii");
const ftp_root = ftp_txt.split("\n")[1];
const ftp_username = ftp_txt.split("\n")[0];
const ftp_password = ftp_txt.split("\n")[2];

function hash512(str) {
  // ? Calculate a SHA 512
  var hash = crypto.createHash("sha512");
  var data = hash.update(str, "utf-8");
  return data.digest("hex");
}

const tlsOptions = {
    key: fs.readFileSync(path.join("../", "selfsigned.key")),
    cert: fs.readFileSync(path.join("../", "selfsigned.crt"))
};

// Create FTP server instance
const ftpServer = new ftp({
  url: "ftp://127.0.0.1:2100",
  greeting: "Zentrox FTP",
  pasv_url: "ftps://127.0.0.1:3000",
  tls: tls.createSecureContext(tlsOptions)
});

// Handle FTP server events
ftpServer.on("login", ({ connection, username, password }, resolve, reject) => {
  if (username != ftp_username)
    return reject(new errors.GeneralError("Login failed", 401));
  if (hash512(password) != ftp_password)
    return reject(new errors.GeneralError("Login failed", 401));
  resolve({ root: ftp_root });
});

ftpServer.on("client-error", (error) => {
  console.error(`Client error: ${error["msg"]}`);
});

ftpServer.on("ftp-error", (error) => {
  console.error(`FTP server error: ${error["msg"]}`);
});

// Start the FTP server
 ftpServer
  .listen()
  .then(() => {
    console.log("FTP server listening on port 2100");
  })
  .catch((error) => {
    console.error(`Error starting FTP server: ${error}`);
  });
