const chpr = require("child_process");
const path = require("path");
eval(fs.readFileSync(path.join(__dirname, "sudo.js")) + "");

function systemctlStatus(daemon) {
  try {
    var active = chpr
      .execSync("systemctl status " + daemon, { timeout: 2000 })
      .toString("ascii")
      .includes("active");
  } catch (e) {
    var active = false;
  }
  return active;
}

function systemctlEnable(daemon, sudo) {
  try {
    execSyncAsSudo("systemctl enable --now " + daemon, sudo);
    return true;
  } catch {
    return false;
  }
}

function systemctlDisable(daemon, sudo) {
  try {
    execSyncAsSudo("systemctl disable --now " + daemon, sudo);
  } catch {
    return false;
  }
}

function systemctlReload(daemon, sudo) {
  try {
    execSyncAsSudo("systemctl reload --now " + daemon, sudo);
    return true;
  } catch {
    return false;
  }
}
